//! Error types for the underwriting engine.

use thiserror::Error;

/// Result alias for underwriting operations.
pub type UnderwriteResult<T> = Result<T, UnderwriteError>;

/// Failures surfaced while scoring risk or pricing premiums.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum UnderwriteError {
    /// A factor input fell outside its valid domain.
    #[error("invalid input for `{field}`: {reason}")]
    InvalidInput {
        /// Offending field name.
        field: &'static str,
        /// Why it was rejected.
        reason: String,
    },

    /// The requested coverage exceeds the pool's free (un-utilised) capital.
    #[error("coverage {requested} exceeds available capacity {available}")]
    InsufficientCapacity {
        /// Coverage amount requested.
        requested: u64,
        /// Capital currently available to back new cover.
        available: u64,
    },

    /// The pool's capital adequacy ratio is below the minimum required.
    #[error("capital adequacy {ratio_bps} bps below minimum {min_bps} bps")]
    Undercapitalised {
        /// Observed adequacy ratio in basis points.
        ratio_bps: u64,
        /// Minimum required ratio in basis points.
        min_bps: u64,
    },

    /// Arithmetic overflow while computing a premium or score.
    #[error("arithmetic overflow in `{op}`")]
    Overflow {
        /// Operation that overflowed.
        op: &'static str,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_include_context() {
        let e = UnderwriteError::InsufficientCapacity {
            requested: 100,
            available: 10,
        };
        assert!(e.to_string().contains("100"));
        assert!(e.to_string().contains("10"));
    }
}
