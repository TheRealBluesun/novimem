mod novimem;
use novimem::{mem_image::MemImage, proc_search::ProcSearch, NoviMem};
use std::io::{stdin, stdout, Write};
use std::{env, f32, i32, mem::size_of, u8, process};

fn do_search(mem: &mut NoviMem, val: &[u8]) {
    let num_results = mem.search(val);
    println!("Found {} results", num_results);
    if num_results <= 10 {
        mem.print_results();
    }
}

macro_rules! readval {
    ($type: ty, $parsed: ident, $mem: ident) => {
        if let Some(addr_str) = $parsed.pop() {
            if let Ok(addr) = u64::from_str_radix(addr_str, 16) {
                if let Some(val) = $mem.getval(addr, size_of::<u32>()) {
                    // TODO: Is there a cleaner way to do this? (slice to fixed size array)
                    let mut arr = [0u8; size_of::<$type>()];
                    arr.copy_from_slice(&val[..size_of::<$type>()]);
                    println!("{}", <$type>::from_le_bytes(arr));
                }
            }
        }
    };
}

macro_rules! writeval {
    ($type: ty, $parsed: ident, $mem: ident) => {
        if let Some(addr_str) = $parsed.pop() {
            if let Ok(addr) = u64::from_str_radix(addr_str, 16) {
                if let Some(val_str) = $parsed.pop() {
                    if let Ok(val) = val_str.parse::<$type>() {
                        $mem.setval(addr, &val.to_le_bytes());
                    } else {
                        println!("Unable to parse {} as value", val_str);
                    }
                } else {
                    println!("Additional arguments required (value)")
                }
            } else {
                println!("Unable to parse {} as address", addr_str);
            }
        } else {
            println!("Additional arguments required (address)");
        }
    };
}

macro_rules! search {
    ($type: ty, $parsed: ident, $mem: ident) => {
        if let Some(search_str) = $parsed.pop() {
            if let Ok(search_int) = search_str.parse::<$type>() {
                do_search($mem, &search_int.to_le_bytes())
            } else {
                println!("Unable to parse input as u8: '{}'", search_str);
            }
        }
    };
}

fn interactive(mut mem: &mut NoviMem) {
    let mut m_img = MemImage::new();
    loop {
        print!("NM>");
        stdout().flush().unwrap();
        let mut input = String::new();
        match stdin().read_line(&mut input) {
            Ok(n) => {
                // Get the command from the input string
                let mut parsed: Vec<&str> = input[..n - 1].split(' ').collect();
                parsed.reverse();
                if let Some(cmd) = parsed.pop() {
                    match cmd {
                        "c" => mem.clear_results(),
                        "b" => search!(u8, parsed, mem),
                        "s" => search!(i16, parsed, mem),
                        "us" => search!(u16, parsed, mem),
                        "i" => search!(i32, parsed, mem),
                        "u" => search!(u32, parsed, mem),
                        "f" => search!(f32, parsed, mem),
                        "p" => mem.print_results(),
                        "pm" => mem.print_modules(),
                        "save" => {
                            if let Some(name) = parsed.pop() {
                                mem.save_search(name.to_string());
                                mem.save_searches_to_file();
                            }
                        }
                        "restore" => {
                            if let Some(name) = parsed.pop() {
                                if !mem.restore_search(name.to_string()) {
                                    println!("Saved search '{}' not found", &name);
                                }
                            }
                        }
                        "delete" => {
                            if let Some(name) = parsed.pop() {
                                if mem.delete_search(name.to_string()) {
                                    println!("Saved search '{}' deleted", &name);
                                } else {
                                    println!("Saved search '{}' not found", &name);
                                }
                            }
                        }
                        "saved" => mem.print_searches(),
                        // Reading/writing values
                        "ru8" => readval!(u8, parsed, mem),
                        "ri8" => readval!(i8, parsed, mem),
                        "ru16" => readval!(u16, parsed, mem),
                        "ri16" => readval!(i16, parsed, mem),
                        "ri32" => readval!(i32, parsed, mem),
                        "ru32" => readval!(u32, parsed, mem),
                        "wu8" => writeval!(u8, parsed, mem),
                        "wi8" => writeval!(i8, parsed, mem),
                        "wu16" => writeval!(u16, parsed, mem),
                        "wi16" => writeval!(i16, parsed, mem),
                        "wu32" => writeval!(u32, parsed, mem),
                        "wi32" => writeval!(i32, parsed, mem),
                        "wf" => writeval!(f32, parsed, mem),
                        "img" => {
                            if let Some(addr_str) = parsed.pop() {
                                if let Ok(addr) = <u64>::from_str_radix(addr_str, 16) {
                                    if let Some(size_str) = parsed.pop() {
                                        if let Ok(size) = size_str.parse::<usize>() {
                                            m_img.print_img(mem, addr, size);
                                        }
                                    }
                                }
                            }
                        }
                        "x" => {
                            mem.save_searches_to_file();
                            break;
                        }
                        _ => println!("Unknown command {}", cmd),
                    };
                }
            }
            Err(error) => println!("error reading stdin: {}", error),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if let Some(search_str) = args.get(1) {
        if let Some(mut procs) = ProcSearch::search(search_str) {
            procs.retain(|(_, pid)| *pid != process::id());
            if !procs.is_empty() {
                println!("Found {} total", procs.len());
                let (name, pid) = if procs.len() > 1 {
                    for (idx, (name, pid)) in procs.iter().enumerate() {
                        println!("{}:\t{}\t{}", idx, name, pid);
                    }
                    print!("Choose pid:");
                    stdout().flush().unwrap();
                    let mut input = String::new();
                    if let Ok(n) = stdin().read_line(&mut input) {
                        if let Ok(choice) = usize::from_str_radix(&input[..n - 1], 10) {
                            if choice <= procs.len() {
                                &procs[choice]
                            } else {
                                panic!("Chose invalid index");
                            }
                        } else {
                            panic!("Unable to parse input as int");
                        }
                    } else {
                        panic! {"I/O error"};
                    }
                } else {
                    &procs[0]
                };
                // remove null chars
                let name = name.replace('\0', "");
                let mut m = NoviMem::new(*pid, String::from(&name));
                println!("loaded proc {}", &name);
                m.load_searches_from_file();
                interactive(&mut m);
            }
        } else {
            println!("{} not found", search_str);
        }
    } else {
        println!("ERR: Requires process name");
    }
}

#[cfg(test)]
#[allow(dead_code)]
use std::process;
#[test]
fn test_search() {
    let pid = process::id();
    let mut m = NoviMem::new(pid, String::from("novimem"));
    let x = 0xDEADBEEFDEADBEEFDEADBEEFDEADBEEFu128;
    m.search(&x.to_le_bytes());
    m.print_results();

    let old_results = m.results();
    let newval = 0xDEADC0DEDEADC0DEDEADC0DEDEADC0DEu128;
    old_results.iter().for_each(|result| {
        println!("Writing to address {:X}", *result);
        // m.setval(*result, &newval.to_le_bytes());
    });

    m.clear_results();
    m.search(&newval.to_le_bytes());
    m.print_results();

    m.results().iter().for_each(|result| {
        assert!(old_results.contains(result));
    });

    assert!(false);
}
