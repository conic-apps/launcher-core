use super::*;

pub async fn get_fabric_artifacts() -> FabricArtifacts {
    reqwest::get("https://meta.fabricmc.net/v2/versions")
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

pub async fn get_yarn_artifact_list() -> Vec<FabricArtifactVersion> {
    reqwest::get("https://meta.fabricmc.net/v2/versions/yarn")
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

pub async fn get_yarn_artifact_list_for(minecraft: &str) -> Vec<FabricArtifactVersion> {
    reqwest::get(format!(
        "https://meta.fabricmc.net/v2/versions/yarn/{}",
        minecraft
    ))
    .await
    .unwrap()
    .json()
    .await
    .unwrap()
}

pub async fn get_loader_artifact_list() -> Vec<FabricArtifactVersion> {
    reqwest::get("https://meta.fabricmc.net/v2/versions/loader")
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

pub async fn get_loader_artifact_list_for(minecraft: &str) -> Vec<FabricLoaderArtifact> {
    reqwest::get(format!(
        "https://meta.fabricmc.net/v2/versions/loader/{}",
        minecraft
    ))
    .await
    .unwrap()
    .json()
    .await
    .unwrap()
}

pub async fn get_fabric_loader_artifact(minecraft: &str, loader: &str) -> FabricLoaderArtifact {
    reqwest::get(format!(
        "https://meta.fabricmc.net/v2/versions/loader/{}/{}",
        minecraft, loader
    ))
    .await
    .unwrap()
    .json()
    .await
    .unwrap()
}
