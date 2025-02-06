use crate::{helpers::one_or_select, prelude::*};
use uniplate::Uniplate;

/// TODO: docs
fn morph_impl<T, M, F>(transforms: Vec<F>, mut tree: T, mut meta: M) -> (T, M)
where
    T: Uniplate,
    F: Fn(&T, &M) -> Option<Reduction<T, M>>,
{
    let mut new_tree = tree;
    'main: loop {
        tree = new_tree;
        for transform in transforms.iter() {
            // Try each transform on the entire tree before moving to the next
            for (node, ctx) in tree.contexts() {
                let red_opt = transform(&node, &meta);

                if let Some(mut red) = red_opt {
                    (new_tree, meta) = red.commands.apply(ctx(red.new_tree), meta);

                    // Restart with the first transform every time a change is made
                    continue 'main;
                }
            }
        }
        // All transforms were attempted without change
        break;
    }
    (tree, meta)
}

/// TODO: docs & example
pub fn morph<T, M, R, S>(groups: Vec<Vec<R>>, select: S, tree: T, meta: M) -> (T, M)
where
    T: Uniplate,
    R: Rule<T, M>,
    S: Fn(&T, &mut dyn Iterator<Item = (&R, Reduction<T, M>)>) -> Option<Reduction<T, M>>,
{
    let transforms: Vec<_> = groups
        .iter()
        .map(|group| {
            |subtree: &T, meta: &M| {
                let applicable = &mut group.iter().filter_map(|rule| {
                    let mut commands = Commands::new();
                    let new_tree = rule.apply(&mut commands, &subtree, &meta)?;
                    Some((rule, Reduction { new_tree, commands }))
                });
                one_or_select(&select, subtree, applicable)
            }
        })
        .collect();
    morph_impl(transforms, tree, meta)
}
