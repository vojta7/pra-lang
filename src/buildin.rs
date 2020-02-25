use crate::ast::{ArgList, VarVal};
use std::collections::HashMap;

pub fn buildins() -> HashMap<String, Box<dyn Fn(ArgList) -> VarVal>> {
    let mut f: HashMap<String, Box<dyn Fn(ArgList) -> VarVal>> = HashMap::new();
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
