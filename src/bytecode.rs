#[derive(Clone, Debug)]
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
    LocalGet(u16, u16),
    If(u32),
    Jump(u32),
    Closure(u16),
    Call(u16),
    Return(u16),
}
