//! `codewarden-core` – Core analysis and security engine for CodeWarden.
//!
//! This crate contains all the business logic, analysis algorithms and data
//! types shared across the CodeWarden toolchain. The CLI and any future

pub mod diff;
pub mod scan;

/// Returns the library version string at compile time.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_not_empty() {
        assert!(!version().is_empty());
    }
}
