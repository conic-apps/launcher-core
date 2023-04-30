use std::path::{Path, PathBuf};
#[derive(Debug)]
pub struct MinecraftLocation {
    root: String,
    libraries: String,
    assets: String,
    resourcepacks: String,
    mods: String,
    logs: String,
    latest_log: String,
    saves: String,
    versions: String,
    options: String,
    screenshots: String,
}

impl MinecraftLocation {
    fn new(root: &str) -> MinecraftLocation {
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

fn get_path(path: PathBuf) -> String {
    match path.to_str() {
        None => panic!("New path is noe a valid UTF-8 sequence!"),
        Some(s) => String::from(s),
    }
}
#[test]
fn test() {
    let a = MinecraftLocation::new("tmp");
    println!("{:#?}", a);
}
