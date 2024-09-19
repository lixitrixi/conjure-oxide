use std::collections::VecDeque;
use uniplate::Uniplate;

pub trait Rule<T, M>
where
    T: Uniplate,
{
    fn apply(&self, commands: &mut Commands<T, M>, subtree: &T, meta: &M) -> Result<T, Error>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// The rule does not apply to the current node, but other rules might.
    NotApplicable,

    /// The current node and its descendants up to a given depth should be ignored.
    /// With a depth of `0`, only the current node is ignored.
    ///
    /// No rules are attempted on the ignored nodes.
    Ignore(u32),

    /// The current node and all its descendants should be ignored.
    /// Equivalent to `Ignore(Infinity)`
    ///
    /// No rules are attempted on the ignored nodes.
    Prune,
}

enum Command<T, M>
where
    T: Uniplate,
{
    Transform(fn(&T) -> T),
    MutMeta(fn(&mut M)),
}

/// A queue of commands to be applied after every successful rule application.
pub struct Commands<T, M>
where
    T: Uniplate,
{
    commands: VecDeque<Command<T, M>>,
}

impl<T, M> Commands<T, M>
where
    T: Uniplate,
{
    pub fn new() -> Self {
        Self {
            commands: VecDeque::new(),
        }
    }

    /// Apply the given transformation to the root node.
    /// Commands are applied in order after the rule is applied.
    pub fn transform(&mut self, f: fn(&T) -> T) {
        self.commands.push_back(Command::Transform(f));
    }

    /// Update the associated metadata.
    /// Commands are applied in order after the rule is applied.
    pub fn mut_meta(&mut self, f: fn(&mut M)) {
        self.commands.push_back(Command::MutMeta(f));
    }

    // Consumes and applies the commands currently in the queue.
    fn apply(&mut self, mut tree: T, mut meta: M) -> (T, M) {
        while let Some(cmd) = self.commands.pop_front() {
            match cmd {
                Command::Transform(f) => tree = f(&tree),
                Command::MutMeta(f) => f(&mut meta),
            }
        }
        (tree, meta)
    }

    fn clear(&mut self) {
        self.commands.clear();
    }
}

// TODO: (Felix) how to allow rewrite selection?
//               add a parameter F: `fn(Iterator<(R, T)>) -> Option<T>`? Vec instead?

// TODO: (Felix) dirty/clean optimisation: replace tree with a custom tree structure,
//               which contains the original tree and adds metadata fields?

/// Continuously apply rules to the tree until no more rules can be applied.
///
/// The tree is traversed top-down, left-to-right.
/// At each node, rules are attempted in the order they are given.
/// If any rule returns a new subtree, that subtree replaces the existing one.
/// If no rules apply, the engine continues further down the tree.
///
/// The command pattern is used to encapsulate side-effects caused by rules.
/// Commands are applied in order after the rule is successfully applied.
/// If a rule fails (returns an `Err`), all commands added by that rule are discarded.
pub fn reduce<T, M, R>(rules: Vec<R>, mut tree: T, mut meta: M) -> (T, M)
where
    T: Uniplate,
    R: Rule<T, M>,
{
    let commands = &mut Commands::new();
    loop {
        match reduce_iteration(commands, &rules, &tree, &meta, 0) {
            Some(new_tree) => {
                // Apply rule side-effects and set the current tree to the new one
                (tree, meta) = commands.apply(new_tree, meta);
            }
            None => break,
        }
    }
    (tree, meta)
}

fn reduce_iteration<T, M, R>(
    commands: &mut Commands<T, M>,
    rules: &Vec<R>,
    subtree: &T,
    meta: &M,
    mut ignore_depth: u32,
) -> Option<T>
where
    T: Uniplate,
    R: Rule<T, M>,
{
    use Error::*;

    if ignore_depth == 0 {
        // Try to apply rules to the current node
        for rule in rules {
            match rule.apply(commands, subtree, meta) {
                Ok(new_tree) => return Some(new_tree),
                Err(err) => {
                    commands.clear(); // Side effects are discarded
                    match err {
                        NotApplicable => continue,
                        Ignore(d) => {
                            ignore_depth = d + 1; // d == 0 -> ignore just this node
                            break;
                        }
                        Prune => return None,
                    }
                }
            }
        }
    }

    // Recursively apply rules to the children and return the updated subtree
    let mut children = subtree.children();
    for i in 0..children.len() {
        if let Some(new_child) = reduce_iteration(
            commands,
            rules,
            &children[i],
            meta,
            if ignore_depth > 0 {
                ignore_depth - 1
            } else {
                0
            },
        ) {
            children[i] = new_child;
            return Some(subtree.with_children(children));
        }
    }

    None
}

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
        fn apply(&self, _: &mut Commands<Expr, ()>, expr: &Expr, _: &()) -> Result<Expr, Error> {
            use ReductionRule::*;
            match self {
                AddZero => match expr {
                    Expr::Add(a, b) if matches!(a.as_ref(), Expr::Val(0)) => Ok(*b.clone()),
                    Expr::Add(a, b) if matches!(b.as_ref(), Expr::Val(0)) => Ok(*a.clone()),
                    _ => Err(Error::NotApplicable),
                },
                MulOne => match expr {
                    Expr::Mul(a, b) if matches!(a.as_ref(), Expr::Val(1)) => Ok(*b.clone()),
                    Expr::Mul(a, b) if matches!(b.as_ref(), Expr::Val(1)) => Ok(*a.clone()),
                    _ => Err(Error::NotApplicable),
                },
                Eval => match expr {
                    Expr::Add(a, b) => match (a.as_ref(), b.as_ref()) {
                        (Expr::Val(x), Expr::Val(y)) => Ok(Expr::Val(x + y)),
                        _ => Err(Error::NotApplicable),
                    },
                    Expr::Mul(a, b) => match (a.as_ref(), b.as_ref()) {
                        (Expr::Val(x), Expr::Val(y)) => Ok(Expr::Val(x * y)),
                        _ => Err(Error::NotApplicable),
                    },
                    _ => Err(Error::NotApplicable),
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
