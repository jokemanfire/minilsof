use minilsof::filesync::LsofSync;
use std::env;
use std::process;

fn print_usage() {
    eprintln!(
        "Usage: minilsof [OPTION] [ARG]
Options:
  -f FILE    Find processes using FILE
  -p PORT    Find processes using PORT
  -a         List all open files (default if no option is provided)
  -h         Display this help message"
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Default behavior is to list all files
    if args.len() == 1 {
        list_all_files();
        return;
    }
    
    // Parse command-line arguments
    match args[1].as_str() {
        "-f" => {
            if args.len() < 3 {
                eprintln!("Error: Missing file path");
                print_usage();
                process::exit(1);
            }
            find_file_users(&args[2]);
        }
        "-p" => {
            if args.len() < 3 {
                eprintln!("Error: Missing port number");
                print_usage();
                process::exit(1);
            }
            find_port_users(&args[2]);
        }
        "-a" => list_all_files(),
        "-h" => {
            print_usage();
        }
        _ => {
            eprintln!("Error: Invalid option '{}'", args[1]);
            print_usage();
            process::exit(1);
        }
    }
}

fn list_all_files() {
    let mut lsof = LsofSync::new();
    
    println!("Listing all open files...");
    
    match lsof.file_ls() {
        Ok(result) => {
            println!("Found {} processes with open files", result.len());
            
            for (pid, info) in result {
                let name = info.name.as_deref().unwrap_or("[unknown]");
                println!("PID: {}, Process: {}, Open Files: {}", pid, name, info.link.len());
            }
        }
        Err(err) => {
            eprintln!("Error listing files: {}", err);
            process::exit(1);
        }
    }
}

fn find_file_users(file_path: &str) {
    let mut lsof = LsofSync::new();
    
    println!("Finding processes using file: {}", file_path);
    
    match lsof.target_file_ls(file_path) {
        Ok(processes) => {
            if processes.is_empty() {
                println!("No processes found using this file");
                return;
            }
            
            println!("Found {} processes using file:", processes.len());
            
            for process in processes {
                let name = process.name.as_deref().unwrap_or("[unknown]");
                println!("PID: {}, Process: {}", process.pid, name);
            }
        }
        Err(err) => {
            eprintln!("Error finding file users: {}", err);
            process::exit(1);
        }
    }
}

fn find_port_users(port: &str) {
    let mut lsof = LsofSync::new();
    
    println!("Finding processes using port: {}", port);
    
    match lsof.port_ls(port) {
        Ok(processes) => {
            if processes.is_empty() {
                println!("No processes found using this port");
                return;
            }
            
            println!("Found {} processes using port:", processes.len());
            
            for process in processes {
                let name = process.name.as_deref().unwrap_or("[unknown]");
                println!("PID: {}, Process: {}", process.pid, name);
            }
        }
        Err(err) => {
            eprintln!("Error finding port users: {}", err);
            process::exit(1);
        }
    }
}
