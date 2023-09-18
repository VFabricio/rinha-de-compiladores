use anyhow::Result;

use rvm::{value::Value, vm::Vm};

fn interpret(program: &str) -> Result<Value> {
    let vm = Vm::new();
    vm.interpret("test", program)
}

#[test]
fn single_int() {
    let result = interpret("42");
    assert_eq!(result.unwrap(), Value::Integer(42));
}

#[test]
fn algebra() {
    let result = interpret("(12 - 5/2) * 4");
    assert_eq!(result.unwrap(), Value::Integer(40));
}

#[test]
fn division_by_zero() {
    let result = interpret("(12 - 5/2) * 4/(-3 + 3)");
    assert!(result.is_err());
}

#[test]
fn true_works() {
    let result = interpret("true");
    assert_eq!(result.unwrap(), Value::Bool(true));
}

#[test]
fn false_works() {
    let result = interpret("false");
    assert_eq!(result.unwrap(), Value::Bool(false));
}
