mod novimem;

use novimem::*;
use std::io::*;

fn main() {
    let pid = 25831;
    let m = NoviMem::new(pid);
    // m.search(b"ELF");
    let results = m.search(&95474000i64.to_le_bytes());
    println!("Found {} results.", results.len());
    println!("{:X?}", results);

    // let mut input = String::new();
    // stdin().read_line(&mut input);
}
