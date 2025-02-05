use crate::commands::Commands;
use uniplate::Uniplate;

pub trait Rule<T: Uniplate, M> {
    /// Applies the rule to the given subtree and returns the result if applicable.
    ///
    /// Any side-effects are encapsulated in the `Commands` object passed by `tree_morph`.
    fn apply(&self, commands: &mut Commands<T, M>, subtree: &T, meta: &M) -> Option<T>;
}

// Allows the user to pass closures and function pointers directly as rules
impl<T: Uniplate, M> Rule<T, M> for dyn Fn(&mut Commands<T, M>, &T, &M) -> Option<T> {
    fn apply(&self, commands: &mut Commands<T, M>, subtree: &T, meta: &M) -> Option<T> {
        (self)(commands, subtree, meta)
    }
}
