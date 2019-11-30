mod novimem;

use novimem::{proc_search::ProcSearch, NoviMem};
use std::i32;
use std::io::{stdin, stdout, Write};

fn interactive(mem: &mut NoviMem) {
    let mut input = String::new();
    match stdin().read_line(&mut input) {
        Ok(n) => {
            if let Ok(search_int) = i32::from_str_radix(&input[..n - 1], 10) {
                if let Some(results) = mem.search(&search_int.to_le_bytes()) {
                    println!("Found results at {:X?}", results);
                }
            }
            else {
                println!("Unable to parse input as i32");
            }
        }
        Err(error) => println!("error: {}", error),
    }
}

fn main() {
    let search_str = "calc";
    if let Some(pids) = ProcSearch::search(search_str) {
        match pids.len() {
            1 => {
                let mut m = NoviMem::new(pids[0]);
                interactive(&mut m);
            }
            _ => (),
        }
    } else {
        println!("{} not found", search_str);
    }
}
