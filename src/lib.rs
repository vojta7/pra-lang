pub mod ast;
mod lexer;

pub use ast::{
    ArgList, Block, Else, Expr, ExprType, Function, If, Opcode, Program, Stmt, VarVal, Variable,
};
use lalrpop_util::{lalrpop_mod, ParseError};
pub use lexer::{Error as LexerError, Lexer, Token};
use serde::Serialize;
use std::collections::HashMap;
use std::fmt;

lalrpop_mod!(pub parser); // synthesized by LALRPOP

#[derive(Debug, Serialize)]
pub struct RuntimeError {
    pub position: usize,
    pub error_type: RuntimeErrorType,
}
#[derive(Debug, Serialize)]
pub enum RuntimeErrorType {
    UndefinedVariable(String),
    UndefinedFunction(String),
    InvalidOpcode,
    InvalidOperands,
    BooleanExpected,
    WrongNumberOfArguments(String),
    NoMain,
}

impl fmt::Display for RuntimeErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            RuntimeErrorType::UndefinedFunction(name) => write!(f, "Undefined function {}", name),
            RuntimeErrorType::UndefinedVariable(name) => write!(f, "Undefined variable {}", name),
            RuntimeErrorType::WrongNumberOfArguments(name) => {
                write!(f, "Wrong number of arguments {}", name)
            }
            RuntimeErrorType::InvalidOperands => write!(f, "Invalid operands"),
            RuntimeErrorType::InvalidOpcode => write!(f, "Invalid opcode"),
            RuntimeErrorType::BooleanExpected => write!(f, "Expected Boolean value"),
            RuntimeErrorType::NoMain => write!(f, "Function main was't found"),
        }
    }
}

fn error(error_type: RuntimeErrorType, position: usize) -> RuntimeError {
    RuntimeError {
        error_type,
        position,
    }
}

pub type Buildins<'a> = HashMap<String, Box<dyn FnMut(ArgList) -> VarVal + 'a>>;

fn eval(
    expr: &Expr,
    globals: &mut HashMap<String, Variable>,
    program: &Program,
    locals: &mut HashMap<String, Variable>,
    buildins: &mut Buildins,
) -> Result<VarVal, RuntimeError> {
    match &expr.expression_type {
        ExprType::Function(name, expr_list) => {
            let arglist = ArgList {
                args: expr_list
                    .iter()
                    .map(|expr| eval(expr, globals, program, locals, buildins))
                    .collect::<Result<_, _>>()?,
            };
            if let Some(f) = buildins.get_mut(name) {
                Ok(f(arglist))
            } else {
                match program.functions.get(name) {
                    Some(f) => eval_function(&f, arglist, globals, program, buildins),
                    None => Err(error(
                        RuntimeErrorType::UndefinedFunction(name.clone()),
                        expr.position,
                    )),
                }
            }
        }
        ExprType::Value(n) => Ok(n.clone()),
        ExprType::Op(lhs, opc, rhs) => {
            let l = eval(&lhs, globals, program, locals, buildins)?;
            let r = eval(&rhs, globals, program, locals, buildins)?;
            if let (VarVal::I32(Some(l)), VarVal::I32(Some(r))) = (&l, &r) {
                match opc {
                    Opcode::Add => Ok(VarVal::I32(Some(l + r))),
                    Opcode::Sub => Ok(VarVal::I32(Some(l - r))),
                    Opcode::Mul => Ok(VarVal::I32(Some(l * r))),
                    Opcode::Div => Ok(VarVal::I32(Some(l / r))),
                    Opcode::Mod => Ok(VarVal::I32(Some(l % r))),
                    Opcode::Eq => Ok(VarVal::BOOL(Some(l == r))),
                    Opcode::Ne => Ok(VarVal::BOOL(Some(l != r))),
                    Opcode::Lt => Ok(VarVal::BOOL(Some(l < r))),
                    Opcode::Le => Ok(VarVal::BOOL(Some(l <= r))),
                    Opcode::Gt => Ok(VarVal::BOOL(Some(l > r))),
                    Opcode::Ge => Ok(VarVal::BOOL(Some(l >= r))),
                    _ => Err(error(RuntimeErrorType::InvalidOpcode, expr.position)),
                }
            } else if let (VarVal::BOOL(Some(l)), VarVal::BOOL(Some(r))) = (&l, &r) {
                match opc {
                    Opcode::Eq => Ok(VarVal::BOOL(Some(l == r))),
                    Opcode::Ne => Ok(VarVal::BOOL(Some(l != r))),
                    Opcode::And => Ok(VarVal::BOOL(Some(*l && *r))),
                    Opcode::Or => Ok(VarVal::BOOL(Some(*l || *r))),
                    _ => Err(error(RuntimeErrorType::InvalidOpcode, expr.position)),
                }
            } else if let (VarVal::STRING(Some(l)), VarVal::STRING(Some(r))) = (&l, &r) {
                match opc {
                    Opcode::Eq => Ok(VarVal::BOOL(Some(l == r))),
                    Opcode::Ne => Ok(VarVal::BOOL(Some(l != r))),
                    _ => Err(error(RuntimeErrorType::InvalidOpcode, expr.position)),
                }
            } else {
                Err(error(RuntimeErrorType::InvalidOperands, expr.position))
            }
        }
        ExprType::Var(id) => globals
            .get(id)
            .map(|v| Ok(v))
            .unwrap_or_else(|| {
                locals.get(id).map_or_else(
                    || {
                        Err(error(
                            RuntimeErrorType::UndefinedVariable(id.clone()),
                            expr.position,
                        ))
                    },
                    |v| Ok(v),
                )
            })
            .map(|v| v.value.clone()),
        ExprType::If(if_expr) => {
            eval_if(if_expr, globals, program, locals, buildins, expr.position)
        }
    }
}

