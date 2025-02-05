//! # Tree Morph
//!
//! **A generic rule-based term rewriting library.**
//!
//! This library provides methods which, given a tree and a collection of node-to-node transformation rules,
//! repeatedly rewrites parts of the tree until no more rules can be applied.

pub mod commands;
pub mod engine;
pub mod helpers;
pub mod reduction;
pub mod traits;

pub mod prelude {
    use super::*;

    pub use commands::Commands;
    pub use engine::{reduce, reduce_with_rule_groups, reduce_with_rules};
    pub use helpers::select_first;
    pub use reduction::Reduction;
    pub use traits::Rule;
}
