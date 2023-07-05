use super::*;

impl FabricArtifacts {
    /// get fabric artifacts
    pub async fn new() -> Self {
        reqwest::get("https://meta.fabricmc.net/v2/versions")
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }
}

impl YarnArtifactList {
    /// get yarn artifacts
    pub async fn new() -> Self {
        reqwest::get("https://meta.fabricmc.net/v2/versions/yarn")
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }
    /// get the yarn of the specified minecraft version
    pub async fn from_mcversion(mcversion: &str) -> Self {
        reqwest::get(format!(
            "https://meta.fabricmc.net/v2/versions/yarn/{}",
            mcversion
        ))
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
    }
}

impl LoaderArtifactList {
    /// get loader artifacts
    pub async fn new() -> Self {
        reqwest::get("https://meta.fabricmc.net/v2/versions/loader")
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }
    /// get the loader of the specified minecraft version
    pub async fn from_mcversion(mcversion: &str) -> Self {
        reqwest::get(format!(
            "https://meta.fabricmc.net/v2/versions/loader/{}",
            mcversion
        ))
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
    }
}

impl FabricLoaderArtifact {
    /// get fabric loader artifact
    pub async fn new(mcversion: &str, loader: &str) -> Self {
        reqwest::get(format!(
            "https://meta.fabricmc.net/v2/versions/loader/{}/{}",
            mcversion, loader
        ))
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
    }
}
