use minilsof::LsofData;

fn main() {
    let mut d = LsofData::new();
    if let Some(result) = d.file_ls(){
        println!("result {:?}",result)
    }

}
