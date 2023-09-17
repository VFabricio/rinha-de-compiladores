use anyhow::Result;

use crate::ast::Term;

pub struct Vm {
    filename: String,
    term: Term,
}

impl Vm {
    pub fn new(filename: String, term: Term) -> Self {
        Self { filename, term }
    }

    pub fn run(self) -> Result<()> {
        Ok(())
    }
}
