#[cfg(feature = "async")]
use tokio::task;
use crate::{Fdinfo, LsofData, Result};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Asynchronous wrapper functions for LsofData
/// 
/// This implementation uses tokio to run the blocking operations
/// in a separate thread pool to avoid blocking the async runtime.
#[cfg(feature = "async")]
pub struct LsofAsync {
    inner: Arc<Mutex<LsofData>>,
}

#[cfg(feature = "async")]
impl LsofAsync {
    /// Create a new LsofAsync instance
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(LsofData::new())),
        }
    }

    /// Get information about all open files by all processes
    pub async fn file_ls(&self) -> Result<HashMap<String, Fdinfo>> {
        let inner = self.inner.clone();
        
        task::spawn_blocking(move || {
            let mut data = inner.lock().map_err(|_| 
                crate::Error::Other("Failed to acquire lock".to_string())
            )?;
            
            let result = data.file_ls().ok_or_else(|| 
                crate::Error::Other("Failed to list all files".to_string())
            )?;
            
            // Clone the result to avoid returning a reference to data inside the lock
            Ok(result.clone())
        }).await.map_err(|_| crate::Error::Other("Task join error".to_string()))?
    }

    /// Get information about processes using a specific file
    pub async fn target_file_ls(&self, path: impl AsRef<Path> + Send + 'static) -> Result<Vec<Fdinfo>> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let inner = self.inner.clone();
        
        task::spawn_blocking(move || {
            let mut data = inner.lock().map_err(|_| 
                crate::Error::Other("Failed to acquire lock".to_string())
            )?;
            
            data.target_file_ls(path_str).ok_or_else(|| 
                crate::Error::Other(format!("Failed to list file: {}", path.as_ref().display()))
            )
        }).await.map_err(|_| crate::Error::Other("Task join error".to_string()))?
    }

    /// Get information about processes using a specific port
    pub async fn port_ls(&self, port: impl AsRef<str> + Send + 'static) -> Result<Vec<Fdinfo>> {
        let port_str = port.as_ref().to_string();
        let inner = self.inner.clone();
        
        task::spawn_blocking(move || {
            let mut data = inner.lock().map_err(|_| 
                crate::Error::Other("Failed to acquire lock".to_string())
            )?;
            
            data.port_ls(port_str).ok_or_else(|| 
                crate::Error::Other(format!("Failed to list port: {}", port.as_ref()))
            )
        }).await.map_err(|_| crate::Error::Other("Task join error".to_string()))?
    }
}

#[cfg(feature = "async")]
impl Default for LsofAsync {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, feature = "async"))]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_async_file_ls() {
        let lsof = LsofAsync::new();
        let result = lsof.file_ls().await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_async_target_file() {
        let lsof = LsofAsync::new();
        // Use a common file that should exist on most systems
        let result = lsof.target_file_ls("/etc/passwd").await;
        // The test passes whether or not processes are using the file
        if let Ok(processes) = &result {
            println!("Found {} processes using /etc/passwd", processes.len());
        }
    }
    
    #[tokio::test]
    async fn test_async_port() {
        let lsof = LsofAsync::new();
        let result = lsof.port_ls("80").await;
        // The test passes whether or not processes are using the port
        if let Ok(processes) = &result {
            println!("Found {} processes using port 80", processes.len());
        }
    }
}
