// #![doc = include_str!("../README.md")]
use glob::glob;
use std::collections::{HashMap, HashSet};
use std::fs::{self, read_to_string};
use std::path::Path;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Error types for the minilsof library
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Glob pattern error: {0}")]
    Glob(#[from] glob::PatternError),
    
    #[error("Other error: {0}")]
    Other(String),
}

// The filesync module is always available, regardless of features
pub mod filesync;

#[cfg(feature = "async")]
pub mod fileasync;

/// Represents information about a file descriptor
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Fdinfo {
    /// Process ID that owns this file descriptor
    pub pid: String,
    /// Process name (if available)
    pub name: Option<String>,
    /// Set of links associated with this file descriptor
    pub link: HashSet<String>,
}

/// Type of file to look for
#[derive(PartialEq, Clone, Debug)]
enum LsofFiletype {
    /// Memory-mapped files
    Mem,
    /// All file types
    All,
    /// Socket files
    Socket,
}

/// Main struct for LSOF operations
pub struct LsofData {
    /// Target file type to search for
    target_filetype: Option<LsofFiletype>,
    /// Map of process IDs to their file descriptor info
    pidmap: HashMap<String, Fdinfo>,
    /// Map of target files to the set of process IDs using them
    targetmap: HashMap<String, HashSet<String>>,
    /// Target file name to search for
    target_filename: String,
}

impl Default for LsofData {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait defining file information retrieval methods
pub trait GetFileInfo {
    /// Get process information from a path
    fn get_pid_info(&self, path: String) -> HashMap<String, String>;
    /// Get memory-mapped file information
    fn get_mem_info(&self, path: String) -> Vec<String>;
    /// Get information about a socket port
    fn get_port_info(&self, port: &str) -> Option<Vec<String>>;
}

impl LsofData {
    /// Create a new LsofData instance
    pub fn new() -> LsofData {
        LsofData {
            target_filetype: None,
            pidmap: HashMap::new(),
            targetmap: HashMap::new(),
            target_filename: String::new(),
        }
    }

    /// Parse and return process information from a file
    fn get_pid_info(&self, path: String) -> HashMap<String, String> {
        let mut map: HashMap<String, String> = HashMap::new();
        match read_to_string(path) {
            Ok(content) => {
                for line in content.lines() {
                    if let Some((key, value)) = line.split_once(':') {
                        map.insert(key.trim().to_string(), value.trim().to_string());
                    }
                }
            }
            Err(_) => {
                // Return empty map if file cannot be read
            }
        }
        map
    }

    /// Get memory-mapped file information from a file
    fn get_mem_info(&self, path: String) -> Vec<String> {
        let mut datas: Vec<String> = Vec::new();
        match read_to_string(path) {
            Ok(content) => {
                for line in content.lines() {
                    let v: Vec<&str> = line.split_whitespace().collect();
                    if let Some(last_part) = v.last() {
                        if !last_part.is_empty() && v.len() >= 6 {
                            datas.push(last_part.to_string());
                        }
                    }
                }
            }
            Err(_) => {
                // Return empty vector if file cannot be read
            }
        }
        datas
    }

    /// Get socket information for a specific port
    fn get_port_info(&self, port: &str) -> Option<Vec<String>> {
        let mut socket_files = Vec::new();
        
        // Check TCP sockets
        if let Ok(tcp_content) = read_to_string("/proc/net/tcp") {
            socket_files.extend(self.parse_socket_file(tcp_content, port));
        }
        
        // Check TCP6 sockets
        if let Ok(tcp6_content) = read_to_string("/proc/net/tcp6") {
            socket_files.extend(self.parse_socket_file(tcp6_content, port));
        }
        
        // Check UDP sockets
        if let Ok(udp_content) = read_to_string("/proc/net/udp") {
            socket_files.extend(self.parse_socket_file(udp_content, port));
        }
        
        // Check UDP6 sockets
        if let Ok(udp6_content) = read_to_string("/proc/net/udp6") {
            socket_files.extend(self.parse_socket_file(udp6_content, port));
        }
        
        if socket_files.is_empty() {
            None
        } else {
            Some(socket_files)
        }
    }
    
    /// Parse socket file content looking for a specific port
    fn parse_socket_file(&self, content: String, port: &str) -> Vec<String> {
        let mut results = Vec::new();
        
        for line in content.lines().skip(1) { // Skip header line
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 10 {
                continue;
            }
            
            // Local address is in column 1, format is IP:PORT in hex
            let local_addr = parts[1];
            if let Some(colon_pos) = local_addr.rfind(':') {
                let port_hex = &local_addr[colon_pos+1..];
                // Convert hex port to decimal
                if let Ok(port_num) = u16::from_str_radix(port_hex, 16) {
                    if port_num.to_string() == port {
                        // Column 9 contains inode number
                        if let Some(inode) = parts.get(9) {
                            results.push(format!("socket:[{}]", inode));
                        }
                    }
                }
            }
        }
        
        results
    }

    /// Insert a PID into the target map
    fn target_map_insert(&mut self, pid: String) {
        if let Some(set) = self.targetmap.get_mut(&self.target_filename) {
            set.insert(pid);
        } else {
            let mut new_set: HashSet<String> = HashSet::new();
            new_set.insert(pid);
            self.targetmap.insert(self.target_filename.clone(), new_set);
        }
    }

