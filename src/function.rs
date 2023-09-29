use crate::bytecode::Instruction;

use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct Local {
    pub name: String,
}

#[derive(Debug)]
pub struct Function {
    pub arity: u16,
    pub bytecode: Vec<Instruction>,
    pub captured: HashSet<String>,
    pub index: u16,
    pub locals: Vec<Local>,
}
