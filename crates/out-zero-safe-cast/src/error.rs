use core::fmt;

/// Error returned when a numeric cast cannot preserve the source value
/// exactly (out of range, fractional truncation, or non-finite input).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CastError {
    from: &'static str,
    to: &'static str,
}

impl CastError {
    /// Creates a new [`CastError`] naming the source and target types.
    #[must_use]
    pub const fn new(from: &'static str, to: &'static str) -> Self {
        Self { from, to }
    }

    /// The name of the source type of the failed cast.
    #[must_use]
    pub const fn from_type(&self) -> &'static str {
        self.from
    }

    /// The name of the target type of the failed cast.
    #[must_use]
    pub const fn to_type(&self) -> &'static str {
        self.to
    }
}

impl fmt::Display for CastError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "cannot safely cast {} to {}: value is not exactly representable",
            self.from, self.to
        )
    }
}

impl core::error::Error for CastError {}
