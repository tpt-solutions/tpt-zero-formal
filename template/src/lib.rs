#![no_std]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

//! {{description}}
//!
//! Part of the [tpt-zero-formal](https://github.com/tpt-solutions/tpt-zero-formal)
//! ecosystem.

/// Example function — replace with the crate's real API.
pub fn example() -> u32 {
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_is_zero() {
        assert_eq!(example(), 0);
    }
}
