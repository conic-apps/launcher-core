use std::path::{Path, PathBuf};
#[derive(Debug)]
pub struct MinecraftLocation {
    pub root: PathBuf,
    pub libraries: PathBuf,
    pub assets: PathBuf,
    pub resourcepacks: PathBuf,
    pub mods: PathBuf,
    pub logs: PathBuf,
    pub latest_log: PathBuf,
    pub saves: PathBuf,
    pub versions: PathBuf,
    pub options: PathBuf,
    pub screenshots: PathBuf,
}

impl MinecraftLocation {
    pub fn new(root: &str) -> MinecraftLocation {
        let path = Path::new(root);
        MinecraftLocation {
            root: path.to_path_buf(),
            assets: path.join("assets"),
            libraries: path.join("libraries"),
            resourcepacks: path.join("resourcepacks"),
            mods: path.join("mods"),
            logs: path.join("logs"),
            latest_log: path.join("logs").join("latest.log"),
            saves: path.join("resourcepacks"),
            versions: path.join("versions"),
            options: path.join("options.txt"),
            screenshots: path.join("screenshots"),
        }
    }
}

pub fn get_path(path: &PathBuf) -> String {
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
