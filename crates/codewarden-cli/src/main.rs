//! `codewarden-cli` – Command-line interface for CodeWarden.
//!
//! This binary crate wires up the CLI layer and delegates all heavy lifting
//! to [`codewarden_core`].

fn main() {
    println!("CodeWarden v{}", codewarden_core::version());
}

#[cfg(test)]
mod tests {
    #[test]
    fn sanity() {
        assert_eq!(2 + 2, 4);
    }
}
