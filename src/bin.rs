use mylib::{execute, parse};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn usage() {
    eprintln!("program <file>");
}

fn load_program(file_path: &Path) -> Result<String, std::io::Error> {
    let mut file = File::open(file_path)?;
    let mut input = String::new();
    file.read_to_string(&mut input)?;
    Ok(input)
}

fn main() {
    let mut args = std::env::args();
    let file = args.nth(1).unwrap_or_else(|| {
        usage();
        std::process::exit(1)
    });
    let file_path = Path::new(&file);
    match load_program(&file_path) {
        Ok(input) => {
            let program = parse(&input);
            //println!("{:#?}", program);
            match execute(&program, &mut HashMap::new()) {
                Ok(_) => (),
                Err(e) => eprintln!("Runtime error: {:?}", e),
            }
        }
        Err(e) => eprintln!("OS error: {:?}", e),
    }
}
