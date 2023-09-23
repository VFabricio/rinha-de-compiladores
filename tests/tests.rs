use anyhow::Result;

use rvm::{value::FinalValue, vm::Vm};

fn compile_and_assert(program: &str, assert: impl Fn(Result<FinalValue>) -> ()) {
    let mut vm = Vm::new();
    let result = vm.interpret("test", program);
    assert(result);
}

#[test]
fn single_int() {
    compile_and_assert("42", |result| {
        assert_eq!(result.unwrap(), FinalValue::Integer(42));
    })
}

#[test]
fn algebra() {
    compile_and_assert("(12 - 5/2) * 4 + 19 % 4", |result| {
        assert_eq!(result.unwrap(), FinalValue::Integer(43));
    })
}

#[test]
fn division_by_zero() {
    compile_and_assert("(12 - 5/2) * 4 / (-3 + 3)", |result| {
        assert!(result.is_err());
    })
}

#[test]
fn remainder_by_zero() {
    compile_and_assert("(12 - 5/2) * 4 % (-3 + 3)", |result| {
        assert!(result.is_err());
    })
}

#[test]
fn true_works() {
    compile_and_assert("true", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(true));
    })
}

#[test]
fn false_works() {
    compile_and_assert("false", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(false));
    })
}

#[test]
fn gt_works() {
    compile_and_assert("2 > 1", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(true));
    });

    compile_and_assert("1 > 2", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(false));
    });
}

#[test]
fn lt_works() {
    compile_and_assert("1 < 2", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(true));
    });

    compile_and_assert("2 < 1", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(false));
    });
}

#[test]
fn gte_works() {
    compile_and_assert("2 >= 1", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(true));
    });

    compile_and_assert("1 >= 2", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(false));
    });

    compile_and_assert("1 >= 1", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(true));
    });
}

#[test]
fn lte_works() {
    compile_and_assert("1 <= 2", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(true));
    });

    compile_and_assert("2 <= 1", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(false));
    });

    compile_and_assert("1 <= 1", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(true));
    });
}

#[test]
fn eq_works() {
    compile_and_assert("42 == 42", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(true));
    });

    compile_and_assert("false == false", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(true));
    });

    compile_and_assert("42 == 0", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(false));
    });

    compile_and_assert("true == false", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(false));
    });

    compile_and_assert("true == 42", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(false));
    });
}

#[test]
fn neq_works() {
    compile_and_assert("42 != 0", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(true));
    });

    compile_and_assert("false != true", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(true));
    });

    compile_and_assert("42 != 42", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(false));
    });

    compile_and_assert("false != false", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(false));
    });

    compile_and_assert("true != 42", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(true));
    });
}

#[test]
fn and_works() {
    compile_and_assert("true && true", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(true));
    });

    compile_and_assert("false && false", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(false));
    });

    compile_and_assert("true && false", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(false));
    });

    compile_and_assert("false && true", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(false));
    });

    compile_and_assert("42 && false", |result| {
        assert!(result.is_err());
    });
}

#[test]
fn or_works() {
    compile_and_assert("true || true", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(true));
    });

    compile_and_assert("false || false", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(false));
    });

    compile_and_assert("true || false", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(true));
    });

    compile_and_assert("false || true", |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(true));
    });

    compile_and_assert("42 || false", |result| {
        assert!(result.is_err());
    });
}

#[test]
fn str_literal_works() {
    compile_and_assert(r#" "test" "#, |result| {
        assert_eq!(result.unwrap(), FinalValue::String("test".to_owned()));
    });
}

#[test]
fn string_concatenation() {
    compile_and_assert(r#" "foo" + "bar" "#, |result| {
        assert_eq!(result.unwrap(), FinalValue::String("foobar".to_owned()));
    });

    compile_and_assert(r#" 42 + "bar" "#, |result| {
        assert_eq!(result.unwrap(), FinalValue::String("42bar".to_owned()));
    });

    compile_and_assert(r#" "foo" + 42 "#, |result| {
        assert_eq!(result.unwrap(), FinalValue::String("foo42".to_owned()));
    });
}

#[test]
fn tuple_works() {
    compile_and_assert(r#" (42, (false, "foo")) "#, |result| {
        assert_eq!(
            result.unwrap(),
            FinalValue::Tuple(
                Box::new(FinalValue::Integer(42)),
                Box::new(FinalValue::Tuple(
                    Box::new(FinalValue::Bool(false)),
                    Box::new(FinalValue::String("foo".to_owned()))
                ))
            )
        );
    });
}

#[test]
fn first_works() {
    compile_and_assert(r#" first((42, true)) "#, |result| {
        assert_eq!(result.unwrap(), FinalValue::Integer(42));
    });

    compile_and_assert(r#" first("foo") "#, |result| {
        assert!(result.is_err());
    });
}

#[test]
fn second_works() {
    compile_and_assert(r#" second((42, true)) "#, |result| {
        assert_eq!(result.unwrap(), FinalValue::Bool(true));
    });

    compile_and_assert(r#" second("foo") "#, |result| {
        assert!(result.is_err());
    });
}

#[test]
fn globals_work() {
    compile_and_assert(
        r#"
        let foo = 42;
        let bar = true;
        (bar, foo)
    "#,
        |result| {
            assert_eq!(
                result.unwrap(),
                FinalValue::Tuple(
                    Box::new(FinalValue::Bool(true)),
                    Box::new(FinalValue::Integer(42))
                )
            );
        },
    );
}

#[test]
fn print_works() {
    compile_and_assert(r#" print((true, 42)) "#, |result| {
        assert_eq!(
            result.unwrap(),
            FinalValue::Tuple(
                Box::new(FinalValue::Bool(true)),
                Box::new(FinalValue::Integer(42))
            )
        );
    });
}

#[test]
fn if_works() {
    compile_and_assert(
        r#"
        if (42 > 0) {
            let a = "foo";
            a + "bar"
        } else {
            1
        }
    "#,
        |result| {
            assert_eq!(result.unwrap(), FinalValue::String("foobar".to_owned()));
        },
    );

    compile_and_assert(
        r#"
        if (42 < 0) {
            let a = "foo";
            a + "bar"
        } else {
            let b = 1;
            b + b
        }
    "#,
        |result| {
            assert_eq!(result.unwrap(), FinalValue::Integer(2));
        },
    );
}

#[test]
fn sum() {
    compile_and_assert(
        r#"
            let sum = fn (n) => {
              if (n == 1) {
                n
              } else {
                n + sum(n - 1)
              }
            };

            print (sum(5))
            "#,
        |result| {
            assert_eq!(result.unwrap(), FinalValue::Integer(15));
        },
    );
}

#[test]
fn combination() {
    compile_and_assert(
        r#"
            let combination = fn (n, k) => {
                let a = k == 0;
                let b = k == n;
                if (a || b)
                {
                    1
                }
                else {
                    combination(n - 1, k - 1) + combination(n - 1, k)
                }
            };

            print(combination(10, 2))
        "#,
        |result| {
            assert_eq!(result.unwrap(), FinalValue::Integer(45));
        },
    );
}

#[test]
fn fibonacci() {
    compile_and_assert(
        r#"
            let fib = fn (n) => {
              if (n < 2) {
                n
              } else {
                fib(n - 1) + fib(n - 2)
              }
            };

            print(fib(10))
        "#,
        |result| {
            assert_eq!(result.unwrap(), FinalValue::Integer(55));
        },
    );
}
