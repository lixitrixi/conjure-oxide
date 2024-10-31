use uniplate::Uniplate;

use crate::{Reduction, Rule};

pub fn select_first<T, M, R>(
    _: &T,
    rs: &mut dyn Iterator<Item = (&R, Reduction<T, M>)>,
) -> Option<Reduction<T, M>>
where
    T: Uniplate,
    R: Rule<T, ()>,
{
    rs.next().map(|(_, r)| r)
}
