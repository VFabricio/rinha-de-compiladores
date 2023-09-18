use rvm::{
    ast::{Operation, Term},
    value::Value,
    vm::Vm,
};

#[test]
fn single_int() {
    let term = Term::Int(42);

    let mut vm = Vm::new();
    let result = vm.interpret("test".into(), term);
    assert_eq!(result.unwrap(), Value::Integer(42));
}

#[test]
fn sum() {
    let term = Term::Binary {
        op: Operation::Add,
        lhs: Box::new(Term::Int(2)),
        rhs: Box::new(Term::Int(3)),
    };

    let mut vm = Vm::new();
    let result = vm.interpret("test".into(), term);
    assert_eq!(result.unwrap(), Value::Integer(5));
}
