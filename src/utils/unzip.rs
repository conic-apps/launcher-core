use std::{fs::File, path::PathBuf};

use zip::{read::ZipFile, CompressionMethod, ZipArchive, DateTime};

pub struct Entry {
    pub version_made_by: (u8, u8),
    pub name: String,
    pub comment: String,
    pub compression: CompressionMethod,
    pub compressed_size: u64,
    pub size: u64,
    pub last_modified: DateTime,
    pub r#type: EntryType
}

pub enum EntryType {
    Dir, File
}

pub fn open(path: PathBuf) -> ZipArchive<File> {
    let file = File::open(path).unwrap();
    ZipArchive::new(file).unwrap()
}

pub(crate) fn filter_entries(zip: &mut ZipArchive<File>, entries: Vec<String>) -> Vec<Entry> {
    let mut data = Vec::with_capacity(entries.len());
    for i in 0..zip.len() {
        let zip_file = zip.by_index(i).unwrap();
        let name = zip_file.name();
        for entry in &entries {
            if name == entry {
                data.push(Entry {
                    version_made_by: zip_file.version_made_by(),
                    name: zip_file.name().to_string(),
                    comment: zip_file.comment().to_string(),
                    compression: zip_file.compression(),
                    compressed_size: zip_file.compressed_size(),
size: zip_file.size(),
last_modified: zip_file.last_modified(),
r#type: {
    
}

                });
                continue;
            }
        }
    }
    data
}
