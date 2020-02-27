use mylib::ast::{ArgList, VarVal};
use mylib::{execute, parse, Buildins};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn buildins() -> Buildins<'static> {
    let mut f: Buildins = HashMap::new();
    f.insert(
        "print".to_owned(),
        Box::from(|args: ArgList| {
            for arg in args.args {
                match arg {
                    VarVal::I32(Some(v)) => print!("{}", v),
                    VarVal::BOOL(Some(v)) => print!("{}", v),
                    VarVal::STRING(Some(v)) => print!("{}", v),
                    VarVal::UNIT => print!("()"),
                    _ => (),
                }
            }
            println!();
            VarVal::UNIT
        }),
    );
    f
}

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
    //let res = load_program(&file_path)
    //    .map(|program| parse(&program).map(|ast| execute(&ast, &mut HashMap::new())));
    //if let Err(e) = res {
    //    eprintln!("{:#?}", e);
    //}
    match load_program(&file_path) {
        Ok(input) => {
            match parse(&input) {
                Ok(program) => {
                    //println!("{:#?}", program);
                    match execute(&program, &mut HashMap::new(), &mut buildins()) {
                        Ok(_) => (),
                        Err(e) => eprintln!("Runtime error: {:#?}", e),
                    }
                }
                Err(e) => eprintln!("Runtime error: {:#?}", e),
            }
        }
        Err(e) => eprintln!("OS error: {:#?}", e),
    }
}
