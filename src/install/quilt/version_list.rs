use super::{QuiltArtifactVersion, DEFAULT_META_URL};

pub async fn get_quilt_version_list(remote: Option<String>) -> Vec<QuiltArtifactVersion> {
    let remote = match remote {
        None => DEFAULT_META_URL.to_string(),
        Some(remote) => remote,
    };
    let url = format!("{remote}/v3/versions/loader");
    let response = reqwest::get(url).await.unwrap();
    response.json().await.unwrap()
}

#[tokio::test]
async fn test() {
    let version_list = get_quilt_version_list(None).await;
    println!("{:#?}", version_list);
}

#[tokio::test]
async fn test1() {
    let version_list = get_quilt_version_list(Some("https://meta.quiltmc.org".to_string())).await;
    println!("{:#?}", version_list);
}

