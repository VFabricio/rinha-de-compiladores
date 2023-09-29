use anyhow::{anyhow, bail, Result};
use rinha::{ast::Term, parser::parse_or_report};
use std::rc::Rc;

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
    current_execution: Option<(u16, i32)>,
    pub functions: Vec<Function>,
    globals: Vec<(&'a str, Rc<Value<'a>>)>,
    identifiers: Vec<String>,
    memoization: Vec<((u16, i32), Rc<Value<'a>>)>,
    pure: bool,
    stack: Vec<Rc<Value<'a>>>,
}

macro_rules! pop_operands {
    ($self: ident) => {{
        let rhs = $self
            .stack
            .pop()
            .ok_or_else(|| anyhow!("Expected operand, but stack was empty."))?;

        let lhs = $self
            .stack
            .pop()
            .ok_or_else(|| anyhow!("Expected operand, but stack was empty."))?;

        let result: Result<(Rc<Value<'_>>, Rc<Value<'_>>)> = Ok((lhs, rhs));
        result
    }};
}

impl<'a> Vm<'a> {
    pub fn new() -> Self {
        Self {
            call_frames: Vec::new(),
            constants: Vec::new(),
            current_execution: None,
            functions: Vec::new(),
            globals: Vec::new(),
            identifiers: Vec::new(),
            memoization: Vec::new(),
            pure: true,
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
            closure: Rc::new(Value::Bool(false)),
            instruction_pointer: 0,
            frame_index: 0,
        };

        self.call_frames.push(initial_frame);

        loop {
            let bytecode;
            let mut instruction_pointer;
            let frame_index;
            let mut environment = &Vec::new();

            if let Some(call_frame) = self.call_frames.last() {
                frame_index = call_frame.frame_index;
                instruction_pointer = call_frame.instruction_pointer;
                bytecode = &call_frame.bytecode[instruction_pointer..];
                if let Value::Closure(_, new_environment) = &*call_frame.closure {
                    environment = new_environment;
                }
            } else {
                break;
            }

            self.pure = true;

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
                        self.stack.push(Rc::new(value));
                    }
                    Instruction::True => {
                        let value = Value::Bool(true);
                        self.stack.push(Rc::new(value));
                    }
                    Instruction::False => {
                        let value = Value::Bool(false);
                        self.stack.push(Rc::new(value));
                    }
                    Instruction::Add => {
                        let (lhs, rhs) = pop_operands!(self)?;

                        match (lhs.as_ref(), rhs.as_ref()) {
                            (Value::Integer(lhs), Value::Integer(rhs)) => {
                                self.stack.push(Rc::new(Value::Integer(lhs + rhs)));
                            }
                            (Value::String(lhs), Value::Integer(rhs)) => {
                                self.stack
                                    .push(Rc::new(Value::String(format!("{lhs}{rhs}"))));
                            }
                            (Value::Integer(lhs), Value::String(rhs)) => {
                                self.stack
                                    .push(Rc::new(Value::String(format!("{lhs}{rhs}"))));
                            }
                            (Value::String(lhs), Value::String(rhs)) => {
                                self.stack
                                    .push(Rc::new(Value::String(format!("{lhs}{rhs}"))));
                            }
                            _ => {
                                bail!("Wrong types for add.");
                            }
                        }
                    }
                    Instruction::Sub => {
                        let (lhs, rhs) = pop_operands!(self)?;

                        if let (Value::Integer(lhs), Value::Integer(rhs)) =
                            (lhs.as_ref(), rhs.as_ref())
                        {
                            self.stack.push(Rc::new(Value::Integer(lhs - rhs)));
                        } else {
                            bail!("Operands must be both integers.");
                        }
                    }
                    Instruction::Mul => {
                        let (lhs, rhs) = pop_operands!(self)?;

                        if let (Value::Integer(lhs), Value::Integer(rhs)) =
                            (lhs.as_ref(), rhs.as_ref())
                        {
                            self.stack.push(Rc::new(Value::Integer(lhs * rhs)));
                        } else {
                            bail!("Operands must be both integers.");
                        }
                    }
                    Instruction::Div => {
                        let (lhs, rhs) = pop_operands!(self)?;

                        if let (Value::Integer(lhs), Value::Integer(rhs)) =
                            (lhs.as_ref(), rhs.as_ref())
                        {
                            let result = lhs
                                .checked_div(*rhs)
                                .ok_or_else(|| anyhow!("Attempted to divide by zero"))?;

                            self.stack.push(Rc::new(Value::Integer(result)));
                        } else {
                            bail!("Operands must be both integers.");
                        }
                    }
                    Instruction::Rem => {
                        let (lhs, rhs) = pop_operands!(self)?;

                        if let (Value::Integer(lhs), Value::Integer(rhs)) =
                            (lhs.as_ref(), rhs.as_ref())
                        {
                            let result = lhs
                                .checked_rem(*rhs)
                                .ok_or_else(|| anyhow!("Attempted to take remainder by zero"))?;

                            self.stack.push(Rc::new(Value::Integer(result)));
                        } else {
                            bail!("Operands must be both integers.");
                        }
                    }
                    Instruction::Eq => {
                        let (lhs, rhs) = pop_operands!(self)?;
                        self.stack.push(Rc::new(Value::Bool(lhs == rhs)));
                    }
                    Instruction::Neq => {
                        let (lhs, rhs) = pop_operands!(self)?;
                        self.stack.push(Rc::new(Value::Bool(lhs != rhs)));
                    }
                    Instruction::Gt => {
                        let (lhs, rhs) = pop_operands!(self)?;

                        if let (Value::Integer(lhs), Value::Integer(rhs)) =
                            (lhs.as_ref(), rhs.as_ref())
                        {
                            self.stack.push(Rc::new(Value::Bool(lhs > rhs)));
                        } else {
                            bail!("Operands must be both integers.");
                        }
                    }
                    Instruction::Lt => {
                        let (lhs, rhs) = pop_operands!(self)?;

                        if let (Value::Integer(lhs), Value::Integer(rhs)) =
                            (lhs.as_ref(), rhs.as_ref())
                        {
                            self.stack.push(Rc::new(Value::Bool(lhs < rhs)));
                        } else {
                            bail!("Operands must be both integers.");
                        }
                    }
                    Instruction::Gte => {
                        let (lhs, rhs) = pop_operands!(self)?;

                        if let (Value::Integer(lhs), Value::Integer(rhs)) =
                            (lhs.as_ref(), rhs.as_ref())
                        {
                            self.stack.push(Rc::new(Value::Bool(lhs >= rhs)));
                        } else {
                            bail!("Operands must be both integers.");
                        }
                    }
                    Instruction::Lte => {
                        let (lhs, rhs) = pop_operands!(self)?;

                        if let (Value::Integer(lhs), Value::Integer(rhs)) =
                            (lhs.as_ref(), rhs.as_ref())
                        {
                            self.stack.push(Rc::new(Value::Bool(lhs <= rhs)));
                        } else {
                            bail!("Operands must be both integers.");
                        }
                    }
                    // TODO: handle short-circuiting
                    Instruction::And => {
                        let (lhs, rhs) = pop_operands!(self)?;

                        if let (Value::Bool(lhs), Value::Bool(rhs)) = (lhs.as_ref(), rhs.as_ref()) {
                            self.stack.push(Rc::new(Value::Bool(*lhs && *rhs)));
                        } else {
                            bail!("Operands must be both integers.");
                        }
                    }
                    Instruction::Or => {
                        let (lhs, rhs) = pop_operands!(self)?;

                        if let (Value::Bool(lhs), Value::Bool(rhs)) = (lhs.as_ref(), rhs.as_ref()) {
                            self.stack.push(Rc::new(Value::Bool(*lhs || *rhs)));
                        } else {
                            bail!("Operands must be both integers.");
                        }
                    }
                    Instruction::Tuple => {
                        let (first, second) = pop_operands!(self)?;
                        let value = Value::Tuple(Box::new(first), Box::new(second));
                        self.stack.push(Rc::new(value));
                    }
                    Instruction::First => {
                        let value = self.stack.pop().ok_or_else(|| {
                            anyhow!("Expected operand, but self.stack was empty.")
                        })?;

                        if let Value::Tuple(first, _) = value.as_ref() {
                            self.stack.push(*first.clone());
                        } else {
                            bail!("Tried to compute `first` of a non tuple type.");
                        }
                    }
                    Instruction::Second => {
                        let value = self.stack.pop().ok_or_else(|| {
                            anyhow!("Expected operand, but self.stack was empty.")
                        })?;

                        if let Value::Tuple(_, second) = value.as_ref() {
                            self.stack.push(*second.clone());
                        } else {
                            bail!("Tried to compute `second` of a non tuple type.");
                        }
                    }
                    Instruction::Print => {
                        self.pure = false;
                        let value = self.stack.last().ok_or_else(|| {
                            anyhow!("Error printing. No value found in the self.stack to be set.")
                        })?;
                        println!("{value}");
                    }
                    Instruction::GlobalSet(index) => {
                        let identifier = &self.identifiers[index as usize];

                        let value = self.stack.pop().ok_or_else(|| { anyhow!(
                            "Error setting global variable. No value found in the self.stack to be set."
                        )})?;
                        let _ = self.globals.push((identifier, value));
                    }
                    Instruction::GlobalGet(index) => {
                        let identifier = self.identifiers[index as usize].as_str();

                        let value = environment
                            .iter()
                            .find(|v| v.0 == identifier)
                            .map(|v| v.1.clone())
                            .or(self
                                .globals
                                .iter()
                                .find(|g| g.0 == identifier)
                                .map(|g| g.1.clone()))
                            .ok_or_else(|| anyhow!("Unknown variable {identifier}."))?
                            .clone();

                        self.stack.push(value);
                    }
                    Instruction::LocalGet(index, identifier_index) => {
                        let absolute_index = frame_index + index as usize;
                        if absolute_index >= self.stack.len() {
                            let identifier = &self.identifiers[identifier_index as usize];
                            bail!("Variable {identifier} not found.");
                        }
                        let value = self.stack[absolute_index].clone();
                        self.stack.push(value);
                    }
                    Instruction::If(jump) => {
                        let value = self.stack.pop().ok_or_else(|| {
                            anyhow!("Error in if. No value found in the self.stack to be tested.")
                        })?;

                        if let Value::Bool(b) = *value {
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

                        let mut environment = Vec::new();

                        if let Value::Closure(parent_function, parent_environment) = parent.as_ref()
                        {
                            for captured in &function.captured {
                                let captured = captured.as_str();
                                let index = parent_function
                                    .locals
                                    .iter()
                                    .position(|l| l.name == *captured);

                                if let Some(index) = index {
                                    let absolute_index = frame_index + index as usize;
                                    environment
                                        .push((captured, self.stack[absolute_index].clone()));
                                } else {
                                    let captured_in_parent = parent_environment
                                        .iter()
                                        .find(|v| v.0 == captured)
                                        .map(|v| v.1.clone());
                                    if let Some(captured_in_parent) = captured_in_parent {
                                        environment.push((captured, captured_in_parent.clone()));
                                    }
                                }
                            }
                        }

                        let closure = Value::Closure(function, environment);
                        self.stack.push(Rc::new(closure));
                    }
                    Instruction::Call(arity) => {
                        let closure_index = self.stack.len() - 1 - arity as usize;
                        let closure = &self.stack[closure_index];
                        let closure = closure.clone();

                        if let Value::Closure(function, _) = *closure {
                            if function.arity != arity {
                                bail!("Attempted to call function with wrong number of arguments.");
                            }

                            if arity == 1 {
                                let last_argument = &self.stack[self.stack.len() - 1];
                                if let Value::Integer(i) = **last_argument {
                                    if let Some((_, memoized)) =
                                        self.memoization.iter().find(|m| m.0 == (function.index, i))
                                    {
                                        self.stack.truncate(self.stack.len() - 2);
                                        self.stack.push(memoized.clone());
                                        continue;
                                    }

                                    self.current_execution = Some((function.index, i));
                                }
                            }

                            let current_frame = self
                                .call_frames
                                .last_mut()
                                .expect("There is at least one active call frame at all times.");

                            current_frame.instruction_pointer = instruction_pointer;

                            let new_frame = CallFrame {
                                bytecode: &function.bytecode,
                                closure,
                                instruction_pointer: 0,
                                frame_index: self.stack.len() - arity as usize,
                            };
                            self.call_frames.push(new_frame);

                            break;
                        } else {
                            bail!("Attempted to call value that is not a function!");
                        }
                    }
                    Instruction::TailCall(arity) => {
                        let closure_index = self.stack.len() - 1 - arity as usize;
                        let closure = &self.stack[closure_index];
                        let closure = closure.clone();

                        if let Value::Closure(function, _) = *closure {
                            if function.arity != arity {
                                bail!("Attempted to call function with wrong number of arguments.");
                            }

                            if arity == 1 {
                                let last_argument = &self.stack[self.stack.len() - 2];
                                if let Value::Integer(i) = **last_argument {
                                    if let Some((_, memoized)) =
                                        self.memoization.iter().find(|m| m.0 == (function.index, i))
                                    {
                                        self.stack.truncate(self.stack.len() - 2);
                                        self.stack.push(memoized.clone());
                                        continue;
                                    }

                                    self.current_execution = Some((function.index, i));
                                }
                            }

                            let current_frame = self
                                .call_frames
                                .last_mut()
                                .expect("There is at least one active call frame at all times.");

                            current_frame.instruction_pointer = instruction_pointer;

                            let last_frame = self
                                .call_frames
                                .pop()
                                .expect("A tail call can only exist within another function");

                            let kept: Vec<Rc<Value<'_>>> = self
                                .stack
                                .drain(self.stack.len() - arity as usize - 1..)
                                .collect();

                            let locals_to_remove = match *last_frame.closure {
                                Value::Closure(f, _) => f.locals.len(),
                                _ => unreachable!(),
                            };

                            self.stack.truncate(self.stack.len() - locals_to_remove - 1);

                            self.stack.extend(kept);

                            let new_frame = CallFrame {
                                bytecode: &function.bytecode,
                                closure,
                                instruction_pointer: 0,
                                frame_index: self.stack.len() - arity as usize,
                            };
                            self.call_frames.push(new_frame);

                            break;
                        } else {
                            bail!("Attempted to call value that is not a function!");
                        }
                    }
                    Instruction::Return(arity) => {
                        let result = self.stack.pop().expect("Function must have a return value");

                        if let Some(execution) = self.current_execution {
                            if self.pure {
                                self.memoization.push((execution, result.clone()));
                            }
                        };

                        self.current_execution = None;

                        for _ in 0..arity + 1 {
                            self.stack.pop();
                        }

                        self.stack.push(result);
                        self.call_frames.pop();

                        break;
                    }
                }
            }
        }

        let value = self.stack.last().expect(
            "At the end of the execution, there must be at least one value in the self.stack.",
        );

        Ok(value.as_ref().into())
    }
}
