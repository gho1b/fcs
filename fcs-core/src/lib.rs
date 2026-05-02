mod error;
mod fixed_point;

pub use error::FixedPointError;
pub use fixed_point::{rounding::RoundingMode, DivResult, FixedPoint};
