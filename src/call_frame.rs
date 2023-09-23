use crate::bytecode::Instruction;

#[derive(Debug)]
pub struct CallFrame<'a> {
    pub bytecode: &'a [Instruction],
    pub instruction_pointer: usize,
}
