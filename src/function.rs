use crate::bytecode::Instruction;

use std::fmt;

#[derive(Clone, Debug)]
pub struct Local {
    pub name: String,
}

pub struct Function {
    pub arity: u16,
    pub bytecode: Vec<Instruction>,
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<#f>")
    }
}
