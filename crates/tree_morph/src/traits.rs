use crate::commands::Commands;
use uniplate::Uniplate;

/// Defines tree rewriting behaviour for a type.
///
/// TODO:
pub trait Rule<T: Uniplate, M> {
    /// Applies the rule to the given subtree and returns the result if applicable.
    ///
    /// Any side-effects are encapsulated in the `Commands` object passed by `tree_morph`.
    fn apply(&self, commands: &mut Commands<T, M>, subtree: &T, meta: &M) -> Option<T>;
}

// Allows the user to pass closures and function pointers directly as rules
impl<T, M, F> Rule<T, M> for F
where
    T: Uniplate,
    F: Fn(&mut Commands<T, M>, &T, &M) -> Option<T>,
{
    fn apply(&self, commands: &mut Commands<T, M>, subtree: &T, meta: &M) -> Option<T> {
        (self)(commands, subtree, meta)
    }
}

/// Converts a function pointer to a uniform type which implements `Rule`.
///
/// Function pointers do not have the same type even if they have the same signature,
///     so this provides a way to pass different functions to the engine.
#[macro_export]
macro_rules! rule_fns {
    [$( $exp:expr ),*] => {
        vec![$( $exp as fn(&mut _, &_, &_) -> Option<_>, )*]
    };
}
