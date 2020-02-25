use mylib::{execute, parse};
use std::collections::HashMap;
use std::io::Read;

fn usage() {
    eprintln!("program <file>");
}

fn main() {
    let mut args = std::env::args();
    let file = args.nth(1).unwrap_or_else(|| {
        usage();
        std::process::exit(1)
    });
    let file_path = std::path::Path::new(&file);
    let mut file = std::fs::File::open(file_path).unwrap();
    let mut input = String::new();
    file.read_to_string(&mut input).unwrap();
    let program = parse(&input);
    //println!("{:#?}", program);
    match execute(&program, &mut HashMap::new()) {
        Ok(_) => (),
        Err(e) => eprintln!("Runtime error: {:?}", e),
    }
}
