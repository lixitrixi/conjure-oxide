use gen_reduce::*;
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
