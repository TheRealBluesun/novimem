use std::{
    fs::File,
    fs::OpenOptions,
    io::{self, prelude::*, BufReader, Seek, SeekFrom, Write},
    str, u64,
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
            pid: pid,
            pname: NoviMem::get_pname(pid),
            regions: Vec::new(),
            results: None,
            memfile: NoviMem::open_mem(pid),
        };
        m.parse_maps();
        // for r in m.regions.clone() {
        //     println!("{:X?}", r);
        // }
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
        use regex::bytes::Regex;
        let mut memfile = self.memfile.try_clone().unwrap();
        // let valstr = String::from_utf8_lossy(&val[..]).to_string();
        let valstr = unsafe { String::from_utf8_unchecked(val.to_vec()) };
        // let valstr = r"\x00\xE1\xF5\x05\x00\x00\x00\x00";
        let re = Regex::new(&valstr).unwrap();
        // let re = Regex::new(val)unwrap();

        println!("Searching for {:X?}", val);
        let mut results = Vec::new();

        self.regions.iter().for_each(|region| {
            // Fill the buffer with this module's memory by seeking to the start address first
            memfile.seek(SeekFrom::Start(region.start_addr)).unwrap();
            let mut reader = BufReader::with_capacity(region.size as usize, &memfile);
            let buf = reader.fill_buf().unwrap();

            // let matches = unsafe{ re.find_iter(&str::from_utf8_unchecked(buf)) };
            let search_data = buf.to_vec();
            let matches = re.find_iter(&search_data);

            for m in matches {
                results.push(m.start() + region.start_addr as usize);
                // println!("{:X?}", m.as_bytes());
                println!(
                    "Found result at {:X} ({:X} + {:X}) in '{}'",
                    m.start() + region.start_addr as usize,
                    region.start_addr,
                    m.start(),
                    region.name,
                );
                // memfile.try_clone().unwrap().seek(SeekFrom::Start(region.start_addr + m.start() as u64)).unwrap();
                // memfile.write(&100000000i64.to_le_bytes()).unwrap();
            }
            // Dump this module to a file
            let mut f = File::create(format!(
                "{:X}.{:X}.{:X}.dump",
                region.start_addr, region.size, region.end_addr
            ))
            .unwrap();
            f.write(buf).unwrap();
        });
        if results.len() > 0 {
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
            .expect("Unable to open file")
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
        self.regions.retain(|x| x.execable && x.readable);
    }
}
