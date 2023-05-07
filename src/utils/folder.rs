use std::path::{Path, PathBuf};
#[derive(Debug)]
pub struct MinecraftLocation {
    pub root: String,
    pub libraries: String,
    pub assets: String,
    pub resourcepacks: String,
    pub mods: String,
    pub logs: String,
    pub latest_log: String,
    pub saves: String,
    pub versions: String,
    pub options: String,
    pub screenshots: String,
}

impl MinecraftLocation {
    pub fn new(root: &str) -> MinecraftLocation {
        let path = Path::new(root);
        MinecraftLocation {
            root: String::from(root),
            assets: get_path(path.join("assets")),
            libraries: get_path(path.join("libraries")),
            resourcepacks: get_path(path.join("resourcepacks")),
            mods: get_path(path.join("mods")),
            logs: get_path(path.join("logs")),
            latest_log: get_path(path.join("logs").join("latest.log")),
            saves: get_path(path.join("resourcepacks")),
            versions: get_path(path.join("versions")),
            options: get_path(path.join("options.txt")),
            screenshots: get_path(path.join("screenshots")),
        }
    }
}

pub fn get_path(path: PathBuf) -> String {
    match path.to_str() {
        None => panic!("New path is noe a valid UTF-8 sequence!"),
        Some(s) => String::from(s),
    }
}
#[test]
fn test() {
    let a = MinecraftLocation::new("/home/CD-DVD/test");
    println!("{:#?}", a);
}
