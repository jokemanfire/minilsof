# Rust - mini lsof

a easy lsof Implemented by Rust


# Example

```rust
use minilsof::LsofData;

//if return data is none unwarp will panic!

//lsof all
#[test]
fn test_lsall(){
    let mut d = LsofData::new();
    let result = d.file_ls().unwrap();
    println!("{:?}",result);
}
//target file
#[test]
fn test_target(){
    let filepath = "/usr/lib64/librt-2.28.so".to_string();
    let mut d = LsofData::new();
    let result = d.target_file_ls(filepath).unwrap();
    println!("{:?}",result);
}


```