use crate::{Commands, Rule};
use uniplate::Uniplate;

// TODO: (Felix) how to allow rewrite selection?
//               add a parameter F: `fn(Iterator<(R, T)>) -> Option<T>`? Vec instead?

// TODO: (Felix) dirty/clean optimisation: replace tree with a custom tree structure,
//               which contains the original tree and adds metadata fields?

// TODO: (Felix) add logging and arbitrary error rule error (handled as not applicable, but logged)

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
pub fn reduce<T, M, F>(transform: F, mut tree: T, mut meta: M) -> (T, M)
where
    T: Uniplate,
    F: Fn(&mut Commands<T, M>, &T, &M) -> Option<T>,
{
    let commands = &mut Commands::new();
    loop {
        match reduce_iteration(commands, &transform, &tree, &meta) {
            Some(new_tree) => {
                // Apply rule side-effects and set the current tree to the new one
                (tree, meta) = commands.apply(new_tree, meta);
            }
            None => break,
        }
    }
    (tree, meta)
}

fn reduce_iteration<T, M, F>(
    commands: &mut Commands<T, M>,
    transform: &F,
    subtree: &T,
    meta: &M,
) -> Option<T>
where
    T: Uniplate,
    F: Fn(&mut Commands<T, M>, &T, &M) -> Option<T>,
{
    // Try to apply the transformation to the current node
    match transform(commands, subtree, meta) {
        Some(new_tree) => return Some(new_tree),
        None => commands.clear(), // Side effects are discarded
    }

    // Recursively apply the transformation to the children and return the updated subtree
    let mut children = subtree.children();
    for i in 0..children.len() {
        if let Some(new_child) = reduce_iteration(commands, transform, &children[i], meta) {
            children[i] = new_child;
            return Some(subtree.with_children(children));
        }
    }

    None
}

pub fn reduce_with_rules<T, M, R>(rules: &[R], tree: T, meta: M) -> (T, M)
where
    T: Uniplate,
    R: Rule<T, M>,
{
    reduce(
        |commands, subtree, meta| {
            for rule in rules {
                if let Some(new_tree) = rule.apply(commands, subtree, meta) {
                    return Some(new_tree);
                }
                commands.clear(); // Side effects are discarded
            }
            None
        },
        tree,
        meta,
    )
}
