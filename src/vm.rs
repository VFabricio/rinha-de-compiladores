use anyhow::{anyhow, bail, Result};
use rinha::{
    ast::{BinaryOp, Term},
    parser::parse_or_report,
};
use std::collections::HashMap;

use crate::{bytecode::Instruction, value::Value};

pub struct Vm {
    constants: Vec<Value>,
    globals: HashMap<String, Value>,
    identifiers: Vec<String>,
    stack: Vec<Value>,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            constants: Vec::new(),
            globals: HashMap::new(),
            identifiers: Vec::new(),
            stack: Vec::new(),
        }
    }

    pub fn interpret(mut self, filename: &str, contents: &str) -> Result<Value> {
        let file = parse_or_report(filename, contents)?;
        let mut bytecode = Vec::new();
        self.compile(file.expression, &mut bytecode)?;

        let result = self.run(bytecode)?;
        Ok(result)
    }

    fn compile(&mut self, term: Term, bytecode: &mut Vec<Instruction>) -> Result<()> {
        match term {
            Term::Int(i) => {
                let value = Value::Integer(i.value);
                self.constants.push(value);

                bytecode.push(Instruction::Constant(self.constants.len() as u16 - 1));
            }
            Term::Bool(b) => {
                let instruction = if b.value {
                    Instruction::True
                } else {
                    Instruction::False
                };
                bytecode.push(instruction);
            }
            Term::Str(s) => {
                let value = Value::String(s.value);
                self.constants.push(value);

                bytecode.push(Instruction::Constant(self.constants.len() as u16 - 1));
            }
            Term::Binary(b) => {
                self.compile(*b.lhs, bytecode)?;
                self.compile(*b.rhs, bytecode)?;

                let instruction = match b.op {
                    BinaryOp::Add => Instruction::Add,
                    BinaryOp::Sub => Instruction::Sub,
                    BinaryOp::Mul => Instruction::Mul,
                    BinaryOp::Div => Instruction::Div,
                    BinaryOp::Rem => Instruction::Rem,
                    BinaryOp::Eq => Instruction::Eq,
                    BinaryOp::Neq => Instruction::Neq,
                    BinaryOp::Gt => Instruction::Gt,
                    BinaryOp::Lt => Instruction::Lt,
                    BinaryOp::Gte => Instruction::Gte,
                    BinaryOp::Lte => Instruction::Lte,
                    BinaryOp::And => Instruction::And,
                    BinaryOp::Or => Instruction::Or,
                };
                bytecode.push(instruction);
            }
            Term::Tuple(t) => {
                self.compile(*t.first, bytecode)?;
                self.compile(*t.second, bytecode)?;

                bytecode.push(Instruction::Tuple);
            }
            Term::First(t) => {
                self.compile(*t.value, bytecode)?;

                bytecode.push(Instruction::First);
            }
            Term::Second(t) => {
                self.compile(*t.value, bytecode)?;

                bytecode.push(Instruction::Second);
            }
            Term::Let(t) => {
                self.compile(*t.value, bytecode)?;

                self.identifiers.push(t.name.text);
                bytecode.push(Instruction::GlobalSet(self.identifiers.len() as u16 - 1));

                self.compile(*t.next, bytecode)?;
            }
            Term::Var(t) => {
                self.identifiers.push(t.text);
                bytecode.push(Instruction::GlobalGet(self.identifiers.len() as u16 - 1));
            }
            Term::Print(t) => {
                self.compile(*t.value, bytecode)?;
                bytecode.push(Instruction::Print);
            }
            Term::If(t) => {
                self.compile(*t.condition, bytecode)?;
                bytecode.push(Instruction::If(0));

                let if_address = bytecode.len() - 1;
                let if_address = if if_address > i32::MAX as usize {
                    bail!("Instruction too long.");
                } else {
                    if_address as u32
                };

                self.compile(*t.then, bytecode)?;
                bytecode.push(Instruction::Jump(0));

                let jump_address = bytecode.len() - 1;
                let jump_address = if jump_address > i32::MAX as usize {
                    bail!("Instruction too long.");
                } else {
                    jump_address as u32
                };

                bytecode[if_address as usize] = Instruction::If(jump_address - if_address);

                self.compile(*t.otherwise, bytecode)?;
                let after_address = bytecode.len() - 1;
                let after_address = if after_address > i32::MAX as usize {
                    bail!("Instruction too long.");
                } else {
                    after_address as u32
                };

                bytecode[jump_address as usize] = Instruction::Jump(after_address - jump_address);
            }
            Term::Error(e) => bail!(anyhow!(e.message)),
            _ => unimplemented!(),
        };
        Ok(())
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
        let mut skip = 0;
        for instruction in bytecode {
            if skip > 0 {
                skip -= 1;
                continue;
            }

            match instruction {
                Instruction::Constant(index) => {
                    let value = self.constants[index as usize].clone();
                    self.stack.push(value);
                }
                Instruction::True => {
                    let value = Value::Bool(true);
                    self.stack.push(value);
                }
                Instruction::False => {
                    let value = Value::Bool(false);
                    self.stack.push(value);
                }
                Instruction::Add => {
                    let (lhs, rhs) = self.pop_operands()?;

                    match (lhs, rhs) {
                        (Value::Integer(lhs), Value::Integer(rhs)) => {
                            self.stack.push(Value::Integer(lhs + rhs));
                        }
                        (Value::String(lhs), Value::Integer(rhs)) => {
                            self.stack.push(Value::String(format!("{lhs}{rhs}")));
                        }
                        (Value::Integer(lhs), Value::String(rhs)) => {
                            self.stack.push(Value::String(format!("{lhs}{rhs}")));
                        }
                        (Value::String(lhs), Value::String(rhs)) => {
                            self.stack.push(Value::String(format!("{lhs}{rhs}")));
                        }
                        _ => {
                            bail!("Wrong types for add.");
                        }
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
                Instruction::Rem => {
                    let (lhs, rhs) = self.pop_operands()?;

                    if let (Value::Integer(lhs), Value::Integer(rhs)) = (lhs, rhs) {
                        let result = lhs
                            .checked_rem(rhs)
                            .ok_or(anyhow!("Attempted to take remainder by zero"))?;

                        self.stack.push(Value::Integer(result));
                    } else {
                        bail!("Operands must be both integers.");
                    }
                }
                Instruction::Eq => {
                    let (lhs, rhs) = self.pop_operands()?;
                    self.stack.push(Value::Bool(lhs == rhs));
                }
                Instruction::Neq => {
                    let (lhs, rhs) = self.pop_operands()?;
                    self.stack.push(Value::Bool(lhs != rhs));
                }
                Instruction::Gt => {
                    let (lhs, rhs) = self.pop_operands()?;

                    if let (Value::Integer(lhs), Value::Integer(rhs)) = (lhs, rhs) {
                        self.stack.push(Value::Bool(lhs > rhs));
                    } else {
                        bail!("Operands must be both integers.");
                    }
                }
                Instruction::Lt => {
                    let (lhs, rhs) = self.pop_operands()?;

                    if let (Value::Integer(lhs), Value::Integer(rhs)) = (lhs, rhs) {
                        self.stack.push(Value::Bool(lhs < rhs));
                    } else {
                        bail!("Operands must be both integers.");
                    }
                }
                Instruction::Gte => {
                    let (lhs, rhs) = self.pop_operands()?;

                    if let (Value::Integer(lhs), Value::Integer(rhs)) = (lhs, rhs) {
                        self.stack.push(Value::Bool(lhs >= rhs));
                    } else {
                        bail!("Operands must be both integers.");
                    }
                }
                Instruction::Lte => {
                    let (lhs, rhs) = self.pop_operands()?;

                    if let (Value::Integer(lhs), Value::Integer(rhs)) = (lhs, rhs) {
                        self.stack.push(Value::Bool(lhs <= rhs));
                    } else {
                        bail!("Operands must be both integers.");
                    }
                }
                // TODO: handle short-circuiting
                Instruction::And => {
                    let (lhs, rhs) = self.pop_operands()?;

                    if let (Value::Bool(lhs), Value::Bool(rhs)) = (lhs, rhs) {
                        self.stack.push(Value::Bool(lhs && rhs));
                    } else {
                        bail!("Operands must be both integers.");
                    }
                }
                Instruction::Or => {
                    let (lhs, rhs) = self.pop_operands()?;

                    if let (Value::Bool(lhs), Value::Bool(rhs)) = (lhs, rhs) {
                        self.stack.push(Value::Bool(lhs || rhs));
                    } else {
                        bail!("Operands must be both integers.");
                    }
                }
                Instruction::Tuple => {
                    let (first, second) = self.pop_operands()?;
                    let value = Value::Tuple(Box::new(first), Box::new(second));
                    self.stack.push(value);
                }
                Instruction::First => {
                    let value = self
                        .stack
                        .pop()
                        .ok_or(anyhow!("Expected operand, but stack was empty."))?;

                    if let Value::Tuple(first, _) = value {
                        self.stack.push(*first);
                    } else {
                        bail!("Tried to compute `first` of a non tuple type.");
                    }
                }
                Instruction::Second => {
                    let value = self
                        .stack
                        .pop()
                        .ok_or(anyhow!("Expected operand, but stack was empty."))?;

                    if let Value::Tuple(_, second) = value {
                        self.stack.push(*second);
                    } else {
                        bail!("Tried to compute `second` of a non tuple type.");
                    }
                }
                Instruction::Print => {
                    let value = self.stack.last().ok_or(anyhow!(
                        "Error printing. No value found in the stack to be set."
                    ))?;
                    println!("{value}");
                }
                Instruction::GlobalSet(index) => {
                    let identifier = self.identifiers[index as usize].clone();

                    let value = self.stack.pop().ok_or(anyhow!(
                        "Error setting global variable. No value found in the stack to be set."
                    ))?;
                    let _ = self.globals.insert(identifier, value);
                }
                Instruction::GlobalGet(index) => {
                    let identifier = self.identifiers[index as usize].clone();

                    let value = self
                        .globals
                        .get(&identifier)
                        .ok_or(anyhow!("Unknown variable {identifier}."))?
                        .clone();

                    self.stack.push(value);
                }
                Instruction::If(jump) => {
                    let value = self.stack.pop().ok_or(anyhow!(
                        "Error in if. No value found in the stack to be tested."
                    ))?;

                    if let Value::Bool(b) = value {
                        if !b {
                            skip = jump;
                            continue;
                        }
                    } else {
                        bail!("Type error: if condition must evaluate to a boolean.");
                    }
                }
                Instruction::Jump(jump) => {
                    skip = jump;
                    continue;
                }
            }
        }

        if self.stack.len() != 1 {
            bail!("At the end of the program the stack should contain only a single item.");
        }
        Ok(self.stack[0].clone())
    }
}
