use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq)]
pub struct ArgList {
    pub args: Vec<VarVal>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Program {
    pub functions: HashMap<String, Function>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Function {
    pub arguments: Vec<Variable>,
    pub name: String,
    pub block: Block,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Variable {
    pub ident: String,
    pub value: VarVal,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum DataType {
    I32,
    BOOL,
    STRING,
    UNIT,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum VarVal {
    I32(Option<i32>),
    BOOL(Option<bool>),
    STRING(Option<String>),
    UNIT,
    ERROR(String),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Block {
    pub statements: Vec<Stmt>,
    pub expr: Box<Expr>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Stmt {
    Expr(Box<Expr>),
    Asgn(String, Box<Expr>),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct If {
    pub condition: Box<Expr>,
    pub if_block: Block,
    pub else_block: Option<Block>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Expr {
    Var(String),
    Value(VarVal),
    Op(Box<Expr>, Opcode, Box<Expr>),
    Function(String, Vec<Box<Expr>>),
    If(If),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Opcode {
    Mul,
    Div,
    Add,
    Sub,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}
