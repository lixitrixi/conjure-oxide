use crate::commands::Commands;
use uniplate::Uniplate;

/// Represents a successful application of a rule.
///
/// This includes the new whole tree and side-effects to be applied.
pub struct Reduction<T, M>
where
    T: Uniplate,
{
    pub new_tree: T,
    pub(crate) commands: Commands<T, M>,
}
