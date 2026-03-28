//! Instruction handlers for the cover executor.

pub mod buy_cover;
pub mod initialize;
pub mod product;
pub mod underwriter;

// Glob re-exports are required so Anchor's `#[program]` macro can resolve the
// generated `__client_accounts_*` helper modules. `buy_cover` and `initialize`
// both define a `handler` fn (called via full path), so the globs collide on
// that one name; the collision is benign and explicitly allowed.
#[allow(ambiguous_glob_reexports)]
pub use buy_cover::*;
#[allow(ambiguous_glob_reexports)]
pub use initialize::*;
#[allow(ambiguous_glob_reexports)]
pub use product::*;
#[allow(ambiguous_glob_reexports)]
pub use underwriter::*;
