/*
 * Amethyst Launcher Core
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

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use anyhow::Result;
use futures::StreamExt;
use reqwest::Response;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::core::HTTP_CLIENT;
use crate::core::task::TaskEventListeners;

use super::sha1::calculate_sha1_from_read;

#[derive(Debug, Clone)]
pub struct Download<P: AsRef<Path> + AsRef<OsStr>> {
    pub url: String,
    pub file: P,
    pub sha1: Option<String>,
}

// todo: 接受url列表以便轮询
// todo: 测试是不是只要把on_progress包在Arc和Mutex里就可以，不需要thread safe版本的实现
pub async fn download<P: AsRef<Path> + AsRef<OsStr>>(
    download_task: Download<P>,
) -> Result<Response> {
    // todo: 读取下载信息结构体中的文件大小
    let file_path = PathBuf::from(&download_task.file);
    let direction = file_path.parent().unwrap();
    if !direction.exists() {
        fs::create_dir_all(&direction).await?
    }
    let mut response = HTTP_CLIENT.get(&download_task.url).send().await?;
    let mut file = fs::File::create(&download_task.file).await?;
    while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk).await?;
    }
    Ok(response)
}

pub async fn download_files(
    download_tasks: Vec<Download<String>>,
    listeners: TaskEventListeners,
    verify_exists: bool,
) -> Result<()> {
    listeners.start();
    listeners.progress(0, 0, 1);
    let download_tasks: Vec<_> = download_tasks
        .iter()
        .filter(|download_task| {
            match std::fs::metadata(&download_task.file) {
                Err(_) => {
                    return true;
                }
                _ => {
                    if !verify_exists {
                        return false;
                    }
                }
            }
            let mut file = match std::fs::File::open(&download_task.file) {
                Ok(file) => file,
                Err(_) => {
                    return true;
                }
            };
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
        })
        .await;

    if counter.load(Ordering::SeqCst) == total {
        listeners.succeed();
    } else {
        listeners.failed();
    }

    Ok(())
}
