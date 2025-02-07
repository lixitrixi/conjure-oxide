use tree_morph::prelude::*;
use uniplate::derive::Uniplate;

#[derive(Debug, Clone, PartialEq, Eq, Uniplate)]
#[uniplate()]
enum Expr {
    A,
    B,
    C,
    D,
}

#[test]
fn test_closure_rules() {
    let expr = Expr::A;

    let (result, _) = morph(
        vec![
            rule_fns![|_, t, _| match t {
                Expr::A => Some(Expr::B),
                _ => None,
            }],
            rule_fns![
                |_, t, _| match t {
                    Expr::B => Some(Expr::C),
                    _ => None,
                },
                |_, t, _| match t {
                    Expr::C => Some(Expr::D),
                    _ => None,
                }
            ],
        ],
        select_first,
        expr,
        (),
    );

    assert_eq!(result, Expr::D);
}
