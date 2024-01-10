Rust - mini lsof

a easy lsof Implemented by Rust


Example:


use minilsof::LsofData;

fn main(){
    let filepath = "/usr/lib64/librt-2.28.so".to_string();
    let mut d = LsofData::new();
    let result = d.target_file_ls(filepath);
    println!("{:?}",result);
}