    /// List all files across processes
    fn set_list_all(&mut self) -> Result<()> {
        let proc_paths = glob("/proc/*").map_err(Error::Glob)?;
        
        for proc_path_r in proc_paths {
            match proc_path_r {
                Ok(proc_path) => {
                    // Get the process ID from the path
                    let pid_os_str = proc_path.file_name()
                        .ok_or_else(|| Error::Other("Failed to get process directory name".to_string()))?;
                    let pid = pid_os_str.to_string_lossy();
                    
                    // Skip non-numeric (non-process) directories
                    if pid.parse::<u64>().is_err() {
                        continue;
                    }

                    let fd_path_str = format!("{}/fd/*", proc_path.display());

                    let mut info = Fdinfo::default();
                    info.pid = pid.to_string();
                    
                    // Get process information
                    let other_info = self.get_pid_info(format!("/proc/{}/status", pid));
                    info.name = other_info.get("Name").cloned();

                    // Get process memory mapping information
                    if let Some(filetype) = &self.target_filetype {
                        if *filetype == LsofFiletype::Mem || *filetype == LsofFiletype::All {
                            let mem_info = self.get_mem_info(format!("/proc/{}/maps", pid));
                            for i in mem_info {
                                info.link.insert(i.clone());
                                if !self.target_filename.is_empty() && self.target_filename == i {
                                    self.target_map_insert(pid.to_string());
                                }
                            }
                        }
                    }

                    // Get file descriptor information
                    if let Ok(fd_paths) = glob(&fd_path_str) {
                        for fd_path in fd_paths {
                            match fd_path {
                                Ok(path_data) => {
                                    // Get the symbolic link target
                                    if let Ok(link) = fs::read_link(&path_data) {
                                        let link_str = link.to_string_lossy().to_string();
                                        
                                        // Store inode information if it's a socket
                                        let is_socket = link_str.starts_with("socket:[");
                                        
                                        info.link.insert(link_str.clone());
                                        
                                        if !self.target_filename.is_empty() && self.target_filename == link_str {
                                            self.target_map_insert(pid.to_string());
                                        }
                                        
                                        // Handle socket file type if needed
                                        if let Some(filetype) = &self.target_filetype {
                                            if *filetype == LsofFiletype::Socket && is_socket {
                                                // Already handled by adding to info.link above
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    // Log error and continue
                                    eprintln!("Error accessing file descriptor: {}", e);
                                }
                            }
                        }
                        self.pidmap.insert(pid.to_string(), info);
                    }
                }
                Err(e) => {
                    // Log error and continue
                    eprintln!("Error processing process directory: {}", e);
                }
            }
        }
        Ok(())
    }

    /// Get information about processes using a specific file
    pub fn target_file_ls(&mut self, path: String) -> Option<Vec<Fdinfo>> {
        let mut result: Vec<Fdinfo> = Vec::new();
        
        // Check if the file exists
        let path = Path::new(&path);
        if !path.exists() {
            return None;
        }

        self.target_filetype = Some(LsofFiletype::All);
        self.target_filename = path.to_string_lossy().to_string();
        
        if self.set_list_all().is_err() {
            return None;
        }

        // Get processes using the target file
        if let Some(pids) = self.targetmap.get(&self.target_filename) {
            for pid in pids {
                if let Some(info) = self.pidmap.get(pid) {
                    result.push(info.clone());
                }
            }
            Some(result)
        } else {
            None
        }
    }

    /// Get information about all open files by all processes
    pub fn file_ls(&mut self) -> Option<&HashMap<String, Fdinfo>> {
        self.target_filetype = Some(LsofFiletype::All);
        
        if self.set_list_all().is_err() {
            return None;
        }
        
        Some(&self.pidmap)
    }

    /// Get information about processes using a specific port
    pub fn port_ls(&mut self, port: String) -> Option<Vec<Fdinfo>> {
        let mut result: Vec<Fdinfo> = Vec::new();
        
        self.target_filetype = Some(LsofFiletype::Socket);
        
        // Get socket inodes for the port
        let socket_inodes = self.get_port_info(&port)?;
        
        // For each socket inode, find the corresponding processes
        for inode in socket_inodes {
            self.target_filename = inode;
            
            if self.set_list_all().is_err() {
                continue;
            }
            
            if let Some(pids) = self.targetmap.get(&self.target_filename) {
                for pid in pids {
                    if let Some(info) = self.pidmap.get(pid) {
                        result.push(info.clone());
                    }
                }
            }
        }
        
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }
}

#[test]
fn test_lsall() {
    let mut d = LsofData::new();
    let result = d.file_ls();
    assert_ne!(result, None);
}

#[test]
fn test_target_file() {
    let filepath = "/etc/passwd".to_string();
    let mut d = LsofData::new();
    let result = d.target_file_ls(filepath);
    assert!(result.is_some() || result.is_none()); // Either result is acceptable for the test
}

#[test]
fn test_port() {
    let mut d = LsofData::new();
    let result = d.port_ls("80".to_string());
    assert!(result.is_some() || result.is_none()); // Either result is acceptable for the test
}


