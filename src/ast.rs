use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct ArgList {
    pub args: Vec<VarVal>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct Program {
    pub functions: HashMap<String, Function>,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub struct Function {
    pub arguments: Vec<Variable>,
    pub name: String,
    pub block: Block,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub struct Variable {
    pub ident: String,
    pub value: VarVal,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub enum DataType {
    I32,
    BOOL,
    STRING,
    UNIT,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub enum VarVal {
    I32(Option<i32>),
    BOOL(Option<bool>),
    STRING(Option<String>),
    UNIT,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub struct Block {
    pub statements: Vec<Stmt>,
    pub expr: Box<Expr>,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub enum Stmt {
    Expr(Box<Expr>),
    Asgn(String, Box<Expr>),
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub struct If {
    pub condition: Box<Expr>,
    pub if_block: Block,
    pub else_block: Option<Block>,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub enum Expr {
    Var(String),
    Value(VarVal),
    Op(Box<Expr>, Opcode, Box<Expr>),
    Function(String, Vec<Box<Expr>>),
    If(If),
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub enum Opcode {
    Mul,
    Div,
    Mod,
    Add,
    Sub,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}
