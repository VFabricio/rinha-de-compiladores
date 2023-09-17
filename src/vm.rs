use anyhow::{bail, Result};

use crate::{ast::Term, bytecode::Instruction, value::Value};

pub struct Vm {
    bytecode: Vec<Instruction>,
    constants: Vec<Value>,
    filename: String,
    stack: Vec<Value>,
    term: Term,
}

impl Vm {
    pub fn new(filename: String, term: Term) -> Self {
        Self {
            bytecode: Vec::new(),
            constants: Vec::new(),
            filename,
            stack: Vec::new(),
            term,
        }
    }

    pub fn interpret(&mut self) -> Result<Value> {
        self.compile();
        let result = self.run()?;
        Ok(result)
    }

    fn compile(&mut self) {
        match self.term {
            Term::Int(i) => {
                let value = Value::Integer(i);
                self.constants.push(value);

                self.bytecode
                    .push(Instruction::Constant(self.constants.len() as u16 - 1));
            }
        };
    }

    fn run(&mut self) -> Result<Value> {
        for instruction in &self.bytecode {
            match *instruction {
                Instruction::Constant(index) => {
                    let value = self.constants[index as usize];
                    self.stack.push(value);
                }
            }
        }

        if self.stack.len() != 1 {
            bail!("At the end of the program the stack should contain only a single item.");
        }
        Ok(self.stack[0])
    }
}
