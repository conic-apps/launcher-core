use reqwest::blocking::Client;
use std::fs::{self, File};
use std::io::copy;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub struct Download {
    pub url: String,
    pub file: String,
}

pub fn download_files(download_tasks: Vec<Download>, num_threads: usize) {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    let (tx, rx) = channel::<usize>();
    let mut task_count = download_tasks.len();
    for download_task in download_tasks {
        let tx = tx.clone();
        pool.spawn(move || {
            let direction = PathBuf::from(&download_task.file);
            create_direction(direction.parent().unwrap());
            if PathBuf::from(&download_task.file).exists() {
                tx.send(1).unwrap();
                return;
            }
            let client = Client::new();
            let mut response = client.get(download_task.url).send().unwrap();
            let mut dest_file = File::create(download_task.file).unwrap();
            if response.status().is_success() {
                copy(&mut response, &mut dest_file).unwrap();
                tx.send(1).unwrap();
            } else {
                // todo: 失败后的重试操作以及错误处理
            }
        })
    }
    loop {
        thread::sleep(Duration::from_micros(3000));
        match rx.try_recv() {
            Ok(_) => task_count -= 1,
            _ => continue,
        }
        println!("还有{}个", &task_count);
        if task_count == 0 {
            break;
        }
    }
}

fn create_direction(direction: &Path) {
    if !direction.exists() {
        match fs::create_dir_all(&direction) {
            Ok(_) => println!("Created directory: {:?}", direction),
            Err(e) => panic!("Failed to create directory: {:?}, error: {}", direction, e),
        }
    }
}

#[test]
fn test_download_files() {
    let mut download_tasks = vec![];
    for i in 0..200 {
        download_tasks.push(Download {
            url: "https://speed.hetzner.de/100MB.bin".to_string(),
            file: format!("/home/CD-DVD/test/test-{}.bin", i + 1),
        });
    }
    download_files(download_tasks, 64);
}
