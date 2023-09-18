use anyhow::{anyhow, bail, Result};

use crate::{
    ast::{Operation, Term},
    bytecode::Instruction,
    value::Value,
};

pub struct Vm {
    bytecode: Vec<Instruction>,
    constants: Vec<Value>,
    stack: Vec<Value>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            bytecode: Vec::new(),
            constants: Vec::new(),
            stack: Vec::new(),
        }
    }

    pub fn interpret(&mut self, filename: String, term: Term) -> Result<Value> {
        self.compile(term);
        let result = self.run()?;
        Ok(result)
    }

    fn compile(&mut self, term: Term) {
        match term {
            Term::Int(i) => {
                let value = Value::Integer(i);
                self.constants.push(value);

                self.bytecode
                    .push(Instruction::Constant(self.constants.len() as u16 - 1));
            }
            Term::Binary { op, lhs, rhs } => {
                self.compile(*lhs);
                self.compile(*rhs);

                match op {
                    Operation::Add => {
                        self.bytecode.push(Instruction::Add);
                    }
                }
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
                Instruction::Add => {
                    let rhs = self
                        .stack
                        .pop()
                        .ok_or(anyhow!("Expected operand, but stack was empty."))?;

                    let lhs = self
                        .stack
                        .pop()
                        .ok_or(anyhow!("Expected operand, but stack was empty."))?;

                    if let (Value::Integer(lhs), Value::Integer(rhs)) = (lhs, rhs) {
                        self.stack.push(Value::Integer(lhs + rhs));
                    } else {
                        bail!("Operands must be both integers.");
                    }
                }
            }
        }

        if self.stack.len() != 1 {
            bail!("At the end of the program the stack should contain only a single item.");
        }
        Ok(self.stack[0])
    }
}
