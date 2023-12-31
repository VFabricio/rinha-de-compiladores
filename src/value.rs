use std::{
    cmp::{Eq, PartialEq},
    convert::From,
    fmt,
    rc::Rc,
};

use crate::function::Function;

#[derive(Clone)]
pub enum Value<'a> {
    Bool(bool),
    Integer(i32),
    String(String),
    Tuple(Box<Rc<Value<'a>>>, Box<Rc<Value<'a>>>),
    Closure(&'a Function, Vec<(&'a str, Rc<Value<'a>>)>),
}

impl<'a> fmt::Debug for Value<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "Bool({b})"),
            Value::Integer(i) => write!(f, "Integer({i})"),
            Value::String(s) => write!(f, "String({s})"),
            Value::Tuple(t1, t2) => write!(f, "Tuple({t1:?}, {t2:?})"),
            Value::Closure(fun, _) => write!(f, "Closure({})", fun.index),
        }
    }
}

impl<'a> fmt::Display for Value<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{b}"),
            Value::Integer(i) => write!(f, "{i}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Tuple(t1, t2) => write!(f, "({t1}, {t2})"),
            Value::Closure { .. } => write!(f, "<#closure>"),
        }
    }
}

impl<'a> PartialEq for Value<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Bool(b1), Value::Bool(b2)) => b1 == b2,
            (Value::Integer(i1), Value::Integer(i2)) => i1 == i2,
            (Value::String(s1), Value::String(s2)) => s1 == s2,
            (Value::Tuple(v1, v2), Value::Tuple(v3, v4)) => v1 == v3 && v2 == v4,
            _ => false,
        }
    }
}

impl<'a> Eq for Value<'a> {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FinalValue {
    Bool(bool),
    Integer(i32),
    String(String),
    Tuple(Box<FinalValue>, Box<FinalValue>),
    Closure,
}

impl<'a> From<&'a Value<'a>> for FinalValue {
    fn from(value: &'a Value<'a>) -> Self {
        match value {
            Value::Bool(b) => Self::Bool(*b),
            Value::Integer(i) => Self::Integer(*i),
            Value::String(s) => Self::String(s.clone()),
            Value::Tuple(v1, v2) => {
                Self::Tuple(Box::new((&***v1).into()), Box::new((&***v2).into()))
            }
            Value::Closure(_, _) => Self::Closure,
        }
    }
}
