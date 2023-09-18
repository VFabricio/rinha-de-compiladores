use anyhow::{bail, Context, Result};
use std::{env::args, fs, io::read_to_string};

use rvm::vm::Vm;

fn main() -> Result<()> {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        bail!("Usage: rvm <filepath>.");
    }

    let path = &args[1];
    let file = fs::File::open(path)?;
    let contents: String = read_to_string(file).context("Could not read file.")?;

    let mut vm = Vm::new();
    let result = vm.interpret(path, &contents)?;

    println!("{}", result);

    Ok(())
}
