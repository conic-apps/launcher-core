use futures::StreamExt;
use once_cell::sync::Lazy;
use reqwest::Client;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

#[derive(Debug)]
pub struct Download {
    pub url: String,
    pub file: String,
}

static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| Client::new());

async fn download(download_task: Download) {
    // todo: 尝试从服务器获取文件大，超过5mb分片下载
    let file_path = PathBuf::from(&download_task.file);
    let direction = file_path.parent().unwrap();
    if !direction.exists() {
        fs::create_dir_all(&direction).unwrap()
    }
    let mut response = HTTP_CLIENT.get(&download_task.url).send().await.unwrap();
    let mut file = tokio::fs::File::create(&download_task.file).await.unwrap();
    while let Some(chunk) = response.chunk().await.unwrap() {
        file.write_all(&chunk).await.unwrap();
    }
}

pub fn filter_existing_files() {}

pub async fn download_files(download_tasks: Vec<Download>, filter_existing: bool) {
    // todo: 已存在文件使用线程池验证哈希，然后修改原有的下载列表
    let task_count = Arc::new(AtomicUsize::new(download_tasks.len()));
    let stream = futures::stream::iter(download_tasks)
        .map(|download_task| {
            let task_count = Arc::clone(&task_count);
            async move {
                let result = download(download_task).await;
                task_count.fetch_sub(1, Ordering::SeqCst);
                result
            }
        })
        .buffer_unordered(16);
    stream
        .for_each_concurrent(1, |_| async {
            println!("还剩{}个", task_count.load(Ordering::SeqCst));
        })
        .await;
    if task_count.load(Ordering::SeqCst) == 0 {
        println!("完成力")
    }
}

#[tokio::test]
async fn test_download_files() {
    let mut download_tasks = vec![];
    for i in 0..200 {
        download_tasks.push(Download {
            url: "https://speed.hetzner.de/100MB.bin".to_string(),
            file: format!("/home/CD-DVD/test/test-{}.bin", i + 1),
        });
    }
    download_files(download_tasks, false).await;
}
