use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Value {
    Integer(i32),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{}", i),
        }
    }
}
