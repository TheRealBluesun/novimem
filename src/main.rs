mod novimem;

use novimem::{proc_search::ProcSearch, NoviMem};
use std::io::{stdin, stdout, Write};
use std::{env, i32};

fn interactive(mem: &mut NoviMem) {
    loop {
        print!("Search value:");
        stdout().flush().unwrap();
        let mut input = String::new();
        match stdin().read_line(&mut input) {
            Ok(n) => {
                if let Ok(search_int) = i32::from_str_radix(&input[..n - 1], 10) {
                    let num_results = mem.search(&search_int.to_le_bytes());
                    println!("Found {} results", num_results);
                    if num_results == 0 {
                        break;
                    } else if num_results <= 10 {
                        mem.print_results();
                    }
                } else {
                    println!("Unable to parse input as i32");
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
                let (name, pid) = &procs[0];
                println!("Using PID {} {}, found {} total", pid, name, procs.len());
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
