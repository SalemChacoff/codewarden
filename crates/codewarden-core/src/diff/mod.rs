pub mod file_change;
pub mod hunk;
mod parser;

pub use parser::parse_diff;
