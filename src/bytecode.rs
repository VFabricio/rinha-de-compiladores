#[derive(Debug)]
pub enum Instruction {
    Constant(u16),
    True,
    False,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Neq,
    Gt,
    Lt,
    Gte,
    Lte,
    And,
    Or,
    Tuple,
    First,
    Second,
    Print,
    GlobalGet(u16),
    GlobalSet(u16),
    If(u32),
    Jump(u32),
}
