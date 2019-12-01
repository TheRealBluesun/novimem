use regex::Regex;
use std::{fs, fs::OpenOptions, io::Read, u32};

pub struct ProcSearch {}

impl ProcSearch {
    pub fn search(procname: &str) -> Option<Vec<(String, u32)>> {
        let paths = fs::read_dir("/proc/").unwrap();
        let re = Regex::new(r"/proc/(\d+)").unwrap();
        let mut resvec = Vec::<(String, u32)>::new();
        for path in paths {
            let p = path.unwrap().path();
            let path_str = p.into_os_string().into_string().unwrap();
            if let Some(cap) = re.captures(&path_str) {
                // PIDs are always base 10
                if let Ok(pid) = u32::from_str_radix(&cap[1], 10) {
                    let pname = ProcSearch::get_pname(pid);
                    if pname.contains(procname) {
                        resvec.push((pname, pid));
                    }
                }
            }
        }
        if !resvec.is_empty() {
            Some(resvec)
        } else {
            None
        }
    }

    fn get_pname(pid: u32) -> String {
        let mut retstr = String::new();
        OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(format!("/proc/{}/cmdline", pid))
            .expect("Unable to open pname file")
            .read_to_string(&mut retstr)
            .unwrap();
        retstr
    }
}
