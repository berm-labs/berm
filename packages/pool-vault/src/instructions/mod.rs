//! Instruction handlers for the cover-pool vault.

pub mod deposit;
pub mod distribute;
pub mod initialize;
pub mod withdraw;

// Glob re-exports are required so Anchor's `#[program]` macro can resolve the
// generated `__client_accounts_*` helper modules. Each handler module also
// defines a `handler` fn (called via its full path), so the globs collide on
// that one name; the collision is benign and explicitly allowed.
#[allow(ambiguous_glob_reexports)]
pub use deposit::*;
#[allow(ambiguous_glob_reexports)]
pub use distribute::*;
#[allow(ambiguous_glob_reexports)]
pub use initialize::*;
#[allow(ambiguous_glob_reexports)]
pub use withdraw::*;
