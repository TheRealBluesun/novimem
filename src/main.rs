mod novimem;

use novimem::{proc_search::ProcSearch, NoviMem};
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

fn print_img(mem: &mut NoviMem) {

}

fn interactive(mut mem: &mut NoviMem) {
    loop {
        print!("NM>");
        stdout().flush().unwrap();
        let mut input = String::new();
        match stdin().read_line(&mut input) {
            Ok(n) => {
                // Get the command from the input string
                let mut parsed: Vec<&str> = input[..n - 1].split(" ").collect();
                parsed.reverse();
                if let Some(cmd) = parsed.pop() {
                    match cmd {
                        "i" => do_searchi32(&mut mem, &mut parsed),
                        "b" => do_searchu8(&mut mem, &mut parsed),
                        "p" => print_img(mem),
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
