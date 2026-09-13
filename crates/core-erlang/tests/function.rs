#[test]
fn function() {
    assert_eq!(
        core_erlang::FunctionName {
            name: "gwah".into(),
            arity: 2
        }
        .to_string(),
        "'gwah'/2"
    );
}

#[test]
fn module() {
    use core_erlang::{Expr, Literal, Module};

    assert_eq!(
        Module::builder("math_utils")
            .attribute("author", Literal::Atom("ream".into()))
            .export("double", 1)
            .function(
                "double",
                1,
                Expr::fun(
                    vec!["N"],
                    Expr::call(
                        Expr::atom("erlang"),
                        Expr::atom("*"),
                        vec![Expr::var("N"), Expr::int(2)],
                    ),
                ),
            )
            .build()
            .to_string(),
        "module 'math_utils' ['double'/1]
  attributes ['author' = 'ream']

'double'/1 = fun (N) ->
    call 'erlang':'*'(N, 2)

end"
    );
}
