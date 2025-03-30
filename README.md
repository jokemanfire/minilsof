# minilsof - A Lightweight Rust Implementation of lsof

`minilsof` is a lightweight Rust library that provides functionality similar to the Linux `lsof` command. It allows you to list open files, find processes using specific files, and identify processes listening on specific ports.

## Features

- Find all open files across all processes
- Find processes using a specific file
- Find processes listening on a specific port
- Synchronous and asynchronous API

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
minilsof = "0.1.2"
```

If you want to use the async API, enable the `async` feature:

```toml
[dependencies]
minilsof = { version = "0.1.2", features = ["async"] }
```

## Usage Examples

### Synchronous API

```rust
use minilsof::filesync::LsofSync;

// List all open files
fn list_all_files() {
    let mut lsof = LsofSync::new();
    match lsof.file_ls() {
        Ok(result) => {
            println!("Found {} processes with open files", result.len());
            // Process the results
        },
        Err(err) => eprintln!("Error: {}", err),
    }
}

// Find processes using a specific file
fn find_processes_using_file() {
    let mut lsof = LsofSync::new();
    let filepath = "/usr/lib/libssl.so.3";
    match lsof.target_file_ls(filepath) {
        Ok(processes) => {
            println!("Found {} processes using {}", processes.len(), filepath);
            for process in processes {
                println!("PID: {}, Name: {:?}", process.pid, process.name);
            }
        },
        Err(err) => eprintln!("Error: {}", err),
    }
}

// Find processes using a specific port
fn find_processes_using_port() {
    let mut lsof = LsofSync::new();
    let port = "80";
    match lsof.port_ls(port) {
        Ok(processes) => {
            println!("Found {} processes using port {}", processes.len(), port);
            for process in processes {
                println!("PID: {}, Name: {:?}", process.pid, process.name);
            }
        },
        Err(err) => eprintln!("Error: {}", err),
    }
}
```

### Asynchronous API

With the `async` feature enabled:

```rust,ignore
// This example requires the "async" feature to be enabled
use minilsof::fileasync::LsofAsync;

// List all open files asynchronously
async fn list_all_files_async() {
    let lsof = LsofAsync::new();
    match lsof.file_ls().await {
        Ok(result) => {
            println!("Found {} processes with open files", result.len());
            // Process the results
        },
        Err(err) => eprintln!("Error: {}", err),
    }
}

// Find processes using a specific file asynchronously
async fn find_processes_using_file_async() {
    let lsof = LsofAsync::new();
    let filepath = "/usr/lib/libssl.so.3";
    match lsof.target_file_ls(filepath).await {
        Ok(processes) => {
            println!("Found {} processes using {}", processes.len(), filepath);
            for process in processes {
                println!("PID: {}, Name: {:?}", process.pid, process.name);
            }
        },
        Err(err) => eprintln!("Error: {}", err),
    }
}
```

## Platform Support

This library is designed for Linux systems and requires access to the `/proc` filesystem.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

```rust
use minilsof::LsofData;

//if return data is none unwarp will panic!

//lsof all
#[test]
fn test_lsall(){
    let mut d = LsofData::new();
    if let Some(result) = d.file_ls(){
        println!("{:?}",result);
    }
}
//target file
#[test]
fn test_target(){
    let filepath = "/usr/lib64/librt-2.28.so".to_string();
    let mut d = LsofData::new();
    let result = d.target_file_ls(filepath);
    if let Some(result) = d.target_file_ls(){
        println!("{:?}",result);
    }
}


```