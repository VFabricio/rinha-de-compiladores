use anyhow::{anyhow, bail, Result};
use rinha::{
    ast::{BinaryOp, Term},
    parser::parse_or_report,
};

use crate::{bytecode::Instruction, value::Value};

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

    pub fn interpret(&mut self, filename: &str, contents: &str) -> Result<Value> {
        let file = parse_or_report(filename, contents)?;
        self.compile(file.expression);
        let result = self.run()?;
        Ok(result)
    }

    fn compile(&mut self, term: Term) {
        match term {
            Term::Int(i) => {
                let value = Value::Integer(i.value);
                self.constants.push(value);

                self.bytecode
                    .push(Instruction::Constant(self.constants.len() as u16 - 1));
            }
            Term::Binary(b) => {
                self.compile(*b.lhs);
                self.compile(*b.rhs);

                match b.op {
                    BinaryOp::Add => {
                        self.bytecode.push(Instruction::Add);
                    }
                    _ => unimplemented!(),
                }
            }
            _ => unimplemented!(),
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
