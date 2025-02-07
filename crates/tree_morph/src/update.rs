use crate::commands::Commands;
use uniplate::Uniplate;

/// Represents the effects of a successful rule application, including the full
/// new tree and any side-effects
///
///
pub struct Update<T, M>
where
    T: Uniplate,
{
    pub new_tree: T,
    pub(crate) commands: Commands<T, M>,
}
