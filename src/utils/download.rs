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

use futures::StreamExt;
use once_cell::sync::Lazy;
use reqwest::{Client, Response};
use std::ffi::OsStr;
use tokio::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

use crate::core::task::TaskEventListeners;

use super::sha1::calculate_sha1_from_read;

#[derive(Debug, Clone)]
pub struct Download<P: AsRef<Path> + AsRef<OsStr>> {
    pub url: String,
    pub file: P,
    pub sha1: Option<String>,
}

static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| Client::new());

pub async fn download<P: AsRef<Path> + AsRef<OsStr>>(download_task: Download<P>) -> Response {
    // todo: 尝试从服务器获取文件大，超过5mb分片下载
    // todo: 错误处理
    let file_path = PathBuf::from(&download_task.file);
    let direction = file_path.parent().unwrap();
    if !direction.exists() {
        fs::create_dir_all(&direction).await.unwrap()
    }
    let mut response = HTTP_CLIENT.get(&download_task.url).send().await.unwrap();
    let mut file = fs::File::create(&download_task.file).await.unwrap();
    while let Some(chunk) = response.chunk().await.unwrap() {
        file.write_all(&chunk).await.unwrap();
    }
    response
}

pub async fn download_files(download_tasks: Vec<Download<String>>, listeners: TaskEventListeners) {
    listeners.start();
    listeners.progress(0, 0, 1);
    let download_tasks: Vec<_> = download_tasks
        .iter()
        .filter(|download_task| {
            match std::fs::metadata(&download_task.file) {
                Err(_) => {
                    return true;
                }
                _ => (),
            }
            let mut file = std::fs::File::open(&download_task.file).unwrap();
            let file_sha1 = calculate_sha1_from_read(&mut file);
            let sha1 = match download_task.sha1.clone() {
                None => return true,
                Some(sha1) => sha1,
            };
            if file_sha1 == sha1 {
                false
            } else {
                true
            }
        })
        .collect();
    let total = download_tasks.len();
    let counter: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let stream = futures::stream::iter(download_tasks)
        .map(|download_task| {
            let counter = Arc::clone(&counter);
            async move {
                let result = download(download_task.clone()).await;
                counter.fetch_add(1, Ordering::SeqCst);
                result
            }
        })
        .buffer_unordered(16);
    stream
        .for_each_concurrent(1, |_| async {
            let completed = counter.clone().load(Ordering::SeqCst);
            listeners.progress(completed, total, 2);
            //println!("{completed}/{total}");
        })
        .await;
    if counter.load(Ordering::SeqCst) == total {
        listeners.succeed();
    } else {
        listeners.failed();
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
