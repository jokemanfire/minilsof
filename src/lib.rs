use glob::glob;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::read_to_string;

#[derive(Default, Debug, Clone)]
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
    //save target-pid
    targetmap: HashMap<String, HashSet<String>>,
    //target_file name
    target_filename: String,
}
impl Default for LsofData {
    fn default() -> Self {
        Self::new()
    }
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
        if path.is_empty() {
            return map;
        }
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
                // println!("Error reading file: {}", e);
            }
        }
        map
    }

    fn get_mem_info(&self, path: String) -> Vec<String> {
        let mut datas: Vec<String> = Vec::new();
        match read_to_string(path) {
            Ok(content) => {
                for line in content.lines() {
                    let v: Vec<&str> = line.split(' ').collect();
                    if v.len() >= 6 && !v.last().unwrap().is_empty() {
                        datas.push(v.last().unwrap().to_string());
                    }
                }
            }
            Err(_) => {
                return datas;
                // println!("Error reading file : {}", e);
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

    fn set_list_all(&mut self) {
        let proc_paths = glob("/proc/*").unwrap();
        for proc_path_r in proc_paths {
            let proc_path_str = proc_path_r.unwrap().to_str().unwrap().to_string();
            let fd_path_str = proc_path_str.clone() + "/fd/*";

            let mut info = Fdinfo::default();
            let pid = *(proc_path_str
                .split('/')
                .collect::<Vec<&str>>()
                .last()
                .unwrap());

            let temp_k = pid.parse::<u64>();
            if temp_k.is_err(){
                continue;
            }
            info.pid = pid.to_string();

            //get process other info
            let other_info = self.get_pid_info(format!("/proc/{}/status", pid).to_string());
            info.name = other_info.get("Name").cloned();

            //get process mem info
            if self.target_filetype.is_some()
                && (self.target_filetype.clone().unwrap() == LsofFiletype::Mem
                    || self.target_filetype.clone().unwrap() == LsofFiletype::All)
            {
                let mem_info = self.get_mem_info(format!("/proc/{}/maps", pid).to_string());
                for i in mem_info {
                    info.link.insert(i.clone());
                    if !self.target_filename.is_empty() && self.target_filename == i {
                        self.target_map_insert(pid.to_string());
                    }
                }
            }

            //get fd link
            let fd_paths = glob(fd_path_str.as_str()).unwrap();
            for fd_path in fd_paths {
                if fd_path.is_err(){
                    continue;
                }
                //else
                let link = fs::read_link(&fd_path.unwrap());
                if let Ok(link) = link {
                    let link_str = link.to_str().unwrap().to_string();
                    info.link.insert(link_str.clone());
                    if !self.target_filename.is_empty() && self.target_filename == link_str {
                        self.target_map_insert(pid.to_string());
                    }
                }
            }
            self.pidmap.insert(pid.to_string(), info);
        }
    }

    //get target info
    pub fn target_file_ls(&mut self, path: String) -> Option<Vec<Fdinfo>> {
        let mut result: Vec<Fdinfo> = Vec::new();
        let metadata = fs::metadata(&path);
        //to do judge file type
        if let Ok(metadata) = metadata {
            let file_type = metadata.file_type();
            println!("File type {:?}", file_type);
        }

        self.target_filetype = Some(LsofFiletype::All);
        self.target_filename = path;
        self.set_list_all();

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
        self.set_list_all();
        Some(&self.pidmap)
    }

    //get socket port used by process
    pub fn port_ls(&mut self, port: String) -> Option<Vec<Fdinfo>> {
        let mut result: Vec<Fdinfo> = Vec::new();
        self.target_filetype = Some(LsofFiletype::Socket);
        self.target_filename = format!("socket:[{}]", port);
        self.set_list_all();
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
    let result = d.file_ls().unwrap();
    println!("{:?}", result);
}

#[test]
fn test_target() {
    let filepath = "/usr/lib64/librt-2.28.so".to_string();
    let mut d = LsofData::new();
    let result = d.target_file_ls(filepath).unwrap();
    // println!("{:?}", result);
    for r in result{
        println!("pid:{}  ,name: {:?} \n",r.pid,r.name)
    }
}
// #[test]
// fn test_port() {
//     let port = "43869".to_string();
//     let mut d = LsofData::new();
//     let result = d.port_ls(port);
//     println!("{:?}", result);
// }
