use rvm::{ast::Term, vm::Vm};

#[test]
fn single_int() {
    let term = Term::Int(42);

    let vm = Vm::new("test".into(), term);
    let result = vm.run();
    assert!(result.is_ok());
}
