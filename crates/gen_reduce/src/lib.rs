//! # Gen_reduce
//!
//! **A generic reduction engine for recursive data types.**
//!
//! This library provides methods which, given a tree and a set of node-to-node transformation rules,
//! repeatedly rewrites parts of the tree until no more rules can be applied.
//!
//! ## A Simple Example
//!
//! ***Adapted from (Mitchell and Runciman 2007)***
//!
//! Below is an example using a "calculator" language. The engine allows us to reduce the expression to a simplified form.
//!
//! ```rust
//! use gen_reduce::*;
//! use uniplate::derive::Uniplate;
//!
//! #[derive(Debug, Clone, PartialEq, Eq, Uniplate)]
//! enum Expr {
//!     Add(Box<Expr>, Box<Expr>),
//!     Mul(Box<Expr>, Box<Expr>),
//!     Neg(Box<Expr>),
//!     Val(i32),
//!     Var(String),
//! }
//!
//! enum ReductionRule {
//!     Eval,       // Evaluate constant expressions
//!
//!     AddZero,   // a + 0 ~> a
//!     AddSame,   // a + a ~> 2 * a
//!     MulOne,    // a * 1 ~> a
//!     MulZero,   // a * 0 ~> 0
//!     DoubleNeg, // -(-a) ~> a
//!
//!     Associativity, // Define a consistent form: a */+ (b */+ c) ~> (a */+ b) */+ c
//!                    // This rule also mixes things up for (a, b) to be tested by other rules
//! }
//!
//! impl Rule<Expr, ()> for ReductionRule {
//!     fn apply(&self, cmd: &mut Commands<Expr, ()>, expr: &Expr, _: &()) -> Option<Expr> {
//!         use ReductionRule::*;
//!         use Expr::*;
//!
//!         match self {
//!             AddZero => match expr {
//!                 Add(a, b) if matches!(a.as_ref(), Val(0)) => Some(*b.clone()),
//!                 Add(a, b) if matches!(b.as_ref(), Val(0)) => Some(*a.clone()),
//!                 _ => None,
//!             },
//!             AddSame => match expr {
//!                 Add(a, b) if a == b => Some(Mul(bx(Val(2)), a.clone())),
//!                 _ => None,
//!             },
//!             MulOne => match expr {
//!                 Mul(a, b) if matches!(a.as_ref(), Val(1)) => Some(*b.clone()),
//!                 Mul(a, b) if matches!(b.as_ref(), Val(1)) => Some(*a.clone()),
//!                 _ => None,
//!             },
//!             MulZero => match expr {
//!                 Mul(a, b) if matches!(a.as_ref(), Val(0)) ||
//!                     matches!(b.as_ref(), Val(0)) => Some(Val(0)),
//!                 _ => None,
//!             },
//!             DoubleNeg => match expr {
//!                 Neg(a) => match a.as_ref() {
//!                     Neg(b) => Some(*b.clone()),
//!                     _ => None,
//!                 },
//!                 _ => None,
//!             },
//!             Eval => match expr {
//!                 Add(a, b) => match (a.as_ref(), b.as_ref()) {
//!                     (Val(x), Val(y)) => Some(Val(x + y)),
//!                     _ => None,
//!                 },
//!                 Mul(a, b) => match (a.as_ref(), b.as_ref()) {
//!                     (Val(x), Val(y)) => Some(Val(x * y)),
//!                     _ => None,
//!                 },
//!                 Neg(a) => match a.as_ref() {
//!                     Val(x) => Some(Val(-x)),
//!                     _ => None,
//!                 },
//!                 _ => None,
//!             },
//!            Associativity => match expr {
//!                 Add(a, b) => match (a.as_ref(), b.as_ref()) {
//!                     (x, Add(y, z)) => Some(Add(bx(Add(a.clone(), y.clone())), z.clone())),
//!                     _ => None,
//!                 },
//!                 Mul(a, b) => match (a.as_ref(), b.as_ref()) {
//!                     (x, Mul(y, z)) => Some(Mul(bx(Mul(a.clone(), y.clone())), z.clone())),
//!                     _ => None,
//!                 },
//!                 _ => None,
//!             },
//!         }
//!     }
//! }
//!
//! // (-(-x) + ((x * 1) + 0)) + ((1 + 1) * x)   ~>   4 * x
//! fn main() {
//!     use Expr::*;
//!     use ReductionRule::*;
//!
//!     let expr = Add(
//!         bx(Add(
//!             bx(Neg(
//!                 bx(Neg(
//!                     bx(Var("x".to_string())),
//!                 )),
//!             )),
//!             bx(Add(
//!                 bx(Mul(
//!                     bx(Var("x".to_string())),
//!                     bx(Val(1)),
//!                 )),
//!                 bx(Val(0)),
//!             )),
//!         )),
//!         bx(Mul(
//!             bx(Add(
//!                 bx(Val(1)),
//!                 bx(Val(1))
//!             )),
//!             bx(Var("x".to_string())),
//!         )),
//!     );
//!
//!     // Ordering is important here: we evaluate first (1), then reduce (2..6), then change form (7)
//!     let rules = vec![Eval, AddZero, AddSame, MulOne, MulZero, DoubleNeg, Associativity];
//!
//!     let (expr, _) = reduce(rules, expr, ());
//!     assert_eq!(expr, Mul(bx(Val(4)), bx(Var("x".to_string()))));
//! }
//!
//! fn bx(expr: Expr) -> Box<Expr> {
//!     Box::new(expr)
//! }
//! ```
//!
//! ## Recommendations
//!
//! Defining rules as an enum can quickly lead to massive match statements.
//! To avoid this, consider instead using a struct containing a function pointer.
//! These functions can then be defined elsewhere for better organization.
//!

