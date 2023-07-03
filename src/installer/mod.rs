use reqwest::Url;
use serde_json::Value;
use tokio::io::AsyncWriteExt;

use crate::{
    core::{
        task::Callbacks,
        version::{
            self, AssetIndex, AssetIndexObject, ResolvedLibraries, ResolvedVersion, VersionManifest,
        },
    },
    utils::{
        download::{download_files, Download},
        folder::{get_path, MinecraftLocation},
    },
};
// todo: crate::core::task里面放 Task 结构体，把Future放进去

pub mod fabric;
pub mod forge;
pub mod quilt;

fn generate_libraries_download_list(
    libraries: ResolvedLibraries,
    minecraft_location: &MinecraftLocation,
) -> Vec<Download> {
    libraries
        .clone()
        .into_iter()
        .map(|library| Download {
            url: format!("https://download.mcbbs.net/maven/{}", library.path),
            file: get_path(&minecraft_location.libraries.join(library.path)),
        })
        .collect()
}

async fn generate_assets_download_list(
    asset_index: AssetIndex,
    minecraft_location: &MinecraftLocation,
) -> Vec<Download> {
    let asset_index_url = Url::parse(&asset_index.url).unwrap();
    let asset_index_raw = reqwest::get(asset_index_url)
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let asset_index_json: Value = serde_json::from_str(&asset_index_raw).unwrap();
    let asset_index_object: AssetIndexObject =
        serde_json::from_value(asset_index_json["objects"].clone()).unwrap();
    let mut assets: Vec<_> = asset_index_object
        .into_iter()
        .map(|obj| Download {
            url: format!(
                "https://download.mcbbs.net/assets/{}/{}",
                &obj.1.hash[0..2],
                obj.1.hash
            ),
            file: get_path(
                &minecraft_location
                    .assets
                    .join("objects")
                    .join(&obj.1.hash[0..2])
                    .join(&obj.1.hash),
            ),
        })
        .collect();
    assets.push(Download {
        url: asset_index.url,
        file: get_path(
            &minecraft_location
                .assets
                .join("indexes")
                .join(format!("{}.json", asset_index.id)),
        ),
    });
    assets
}

pub async fn install_dependencies(
    version: ResolvedVersion,
    minecraft_location: MinecraftLocation,
    callbacks: Callbacks,
) {
    let mut download_list = Vec::new();
    download_list.extend(generate_libraries_download_list(
        version.libraries,
        &minecraft_location,
    ));
    download_list.extend(
        generate_assets_download_list(version.asset_index.unwrap(), &minecraft_location).await,
    );
    download_files(download_list, callbacks).await;
}

pub async fn install(
    version_id: &str,
    minecraft_location: MinecraftLocation,
    callbacks: Callbacks,
) {
    let versions = VersionManifest::new().await.versions;
    let version_metadata: Vec<_> = versions
        .into_iter()
        .filter(|v| v.id == version_id)
        .collect();
    if version_metadata.len() != 1 {
        panic!("Bad version manifest!!!")
    };
    let version_metadata = version_metadata.get(0).unwrap();
    println!("{:#?}", version_metadata); // todo: 感觉没有获取到正确的版本所以打印出来看看是不是要求安装的，回来必须删掉

    let version_json_raw = reqwest::get(version_metadata.url.clone())
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let version = version::Version::from_str(&version_json_raw)
        .unwrap()
        .parse(minecraft_location.clone());
    let id = &version.id;

    let version_json_path = minecraft_location.versions.join(format!("{id}/{id}.json"));
    tokio::fs::create_dir_all(version_json_path.parent().unwrap())
        .await
        .unwrap();
    let mut file = tokio::fs::File::create(&version_json_path).await.unwrap();
    file.write_all(version_json_raw.as_bytes()).await.unwrap();

    let mut download_list = vec![];
    download_list.push(Download {
        url: format!("https://download.mcbbs.net/version/{version_id}/client"),
        file: get_path(&minecraft_location.versions.join(format!("{id}/{id}.jar"))),
    });

    download_list.extend(generate_libraries_download_list(
        version.libraries,
        &minecraft_location,
    ));
    download_list.extend(
        generate_assets_download_list(version.asset_index.unwrap(), &minecraft_location).await,
    );

    download_files(download_list, callbacks).await
}

#[tokio::test]
async fn test() {
    let a = Box::new(|completed, total| {
        println!("{}/{}", completed, total);
    });
    let cb = Callbacks::new().on_progress(a);
    install("1.20", MinecraftLocation::new("test"), cb).await;
    // let minecraft_location = MinecraftLocation::new("test");
    // let raw = read_to_string(minecraft_location.versions.clone().join("1.20").join("1.20.json")).unwrap();
    // let version = crate::core::version::Version::from_str(&raw).unwrap().parse(minecraft_location.clone());
    // install_dependencies(version, a, minecraft_location.clone()).await;
    // c.task.await;
}
