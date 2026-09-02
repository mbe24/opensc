//! Game assets, compiled straight into the binary with `include_bytes!`.
//!
//! Embedding (rather than `load_texture(...).await`) keeps native and web builds
//! byte-identical, needs no filesystem or HTTP fetch, and sidesteps asset-path
//! problems under the GitHub Pages `/<repo>/` subpath.

use macroquad::prelude::*;

/// The packed atlas: every sprite as a white silhouette on transparency, so it
/// can be tinted to any colour at draw time (see [`crate::atlas::Sprite`]).
const ATLAS_PNG: &[u8] = include_bytes!("../assets/atlas.png");

/// Handles to every loaded asset. Built once at startup and passed by reference.
pub struct Assets {
    pub atlas: Texture2D,
}

impl Assets {
    /// Decode the embedded PNG. Synchronous and infallible for embedded bytes,
    /// but returns the texture ready to draw with crisp nearest-neighbour scaling.
    #[must_use]
    pub fn load() -> Self {
        let atlas = Texture2D::from_file_with_format(ATLAS_PNG, Some(ImageFormat::Png));
        atlas.set_filter(FilterMode::Nearest);
        Self { atlas }
    }
}
