use std::{path::PathBuf, fs::File};

use zip::ZipArchive;

pub fn open(path: PathBuf) -> ZipArchive<File> {
    let file = File::open(path).unwrap();
    ZipArchive::new(file).unwrap()
}