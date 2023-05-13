use std::{
    print, println,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use crate::utils::{download, versions::VersionManifest};
use serde_json as JSON;
use tokio::io::AsyncWriteExt;
use JSON::Value;

use reqwest::Url;

use crate::utils::{
    download::{download_files, Download},
    folder::{get_path, MinecraftLocation},
    versions::{self, Artifact, AssetIndexDownload},
};

#[derive(Clone)]
pub enum TaskState {
    Idle,
    Running,
    Cancelled,
    Paused,
    Succeed,
    Failed,
}

#[derive(Clone)]
/// 用来保存任务执行的状态
pub struct Task {
    name: String,
    total: Arc<AtomicUsize>,
    progress: Arc<AtomicUsize>,
    path: String,
    state: TaskState,
}

impl Task {
    pub fn new(name: &str) -> Task {
        Task {
            name: name.to_string(),
            total: Arc::new(AtomicUsize::new(0)),
            progress: Arc::new(AtomicUsize::new(0)),
            path: "".to_string(),
            state: TaskState::Idle,
        }
    }
}

// todo: 把所有生成下载列表用的东西放进impl里

fn generate_libraries_download_list(
    libraries: Vec<Artifact>,
    minecraft_location: &MinecraftLocation,
) -> Vec<Download> {
    let mut download_list = Vec::with_capacity(libraries.len());
    for library in libraries {
        download_list.push(Download {
            url: format!("https://download.mcbbs.net/maven/{}", library.path),
            file: get_path(&minecraft_location.libraries.join(library.path)),
        })
    }
    download_list
}

pub async fn generate_assets_download_list(
    assets_index: AssetIndexDownload,
    minecraft_location: &MinecraftLocation,
) -> Vec<Download> {
    let assets_index_url = Url::parse(&assets_index.url).unwrap();
    let assets_index_data = reqwest::get(assets_index_url)
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let assets_index_json: Value = serde_json::from_str(&assets_index_data).unwrap();
    let assets_index_object = assets_index_json["objects"].as_object().unwrap();
    let mut assets: Vec<download::Download> = Vec::with_capacity(assets_index_object.len());
    let mut index = 0;
    assets.push(download::Download {
        url: assets_index.url.to_string(),
        file: get_path(
            &minecraft_location
                .assets
                .join("indexes")
                .join(format!("{}.json", assets_index.id)),
        ),
    });
    for key in assets_index_object.keys() {
        let source = "https://download.mcbbs.net/assets";
        let hash = assets_index_json["objects"][key]["hash"].as_str().unwrap();
        let hash_first_two = &hash[0..2];
        assets.push(download::Download {
            url: format!("{}/{}/{}", source, hash_first_two, hash),
            file: get_path(
                &minecraft_location
                    .assets
                    .join("objects")
                    .join(hash_first_two)
                    .join(hash),
            ),
        });
    }
    assets
}

async fn install(version_id: &str, minecraft_location: MinecraftLocation, task: Task) {
    let versions = VersionManifest::new().await.versions;
    let version_metadata = versions
        .into_iter()
        .filter(|v| v.id == version_id)
        .next()
        .unwrap();
    let version_json_data = reqwest::get(version_metadata.url)
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let version = versions::Version::new(&version_json_data);
    let version_json_path = minecraft_location
        .versions
        .join(format!("{}/{}.json", version.id, version.id));
    tokio::fs::create_dir_all(version_json_path.parent().unwrap())
        .await
        .unwrap();
    let mut file = tokio::fs::File::create(&version_json_path).await.unwrap();
    file.write_all(version_json_data.as_bytes()).await.unwrap();
    let mut download_list = Vec::new();
    download_list.push(Download {
        url: format!("https://download.mcbbs.net/version/{}/client", version_id),
        file: get_path(
            &minecraft_location
                .versions
                .join(format!("{}/{}.jar", version.id, version.id)),
        ),
    });
    download_list.extend(generate_libraries_download_list(
        version.libraries,
        &minecraft_location,
    ));
    download_list
        .extend(generate_assets_download_list(version.asset_index, &minecraft_location).await);
    task.total.store(download_list.len(), Ordering::SeqCst);
    download_files(
        download_list,
        task.total.load(Ordering::SeqCst),
        task.progress,
        false,
    )
    .await;
}

#[tokio::test]
async fn test() {
    // todo: 支持模组加载器，但fabric和quilt必须事先安装好原版游戏
    // so Optifine, Fuck you
    let task = Task::new("install-game");
    let task_clone = task.clone();
    thread::spawn(move || loop {
        let progress = task_clone.progress.load(Ordering::SeqCst);
        let total = task_clone.total.load(Ordering::SeqCst);
        let percentage = if total == 0 {
            0.0
        } else {
            progress as f64 / total as f64 * 100.0
        };
        println!("{}% {}/{}", format!("{:.2}", percentage), progress, total);
        thread::sleep(Duration::from_micros(50000));
    });
    install("1.19.4", MinecraftLocation::new("test"), task).await;
}
