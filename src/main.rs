mod novimem;

use novimem::*;
use std::io::{stdin, stdout, Write};

fn main() {
    let pid = 25831;
    let mut m = NoviMem::new(pid);
    if let Some(results) = m.search(&100000000u64.to_le_bytes()) {
        println!("Found {} results.", results.len());
        println!("{:X?}", results);
        if results.len() == 1 {
            print!("Enter the new value (or press ENTER to exit)");
            stdout().flush().unwrap();
            let mut input = String::new();
            match stdin().read_line(&mut input) {
                Ok(n) => match n {
                    0 => (),
                    1 => (),
                    _ => {
                        let val = input[..input.len() - 1].parse::<u64>().unwrap();
                        let addr = results.first().unwrap();
                        if !m.setval(*addr as u64, &val.to_le_bytes()) {
                            println!("Failed to write value {} at addr {:X}", val, addr);
                        }
                    }
                },
                Err(error) => println!("error: {}", error),
            }
        } else {
            print!("Found {} results", results.len());
            stdout().flush().unwrap();
        }
    }

    // let mut input = String::new();
    // stdin().read_line(&mut input);
}
