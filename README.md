# Rust - mini lsof

A easy lsof Implemented by Rust

Include library and example

example is a easy cli program

library will use async and sync implementions
# Example

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