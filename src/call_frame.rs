use crate::{bytecode::Instruction, value::Value};
use std::rc::Rc;

#[derive(Debug)]
pub struct CallFrame<'a> {
    pub bytecode: &'a [Instruction],
    pub closure: Rc<Value<'a>>,
    pub instruction_pointer: usize,
    pub frame_index: usize,
}
