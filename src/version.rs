pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[must_use]
pub fn version_string() -> String {
    format!("{NAME} {VERSION}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_matches_package() {
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
        assert!(!version_string().is_empty());
    }
}
