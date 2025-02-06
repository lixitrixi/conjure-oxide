//! Here we test the `reduce_with_rule_groups` function.
//! Each rule group is applied to the whole tree as with `reduce_with_rules`, before the next group is tried.
//! Every time a change is made, the algorithm starts again with the first group.
//!
//! This lets us make powerful "evaluation" rules which greedily reduce the tree as much as possible, before other
//! "rewriting" rules are applied.

use tree_morph::prelude::*;
use uniplate::derive::Uniplate;

/// A simple language of two literals and a wrapper
#[derive(Debug, Clone, PartialEq, Eq, Uniplate)]
#[uniplate()]
enum Expr {
    A,               // a
    B,               // b
    Wrap(Box<Expr>), // [E]
}

/// [a] ~> a
fn rule_unwrap_a(_: &mut Commands<Expr, ()>, expr: &Expr, _: &()) -> Option<Expr> {
    if let Expr::Wrap(inner) = expr {
        if let Expr::A = **inner {
            return Some(Expr::A);
        }
    }
    None
}

/// a ~> b
fn rule_a_to_b(_: &mut Commands<Expr, ()>, expr: &Expr, _: &()) -> Option<Expr> {
    if let Expr::A = expr {
        return Some(Expr::B);
    }
    None
}

#[test]
fn test_same_group() {
    // If the rules are in the same group, unwrap_a will apply higher in the tree

    // [a]
    let expr = Expr::Wrap(Box::new(Expr::A));

    let (result, _) = morph(
        vec![vec![rule_fn!(rule_unwrap_a), rule_fn!(rule_a_to_b)]],
        select_first,
        expr,
        (),
    );

    // [a] ~> a ~> b
    assert_eq!(result, Expr::B);
}

#[test]
fn test_a_to_b_first() {
    // a_to_b is in a higher group than unwrap_a, so it will be applied first to the lower expression

    // [a]
    let expr = Expr::Wrap(Box::new(Expr::A));

    let (result, _) = morph(
        vec![vec![rule_fn!(rule_a_to_b)], vec![rule_fn!(rule_unwrap_a)]],
        select_first,
        expr,
        (),
    );

    // [a] ~> [b]
    assert_eq!(result, Expr::Wrap(Box::new(Expr::B)));
}
