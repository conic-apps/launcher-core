use futures::StreamExt;
use once_cell::sync::Lazy;
use reqwest::Client;
use tokio::fs;
// use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

use crate::core::task::Callbacks;

#[derive(Debug)]
pub struct Download {
    pub url: String,
    pub file: String,
}

static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| Client::new());

pub async fn download(download_task: Download) {
    // todo: 尝试从服务器获取文件大，超过5mb分片下载
    // todo: 错误处理
    let file_path = PathBuf::from(&download_task.file);
    let direction = file_path.parent().unwrap();
    if !direction.exists() {
        fs::create_dir_all(&direction).await.unwrap()
    }
    let mut response = HTTP_CLIENT.get(&download_task.url).send().await.unwrap();
    let mut file = tokio::fs::File::create(&download_task.file).await.unwrap();
    while let Some(chunk) = response.chunk().await.unwrap() {
        file.write_all(&chunk).await.unwrap();
    }
}

pub fn filter_existing_files() {}

pub async fn download_files(download_tasks: Vec<Download>, callbacks: Callbacks) {
    let total = download_tasks.len();
    let counter: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    (callbacks.on_start)();
    let stream = futures::stream::iter(download_tasks)
        .map(|download_task| {
            let counter = Arc::clone(&counter);
            async move {
                let result = download(download_task).await;
                counter.fetch_add(1, Ordering::SeqCst);
                result
            }
        })
        .buffer_unordered(16);
    stream
        .for_each_concurrent(1, |_| async {
            let completed = counter.clone().load(Ordering::SeqCst);
            (callbacks.on_progress)(completed, total);
            //println!("{completed}/{total}");
        })
        .await;
    if counter.load(Ordering::SeqCst) == total {
        (callbacks.on_succeed)();
    }
}

// #[tokio::test]
// async fn test_download_files() {
//     let mut download_tasks = vec![];
//     for i in 0..200 {
//         download_tasks.push(Download {
//             url: "https://speed.hetzner.de/100MB.bin".to_string(),
//             file: format!("/home/CD-DVD/test/test-{}.bin", i + 1),
//         });
//     }
//     download_files(download_tasks, false).await;
// }
