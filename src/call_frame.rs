use crate::{bytecode::Instruction, value::Value};

#[derive(Debug)]
pub struct CallFrame<'a> {
    pub bytecode: &'a [Instruction],
    pub instruction_pointer: usize,
    pub closure: Value<'a>,
}
