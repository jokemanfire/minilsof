use glob::glob;
use std::collections::{HashMap, HashSet};
use std::fs::{self, read_to_string};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error is other: {0}")]
    Other(String),
}

#[cfg(not(feature = "async"))]
pub mod filesync;

pub mod fileasync;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Fdinfo {
    pid: String,
    name: Option<String>,
    link: HashSet<String>,
}
#[derive(PartialEq, Clone)]
enum LsofFiletype {
    Mem,
    All,
    Socket,
}

pub struct LsofData {
    //target filetype
    target_filetype: Option<LsofFiletype>,
    //save pid hash
    pidmap: HashMap<String, Fdinfo>,
    //save target file and pid
    targetmap: HashMap<String, HashSet<String>>,
    //target_file name
    target_filename: String,
}
impl Default for LsofData {
    fn default() -> Self {
        Self::new()
    }
}

pub trait GetFileInfo {
    fn get_pid_info(&self, path: String) -> HashMap<String, String>;
    fn get_mem_info(&self, path: String) -> Vec<String>;
    fn get_port_info(&self, port: String);
}

impl LsofData {
    pub fn new() -> LsofData {
        LsofData {
            target_filetype: None,
            pidmap: HashMap::new(),
            targetmap: HashMap::new(),
            target_filename: String::new(),
        }
    }

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
                return map;
            }
        }
        map
    }

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
                return datas;
            }
        }
        datas
    }

    fn target_map_insert(&mut self, pid: String) {
        let r = self.targetmap.get_mut(&self.target_filename);
        match r {
            Some(set) => {
                set.insert(pid);
            }
            None => {
                let mut new_set: HashSet<String> = HashSet::new();
                new_set.insert(pid);
                self.targetmap.insert(self.target_filename.clone(), new_set);
            }
        }
    }

    fn set_list_all(&mut self) -> Result<()> {
        let proc_paths =
            glob("/proc/*").map_err(|e| Error::Other(format!("Get proc path fail {}", e)))?;
        for proc_path_r in proc_paths {
            match proc_path_r {
                Ok(proc_path) => {
                    let fd_path_str = proc_path.as_os_str().to_string_lossy() + "/fd/*";

                    let pid = proc_path.file_name().unwrap().to_string_lossy();
                    if pid.parse::<u64>().is_err() {
                        continue;
                    };

                    let mut info = Fdinfo::default();
                    info.pid = pid.to_string();
                    //get process other info
                    let other_info = self.get_pid_info(format!("/proc/{}/status", pid).to_string());
                    info.name = other_info.get("Name").cloned();

                    //get process mem info
                    if let Some(filetype) = &self.target_filetype {
                        if *filetype == LsofFiletype::Mem || *filetype == LsofFiletype::All {
                            let mem_info = self.get_mem_info(format!("/proc/{}/maps", pid));
                            for i in mem_info {
                                info.link.insert(i.clone());
                                if !self.target_filename.is_empty() && self.target_filename == *i {
                                    self.target_map_insert(pid.to_string());
                                }
                            }
                        }
                    }

                    //get fd link
                    if let Ok(fd_paths) = glob(&fd_path_str) {
                        for fd_path in fd_paths {
                            if fd_path.is_err() {
                                continue;
                            }
                            let path_data = fd_path.unwrap().clone();

                            let link = fs::read_link(&path_data);
                            //TODO inode info
                            if let Ok(link) = link {
                                let link_str = link.to_string_lossy();
                                info.link.insert(link_str.to_string());
                                if !self.target_filename.is_empty()
                                    && self.target_filename == link_str
                                {
                                    self.target_map_insert(pid.to_string());
                                }
                                if self.target_filetype.clone().unwrap() == LsofFiletype::Socket {
                                    todo!()
                                }
                            }
                        }
                        self.pidmap.insert(pid.to_string(), info);
                    }
                }
                Err(_) => todo!(),
            }
        }
        Ok(())
    }

    //get target info
    pub fn target_file_ls(&mut self, path: String) -> Option<Vec<Fdinfo>> {
        let mut result: Vec<Fdinfo> = Vec::new();
        let metadata = fs::metadata(&path);
        //to do judge file type
        if let Ok(metadata) = metadata {
            let _file_type = metadata.file_type();
            //file_type judgement
        }

        self.target_filetype = Some(LsofFiletype::All);
        self.target_filename = path;
        if self.set_list_all().is_err() {
            return None;
        }

        let t_result = self.targetmap.get(&self.target_filename);
        match t_result {
            Some(t) => {
                for s in t {
                    if let Some(d) = self.pidmap.get(s) {
                        result.push(d.clone());
                    }
                }
            }
            None => {
                return None;
            }
        }
        Some(result)
    }

    //get all infomation
    pub fn file_ls(&mut self) -> Option<&HashMap<String, Fdinfo>> {
        self.target_filetype = Some(LsofFiletype::All);
        if self.set_list_all().is_err() {
            return None;
        }
        Some(&self.pidmap)
    }

    //get socket port used by process
    pub fn port_ls(&mut self, port: String) -> Option<Vec<Fdinfo>> {
        let mut result: Vec<Fdinfo> = Vec::new();
        self.target_filetype = Some(LsofFiletype::Socket);
        self.target_filename = format!("socket:[{}]", port);
        if self.set_list_all().is_err() {
            return None;
        }
        let t_result: Option<&HashSet<String>> = self.targetmap.get(&self.target_filename);
        match t_result {
            Some(t) => {
                for s in t {
                    if let Some(d) = self.pidmap.get(s) {
                        result.push(d.clone());
                    }
                }
            }
            None => {
                return None;
            }
        }
        Some(result)
    }
}

#[test]
fn test_lsall() {
    let mut d = LsofData::new();
    let result = d.file_ls();
    assert_ne!(result, None);
}


