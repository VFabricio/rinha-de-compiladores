use crate::{bytecode::Instruction, value::Value};

#[derive(Debug)]
pub struct CallFrame<'a> {
    pub bytecode: &'a [Instruction],
    pub closure: Value<'a>,
    pub instruction_pointer: usize,
    pub frame_index: usize,
}
