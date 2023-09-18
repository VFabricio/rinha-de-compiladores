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
    let result = interpret("(12 - 5/2) * 4 + 19 % 4");
    assert_eq!(result.unwrap(), Value::Integer(43));
}

#[test]
fn division_by_zero() {
    let result = interpret("(12 - 5/2) * 4 / (-3 + 3)");
    assert!(result.is_err());
}

#[test]
fn remainder_by_zero() {
    let result = interpret("(12 - 5/2) * 4 % (-3 + 3)");
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

#[test]
fn gt_works() {
    let result = interpret("2 > 1");
    assert_eq!(result.unwrap(), Value::Bool(true));

    let result2 = interpret("1 > 2");
    assert_eq!(result2.unwrap(), Value::Bool(false));
}

#[test]
fn lt_works() {
    let result = interpret("1 < 2");
    assert_eq!(result.unwrap(), Value::Bool(true));

    let result2 = interpret("2 < 1");
    assert_eq!(result2.unwrap(), Value::Bool(false));
}

#[test]
fn gte_works() {
    let result = interpret("2 >= 1");
    assert_eq!(result.unwrap(), Value::Bool(true));

    let result2 = interpret("1 >= 2");
    assert_eq!(result2.unwrap(), Value::Bool(false));

    let result3 = interpret("1 >= 1");
    assert_eq!(result3.unwrap(), Value::Bool(true));
}

#[test]
fn lte_works() {
    let result = interpret("1 <= 2");
    assert_eq!(result.unwrap(), Value::Bool(true));

    let result2 = interpret("2 <= 1");
    assert_eq!(result2.unwrap(), Value::Bool(false));

    let result3 = interpret("1 <= 1");
    assert_eq!(result3.unwrap(), Value::Bool(true));
}
