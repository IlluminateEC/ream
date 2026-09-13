pub mod bitstrings;
pub use bitstrings::*;
pub mod expr;
pub use expr::*;
pub mod clauses;
pub use clauses::*;
pub mod patterns;
pub use patterns::*;
pub mod primitives;
pub use primitives::*;
pub mod module;
pub use module::*;

fn stringify_all(items: &[impl ToString]) -> Vec<String> {
    items.iter().map(ToString::to_string).collect()
}
