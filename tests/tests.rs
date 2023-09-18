use anyhow::Result;

use rvm::{value::Value, vm::Vm};

fn interpret(program: &str) -> Result<Value> {
    let mut vm = Vm::new();
    vm.interpret("test", program)
}

#[test]
fn single_int() {
    let result = interpret("42");
    assert_eq!(result.unwrap(), Value::Integer(42));
}

#[test]
fn sum() {
    let result = interpret("2 + 3");
    assert_eq!(result.unwrap(), Value::Integer(5));
}
