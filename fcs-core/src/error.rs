use thiserror::Error;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum FixedPointError {
    #[error("arithmetic overflow")]
    ArithmeticOverflow,

    #[error("scale incompatible for {operation}: expected {expected} but got {got}")]
    IncompatibleScale {
        operation: &'static str,
        expected: i64,
        got: i64,
    },

    #[error("invalid divisor for {operation}: got {divisor}")]
    InvalidDivisor {
        operation: &'static str,
        divisor: i64,
    },
}
