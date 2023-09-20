use anyhow::{anyhow, bail, Result};
use rinha::{ast::Term, parser::parse_or_report};
use std::collections::HashMap;

use crate::{bytecode::Instruction, compiler::Compiler, function::Function, value::Value};

pub struct Vm<'a> {
    constants: Vec<Value<'a>>,
    pub functions: Vec<Function>,
    identifiers: Vec<String>,
    frame_index: usize,
}

macro_rules! pop_operands {
    ($stack: ident) => {{
        let rhs = $stack
            .pop()
            .ok_or(anyhow!("Expected operand, but stack was empty."))?;

        let lhs = $stack
            .pop()
            .ok_or(anyhow!("Expected operand, but stack was empty."))?;

        let result: Result<(Value<'_>, Value<'_>)> = Ok((lhs, rhs));
        result
    }};
}

impl<'a> Vm<'a> {
    pub fn new() -> Self {
        Self {
            constants: Vec::new(),
            functions: Vec::new(),
            identifiers: Vec::new(),
            frame_index: 0,
        }
    }

    pub fn interpret(&'a mut self, filename: &str, contents: &str) -> Result<Value<'a>> {
        let file = parse_or_report(filename, contents)?;
        let bytecode = self.compile(file.expression)?;

        let result = self.run(&bytecode, Vec::new(), &HashMap::new(), false)?;
        Ok(result)
    }

    pub fn create_constant(&mut self, value: Value<'a>) -> Result<u16> {
        if self.constants.len() >= u16::MAX as usize {
            bail!("Cannot create more than {} constants.", u16::MAX);
        }

        let position = self.constants.iter().position(|v| *v == value);

        Ok(position.unwrap_or_else(|| {
            self.constants.push(value);
            self.constants.len() - 1
        }) as u16)
    }

    pub fn create_identifier(&mut self, identifier: String) -> Result<u16> {
        if self.identifiers.len() >= u16::MAX as usize {
            bail!("Cannot create more than {} identifiers.", u16::MAX);
        }

        let position = self.identifiers.iter().position(|i| *i == identifier);

        Ok(position.unwrap_or_else(|| {
            self.identifiers.push(identifier);
            self.identifiers.len() - 1
        }) as u16)
    }

    fn compile(&mut self, term: Term) -> Result<Vec<Instruction>> {
        let mut compiler = Compiler::new(None);
        compiler.compile(term, self)
    }

    fn run(
        &'a self,
        bytecode: &[Instruction],
        stack: Vec<Value<'a>>,
        global_references: &HashMap<String, Value<'a>>,
        is_function: bool,
    ) -> Result<Value<'a>> {
        let mut globals = HashMap::new();
        let mut stack = stack;

        let mut skip = 0;
        for instruction in bytecode {
            if skip > 0 {
                skip -= 1;
                continue;
            }

            match *instruction {
                Instruction::Constant(index) => {
                    let value = self.constants[index as usize].clone();
                    stack.push(value);
                }
                Instruction::True => {
                    let value = Value::Bool(true);
                    stack.push(value);
                }
                Instruction::False => {
                    let value = Value::Bool(false);
                    stack.push(value);
                }
                Instruction::Add => {
                    let (lhs, rhs) = pop_operands!(stack)?;

                    match (lhs, rhs) {
                        (Value::Integer(lhs), Value::Integer(rhs)) => {
                            stack.push(Value::Integer(lhs + rhs));
                        }
                        (Value::String(lhs), Value::Integer(rhs)) => {
                            stack.push(Value::String(format!("{lhs}{rhs}")));
                        }
                        (Value::Integer(lhs), Value::String(rhs)) => {
                            stack.push(Value::String(format!("{lhs}{rhs}")));
                        }
                        (Value::String(lhs), Value::String(rhs)) => {
                            stack.push(Value::String(format!("{lhs}{rhs}")));
                        }
                        _ => {
                            bail!("Wrong types for add.");
                        }
                    }
                }
                Instruction::Sub => {
                    let (lhs, rhs) = pop_operands!(stack)?;

                    if let (Value::Integer(lhs), Value::Integer(rhs)) = (lhs, rhs) {
                        stack.push(Value::Integer(lhs - rhs));
                    } else {
                        bail!("Operands must be both integers.");
                    }
                }
                Instruction::Mul => {
                    let (lhs, rhs) = pop_operands!(stack)?;

                    if let (Value::Integer(lhs), Value::Integer(rhs)) = (lhs, rhs) {
                        stack.push(Value::Integer(lhs * rhs));
                    } else {
                        bail!("Operands must be both integers.");
                    }
                }
                Instruction::Div => {
                    let (lhs, rhs) = pop_operands!(stack)?;

                    if let (Value::Integer(lhs), Value::Integer(rhs)) = (lhs, rhs) {
                        let result = lhs
                            .checked_div(rhs)
                            .ok_or(anyhow!("Attempted to divide by zero"))?;

                        stack.push(Value::Integer(result));
                    } else {
                        bail!("Operands must be both integers.");
                    }
                }
                Instruction::Rem => {
                    let (lhs, rhs) = pop_operands!(stack)?;

                    if let (Value::Integer(lhs), Value::Integer(rhs)) = (lhs, rhs) {
                        let result = lhs
                            .checked_rem(rhs)
                            .ok_or(anyhow!("Attempted to take remainder by zero"))?;

                        stack.push(Value::Integer(result));
                    } else {
                        bail!("Operands must be both integers.");
                    }
                }
                Instruction::Eq => {
                    let (lhs, rhs) = pop_operands!(stack)?;
                    stack.push(Value::Bool(lhs == rhs));
                }
                Instruction::Neq => {
                    let (lhs, rhs) = pop_operands!(stack)?;
                    stack.push(Value::Bool(lhs != rhs));
                }
                Instruction::Gt => {
                    let (lhs, rhs) = pop_operands!(stack)?;

                    if let (Value::Integer(lhs), Value::Integer(rhs)) = (lhs, rhs) {
                        stack.push(Value::Bool(lhs > rhs));
                    } else {
                        bail!("Operands must be both integers.");
                    }
                }
                Instruction::Lt => {
                    let (lhs, rhs) = pop_operands!(stack)?;

                    if let (Value::Integer(lhs), Value::Integer(rhs)) = (lhs, rhs) {
                        stack.push(Value::Bool(lhs < rhs));
                    } else {
                        bail!("Operands must be both integers.");
                    }
                }
                Instruction::Gte => {
                    let (lhs, rhs) = pop_operands!(stack)?;

                    if let (Value::Integer(lhs), Value::Integer(rhs)) = (lhs, rhs) {
                        stack.push(Value::Bool(lhs >= rhs));
                    } else {
                        bail!("Operands must be both integers.");
                    }
                }
                Instruction::Lte => {
                    let (lhs, rhs) = pop_operands!(stack)?;

                    if let (Value::Integer(lhs), Value::Integer(rhs)) = (lhs, rhs) {
                        stack.push(Value::Bool(lhs <= rhs));
                    } else {
                        bail!("Operands must be both integers.");
                    }
                }
                // TODO: handle short-circuiting
                Instruction::And => {
                    let (lhs, rhs) = pop_operands!(stack)?;

                    if let (Value::Bool(lhs), Value::Bool(rhs)) = (lhs, rhs) {
                        stack.push(Value::Bool(lhs && rhs));
                    } else {
                        bail!("Operands must be both integers.");
                    }
                }
                Instruction::Or => {
                    let (lhs, rhs) = pop_operands!(stack)?;

                    if let (Value::Bool(lhs), Value::Bool(rhs)) = (lhs, rhs) {
                        stack.push(Value::Bool(lhs || rhs));
                    } else {
                        bail!("Operands must be both integers.");
                    }
                }
                Instruction::Tuple => {
                    let (first, second) = pop_operands!(stack)?;
                    let value = Value::Tuple(Box::new(first), Box::new(second));
                    stack.push(value);
                }
                Instruction::First => {
                    let value = stack
                        .pop()
                        .ok_or(anyhow!("Expected operand, but stack was empty."))?;

                    if let Value::Tuple(first, _) = value {
                        stack.push(*first);
                    } else {
                        bail!("Tried to compute `first` of a non tuple type.");
                    }
                }
                Instruction::Second => {
                    let value = stack
                        .pop()
                        .ok_or(anyhow!("Expected operand, but stack was empty."))?;

                    if let Value::Tuple(_, second) = value {
                        stack.push(*second);
                    } else {
                        bail!("Tried to compute `second` of a non tuple type.");
                    }
                }
                Instruction::Print => {
                    let value = stack.last().ok_or(anyhow!(
                        "Error printing. No value found in the stack to be set."
                    ))?;
                    println!("{value}");
                }
                Instruction::GlobalSet(index) => {
                    let identifier = self.identifiers[index as usize].clone();

                    let value = stack.pop().ok_or(anyhow!(
                        "Error setting global variable. No value found in the stack to be set."
                    ))?;
                    let _ = globals.insert(identifier, value);
                }
                Instruction::GlobalGet(index) => {
                    let identifier = self.identifiers[index as usize].clone();

                    let global_references = if is_function {
                        global_references
                    } else {
                        &globals
                    };

                    let value = global_references
                        .get(&identifier)
                        .ok_or(anyhow!("Unknown variable {identifier}."))?
                        .clone();

                    stack.push(value);
                }
                Instruction::LocalGet(index, identifier_index) => {
                    let absolute_index = self.frame_index + index as usize;
                    if absolute_index >= stack.len() {
                        let identifier = &self.identifiers[identifier_index as usize];
                        bail!("Variable {identifier} not found.");
                    }
                    let value = stack[absolute_index].clone();
                    stack.push(value);
                }
                Instruction::If(jump) => {
                    let value = stack.pop().ok_or(anyhow!(
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
                }
                Instruction::Closure(index) => {
                    let function = &self.functions[index as usize];
                    let closure = Value::Closure(function);
                    stack.push(closure);
                }
                Instruction::Call(arity) => {
                    let mut child_stack = Vec::new();

                    for _ in 0..arity {
                        let value = stack.pop().ok_or(anyhow!("Missing arguments."))?;
                        child_stack.push(value);
                    }

                    let closure = stack.pop().ok_or(anyhow!("Missing function."))?;

                    if let Value::Closure(f) = closure {
                        if f.arity != arity {
                            bail!("Function called with wrong number of arguments.");
                        }

                        let global_references = if is_function {
                            global_references
                        } else {
                            &globals
                        };

                        let value = self.run(&f.bytecode, child_stack, global_references, true)?;
                        stack.push(value);
                    } else {
                        bail!("Attempted to call a value that is not a function.");
                    }
                }
            }
        }

        Ok(stack
            .last()
            .expect("At the end of the execution, there must be at least one value in the stack.")
            .clone())
    }
}
