pub mod ast;
pub mod buildin;

use ast::{ArgList, Block, Expr, Function, Opcode, Program, Stmt, VarVal, Variable};
use lalrpop_util::lalrpop_mod;
use std::collections::HashMap;
use std::io::Read;

lalrpop_mod!(pub calculator1); // synthesized by LALRPOP

fn eval(
    expr: &Expr,
    globals: &mut HashMap<String, Variable>,
    program: &Program,
    locals: &mut HashMap<String, Variable>,
) -> VarVal {
    match expr {
        Expr::Function(name, expr) => {
            let functions = buildin::buildins();
            let arglist = ArgList {
                args: expr
                    .iter()
                    .map(|expr| eval(expr, globals, program, locals))
                    .collect(),
            };
            match functions.get(name) {
                Some(f) => f(arglist),
                None => match program.functions.get(name) {
                    Some(f) => eval_function(&f, arglist, globals, program),
                    None => VarVal::ERROR("Unknown function".to_owned()),
                },
            }
        }
        Expr::Value(n) => n.clone(),
        Expr::Op(lhs, opc, rhs) => {
            let l = eval(lhs, globals, program, locals);
            let r = eval(rhs, globals, program, locals);
            if let (VarVal::I32(Some(l)), VarVal::I32(Some(r))) = (&l, &r) {
                match opc {
                    Opcode::Add => VarVal::I32(Some(l + r)),
                    Opcode::Sub => VarVal::I32(Some(l - r)),
                    Opcode::Mul => VarVal::I32(Some(l * r)),
                    Opcode::Div => VarVal::I32(Some(l / r)),
                    Opcode::Eq => VarVal::BOOL(Some(l == r)),
                    Opcode::Ne => VarVal::BOOL(Some(l != r)),
                    Opcode::Lt => VarVal::BOOL(Some(l < r)),
                    Opcode::Le => VarVal::BOOL(Some(l <= r)),
                    Opcode::Gt => VarVal::BOOL(Some(l > r)),
                    Opcode::Ge => VarVal::BOOL(Some(l >= r)),
                }
            } else if let (VarVal::BOOL(Some(l)), VarVal::BOOL(Some(r))) = (&l, &r) {
                match opc {
                    Opcode::Eq => VarVal::BOOL(Some(l == r)),
                    Opcode::Ne => VarVal::BOOL(Some(l != r)),
                    _ => VarVal::ERROR("invalid opcode".to_owned()),
                }
            } else if let (VarVal::STRING(Some(l)), VarVal::STRING(Some(r))) = (&l, &r) {
                match opc {
                    Opcode::Eq => VarVal::BOOL(Some(l == r)),
                    Opcode::Ne => VarVal::BOOL(Some(l != r)),
                    _ => VarVal::ERROR("invalid opcode".to_owned()),
                }
            } else {
                VarVal::ERROR("invalid operands".to_owned())
            }
        }
        Expr::Var(id) => globals
            .get(id)
            .unwrap_or_else(|| locals.get(id).unwrap())
            .value
            .clone(),
        Expr::If(if_expr) => {
            let predicate = eval(&if_expr.condition, globals, program, locals);
            match predicate {
                VarVal::BOOL(Some(v)) => {
                    if v {
                        eval_block(&if_expr.if_block, globals, program, locals)
                    } else if let Some(else_block) = &if_expr.else_block {
                        eval_block(else_block, globals, program, locals)
                    } else {
                        VarVal::UNIT
                    }
                }
                _ => VarVal::ERROR("Expected boolean".to_owned()),
            }
        }
    }
}

fn eval_block(
    block: &Block,
    globals: &mut HashMap<String, Variable>,
    program: &Program,
    locals: &mut HashMap<String, Variable>,
) -> VarVal {
    for stmt in &block.statements {
        match stmt {
            Stmt::Expr(expr) => {
                eval(&expr, globals, program, locals);
            }
            Stmt::Asgn(id, expr) => {
                let res = eval(&expr, globals, program, locals);
                locals.insert(
                    id.to_string(),
                    Variable {
                        ident: id.to_string(),
                        value: res,
                    },
                );
            }
        };
    }
    eval(&block.expr, globals, program, locals)
}

fn eval_function(
    function: &Function,
    arglist: ArgList,
    globals: &mut HashMap<String, Variable>,
    program: &Program,
) -> VarVal {
    let mut locals = HashMap::new();
    if arglist.args.len() != function.arguments.len() {
        return VarVal::ERROR("Wrong number of arguments".to_owned());
    }
    for (var, arg_value) in function.arguments.iter().zip(arglist.args.iter()) {
        let mut var = var.clone();
        var.value = arg_value.clone();
        locals.insert(var.ident.clone(), var);
    }
    eval_block(&function.block, globals, program, &mut locals)
}

fn execute(program: &Program, globals: &mut HashMap<String, Variable>) {
    eval_function(
        &program.functions["main"],
        ArgList { args: Vec::new() },
        globals,
        program,
    );
}

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
    let program = calculator1::ProgramParser::new().parse(&input).unwrap();
    println!("{:#?}", program);
    execute(&program, &mut HashMap::new());
}
