use anyhow::{bail, Context, Result};
use serde_json::from_reader;
use std::{env::args, fs};

use rvm::{ast::File, vm::Vm};

fn main() -> Result<()> {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        bail!("Usage: rvm <filepath>.");
    }

    let path = &args[1];
    let file = fs::File::open(path)?;
    let contents: File =
        from_reader(file).context("File does not contain a JSON representation of the AST.")?;
    let vm = Vm::new(contents.name, contents.expression);
    Ok(())
}
