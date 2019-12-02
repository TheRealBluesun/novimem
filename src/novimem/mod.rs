pub mod mem_image;
pub mod proc_search;

use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::{
    collections::HashMap,
    fs::read,
    fs::File,
    fs::OpenOptions,
    io::{prelude::*, BufReader, Seek, SeekFrom, Write},
    u64,
};

#[derive(Debug, Clone)]
struct MemRegion {
    start_addr: u64,
    end_addr: u64,
    size: usize,
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
        f.write_all(buf).unwrap();
        f.flush().unwrap();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchResult {
    region_key: String,
    offset: usize,
    address: u64,
}

pub struct NoviMem {
    pid: u32,
    pname: String,
    regions: HashMap<String, MemRegion>,
    searches: HashMap<String, Vec<SearchResult>>,
    results: Vec<SearchResult>,
    memfile: File,
}

impl NoviMem {
    pub fn new(pid: u32, pname: String) -> NoviMem {
        let mut m = NoviMem {
            pid,
            pname,
            regions: HashMap::<String, MemRegion>::new(),
            searches: HashMap::<String, Vec<SearchResult>>::new(),
            results: Vec::new(),
            memfile: NoviMem::open_mem(pid),
        };
        m.parse_maps();
        m
    }

    pub fn print_modules(&self) {
        self.regions.iter().for_each(|(name, region)| {
            println!("{:X}:{:X}\t{}", region.start_addr, region.end_addr, name)
        });
    }

    pub fn save_searches_to_file(&self) {
        if !self.searches.is_empty() {
            let json = serde_json::to_string(&self.searches).unwrap();
            // write(format!("{}.searches", self.pname), json).unwrap();
            let fname = format!("./{}.searches", self.pname);
            match File::create(&fname.to_string()) {
                Ok(mut f) => {
                    f.write(json.to_string().as_bytes()).unwrap();
                }
                Err(e) => println!(
                    "Unable to create file '{}' in save_searches_to_file(): {}",
                    &fname, e
                ),
            };
        }
    }

    pub fn load_searches_from_file(&mut self) {
        if let Ok(f) = read(format!("./{}.searches", self.pname)) {
            let json: String = String::from_utf8_lossy(&f).to_string();
            if !json.is_empty() {
                self.searches = serde_json::from_str(&json).unwrap();
            }
        }
    }

    pub fn save_search(&mut self, name: String) {
        self.searches.insert(name, self.results.to_owned());
        self.results.clear();
        self.save_searches_to_file()
    }

    pub fn restore_search(&mut self, name: String) -> bool {
        if let Some(result) = self.searches.get(&name) {
            self.results = result.to_vec();
            true
        } else {
            false
        }
    }

    pub fn delete_search(&mut self, name: String) -> bool {
        if self.searches.remove(&name).is_some() {
            true
        } else {
            false
        }
    }

    pub fn clear_results(&mut self) {
        self.results.clear();
    }

    pub fn print_searches(&self) {
        self.searches.keys().for_each(|name| println!("\t{}", name));
    }

    pub fn setval(&mut self, addr: u64, val: &[u8]) -> bool {
        self.memfile.seek(SeekFrom::Start(addr)).unwrap();
        self.memfile.write(val).unwrap() == val.len()
    }

    pub fn getval(&mut self, addr: u64, size: usize) -> Option<Vec<u8>> {
        if self.memfile.seek(SeekFrom::Start(addr)).is_ok() {
            let mut reader = BufReader::with_capacity(size as usize, &self.memfile);
            match reader.fill_buf() {
                Ok(buf) => Some(buf.to_vec()),
                Err(e) => {
                    println!(
                        "Unable to fill buffer in getval() at address {:X} with size {}: {}",
                        addr, size, e
                    );
                    None
                }
            }
        } else {
            println!("Unable to seek to address {:X} in getval()", addr);
            None
        }
    }

    pub fn get_region_contents(&mut self, region_key: &str) -> Option<Vec<u8>> {
        if let Some(region) = self.regions.get(region_key) {
            self.getval(region.start_addr, region.size)
        } else {
            println!("Unable to find region {}", region_key);
            None
        }
    }

    pub fn print_results(&self) {
        self.results.iter().for_each(|result| {
            if let Some(region) = self.regions.get(&result.region_key) {
                println!(
                    "\t{:X} ({} + {:X})",
                    result.address, region.name, result.offset
                );
            }
        });
        println!("\t{} results", self.results.len());
    }

