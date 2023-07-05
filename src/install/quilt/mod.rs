use serde::{Deserialize, Serialize};

pub mod install;
pub mod version_list;

const DEFAULT_META_URL: &str = "https://meta.quiltmc.org";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QuiltArtifactVersion {
    separator: String,
    build: u32,

    /// e.g. "org.quiltmc.quilt-loader:0.16.1"
    maven: String,
    version: String,
}
