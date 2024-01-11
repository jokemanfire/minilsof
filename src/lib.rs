use glob::glob;
use std::collections::{HashMap, HashSet};
use std::fs::read_to_string;
use std::hash::Hash;
use std::path::Path;
use std::{fs, vec};

#[derive(Default, Debug, Clone)]
pub struct Fdinfo {
    pid: String,
    name: Option<String>,
    link: HashSet<String>,
}
#[derive(PartialEq, Clone)]
enum LsofFiletype {
    mem,
    all,
    socket,
}

pub struct LsofData {
    //target filetype
    targetFiletype: Option<LsofFiletype>,
    //save pid hash
    pidmap: HashMap<String, Fdinfo>,
    //save target-pid
    targetmap: HashMap<String, HashSet<String>>,
    //target_file name
    targetFilename: String,
}

impl LsofData {
    pub fn new() -> LsofData {
        LsofData {
            targetFiletype: None,
            pidmap: HashMap::new(),
            targetmap: HashMap::new(),
            targetFilename: String::new(),
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
                    if let Some((key, value)) = line.split_once(":") {
                        map.insert(key.trim().to_string(), value.trim().to_string());
                    }
                }
            }
            Err(e) => {
                println!("Error reading file: {}", e);
            }
        }
        return map;
    }

    fn get_mem_info(&self, path: String) -> Vec<String> {
        let mut datas: Vec<String> = Vec::new();
        match read_to_string(path) {
            Ok(content) => {
                for line in content.lines() {
                    let v: Vec<&str> = line.split(" ").collect();
                    if v.len() >= 6 && !v.last().unwrap().is_empty() {
                        datas.push(v.last().unwrap().to_string());
                    }
                }
            }
            Err(e) => {
                println!("Error reading file : {}", e);
            }
        }
        return datas;
    }
    fn target_map_insert(&mut self, pid: String) {
        let r = self.targetmap.get_mut(&self.targetFilename);
        match r {
            Some(set) => {
                set.insert(pid);
            }
            None => {
                let mut new_set: HashSet<String> = HashSet::new();
                new_set.insert(pid);
                self.targetmap.insert(self.targetFilename.clone(), new_set);
            }
        }
    }

    fn set_list_all(&mut self) {
        let proc_paths = glob("/proc/*").unwrap();
        for proc_path_r in proc_paths {
            let proc_path_str = proc_path_r.unwrap().to_str().unwrap().to_string();
            let fd_path_str = String::from(proc_path_str.clone() + "/fd/*");

            let mut info = Fdinfo::default();
            let pid = proc_path_str
                .split("/")
                .collect::<Vec<&str>>()
                .last()
                .unwrap()
                .clone();

            let temp_k = pid.parse::<u64>();
            if let Err(_) = temp_k {
                continue;
            }
            info.pid = pid.to_string();

            //get process other info
            let other_info = self.get_pid_info(format!("/proc/{}/status", pid).to_string());
            info.name = other_info.get("Name").cloned();

            //get process mem info
            if self.targetFiletype.is_some()
                && (self.targetFiletype.clone().unwrap() == LsofFiletype::mem
                    || self.targetFiletype.clone().unwrap() == LsofFiletype::all)
            {
                let mem_info = self.get_mem_info(format!("/proc/{}/maps", pid).to_string());
                for i in mem_info {
                    info.link.insert(i.clone());
                    if !self.targetFilename.is_empty() && self.targetFilename == i {
                        self.target_map_insert(pid.to_string());
                    }
                }
            }

            //get fd link
            let fd_paths = glob(fd_path_str.as_str()).unwrap();
            for fd_path in fd_paths {
                if let Err(_) = fd_path {
                    continue;
                }
                //else
                let link = fs::read_link(&fd_path.unwrap());
                if let Ok(link) = link {
                    let link_str = link.to_str().unwrap().to_string();
                    info.link.insert(link_str.clone());
                    if !self.targetFilename.is_empty() && self.targetFilename == link_str{
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
        ///to do judge file type
        if let Ok(metadata) = metadata {
            let file_type = metadata.file_type();
            println!("File type {:?}", file_type);
        }

        self.targetFiletype = Some(LsofFiletype::all);
        self.targetFilename = path;
        self.set_list_all();

        let t_result = self.targetmap.get(&self.targetFilename);
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
        return Some(result);
    }
    //get all infomation
    pub fn file_ls(&mut self) -> Option<&HashMap<String, Fdinfo>> {
        self.targetFiletype = Some(LsofFiletype::all);
        self.set_list_all();
        return Some(&self.pidmap);
    }

    //get port used by process
    pub fn port_ls(&mut self,port:String) -> Option<Vec<Fdinfo>>{
        let mut result: Vec<Fdinfo> = Vec::new();
        self.targetFiletype = Some(LsofFiletype::socket);
        self.targetFilename = format!("socket:[{}]",port);
        self.set_list_all();
        let t_result = self.targetmap.get(&self.targetFilename);
        match t_result{
            Some(t) => {
                for s in t{
                    if let Some(d) = self.pidmap.get(s) {
                        result.push(d.clone());
                    }
                }
            }
            None =>{
                return None;
            }
        }
        return  Some(result);
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
    println!("{:?}", result);
}
#[test]
fn test_port(){
    let port = "46578".to_string();
    let mut d = LsofData::new();
    let result = d.port_ls(port);
    println!("{:?}", result);
}