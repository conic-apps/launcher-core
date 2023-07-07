/*
 * Magical Launcher Core
 * Copyright (C) 2023 Broken-Deer <old_driver__@outlook.com> and contributors
 *
 * This program is free software, you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use reqwest::Url;
use serde_json::Value;
use tokio::io::AsyncWriteExt;

use crate::core::version::ResolvedLibrary;
use crate::{
    core::{
        folder::{get_path, MinecraftLocation},
        task::TaskEventListeners,
        version::{self, AssetIndex, AssetIndexObject, ResolvedVersion, VersionManifest},
        PlatformInfo,
    },
    utils::download::{download_files, Download},
};
// todo: crate::core::task里面放 Task 结构体，把Future放进去

pub mod fabric;
pub mod forge;
pub mod optifine;
pub mod quilt;

pub(crate) fn generate_libraries_download_list(
    libraries: Vec<ResolvedLibrary>,
    minecraft_location: &MinecraftLocation,
) -> Vec<Download<String>> {
    libraries
        .clone()
        .into_iter()
        .map(|library| Download {
            url: format!("https://download.mcbbs.net/maven/{}", library.artifact.path),
            file: minecraft_location
                .libraries
                .join(library.artifact.path)
                .to_str()
                .unwrap()
                .to_string(),
            sha1: Some(library.artifact.sha1),
        })
        .collect()
}

pub(crate) async fn generate_assets_download_list(
    asset_index: AssetIndex,
    minecraft_location: &MinecraftLocation,
) -> Vec<Download<String>> {
    let asset_index_url = Url::parse((&asset_index.url).as_ref()).unwrap();
    let asset_index_raw = reqwest::get(asset_index_url)
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let asset_index_json: Value = serde_json::from_str((&asset_index_raw).as_ref()).unwrap();
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
            file: minecraft_location
                .assets
                .join("objects")
                .join(&obj.1.hash[0..2])
                .join(&obj.1.hash)
                .to_str()
                .unwrap()
                .to_string(),
            sha1: Some(obj.1.hash),
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
        sha1: None,
    });
    assets
}

pub async fn install_dependencies(
    version: ResolvedVersion,
    minecraft_location: MinecraftLocation,
    listeners: TaskEventListeners,
) {
    let mut download_list = Vec::new();
    download_list.extend(generate_libraries_download_list(
        version.libraries,
        &minecraft_location,
    ));
    download_list.extend(
        generate_assets_download_list(version.asset_index.unwrap(), &minecraft_location).await,
    );
    download_files(download_list, listeners).await;
}

pub async fn install(
    version_id: &str,
    minecraft_location: MinecraftLocation,
    listeners: TaskEventListeners,
) {
    let platform = PlatformInfo::new().await;

    let versions = VersionManifest::new().await.versions;
    let version_metadata: Vec<_> = versions
        .into_iter()
        .filter(|v| v.id == version_id)
        .collect();
    if version_metadata.len() != 1 {
        panic!("Bad version manifest!!!")
    };
    let version_metadata = version_metadata.get(0).unwrap();

    let version_json_raw = reqwest::get(version_metadata.url.clone())
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let version = version::Version::from_str(&version_json_raw)
        .unwrap()
        .parse(&minecraft_location, &platform)
        .await;
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
        sha1: None,
    });

    download_list.extend(generate_libraries_download_list(
        version.libraries,
        &minecraft_location,
    ));
    download_list.extend(
        generate_assets_download_list(version.asset_index.unwrap(), &minecraft_location).await,
    );

    download_files(download_list, listeners).await
}

#[tokio::test]
async fn test() {
    // let a = Box::new(|completed, total, step| {
    //     println!("progress: {completed}/{total}  step: {step}");
    // });
    // let cb = EventListeners::new().on_progress(a);
    // install("1.20.1", MinecraftLocation::new("test"), cb).await;
    // let minecraft_location = MinecraftLocation::new("test");
    // let raw = read_to_string(minecraft_location.versions.clone().join("1.20").join("1.20.json")).unwrap();
    // let version = crate::core::version::Version::from_str(&raw).unwrap().parse(minecraft_location.clone());
    // install_dependencies(version, a, minecraft_location.clone()).await;
    // c.task.await;
}
