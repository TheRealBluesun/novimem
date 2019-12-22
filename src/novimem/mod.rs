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

use rayon::prelude::*;

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
    region_key: u64,
    offset: usize,
    address: u64,
}

#[derive(Clone)]
struct SnapShot {
    region_key: u64,
    data: Vec<u8>,
}

impl PartialEq for SnapShot {
    fn eq(&self, other: &Self) -> bool {
        self.region_key == other.region_key
    }
}

impl Eq for SnapShot {}

pub struct NoviMem {
    pid: u32,
    pname: String,
    regions: Vec<MemRegion>,
    searches: HashMap<String, Vec<u64>>,
    results: Vec<u64>,
    values: Vec<u8>,
    snapshots: Vec<SnapShot>,
    memfile: File,
}

pub enum SearchType {
    Changed,
    Unchanged,
}

impl NoviMem {
    pub fn new(pid: u32, pname: String) -> NoviMem {
        let mut m = NoviMem {
            pid,
            pname,
            regions: Vec::new(),
            searches: HashMap::new(),
            results: Vec::new(),
            values: Vec::new(),
            snapshots: Vec::new(),
            memfile: NoviMem::open_mem(pid),
        };
        m.parse_maps();
        m
    }

    pub fn print_modules(&self) {
        self.regions.iter().for_each(|region| {
            println!(
                "{:X}:{:X}\t{}",
                region.start_addr, region.end_addr, region.name
            )
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

    pub fn setval(&mut self, addr: u64, val: &[u8]) {
        self.memfile.seek(SeekFrom::Start(addr)).unwrap();
        if self.memfile.write(val).is_err() {
            println!("Unable to write val at address {:X}", addr);
        }
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

    // pub fn get_region(&self, addr: u64) -> Option<&MemRegion> {
    //     let found: Vec<MemRegion> = self
    //         .regions
    //         .into_iter()
    //         .filter(|region| region.start_addr <= addr && region.end_addr >= addr)
    //         .collect();
    //     if found.len() == 1 {
    //         found.first()
    //     } else {
    //         None
    //     }
    // }

    // pub fn get_region_contents(&mut self, region_key: &str) -> Option<Vec<u8>> {
    //     if let Some(region) = self.regions.get(region_key) {
    //         self.getval(region.start_addr, region.size)
    //     } else {
    //         println!("Unable to find region {}", region_key);
    //         None
    //     }
    // }

    pub fn get_containing_region(&self, addr: u64) -> Option<(u64, &String)> {
        if let Some((addr, name)) = self
            .regions
            .iter()
            .filter_map(|r| {
                if r.start_addr <= addr && r.end_addr >= addr {
                    Some((r.start_addr, &r.name))
                } else {
                    None
                }
            })
            .collect::<Vec<(u64, &String)>>()
            .first()
        {
            Some((*addr, name))
        } else {
            None
        }
    }

    pub fn take_snapshots(&mut self, stype: Option<SearchType>) -> usize {
        // Get the current snapshot of all regions
        let mut snapshots = Vec::<SnapShot>::with_capacity(self.regions.len());
        self.regions.clone().iter().for_each(|r| {
            if let Some(data) = self.getval(r.start_addr, r.size) {
                snapshots.push(SnapShot {
                    region_key: r.start_addr,
                    data: data,
                });
            }
        });
        let mut resvec: Vec<u64> = Vec::new();
        let mut values: Vec<u8> = Vec::new();

        if let Some(t) = stype {
            if !self.snapshots.is_empty() {
                // We have a search type specified and we have a previous snapshot
                // Only retain the snapshots that exist in both
                // snapshots.retain(|s| self.snapshots.contains(s));
                // TODO: This may become more complex in the future
                let should_equal = match t {
                    SearchType::Changed => false,
                    SearchType::Unchanged => true,
                };
                if self.results.is_empty() {
                    // Compare our new snapshot with the existing snapshots
                    // For each snapshot, use our chosen compare method
                    // to decide which addresses to add to our results
                    snapshots.iter().for_each(|s| {
                        // For each byte in this snapshot's data, compare with
                        // the comparable self.snapshot's data
                        if let Some(idx) = self
                            .snapshots
                            .iter()
                            .position(|prev_snap| s.region_key == prev_snap.region_key)
                        {
                            let prev_snap = &self.snapshots.clone()[idx];
                            // Now we have our previous snapshot and our existing snapshot -- let's compare the data
                            // and save off the indeces where they match

                            resvec.extend(
                                prev_snap
                                    .data
                                    .iter()
                                    .zip(&s.data)
                                    .enumerate()
                                    .filter_map(|(i, (a, b))| {
                                        if (*a == *b) == should_equal {
                                            values.push(*a);
                                            Some(i as u64 + s.region_key)
                                        } else {
                                            None
                                        }
                                    })
                                    .collect::<Vec<u64>>(),
                            );
                        } else {
                            println!(
                            "Snapshot contains region not included in existing snapshots...huh?"
                        );
                        }
                    });
                } else {
                    // We have results, search through them instead
                    resvec = self
                        .results()
                        .clone()
                        .iter()
                        .zip(self.values.clone())
                        .enumerate()
                        .filter_map(|(i, (a, v))| {
                            if let Some(val) = self.getval(*a, 1) {
                                if let Some(val) = val.first() {
                                    if (*val == v) == should_equal {
                                        values.push(*val);
                                        Some(*a)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect();
                }
            } else {
                println!("Search type specified, but no snapshot currently exists!");
            }
        }
        self.snapshots = snapshots;
        self.results = resvec;
        self.values = values;
        self.results.len()
    }

    pub fn print_results(&self) {
        self.results.iter().for_each(|result| {
            if let Some((region_addr, region_name)) = self.get_containing_region(*result) {
                println!(
                    "\t{:X} ({:X} + {:X} in {})",
                    result,
                    region_addr,
                    result - region_addr,
                    region_name
                );
            } else {
                println!("\t{:X}", result);
            }
        });
        println!("\t{} results", self.results.len());
    }

    pub fn results(&self) -> &Vec<u64> {
        &self.results
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
                self.regions.iter().for_each(|region| {
                    // Fill the buffer with this module's memory by seeking to the start address first
                    if memfile.seek(SeekFrom::Start(region.start_addr)).is_ok() {
                        let mut reader = BufReader::with_capacity(region.size, &memfile);
                        if let Ok(buf) = reader.fill_buf() {
                            re.find_iter(buf)
                                .for_each(|m| results.push(region.start_addr + m.start() as u64));
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
                })
            } else {
                // Otherwise, only look through our existing results
                let results_cpy = self.results.clone();
                results = results_cpy
                    .iter()
                    .filter_map(|r| {
                        if let Some(read_val) = self.getval(*r, val.len()) {
                            if read_val == val {
                                Some(*r)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();
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
        use regex::RegexBuilder;
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
                BufReader::new(mapsfile).lines().for_each(|line| {
                    if let Ok(resline) = line {
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
                                        format!("{:X}", start)
                                    },
                                };
                                self.regions.push(region);
                            } else {
                                println!("Did not include maps line {}", resline);
                            }
                        } else {
                            println!("Failed to parse {}", &resline)
                        }
                    }
                });
            }
            Err(e) => println!("ERR: Unable to build regex in parse_maps(): {}", e), // We only care about modules that are marked as executable
        }
        self.regions
            .retain(|r| r.readable && r.writeable && r.name != "[stack]");
    }
}
