use crate::bytecode::Instruction;

#[derive(Debug)]
pub struct CallFrame {
    pub bytecode: Vec<Instruction>,
    pub instruction_pointer: usize,
}
