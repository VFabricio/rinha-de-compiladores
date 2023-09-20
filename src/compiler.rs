use anyhow::{anyhow, bail, Result};
use rinha::ast::{BinaryOp, Term};

use crate::{
    bytecode::Instruction,
    function::{Function, Local},
    value::Value,
    vm::Vm,
};

pub struct Compiler<'a> {
    parent: Option<&'a Compiler<'a>>,
    bytecode: Vec<Instruction>,
    locals: Vec<Local>,
}

impl<'a> Compiler<'a> {
    pub fn new(parent: Option<&'a Compiler<'a>>) -> Self {
        Self {
            parent,
            bytecode: Vec::new(),
            locals: Vec::new(),
        }
    }
    pub fn compile(&mut self, term: Term, vm: &mut Vm) -> Result<Vec<Instruction>> {
        match term {
            Term::Int(i) => {
                let value = Value::Integer(i.value);
                let index = vm.create_constant(value)?;

                self.bytecode.push(Instruction::Constant(index));
            }
            Term::Bool(b) => {
                let instruction = if b.value {
                    Instruction::True
                } else {
                    Instruction::False
                };
                self.bytecode.push(instruction);
            }
            Term::Str(s) => {
                let value = Value::String(s.value);
                let index = vm.create_constant(value)?;

                self.bytecode.push(Instruction::Constant(index));
            }
            Term::Binary(b) => {
                self.compile(*b.lhs, vm)?;
                self.compile(*b.rhs, vm)?;

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
                self.bytecode.push(instruction);
            }
            Term::Tuple(t) => {
                self.compile(*t.first, vm)?;
                self.compile(*t.second, vm)?;

                self.bytecode.push(Instruction::Tuple);
            }
            Term::First(t) => {
                self.compile(*t.value, vm)?;

                self.bytecode.push(Instruction::First);
            }
            Term::Second(t) => {
                self.compile(*t.value, vm)?;

                self.bytecode.push(Instruction::Second);
            }
            Term::Let(t) => {
                self.compile(*t.value, vm)?;

                let index = vm.create_identifier(t.name.text.clone())?;

                if self.parent.is_some() {
                    self.locals.push(Local { name: t.name.text });
                } else {
                    self.bytecode.push(Instruction::GlobalSet(index));
                }

                self.compile(*t.next, vm)?;
            }
            Term::Var(t) => {
                let identifier_index = vm.create_identifier(t.text.clone())?;

                let local_index = self.locals.iter().position(|l| l.name == t.text);
                if let Some(index) = local_index {
                    self.bytecode
                        .push(Instruction::LocalGet(index as u16, identifier_index));
                } else {
                    self.bytecode.push(Instruction::GlobalGet(identifier_index));
                }
            }
            Term::Print(t) => {
                self.compile(*t.value, vm)?;
                self.bytecode.push(Instruction::Print);
            }
            Term::If(t) => {
                self.compile(*t.condition, vm)?;
                self.bytecode.push(Instruction::If(0));

                let if_address = self.bytecode.len() - 1;
                let if_address = if if_address > i32::MAX as usize {
                    bail!("Instruction too long.");
                } else {
                    if_address as u32
                };

                self.compile(*t.then, vm)?;
                self.bytecode.push(Instruction::Jump(0));

                let jump_address = self.bytecode.len() - 1;
                let jump_address = if jump_address > i32::MAX as usize {
                    bail!("Instruction too long.");
                } else {
                    jump_address as u32
                };

                self.bytecode[if_address as usize] = Instruction::If(jump_address - if_address);

                self.compile(*t.otherwise, vm)?;
                let after_address = self.bytecode.len() - 1;
                let after_address = if after_address > i32::MAX as usize {
                    bail!("Instruction too long.");
                } else {
                    after_address as u32
                };

                self.bytecode[jump_address as usize] =
                    Instruction::Jump(after_address - jump_address);
            }
            Term::Function(mut f) => {
                let mut compiler = Compiler::new(Some(self));

                let arity = f.parameters.len() as u16;

                f.parameters.reverse();
                for parameter in f.parameters {
                    compiler.locals.push(Local {
                        name: parameter.text,
                    });
                }

                let bytecode = compiler.compile(*f.value, vm)?;

                let function = Function { arity, bytecode };
                vm.functions.push(function);

                self.bytecode
                    .push(Instruction::Closure(vm.functions.len() as u16 - 1));
            }
            Term::Call(c) => {
                self.compile(*c.callee, vm)?;

                let arity = c.arguments.len() as u16;

                for argument in c.arguments {
                    self.compile(argument, vm)?;
                }

                self.bytecode.push(Instruction::Call(arity))
            }
            Term::Error(e) => bail!(anyhow!(e.message)),
        };
        Ok(self.bytecode.clone())
    }
}
