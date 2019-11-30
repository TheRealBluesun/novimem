pub mod proc_search;

use std::{
    fs::File,
    fs::OpenOptions,
    io::{prelude::*, BufReader, Seek, SeekFrom, Write},
    u64,
};

#[derive(Debug, Clone)]
struct MemRegion {
    start_addr: u64,
    end_addr: u64,
    size: u64,
    readable: bool,
    writeable: bool,
    execable: bool,
    private: bool,
    shared: bool,
    name: String,
}

impl MemRegion {
    pub fn dump_to_file(&self, buf: &[u8]) {
        let mut f = File::create(format!(
            "{:X}.{:X}.{:X}.dump",
            self.start_addr, self.size, self.end_addr
        ))
        .unwrap();
        f.write(buf).unwrap();
        f.flush().unwrap();
    }
}

pub struct NoviMem {
    pid: u32,
    pname: String,
    regions: Vec<MemRegion>,
    results: Option<Vec<usize>>,
    memfile: File,
}

impl NoviMem {
    pub fn new(pid: u32) -> NoviMem {
        let mut m = NoviMem {
            pid,
            pname: String::new(), //NoviMem::get_pname(pid),
            regions: Vec::new(),
            results: None,
            memfile: NoviMem::open_mem(pid),
        };
        m.parse_maps();
        m
    }

    // pub fn search_num<T>(&self, val: T) -> Vec<usize> where T: Bytes {
    //     self.search(&val.to_le_bytes())
    // }

    pub fn setval(&mut self, addr: u64, val: &[u8]) -> bool {
        self.memfile.seek(SeekFrom::Start(addr)).unwrap();
        self.memfile.write(val).unwrap() == val.len()
    }

    pub fn search(&self, val: &[u8]) -> Option<Vec<usize>> {
        use regex::bytes::RegexBuilder;
        let mut memfile = self.memfile.try_clone().unwrap();
        let mut valstr = String::new();
        for b in val {
            valstr.push_str(&format!("\\x{:02x}", b).to_string());
        }

        let mut builder = RegexBuilder::new(&valstr.to_string());
        builder.unicode(false);
        builder.dot_matches_new_line(true);
        builder.case_insensitive(false);

        let re = builder.build().unwrap();

        println!("Searching for {}", valstr);
        let mut results = Vec::new();

        self.regions.iter().for_each(|region| {
            // Fill the buffer with this module's memory by seeking to the start address first
            memfile.seek(SeekFrom::Start(region.start_addr)).unwrap();
            let mut reader = BufReader::with_capacity(region.size as usize, &memfile);
            let buf = reader.fill_buf().unwrap();
            let search_data = buf.to_vec();
            let matches = re.find_iter(&search_data);

            for m in matches {
                results.push(m.start() + region.start_addr as usize);
                println!(
                    "Found result at {:X} ({:X} + {:X}) in '{}'",
                    m.start() + region.start_addr as usize,
                    region.start_addr,
                    m.start(),
                    region.name,
                );
            }
        });
        if !results.is_empty() {
            Some(results)
        } else {
            None
        }
    }

    fn open_mem(pid: u32) -> File {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(false)
            .open(format!("/proc/{}/mem", pid))
            // .open("/proc/self/mem")
            .expect(&format!("Unable to open memory for pid {}", pid))
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

    fn parse_maps(&mut self) {
        use regex::Regex;
        let mapsfile = OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(format!("/proc/{}/maps", self.pid))
            // .open("/proc/self/maps")
            .expect("Unable to open file");

        // Parse the maps file to find regions of interest
        let re = Regex::new(
        //address                       perms                     offset           dev                           inode     pathname
        r"([0-9A-Fa-f]+)-([0-9A-Fa-f]+) ([-r])([-w])([-x])([-ps]) (?:[0-9A-Fa-f]+) (?:[0-9A-Fa-f]+:[0-9A-Fa-f]+) (?:\d*)\s+(.+$)?")
    .unwrap();
        for line in BufReader::new(mapsfile).lines() {
            let resline = line.unwrap();
            // println!("{}", &resline );
            if let Some(cap) = re.captures(resline.as_str()) {
                if cap.len() > 0 {
                    // println!("{:?}", &cap);
                    let start = u64::from_str_radix(&cap[1], 16).unwrap();
                    let end = u64::from_str_radix(&cap[2], 16).unwrap();
                    let region = MemRegion {
                        start_addr: start,
                        end_addr: end,
                        size: end - start,
                        readable: &cap[3] == "r",
                        writeable: &cap[4] == "w",
                        execable: &cap[5] == "x",
                        private: &cap[6] == "p",
                        shared: &cap[6] == "s",
                        name: if let Some(n) = cap.get(7) {
                            n.as_str().to_string()
                        } else {
                            String::from("")
                        },
                    };
                    self.regions.push(region);
                }
            } else {
                println!("Failed to parse {}", &resline)
            }
        }
        // We only care about modules that are marked as executable
        self.regions.retain(|x| x.readable && x.writeable);
    }
}
