use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod install;
pub mod version_list;

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

/// Fabric Artifacts
///
/// ### Example
///
/// basic usage:
///
/// ```rust
/// use mgl_core::installer::fabric::FabricArtifacts;
///
/// async fn fn_name() {
///     let artifacts = FabricArtifacts::new().await;
///     println!("{:#?}", artifacts);
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct FabricArtifacts {
    pub mappings: Vec<FabricArtifactVersion>,
    pub loader: Vec<FabricArtifactVersion>,
}

/// Fabric Loader Artifact
///
/// ### Example
///
/// basic usage:
///
/// ```rust
/// use mgl_core::installer::fabric::FabricLoaderArtifact;
///
/// async fn fn_name() {
///     let artifact = FabricLoaderArtifact::new("1.19.4", "0.1.0.48").await;
///     println!("{:#?}", artifact);
/// }
/// ```
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabricLoaderArtifact {
    pub loader: FabricArtifactVersion,
    pub intermediary: FabricArtifactVersion,
    pub launcher_meta: LauncherMeta,
}

/// Yarn Artifacts
///
/// ### Example
///
/// basic usage:
///
/// ```rust
/// use mgl_core::installer::fabric::YarnArtifactList;
/// 
/// async fn fn_name() {
///     let artifacts = YarnArtifactList::new().await;
///     println!("{:#?}", artifacts);
/// }
/// ```
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YarnArtifactList(Vec<FabricArtifactVersion>);

/// Loader Artifacts
///
/// ### Example
///
/// basic usage:
///
/// ```rust
/// use mgl_core::installer::fabric::LoaderArtifactList;
///
/// async fn fn_name() {
///     let artifacts = LoaderArtifactList::new().await;
///     println!("{:#?}", artifacts);
/// }
/// ```
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaderArtifactList(Vec<FabricArtifactVersion>);

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
