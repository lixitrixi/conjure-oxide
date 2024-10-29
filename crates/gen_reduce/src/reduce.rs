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
pub fn reduce<T, M, R>(rules: Vec<R>, mut tree: T, mut meta: M) -> (T, M)
where
    T: Uniplate,
    R: Rule<T, M>,
{
    let commands = &mut Commands::new();
    loop {
        match reduce_iteration(commands, &rules, &tree, &meta) {
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
) -> Option<T>
where
    T: Uniplate,
    R: Rule<T, M>,
{
    // Try to apply rules to the current node
    for rule in rules {
        match rule.apply(commands, subtree, meta) {
            Some(new_tree) => return Some(new_tree),
            None => commands.clear(), // Side effects are discarded
        }
    }

    // Recursively apply rules to the children and return the updated subtree
    let mut children = subtree.children();
    for i in 0..children.len() {
        if let Some(new_child) = reduce_iteration(commands, rules, &children[i], meta) {
            children[i] = new_child;
            return Some(subtree.with_children(children));
        }
    }

    None
}