    pub fn search(&mut self, val: &[u8]) -> usize {
        // Explicitly use the bytes regex
        use regex::bytes::RegexBuilder;
        let mut memfile = self.memfile.try_clone().unwrap();
        let mut valstr = String::new();
        val.iter()
            .for_each(|b| valstr.push_str(&format!("\\x{:02x}", b).to_string()));
        println!("Searching for {}", valstr);
        let mut builder = RegexBuilder::new(&valstr.to_string());
        builder
            .unicode(false)
            .dot_matches_new_line(true)
            .case_insensitive(false);
        if let Ok(re) = builder.build() {
            let mut results = Vec::new();
            // If this is a new search, look through everything
            if self.results.is_empty() {
                for (key, region) in self.regions.iter() {
                    // Fill the buffer with this module's memory by seeking to the start address first
                    if memfile.seek(SeekFrom::Start(region.start_addr)).is_ok() {
                        let mut reader = BufReader::with_capacity(region.size, &memfile);
                        if let Ok(buf) = reader.fill_buf() {
                            re.find_iter(buf).for_each(|m| {
                                results.push(SearchResult {
                                    region_key: key.to_string(),
                                    offset: m.start(),
                                    address: region.start_addr + m.start() as u64,
                                })
                            });
                        } else {
                            println!(
                                "Unable to fill buffer from memory region {} :-(",
                                region.name
                            );
                        }
                    } else {
                        println!(
                            "Unable to seek to region {} start address {:X}",
                            region.name, region.start_addr
                        );
                    }
                }
            } else {
                // Otherwise, only look through our existing results
                let results_cpy = self.results.clone();
                results_cpy.iter().for_each(|search_result| {
                    if let Some(read_val) = self.getval(search_result.address, val.len()) {
                        if read_val == val {
                            results.push(search_result.to_owned());
                        }
                    }
                })
            }
            self.results = results;
        } else {
            println!("Unable to build search :-(");
        }
        self.results.len()
    }

    fn open_mem(pid: u32) -> File {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(false)
            .open(format!("/proc/{}/mem", pid))
            .unwrap_or_else(|_| panic!("Unable to open memory for pid {}", pid))
    }

    fn parse_maps(&mut self) {
        use regex::{Regex, RegexBuilder};
        let mapsfile = OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(format!("/proc/{}/maps", self.pid))
            .expect("Unable to open file");
        let regex_str = 
            //address 1,2                    perms 3,4,5,6            offset           dev                           inode     pathname 7
            r"([0-9A-Fa-f]+)-([0-9A-Fa-f]+) ([-r])([-w])([-x])([-ps]) (?:[0-9A-Fa-f]+) (?:[0-9A-Fa-f]+:[0-9A-Fa-f]+) (?:\d+)\s+(.*)?";
        let mut builder = RegexBuilder::new(regex_str);
        builder
            .unicode(true)
            .dot_matches_new_line(true)
            .case_insensitive(false);
        // Parse the maps file to find regions of interest
        match builder.build() {
            Ok(re) => {
            for line in BufReader::new(mapsfile).lines() {
                let resline = line.unwrap();
                if let Some(cap) = re.captures(resline.as_str()) {
                    if cap.len() > 0 {
                        let start = u64::from_str_radix(&cap[1], 16).unwrap();
                        let end = u64::from_str_radix(&cap[2], 16).unwrap();
                        let region = MemRegion {
                            start_addr: start,
                            end_addr: end,
                            size: (end - start) as usize,
                            readable: &cap[3] == "r",
                            writeable: &cap[4] == "w",
                            execable: &cap[5] == "x",
                            private: &cap[6] == "p",
                            shared: &cap[6] == "s",
                            name: if let Some(n) = cap.get(7) {
                                let name = n.as_str().to_string();
                                if name.is_empty() {
                                    format!("{:X}", start)
                                } else {
                                    name.replace('\0', "")
                                }
                            } else {
                                println!("Failed to capture module name {}", resline);
                                String::from("[anon]")
                            },
                        };
                        let name_key = region.name.clone();
                        if self.regions.insert(region.name.to_string(), region).is_some() {
                            println!("Inserted duplicate region {}", name_key);
                        }
                    } else {
                        println!("Did not include maps line {}", resline);
                    }
                    println!("'{}'\thas {} matches", resline, cap.len());
                } else {
                    println!("Failed to parse {}", &resline)
                }
            }
        }
        Err(e) =>   println!("ERR: Unable to build regex in parse_maps(): {}", e)
        
        // We only care about modules that are marked as executable
    }
    self.regions
        .retain(|_, r| r.readable && r.writeable);
    }
}
