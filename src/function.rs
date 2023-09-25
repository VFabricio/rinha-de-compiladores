use crate::bytecode::Instruction;

use std::collections::HashSet;
use std::fmt;

#[derive(Clone, Debug)]
pub struct Local {
    pub name: String,
}

pub struct Function {
    pub arity: u16,
    pub bytecode: Vec<Instruction>,
    pub captured: HashSet<String>,
    pub index: u16,
    pub locals: Vec<Local>,
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#f")
    }
}
