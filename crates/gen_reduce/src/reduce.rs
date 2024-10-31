use crate::{Commands, Reduction, Rule};
use uniplate::Uniplate;

// TODO: (Felix) dirty/clean optimisation: replace tree with a custom tree structure,
//               which contains the original tree and adds metadata fields?

// TODO: (Felix) add logging via `log` crate; possibly need tree type to be Debug?
//               could be a crate feature?

// TODO: (Felix) add "control" rules; e.g. ignore a subtree to a certain depth
//               test by ignoring everything once a metadata field is set? e.g. "reduce until contains X"

pub fn reduce<T, M, F>(transform: F, mut tree: T, mut meta: M) -> (T, M)
where
    T: Uniplate,
    F: Fn(&mut Commands<T, M>, &T, &M) -> Option<T>,
{
    // Apply the transformation to the tree until no more changes are made
    while let Some(mut reduction) = reduce_iteration(&transform, &tree, &meta) {
        // Apply rule side-effects and set the current tree to the new one
        (tree, meta) = reduction.commands.apply(reduction.new_tree, meta);
    }
    (tree, meta)
}

fn reduce_iteration<T, M, F>(transform: &F, subtree: &T, meta: &M) -> Option<Reduction<T, M>>
where
    T: Uniplate,
    F: Fn(&mut Commands<T, M>, &T, &M) -> Option<T>,
{
    // Try to apply the transformation to the current node
    let reduction = Reduction::apply_transform(transform, subtree, meta);
    if reduction.is_some() {
        return reduction;
    }

    // Try to call the transformation on the children of the current node
    // If successful, return the new subtree
    let mut children = subtree.children();
    for c in children.iter_mut() {
        if let Some(reduction) = reduce_iteration(transform, c, meta) {
            *c = reduction.new_tree;
            return Some(Reduction {
                new_tree: subtree.with_children(children),
                ..reduction
            });
        }
    }

    None
}

pub fn reduce_with_rules<T, M, R, S>(rules: &[R], select: S, tree: T, meta: M) -> (T, M)
where
    T: Uniplate,
    R: Rule<T, M> + 'static,
    S: Fn(&T, &mut dyn Iterator<Item = (&R, Reduction<T, M>)>) -> Option<Reduction<T, M>>,
{
    reduce(
        |commands, subtree, meta| {
            let selection = select(
                subtree,
                &mut rules.iter().filter_map(|rule| {
                    Reduction::apply_transform(|c, t, m| rule.apply(c, t, m), subtree, meta)
                        .map(|r| (rule, r))
                }),
            );
            selection.map(|r| {
                // Ensure commands used by the engine are the ones resulting from this reduction
                *commands = r.commands;
                r.new_tree
            })
        },
        tree,
        meta,
    )
}
