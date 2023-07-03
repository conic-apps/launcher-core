use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod version_list;
pub mod install;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabricArtifactVersion {
    pub game_version: Option<String>,
    pub separator: Option<String>,
    pub build: Option<usize>,
    pub maven: String,
    pub version: String,
    pub stable: bool,
}

#[derive(Debug, Deserialize)]
pub struct FabricArtifacts {
    pub mappings: Vec<FabricArtifactVersion>,
    pub loader: Vec<FabricArtifactVersion>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabricLoaderArtifact {
    pub loader: FabricArtifactVersion,
    pub intermediary: FabricArtifactVersion,
    pub launcher_meta: LauncherMeta,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LauncherMeta {
    pub version: usize,
    pub libraries: LauncherMetaLibraries,
    pub main_class: Value,
}

#[derive(Debug, Deserialize)]
pub struct LauncherMetaLibraries {
    pub client: Vec<LauncherMetaLibrariesItems>,
    pub common: Vec<LauncherMetaLibrariesItems>,
    pub server: Vec<LauncherMetaLibrariesItems>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LauncherMetaLibrariesItems {
    pub name: Option<String>,
    pub url: Option<String>,
}

pub enum FabricInstallSide {
    Client,
    Server,
}

pub enum YarnVersion {
    String(String),
    FabricArtifactVersion(FabricArtifactVersion),
}
pub struct FabricInstallOptions {
    /// 当你想要在另一个版本的基础上安装一个版本时。
    pub inherits_from: Option<String>,

    /// 覆盖新安装的版本 id。
    pub version_id: Option<String>,
    pub size: Option<FabricInstallSide>,
    pub yarn_version: Option<YarnVersion>,
}