mod novimem;

use novimem::*;
use std::{
    i64,
    io::{stdin, stdout, Write},
};

fn main() {
    let pid = 25831;
    let mut m = NoviMem::new(pid);
    // m.search(b"ELF");
    while let Some(results) = m.search(&100000000i64.to_le_bytes()) {
    // while let Some(results) = m.search(b"ELF") {
        println!("Found {} results.", results.len());
        println!("{:X?}", results);
        if results.len() == 1 {
            print!("Enter the new value (or press ENTER to exit)");
            stdout().flush().unwrap();
            let mut input = String::new();
            match stdin().read_line(&mut input) {
                Ok(n) => match n {
                    0 => break,
                    1 => break,
                    _ => {
                        let val = input[..input.len() - 1].parse::<i64>().unwrap();
                        let addr = results.first().unwrap();
                        if !m.setval(*addr as u64, &val.to_le_bytes()) {
                            println!("Failed to write value {} at addr {:X}", val, addr);
                        }
                    }
                },
                Err(error) => println!("error: {}", error),
            }
            break;
        } else {
            print!("Found {} results", results.len());
            stdout().flush().unwrap();
            break;
        }
    }

    // let mut input = String::new();
    // stdin().read_line(&mut input);
}
