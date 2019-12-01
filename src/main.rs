mod novimem;

use novimem::{mem_image::MemImage, proc_search::ProcSearch, NoviMem};
use std::io::{stdin, stdout, Write};
use std::{env, i32, u8};

fn do_search(mem: &mut NoviMem, val: &[u8]) {
    let num_results = mem.search(val);
    println!("Found {} results", num_results);
    if num_results <= 10 {
        mem.print_results();
    }
}

fn do_searchi32(mem: &mut NoviMem, cmd: &mut Vec<&str>) {
    if let Some(search_str) = cmd.pop() {
        if let Ok(search_int) = i32::from_str_radix(search_str, 10) {
            do_search(mem, &search_int.to_le_bytes())
        } else {
            println!("Unable to parse input as u8: '{}'", search_str);
        }
    }
}

fn do_searchu8(mem: &mut NoviMem, cmd: &mut Vec<&str>) {
    if let Some(search_str) = cmd.pop() {
        if let Ok(search_int) = u8::from_str_radix(search_str, 10) {
            do_search(mem, &search_int.to_le_bytes())
        } else {
            println!("Unable to parse input as i32: '{}'", search_str);
        }
    }
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
                        "i" => do_searchi32(&mut mem, &mut parsed),
                        "b" => do_searchu8(&mut mem, &mut parsed),
                        "p" => mem.print_results(),
                        "ru8" => {
                            if let Some(addr_str) = parsed.pop() {
                                if let Ok(addr) = u64::from_str_radix(addr_str, 16) {
                                    if let Some(val) = mem.getval(addr, 1) {
                                        println!("{}", u8::from_le_bytes([val[0]]));
                                    }
                                }
                            }
                        }
                        "wu8" => {
                            if let Some(addr_str) = parsed.pop() {
                                if let Ok(addr) = u64::from_str_radix(addr_str, 16) {
                                    if let Some(val_str) = parsed.pop() {
                                        if let Ok(val) = u8::from_str_radix(val_str, 10) {
                                            mem.setval(addr, &[val]);
                                        }
                                    }
                                }
                            }
                        }
                        "img" => m_img.print_img(mem),
                        "x" => break,
                        _ => println!("Unknown command {}", cmd),
                    };
                }
            }
            Err(error) => println!("error: {}", error),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if let Some(search_str) = args.get(1) {
        if let Some(procs) = ProcSearch::search(search_str) {
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
                let mut m = NoviMem::new(*pid);
                interactive(&mut m);
            }
        } else {
            println!("{} not found", search_str);
        }
    } else {
        println!("ERR: Requires process name");
    }
}
