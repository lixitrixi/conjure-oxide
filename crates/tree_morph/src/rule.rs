use crate::commands::Commands;
use uniplate::Uniplate;

/// Defines behaviour for rewriting a term in an AST.
pub trait Rule<T: Uniplate, M> {
    /// Applies the rule to the given subtree and returns the result if applicable.
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

/// A uniform type for `fn` pointers and closures, which implements the [`Rule`] trait.
///
/// Casting a `fn` pointer or closure to this type allows it to be passed directly to the engine.
/// See the [`rule_fns!`] macro for a convenient way to do this.
pub type RuleFn<T, M> = fn(&mut Commands<T, M>, &T, &M) -> Option<T>;

/// A convenience macro to cast a list of `fn` pointers or closures to a uniform type which implements
/// [`Rule`], to allow these to be passed directly to the engine.
///
/// This makes simple cases less verbose. For more complex use cases with many rules it may better to
/// define your own type which implements [`Rule`] directly.
///
/// # Example
/// ```
/// use tree_morph::prelude::*;
/// use uniplate::derive::Uniplate;
///
/// #[derive(Debug, Clone, PartialEq, Eq, Uniplate)]
/// #[uniplate()]
/// struct Foo;
///
/// fn rule_a(_: &mut Commands<Foo, ()>, _: &Foo, _: &()) -> Option<Foo> {
///     None
/// }
///
/// fn rule_b(_: &mut Commands<Foo, ()>, _: &Foo, _: &()) -> Option<Foo> {
///     None
/// }
///
/// // Closures and fn pointers can be passed directly and even together
/// let rules = vec![
///     rule_fns![rule_a],
///     rule_fns![rule_b, |_, _, _| None],
/// ];
///
/// morph(rules, select_first, Foo, ());
/// ```
#[macro_export]
macro_rules! rule_fns {
    [$($x:expr),+ $(,)?] => {
        vec![$( $x as ::tree_morph::rule::RuleFn<_, _>, )*]
    };
}
