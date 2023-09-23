use anyhow::{anyhow, bail, Result};
use rinha::{ast::Term, parser::parse_or_report};
use std::collections::HashMap;

use crate::{
    bytecode::Instruction,
    call_frame::CallFrame,
    compiler::{CallPosition, Compiler},
    function::Function,
    value::{FinalValue, Value},
};

pub struct Vm<'a> {
    call_frames: Vec<CallFrame<'a>>,
    constants: Vec<Value<'a>>,
    pub functions: Vec<Function>,
    globals: HashMap<String, Value<'a>>,
    identifiers: Vec<String>,
    frame_index: usize,
    stack: Vec<Value<'a>>,
}

macro_rules! pop_operands {
    ($self: ident) => {{
        let rhs = $self
            .stack
            .pop()
            .ok_or(anyhow!("Expected operand, but stack was empty."))?;

        let lhs = $self
            .stack
            .pop()
            .ok_or(anyhow!("Expected operand, but stack was empty."))?;

        let result: Result<(Value<'_>, Value<'_>)> = Ok((lhs, rhs));
        result
    }};
}

impl<'a> Vm<'a> {
    pub fn new() -> Self {
        Self {
            call_frames: Vec::new(),
            constants: Vec::new(),
            functions: Vec::new(),
            globals: HashMap::new(),
            identifiers: Vec::new(),
            frame_index: 0,
            stack: Vec::new(),
        }
    }

    pub fn interpret(&'a mut self, filename: &str, contents: &str) -> Result<FinalValue> {
        let file = parse_or_report(filename, contents)?;

        let mut bytecode = self.compile(file.expression)?;
        bytecode.push(Instruction::Return(0));
        let bytecode = Box::leak(Box::new(bytecode));

        let result = self.run(bytecode)?;
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
        compiler.compile(term, self, CallPosition::Unknown)
    }

