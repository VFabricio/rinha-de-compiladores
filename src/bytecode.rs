#[derive(Debug)]
pub enum Instruction {
    Constant(u16),
    True,
    False,
    Add,
    Sub,
    Mul,
    Div,
}