mod commands;
mod reduce;
mod rule;

pub use commands::Commands;
pub use reduce::reduce;
pub use rule::Rule;

#[cfg(test)]
mod tests {
    use super::*;
    use uniplate::derive::Uniplate;

    #[derive(Debug, Clone, PartialEq, Eq, Uniplate)]
    #[uniplate()]
    enum Expr {
        Add(Box<Expr>, Box<Expr>),
        Mul(Box<Expr>, Box<Expr>),
        Val(i32),
    }

    enum ReductionRule {
        AddZero,
        MulOne,
        Eval,
    }

    impl Rule<Expr, ()> for ReductionRule {
        fn apply(&self, _: &mut Commands<Expr, ()>, expr: &Expr, _: &()) -> Option<Expr> {
            use Expr::*;
            use ReductionRule::*;

            match self {
                AddZero => match expr {
                    Add(a, b) if matches!(a.as_ref(), Val(0)) => Some(*b.clone()),
                    Add(a, b) if matches!(b.as_ref(), Val(0)) => Some(*a.clone()),
                    _ => None,
                },
                MulOne => match expr {
                    Mul(a, b) if matches!(a.as_ref(), Val(1)) => Some(*b.clone()),
                    Mul(a, b) if matches!(b.as_ref(), Val(1)) => Some(*a.clone()),
                    _ => None,
                },
                Eval => match expr {
                    Add(a, b) => match (a.as_ref(), b.as_ref()) {
                        (Val(x), Val(y)) => Some(Val(x + y)),
                        _ => None,
                    },
                    Mul(a, b) => match (a.as_ref(), b.as_ref()) {
                        (Val(x), Val(y)) => Some(Val(x * y)),
                        _ => None,
                    },
                    _ => None,
                },
            }
        }
    }

    #[test]
    fn test_single_var() {
        let expr = Expr::Val(42);
        let (expr, _) = reduce(vec![ReductionRule::Eval], expr, ());
        assert_eq!(expr, Expr::Val(42));
    }

    #[test]
    fn test_add_zero() {
        let expr = Expr::Add(Box::new(Expr::Val(0)), Box::new(Expr::Val(42)));
        let (expr, _) = reduce(vec![ReductionRule::AddZero], expr, ());
        assert_eq!(expr, Expr::Val(42));
    }

    #[test]
    fn test_mul_one() {
        let expr = Expr::Mul(Box::new(Expr::Val(1)), Box::new(Expr::Val(42)));
        let (expr, _) = reduce(vec![ReductionRule::MulOne], expr, ());
        assert_eq!(expr, Expr::Val(42));
    }

    #[test]
    fn test_eval() {
        let expr = Expr::Add(Box::new(Expr::Val(1)), Box::new(Expr::Val(2)));
        let (expr, _) = reduce(vec![ReductionRule::Eval], expr, ());
        assert_eq!(expr, Expr::Val(3));
    }

    #[test]
    fn test_eval_nested() {
        let expr = Expr::Mul(
            Box::new(Expr::Add(Box::new(Expr::Val(1)), Box::new(Expr::Val(2)))),
            Box::new(Expr::Val(3)),
        );
        let (expr, _) = reduce(vec![ReductionRule::Eval], expr, ());
        assert_eq!(expr, Expr::Val(9));
    }
}
