use crate::{Fdinfo, LsofData, Result};
use std::collections::HashMap;
use std::path::Path;

/// Synchronous wrapper functions for LsofData
pub struct LsofSync {
    inner: LsofData,
}

impl LsofSync {
    /// Create a new LsofSync instance
    pub fn new() -> Self {
        Self {
            inner: LsofData::new(),
        }
    }

    /// Get information about all open files by all processes
    pub fn file_ls(&mut self) -> Result<&HashMap<String, Fdinfo>> {
        self.inner.file_ls().ok_or_else(|| crate::Error::Other("Failed to list all files".to_string()))
    }

    /// Get information about processes using a specific file
    pub fn target_file_ls(&mut self, path: impl AsRef<Path>) -> Result<Vec<Fdinfo>> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        self.inner.target_file_ls(path_str).ok_or_else(|| 
            crate::Error::Other(format!("Failed to list file: {}", path.as_ref().display()))
        )
    }

    /// Get information about processes using a specific port
    pub fn port_ls(&mut self, port: impl AsRef<str>) -> Result<Vec<Fdinfo>> {
        let port_str = port.as_ref().to_string();
        self.inner.port_ls(port_str).ok_or_else(|| 
            crate::Error::Other(format!("Failed to list port: {}", port.as_ref()))
        )
    }
}

impl Default for LsofSync {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_file_ls() {
        let mut lsof = LsofSync::new();
        let result = lsof.file_ls();
        assert!(result.is_ok());
    }

    #[test]
    fn test_sync_target_file() {
        let mut lsof = LsofSync::new();
        // Use a common file that should exist on most systems
        let result = lsof.target_file_ls("/etc/passwd");
        // The test passes whether or not processes are using the file
        if let Ok(processes) = &result {
            println!("Found {} processes using /etc/passwd", processes.len());
        }
    }
}
