#[derive(Debug)]
pub enum Instruction {
    Constant(u16),
    Add,
    Sub,
    Mul,
    Div,
}
