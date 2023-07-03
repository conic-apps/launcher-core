use std::{collections::HashMap, fs::File, path::PathBuf};

use zip::{read::ZipFile, CompressionMethod, DateTime, ZipArchive};

pub(crate) type UnzipFrom = String;
pub(crate) type UnzipTo = PathBuf;

#[derive(Debug, Clone)]
pub enum EntryType {
    Dir,
    File,
}

pub fn open(path: PathBuf) -> ZipArchive<File> {
    let file = File::open(path).unwrap();
    ZipArchive::new(file).unwrap()
}

pub fn filter_entries(zip: &mut ZipArchive<File>, entries: &Vec<String>) -> HashMap<String, Entry> {
    let mut resolved_entries = HashMap::with_capacity(entries.len());
    for i in 0..zip.len() {
        let zip_file = zip.by_index(i).unwrap();
        let name = zip_file.name();
        for entry in entries {
            if name == entry {
                resolved_entries.insert(entry.clone(), Entry::from_zip_file(zip_file));
            }
        }
    }
    resolved_entries
}
