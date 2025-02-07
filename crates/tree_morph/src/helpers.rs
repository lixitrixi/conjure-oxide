use std::{collections::VecDeque, fmt::Display, io::Write, sync::Arc};

use crate::prelude::*;
use multipeek::multipeek;
use uniplate::Uniplate;

/// Returns the first [`Update`] if the iterator only yields one, otherwise calls `select`.
pub(crate) fn one_or_select<T, M, R>(
    select: impl Fn(&T, &mut dyn Iterator<Item = (&R, Update<T, M>)>) -> Option<Update<T, M>>,
    t: &T,
    rs: &mut dyn Iterator<Item = (&R, Update<T, M>)>,
) -> Option<Update<T, M>>
where
    T: Uniplate,
    R: Rule<T, M>,
{
    let mut rs = multipeek(rs);
    if rs.peek_nth(1).is_none() {
        return rs.next().map(|(_, u)| u);
    }
    select(t, &mut rs)
}

/// Returns the first available [`Update`] if there is one, otherwise returns `None`.
///
/// This is a good default selection strategy, especially when you expect only one possible
/// rule to apply to any one term.
pub fn select_first<T, M, R>(
    _: &T,
    rs: &mut dyn Iterator<Item = (&R, Update<T, M>)>,
) -> Option<Update<T, M>>
where
    T: Uniplate,
    R: Rule<T, M>,
{
    rs.next().map(|(_, u)| u)
}

/// Select the first [`Update`] or panic if there is more than one.
///
/// This is useful when you expect exactly one rule to be applicable in all cases.
pub fn select_first_or_panic<T, M, R>(
    t: &T,
    rs: &mut dyn Iterator<Item = (&R, Update<T, M>)>,
) -> Option<Update<T, M>>
where
    T: Uniplate + std::fmt::Debug,
    R: Rule<T, M> + std::fmt::Debug,
{
    let mut rs = multipeek(rs);
    if rs.peek_nth(1).is_some() {
        let rules = rs.map(|(r, _)| r).collect::<Vec<_>>();
        panic!(
            "Multiple rules applicable to expression {:?}\n{:?}",
            t, rules
        );
    }
    rs.next().map(|(_, u)| u)
}

pub fn select_user_input<T, M, R>(
    t: &T,
    rs: &mut dyn Iterator<Item = (&R, Update<T, M>)>,
) -> Option<Update<T, M>>
where
    T: Uniplate + Display,
    R: Rule<T, M> + Display,
{
    let mut choices: Vec<_> = rs.collect();

    let rules = choices
        .iter()
        .enumerate()
        .map(|(i, (r, Update { new_tree, .. }))| {
            format!(
                "{}. {}
   ~> {}",
                i + 1,
                r,
                new_tree
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    loop {
        print!(
            "--- Current Expression ---
{}

--- Rules ---
{}

---
q   No change
<n> Apply rule n

:",
            t, rules
        );
        std::io::stdout().flush().unwrap(); // Print the : on same line

        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();

        match line.trim() {
            "q" => return None,
            n => {
                if let Ok(n) = n.parse::<usize>() {
                    if n > 0 && n <= choices.len() {
                        let ret = choices.swap_remove(n - 1).1;
                        return Some(ret);
                    }
                }
            }
        }
    }
}

/// Selects a random `Reduction` from the iterator.
pub fn select_random<T, M, R>(
    _: &T,
    rs: &mut dyn Iterator<Item = (&R, Update<T, M>)>,
) -> Option<Update<T, M>>
where
    T: Uniplate,
    R: Rule<T, M>,
{
    use rand::seq::IteratorRandom;
    let mut rng = rand::rng();
    rs.choose(&mut rng).map(|(_, u)| u)
}

/// Selects the `Reduction` which results in the smallest subtree.
///
/// Subtree size is determined by maximum depth.
/// Among trees with the same depth, the first in the iterator order is selected.
pub fn select_smallest_subtree<T, M, R>(
    _: &T,
    rs: &mut dyn Iterator<Item = (&R, Update<T, M>)>,
) -> Option<Update<T, M>>
where
    T: Uniplate,
    R: Rule<T, M>,
{
    rs.min_by_key(|(_, u)| {
        u.new_tree.cata(Arc::new(|_, cs: VecDeque<i32>| {
            // Max subtree height + 1
            cs.iter().max().unwrap_or(&0) + 1
        }))
    })
    .map(|(_, u)| u)
}
