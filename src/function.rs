use crate::bytecode::Instruction;

#[derive(Clone, Debug)]
pub struct Local {
    pub name: String,
}

#[derive(Debug)]
pub struct Function {
    pub arity: u16,
    pub bytecode: Vec<Instruction>,
}
