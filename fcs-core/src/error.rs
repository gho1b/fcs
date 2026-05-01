use std::num::TryFromIntError;
use thiserror::Error;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum FixedPointError {
    #[error("arithmetic overflow")]
    ArithmeticOverflow,

    #[error("invalid scale: got {scale}")]
    InvalidScale { scale: i64 },

    #[error("scale incompatible for {operation}: expected {expected} but got {got}")]
    IncompatibleScale {
        operation: &'static str,
        expected: i64,
        got: i64,
    },

    #[error("non-exact rescale from {from} to {to}")]
    NonExactRescale { from: i64, to: i64 },

    #[error("invalid divisor for {operation}: got {divisor}")]
    InvalidDivisor {
        operation: &'static str,
        divisor: i64,
    },

    #[error(transparent)]
    TryFromIntError(#[from] TryFromIntError),
}
