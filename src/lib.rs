pub mod ast;
pub mod buildin;

use ast::{ArgList, Block, Expr, Function, Opcode, Program, Stmt, VarVal, Variable};
use lalrpop_util::lalrpop_mod;
use std::collections::HashMap;

lalrpop_mod!(pub calculator1); // synthesized by LALRPOP

#[derive(Debug)]
pub enum RuntimeError {
    UndefinedVariable,
    UndefinedFunction(String),
    InvalidOpcode,
    InvalidOperands,
    BooleanExpected,
    WrongNumberOfArguments,
}

fn eval(
    expr: &Expr,
    globals: &mut HashMap<String, Variable>,
    program: &Program,
    locals: &mut HashMap<String, Variable>,
) -> Result<VarVal, RuntimeError> {
    match expr {
        Expr::Function(name, expr) => {
            let functions = buildin::buildins();
            let arglist = ArgList {
                args: expr
                    .iter()
                    .map(|expr| eval(expr, globals, program, locals))
                    .collect::<Result<_, _>>()?,
            };
            match functions.get(name) {
                Some(f) => Ok(f(arglist)),
                None => match program.functions.get(name) {
                    Some(f) => eval_function(&f, arglist, globals, program),
                    None => Err(RuntimeError::UndefinedFunction(name.clone())),
                },
            }
        }
        Expr::Value(n) => Ok(n.clone()),
        Expr::Op(lhs, opc, rhs) => {
            let l = eval(lhs, globals, program, locals)?;
            let r = eval(rhs, globals, program, locals)?;
            if let (VarVal::I32(Some(l)), VarVal::I32(Some(r))) = (&l, &r) {
                Ok(match opc {
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
                })
            } else if let (VarVal::BOOL(Some(l)), VarVal::BOOL(Some(r))) = (&l, &r) {
                match opc {
                    Opcode::Eq => Ok(VarVal::BOOL(Some(l == r))),
                    Opcode::Ne => Ok(VarVal::BOOL(Some(l != r))),
                    _ => Err(RuntimeError::InvalidOpcode),
                }
            } else if let (VarVal::STRING(Some(l)), VarVal::STRING(Some(r))) = (&l, &r) {
                match opc {
                    Opcode::Eq => Ok(VarVal::BOOL(Some(l == r))),
                    Opcode::Ne => Ok(VarVal::BOOL(Some(l != r))),
                    _ => Err(RuntimeError::InvalidOpcode),
                }
            } else {
                Err(RuntimeError::InvalidOperands)
            }
        }
        Expr::Var(id) => globals
            .get(id)
            .map(|v| Ok(v))
            .unwrap_or_else(|| {
                locals
                    .get(id)
                    .map_or_else(|| Err(RuntimeError::UndefinedVariable), |v| Ok(v))
            })
            .map(|v| v.value.clone()),
        Expr::If(if_expr) => {
            let predicate = eval(&if_expr.condition, globals, program, locals)?;
            match predicate {
                VarVal::BOOL(Some(v)) => {
                    if v {
                        eval_block(&if_expr.if_block, globals, program, locals)
                    } else if let Some(else_block) = &if_expr.else_block {
                        eval_block(else_block, globals, program, locals)
                    } else {
                        Ok(VarVal::UNIT)
                    }
                }
                _ => Err(RuntimeError::BooleanExpected),
            }
        }
    }
}

fn eval_block(
    block: &Block,
    globals: &mut HashMap<String, Variable>,
    program: &Program,
    locals: &mut HashMap<String, Variable>,
) -> Result<VarVal, RuntimeError> {
    for stmt in &block.statements {
        match stmt {
            Stmt::Expr(expr) => {
                eval(&expr, globals, program, locals)?;
            }
            Stmt::Asgn(id, expr) => {
                let res = eval(&expr, globals, program, locals)?;
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
) -> Result<VarVal, RuntimeError> {
    let mut locals = HashMap::new();
    if arglist.args.len() != function.arguments.len() {
        return Err(RuntimeError::WrongNumberOfArguments);
    }
    for (var, arg_value) in function.arguments.iter().zip(arglist.args.iter()) {
        let mut var = var.clone();
        var.value = arg_value.clone();
        locals.insert(var.ident.clone(), var);
    }
    eval_block(&function.block, globals, program, &mut locals)
}

pub fn execute(
    program: &Program,
    globals: &mut HashMap<String, Variable>,
) -> Result<VarVal, RuntimeError> {
    eval_function(
        &program.functions["main"],
        ArgList { args: Vec::new() },
        globals,
        program,
    )
}

pub fn parse(input: &str) -> Program {
    calculator1::ProgramParser::new().parse(&input).unwrap() // TODO remove unwrap
}
