use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{format, fs, io::copy, println, vec};

use crate::utils::folder::MinecraftLocation;

#[derive(Debug, Deserialize)]
pub struct FabricArtifactVersion {
    pub gameVersion: Option<String>,
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
pub struct FabricLoaderArtifact {
    pub loader: FabricArtifactVersion,
    pub intermediary: FabricArtifactVersion,
    pub launcherMeta: LauncherMeta,
}

#[derive(Debug, Deserialize)]
pub struct LauncherMeta {
    pub version: usize,
    pub libraries: LauncherMetaLibraries,
    pub mainClass: Value,
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

/// 根据 yarn 和 loader 生成 fabric 版本的 JSON 文件到磁盘中。
pub async fn install_fabric(
    loader: FabricLoaderArtifact,
    minecraft_location: MinecraftLocation,
    options: FabricInstallOptions,
) -> String {
    let yarn: Option<String>;
    let side = options.size.unwrap_or(FabricInstallSide::Client);
    let mut id = options.version_id;
    let mut minecraft_version = "".to_string();

    match options.yarn_version {
        Some(yarn_version) => match yarn_version {
            YarnVersion::String(yarn_version) => {
                yarn = Some(yarn_version);
            }
            YarnVersion::FabricArtifactVersion(yarn_version) => {
                yarn = Some(yarn_version.version);
            }
        },
        None => {
            yarn = None;
            minecraft_version = loader.intermediary.version;
        }
    }
    if let None = id {
        if let Some(yarn) = yarn.clone() {
            id = Some(format!(
                "{}-loader{}",
                minecraft_version, loader.loader.version
            ));
        } else {
            id = Some(format!(
                "{}-fabric{}",
                minecraft_version, loader.loader.version
            ))
        }
    }
    let mut libraries = vec![
        LauncherMetaLibrariesItems {
            name: Some(loader.loader.maven.clone()),
            url: Some(String::from("https://maven.fabricmc.net/")),
        },
        LauncherMetaLibrariesItems {
            name: Some(loader.intermediary.maven.clone()),
            url: Some(String::from("https://maven.fabricmc.net/")),
        },
    ];
    if let Some(yarn) = yarn.clone() {
        libraries.push(LauncherMetaLibrariesItems {
            name: Some(format!("net.fabricmc:yarn:{}", yarn)),
            url: Some(String::from("https://maven.fabricmc.net/")),
        });
    }
    libraries.extend(loader.launcherMeta.libraries.common.iter().cloned());
    match side {
        FabricInstallSide::Client => {
            libraries.extend(loader.launcherMeta.libraries.client.iter().cloned())
        }
        FabricInstallSide::Server => {
            libraries.extend(loader.launcherMeta.libraries.server.iter().cloned())
        }
    }
    let main_class = match side {
        FabricInstallSide::Client => loader.launcherMeta.mainClass["client"]
            .as_str()
            .unwrap_or(loader.launcherMeta.mainClass.as_str().unwrap_or(""))
            .to_string(),
        FabricInstallSide::Server => loader.launcherMeta.mainClass["server"]
            .as_str()
            .unwrap_or(loader.launcherMeta.mainClass.as_str().unwrap_or(""))
            .to_string(),
    };
    let inherits_from = options.inherits_from.unwrap_or(minecraft_version);

    let json_file_path = minecraft_location.get_version_json(&id.clone().unwrap());
    fs::create_dir_all(json_file_path.parent().unwrap()).unwrap();
    if let Ok(metadata) = fs::metadata(&json_file_path) {
        if metadata.is_file() {
            fs::remove_file(&json_file_path).unwrap();
        } else {
            fs::remove_dir_all(&json_file_path).unwrap();
        }
    }
    #[derive(Serialize)]
    struct FabricVersionJSON {
        id: String,
        inheritsFrom: String,
        mainClass: String,
        libraries: String,
        arguments: FabricVersionJSONArg,
        releaseTime: String,
        time: String,
    }
    #[derive(Serialize)]
    struct FabricVersionJSONArg {
        game: Vec<i32>,
        jvm: Vec<i32>,
    }
    let version_json = FabricVersionJSON {
        id: id.clone().unwrap_or("".to_string()),
        inheritsFrom: inherits_from,
        mainClass: main_class,
        libraries: serde_json::to_string(&libraries).unwrap_or("".to_string()),
        arguments: FabricVersionJSONArg {
            game: vec![],
            jvm: vec![],
        },
        releaseTime: "2023-05-13T15:58:54.493Z".to_string(),
        time: "2023-05-13T15:58:54.493Z".to_string(),
    };
    let json_data = serde_json::to_string_pretty(&version_json)
        .unwrap_or("".to_string())
        .to_string();
    tokio::fs::write(json_file_path, json_data).await.unwrap();

    id.unwrap_or("".to_string())
}

#[tokio::test]
async fn test() {
    // let b = get_loader_artifact_list().await;
    let a = get_fabric_loader_artifact("1.19.4", "0.1.0.48").await;
    let options = FabricInstallOptions {
        inherits_from: None,
        version_id: None,
        size: None,
        yarn_version: None
    };
    let location = MinecraftLocation::new("test");
    // println!("{:#?}",a);
        install_fabric(a, location, options).await;
}
