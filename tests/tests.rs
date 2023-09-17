use rvm::{ast::Term, value::Value, vm::Vm};

#[test]
fn single_int() {
    let term = Term::Int(42);

    let mut vm = Vm::new("test".into(), term);
    let result = vm.interpret();
    assert_eq!(result.unwrap(), Value::Integer(0));
}
