use tokio::fs::{self, create_dir_all};

use crate::core::{folder::MinecraftLocation, version::Version};

use super::DEFAULT_META_URL;

pub async fn install_quilt_version(
    mcversion: &str,
    quilt_version: &str,
    minecraft: MinecraftLocation,
    remote: Option<String>,
) {
    let remote = remote.unwrap_or(DEFAULT_META_URL.to_string());
    let url = format!("{remote}/v3/versions/loader/{mcversion}/{quilt_version}/profile/json");

    let response = reqwest::get(url).await.unwrap();

    let quilt_version: Version = response.json().await.unwrap();

    let version_name = quilt_version.id.clone();

    let json_path = minecraft.get_version_json(&version_name);
    println!("{:?}", json_path);
    // let libraries = quilt_version.libraries.clone().unwrap();
    // let hashed = libraries.iter().find(|l| match l["name"].as_str() {
    //     None => false,
    //     Some(name) => name.starts_with("org.quiltmc:hashed"),
    // });

    create_dir_all(json_path.parent().unwrap()).await.unwrap();
    fs::write(
        json_path,
        serde_json::to_string_pretty(&quilt_version).unwrap(),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn test() {
    let mcversion = "1.19.3";
    let quilt_version = "0.19.1";
    let minecraft = MinecraftLocation::new("test");
    install_quilt_version(mcversion, quilt_version, minecraft, None).await;
}
