# Rust - mini lsof

a easy lsof Implemented by Rust


# Example

```rust
use minilsof::LsofData;


#[test]
fn test_lsall(){
    let mut d = LsofData::new();
    let result = d.file_ls().unwrap();
    println!("{:?}",result);
}

#[test]
fn test_target(){
    let filepath = "/usr/lib64/librt-2.28.so".to_string();
    let mut d = LsofData::new();
    let result = d.target_file_ls(filepath).unwrap();
    println!("{:?}",result);
}

```