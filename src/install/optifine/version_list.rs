use serde::{Deserialize, Serialize};

use super::DEFAULT_META_URL;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptifineVersionListItem {
    pub _id: String,
    pub mcversion: String,
    pub patch: String,
    pub r#type: String,
    pub __v: i32,
    pub filename: String,
    pub forge: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptifineVersionList(Vec<OptifineVersionListItem>);

impl OptifineVersionList {
    pub async fn new(mcversion: &str, remote: Option<String>) -> Self {
        let url = match remote {
            Some(remote) => format!("{remote}/{mcversion}"),
            None => format!("{DEFAULT_META_URL}/{mcversion}"),
        };
        reqwest::get(url)
            .await
            .unwrap()
            .json::<OptifineVersionList>()
            .await
            .unwrap()
        // todo: 返回404时会导致解析失败，要返回Err()
    }
}

#[tokio::test]
async fn test() {
    let list = OptifineVersionList::new("1.19.4", None).await;
    println!("{:#?}", list);
}
