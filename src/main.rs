use anyhow::{bail, Context, Result};
use std::{env::args, fs, io::read_to_string};

use rvm::vm::Vm;

fn main() -> Result<()> {
    let args: Vec<String> = args().collect();

    let path = match args.len() {
        1 => "/var/rinha/source.rinha",
        2 => &args[1],
        _ => bail!("Usage: rvm <filepath>."),
    };

    let file = fs::File::open(path)?;
    let contents: String = read_to_string(file).context("Could not read file.")?;

    let mut vm = Vm::new();
    let _result = vm.interpret(path, &contents)?;

    //println!("{}", result);

    Ok(())
}
