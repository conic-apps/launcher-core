
use std::io::Read;

use sha1::Sha1;

pub fn calculate_sha1_from_read<R: Read>(source:&mut R) -> String {
    let mut hasher = Sha1::new();
    let mut buffer = [0; 1024];
    loop {
        let bytes_read = source.read(&mut buffer).unwrap();
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    hasher.digest().to_string()
}