use std::path::PathBuf;

/// Embeds bytes of a file from the crate's `assets/` directory at compile time.
#[macro_export]
macro_rules! asset_bytes {
    ($file:expr) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $file))
    };
}

/// Resolves filesystem path for an asset from the crate's `assets/` directory at runtime.
#[allow(dead_code)]
pub fn asset_path(file: &str) -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_helpers() {
        assert!(asset_path("app-icon.png").exists());
        let bytes = asset_bytes!("app-icon.png");
        assert!(!bytes.is_empty());
    }
}