    fn run(&'a mut self, bytecode: &'a [Instruction]) -> Result<FinalValue> {
        let initial_frame = CallFrame {
            bytecode: &bytecode,
            instruction_pointer: 0,
            closure: Value::Bool(false),
        };

        self.call_frames.push(initial_frame);

        loop {
            let bytecode;
            let mut instruction_pointer;
            let mut environment = &HashMap::new();

            if let Some(call_frame) = self.call_frames.last() {
                instruction_pointer = call_frame.instruction_pointer;
                bytecode = &call_frame.bytecode[instruction_pointer..];
                if let Value::Closure(_, new_environment) = &call_frame.closure {
                    environment = new_environment;
                }
            } else {
                break;
            }

            let mut skip = 0;
            for instruction in bytecode {
                instruction_pointer += 1;

                if skip > 0 {
                    skip -= 1;
                    continue;
                }

                match *instruction {
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
                        let (lhs, rhs) = pop_operands!(self)?;

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
                        let (lhs, rhs) = pop_operands!(self)?;

                        if let (Value::Integer(lhs), Value::Integer(rhs)) = (lhs, rhs) {
                            self.stack.push(Value::Integer(lhs - rhs));
                        } else {
                            bail!("Operands must be both integers.");
                        }
                    }
                    Instruction::Mul => {
                        let (lhs, rhs) = pop_operands!(self)?;

                        if let (Value::Integer(lhs), Value::Integer(rhs)) = (lhs, rhs) {
                            self.stack.push(Value::Integer(lhs * rhs));
                        } else {
                            bail!("Operands must be both integers.");
                        }
                    }
                    Instruction::Div => {
                        let (lhs, rhs) = pop_operands!(self)?;

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
                        let (lhs, rhs) = pop_operands!(self)?;

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
                        let (lhs, rhs) = pop_operands!(self)?;
                        self.stack.push(Value::Bool(lhs == rhs));
                    }
                    Instruction::Neq => {
                        let (lhs, rhs) = pop_operands!(self)?;
                        self.stack.push(Value::Bool(lhs != rhs));
                    }
                    Instruction::Gt => {
                        let (lhs, rhs) = pop_operands!(self)?;

                        if let (Value::Integer(lhs), Value::Integer(rhs)) = (lhs, rhs) {
                            self.stack.push(Value::Bool(lhs > rhs));
                        } else {
                            bail!("Operands must be both integers.");
                        }
                    }
                    Instruction::Lt => {
                        let (lhs, rhs) = pop_operands!(self)?;

                        if let (Value::Integer(lhs), Value::Integer(rhs)) = (lhs, rhs) {
                            self.stack.push(Value::Bool(lhs < rhs));
                        } else {
                            bail!("Operands must be both integers.");
                        }
                    }
                    Instruction::Gte => {
                        let (lhs, rhs) = pop_operands!(self)?;

                        if let (Value::Integer(lhs), Value::Integer(rhs)) = (lhs, rhs) {
                            self.stack.push(Value::Bool(lhs >= rhs));
                        } else {
                            bail!("Operands must be both integers.");
                        }
                    }
                    Instruction::Lte => {
                        let (lhs, rhs) = pop_operands!(self)?;

                        if let (Value::Integer(lhs), Value::Integer(rhs)) = (lhs, rhs) {
                            self.stack.push(Value::Bool(lhs <= rhs));
                        } else {
                            bail!("Operands must be both integers.");
                        }
                    }
                    // TODO: handle short-circuiting
                    Instruction::And => {
                        let (lhs, rhs) = pop_operands!(self)?;

                        if let (Value::Bool(lhs), Value::Bool(rhs)) = (lhs, rhs) {
                            self.stack.push(Value::Bool(lhs && rhs));
                        } else {
                            bail!("Operands must be both integers.");
                        }
                    }
                    Instruction::Or => {
                        let (lhs, rhs) = pop_operands!(self)?;

                        if let (Value::Bool(lhs), Value::Bool(rhs)) = (lhs, rhs) {
                            self.stack.push(Value::Bool(lhs || rhs));
                        } else {
                            bail!("Operands must be both integers.");
                        }
                    }
                    Instruction::Tuple => {
                        let (first, second) = pop_operands!(self)?;
                        let value = Value::Tuple(Box::new(first), Box::new(second));
                        self.stack.push(value);
                    }
                    Instruction::First => {
                        let value = self
                            .stack
                            .pop()
                            .ok_or(anyhow!("Expected operand, but self.stack was empty."))?;

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
                            .ok_or(anyhow!("Expected operand, but self.stack was empty."))?;

                        if let Value::Tuple(_, second) = value {
                            self.stack.push(*second);
                        } else {
                            bail!("Tried to compute `second` of a non tuple type.");
                        }
                    }
                    Instruction::Print => {
                        let value = self.stack.last().ok_or(anyhow!(
                            "Error printing. No value found in the self.stack to be set."
                        ))?;
                        println!("{value}");
                    }
                    Instruction::GlobalSet(index) => {
                        let identifier = self.identifiers[index as usize].clone();

                        let value = self.stack.pop().ok_or(anyhow!(
                            "Error setting global variable. No value found in the self.stack to be set."
                        ))?;
                        let _ = self.globals.insert(identifier, value);
                    }
                    Instruction::GlobalGet(index) => {
                        let identifier = self.identifiers[index as usize].clone();

                        let value = environment
                            .get(&identifier)
                            .or(self.globals.get(&identifier))
                            .ok_or(anyhow!("Unknown variable {identifier}."))?
                            .clone();

                        self.stack.push(value);
                    }
                    Instruction::LocalGet(index, identifier_index) => {
                        let absolute_index = self.frame_index + index as usize;
                        if absolute_index >= self.stack.len() {
                            let identifier = &self.identifiers[identifier_index as usize];
                            bail!("Variable {identifier} not found.");
                        }
                        let value = self.stack[absolute_index].clone();
                        self.stack.push(value);
                    }
                    Instruction::If(jump) => {
                        let value = self.stack.pop().ok_or(anyhow!(
                            "Error in if. No value found in the self.stack to be tested."
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
                        let parent = &self
                            .call_frames
                            .last()
                            .expect("There is always at least one call frame active.")
                            .closure;

                        let mut environment = HashMap::new();

                        if let Value::Closure(parent_function, parent_environment) = parent {
                            for captured in &function.captured {
                                let index = parent_function
                                    .locals
                                    .iter()
                                    .position(|l| l.name == *captured);

                                if let Some(index) = index {
                                    let absolute_index = self.frame_index + index as usize;
                                    environment.insert(
                                        captured.clone(),
                                        self.stack[absolute_index].clone(),
                                    );
                                } else {
                                    let captured_in_parent = parent_environment.get(captured);
                                    if let Some(captured_in_parent) = captured_in_parent {
                                        environment
                                            .insert(captured.clone(), captured_in_parent.clone());
                                    }
                                }
                            }
                        }

                        let closure = Value::Closure(function, environment);
                        self.stack.push(closure);
                    }
                    Instruction::Call(arity) => {
                        let closure_index = self.stack.len() - 1 - arity as usize;
                        let closure = &self.stack[closure_index];
                        let closure = closure.clone();

                        if let Value::Closure(function, _) = closure {
                            if function.arity != arity {
                                bail!("Attempted to call function with wrong number of arguments.");
                            }
                            let current_frame = self
                                .call_frames
                                .last_mut()
                                .expect("There is at least one active call frame at all times.");

                            current_frame.instruction_pointer = instruction_pointer;

                            let new_frame = CallFrame {
                                bytecode: &function.bytecode,
                                instruction_pointer: 0,
                                closure,
                            };
                            self.call_frames.push(new_frame);

                            self.frame_index = self.stack.len() - arity as usize;
                            break;
                        } else {
                            bail!("Attempted to call value that is not a function!");
                        }
                    }
                    Instruction::TailCall(arity) => {
                        let closure_index = self.stack.len() - 1 - arity as usize;
                        let closure = &self.stack[closure_index];
                        let closure = closure.clone();

                        if let Value::Closure(function, _) = closure {
                            if function.arity != arity {
                                bail!("Attempted to call function with wrong number of arguments.");
                            }
                            let current_frame = self
                                .call_frames
                                .last_mut()
                                .expect("There is at least one active call frame at all times.");

                            current_frame.instruction_pointer = instruction_pointer;

                            let new_frame = CallFrame {
                                bytecode: &function.bytecode,
                                instruction_pointer: 0,
                                closure,
                            };
                            self.call_frames.pop();

                            self.call_frames.push(new_frame);

                            self.frame_index = self.stack.len() - arity as usize;
                            break;
                        } else {
                            bail!("Attempted to call value that is not a function!");
                        }
                    }
                    Instruction::Return(arity) => {
                        let result = self.stack.pop().expect("Function must have a return value");

                        for _ in 0..arity + 1 {
                            self.stack.pop();
                        }

                        self.stack.push(result);
                        self.call_frames.pop();

                        if self.call_frames.len() > 1 {
                            self.frame_index =
                                (self.stack.len() - 1).saturating_sub(arity as usize);
                        }
                        break;
                    }
                }
            }
        }

        let value = self.stack.last().expect(
            "At the end of the execution, there must be at least one value in the self.stack.",
        );

        let value: FinalValue = value.into();

        Ok(value)
    }
}
