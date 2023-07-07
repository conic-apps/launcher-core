use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ForgeVersionListItem {
    pub _id: String,
    pub build: u32,
    pub __v: u32,
    pub version: String,
    pub modified: String,
    pub mcversion: String,
    pub files: Vec<ForgeInstallerFile>,
    pub branch: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ForgeInstallerFile {
    pub format: String,
    pub category: String,
    pub hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ForgeVersionList(Vec<ForgeVersionListItem>);

impl ForgeVersionList {
    pub async fn new() -> Self {
        reqwest::get("https://bmclapi2.bangbang93.com/forge/list/0")
            .await
            .unwrap()
            .json::<Self>()
            .await
            .unwrap()
    }

    pub async fn from_mcversion(mcversion: &str) -> Self {
        reqwest::get(format!(
            "https://bmclapi2.bangbang93.com/forge/minecraft/{mcversion}"
        ))
        .await
        .unwrap()
        .json::<Self>()
        .await
        .unwrap()
    }
}
