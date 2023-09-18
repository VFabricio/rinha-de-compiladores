use anyhow::{anyhow, bail, Result};
use rinha::{
    ast::{BinaryOp, Term},
    parser::parse_or_report,
};

use crate::{bytecode::Instruction, value::Value};

pub struct Vm {
    constants: Vec<Value>,
    stack: Vec<Value>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            constants: Vec::new(),
            stack: Vec::new(),
        }
    }

    pub fn interpret(mut self, filename: &str, contents: &str) -> Result<Value> {
        let file = parse_or_report(filename, contents)?;
        let mut bytecode = Vec::new();
        self.compile(file.expression, &mut bytecode);
        let result = self.run(bytecode)?;
        Ok(result)
    }

    fn compile(&mut self, term: Term, bytecode: &mut Vec<Instruction>) {
        match term {
            Term::Int(i) => {
                let value = Value::Integer(i.value);
                self.constants.push(value);

                bytecode.push(Instruction::Constant(self.constants.len() as u16 - 1));
            }
            Term::Binary(b) => {
                self.compile(*b.lhs, bytecode);
                self.compile(*b.rhs, bytecode);

                match b.op {
                    BinaryOp::Add => {
                        bytecode.push(Instruction::Add);
                    }
                    BinaryOp::Sub => {
                        bytecode.push(Instruction::Sub);
                    }
                    BinaryOp::Mul => {
                        bytecode.push(Instruction::Mul);
                    }
                    BinaryOp::Div => {
                        bytecode.push(Instruction::Div);
                    }
                    _ => unimplemented!(),
                }
            }
            _ => unimplemented!(),
        };
    }

    fn pop_operands(&mut self) -> Result<(Value, Value)> {
        let rhs = self
            .stack
            .pop()
            .ok_or(anyhow!("Expected operand, but stack was empty."))?;

        let lhs = self
            .stack
            .pop()
            .ok_or(anyhow!("Expected operand, but stack was empty."))?;

        Ok((lhs, rhs))
    }

    fn run(mut self, bytecode: Vec<Instruction>) -> Result<Value> {
        for instruction in bytecode {
            match instruction {
                Instruction::Constant(index) => {
                    let value = self.constants[index as usize];
                    self.stack.push(value);
                }
                Instruction::Add => {
                    let (lhs, rhs) = self.pop_operands()?;

                    if let (Value::Integer(lhs), Value::Integer(rhs)) = (lhs, rhs) {
                        self.stack.push(Value::Integer(lhs + rhs));
                    } else {
                        bail!("Operands must be both integers.");
                    }
                }
                Instruction::Sub => {
                    let (lhs, rhs) = self.pop_operands()?;

                    if let (Value::Integer(lhs), Value::Integer(rhs)) = (lhs, rhs) {
                        self.stack.push(Value::Integer(lhs - rhs));
                    } else {
                        bail!("Operands must be both integers.");
                    }
                }
                Instruction::Mul => {
                    let (lhs, rhs) = self.pop_operands()?;

                    if let (Value::Integer(lhs), Value::Integer(rhs)) = (lhs, rhs) {
                        self.stack.push(Value::Integer(lhs * rhs));
                    } else {
                        bail!("Operands must be both integers.");
                    }
                }
                Instruction::Div => {
                    let (lhs, rhs) = self.pop_operands()?;

                    if let (Value::Integer(lhs), Value::Integer(rhs)) = (lhs, rhs) {
                        let result = lhs
                            .checked_div(rhs)
                            .ok_or(anyhow!("Attempted to divide by zero"))?;

                        self.stack.push(Value::Integer(result));
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