fn eval_if(
    if_expr: &If,
    globals: &mut HashMap<String, Variable>,
    program: &Program,
    locals: &mut HashMap<String, Variable>,
    buildins: &mut Buildins,
    position: usize,
) -> Result<VarVal, RuntimeError> {
    let predicate = eval(&if_expr.condition, globals, program, locals, buildins)?;
    match predicate {
        VarVal::BOOL(Some(v)) => {
            if v {
                eval_block(&if_expr.if_block, globals, program, locals, buildins)
            } else {
                match &if_expr.else_part {
                    Else::Else(block) => eval_block(block, globals, program, locals, buildins),
                    Else::ElseIf(next_if) => {
                        eval_if(&**next_if, globals, program, locals, buildins, position)
                    }
                    Else::None => Ok(VarVal::UNIT),
                }
            }
        }
        _ => Err(error(RuntimeErrorType::BooleanExpected, position)),
    }
}

fn eval_block(
    block: &Block,
    globals: &mut HashMap<String, Variable>,
    program: &Program,
    locals: &mut HashMap<String, Variable>,
    buildins: &mut Buildins,
) -> Result<VarVal, RuntimeError> {
    for stmt in &block.statements {
        match stmt {
            Stmt::Expr(expr) => {
                eval(&expr, globals, program, locals, buildins)?;
            }
            Stmt::Asgn(id, expr) => {
                let res = eval(&expr, globals, program, locals, buildins)?;
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
    eval(&block.expr, globals, program, locals, buildins)
}

fn eval_function(
    function: &Function,
    arglist: ArgList,
    globals: &mut HashMap<String, Variable>,
    program: &Program,
    buildins: &mut Buildins,
) -> Result<VarVal, RuntimeError> {
    let mut locals = HashMap::new();
    if arglist.args.len() != function.arguments.len() {
        return Err(error(
            RuntimeErrorType::WrongNumberOfArguments(function.name.clone()),
            function.position,
        ));
    }
    for (var, arg_value) in function.arguments.iter().zip(arglist.args.iter()) {
        let mut var = var.clone();
        var.value = arg_value.clone();
        locals.insert(var.ident.clone(), var);
    }
    eval_block(&function.block, globals, program, &mut locals, buildins)
}

pub fn execute(
    program: &Program,
    globals: &mut HashMap<String, Variable>,
    buildins: &mut Buildins,
) -> Result<VarVal, RuntimeError> {
    if let Some(main) = program.functions.get("main") {
        eval_function(
            main,
            ArgList { args: Vec::new() },
            globals,
            program,
            buildins,
        )
    } else {
        Err(error(RuntimeErrorType::NoMain, 0))
    }
}

#[derive(Debug, Serialize)]
pub struct ParsingError {
    pub from: usize,
    pub to: usize,
    pub description: String,
}

fn parsing_err(from: usize, to: usize, description: String) -> ParsingError {
    ParsingError {
        from,
        to,
        description,
    }
}

pub fn parse(input: &str) -> Result<Program, ParsingError> {
    let lexer = lexer::Lexer::new(input);
    parser::ProgramParser::new()
        .parse(&input, lexer)
        .map_err(|e| match e {
            ParseError::User { error } => parsing_err(
                error.location,
                error.location + 1,
                format!("Unexpected character {}", error.char.unwrap_or(' ')),
            ),
            ParseError::InvalidToken { location } => {
                parsing_err(location, location, "invalid token".to_string())
            }
            ParseError::UnrecognizedToken {
                token: (l, token, r),
                expected,
            } => parsing_err(
                l,
                r,
                format!(
                    "unexpected token {:?}, expected {}",
                    token,
                    expected.join(",")
                ),
            ),
            ParseError::ExtraToken {
                token: (l, token, r),
            } => parsing_err(l, r, format!("extra token '{:?}' encountered", token)),
            ParseError::UnrecognizedEOF { location, expected } => parsing_err(
                location,
                location,
                format!("unexpected end of file, expecting {}", expected.join(", ")),
            ),
        })
}
