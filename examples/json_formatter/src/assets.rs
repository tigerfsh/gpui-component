use gpui::{AssetSource, Result, SharedString};
use std::borrow::Cow;

/// Asset source for the JSON formatter application
pub struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        // For now, we're relying on cargo-bundle to include the icon files
        // in the final application bundle, rather than embedding them in the binary
        Ok(None)
    }

    fn list(&self, _path: &str) -> Result<Vec<SharedString>> {
        // We don't need to list any embedded assets since we're using cargo-bundle
        // to handle icon files
        Ok(Vec::new())
    }
}