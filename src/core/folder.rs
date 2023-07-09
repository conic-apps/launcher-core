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

//! The game folders parser
//!
//! # Example
//!
//! ```
//! use std::path::Path;
//! use mgl_core::core::folder::MinecraftLocation;
//!
//! let minecraft_location = MinecraftLocation::new(".minecraft");
//!
//! assert_eq!(Path::new(".minecraft/mods").to_path_buf(), minecraft_location.mods);
//! assert_eq!(
//!     Path::new(".minecraft/versions/1.19.4/1.19.4.json").to_path_buf(),
//!     minecraft_location.get_version_json("1.19.4")
//! );
//! ```

use std::ffi::OsStr;
use std::{
    fmt::Display,
    format,
    path::{Path, PathBuf},
};

use super::PlatformInfo;

#[derive(Debug, Clone)]
/// The Minecraft folder structure. All method will return the path related to a minecraft root like .minecraft.
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
    pub fn new<S: AsRef<OsStr> + ?Sized>(root: &S) -> MinecraftLocation {
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

    pub fn get_natives_root(&self, version: &str, platform: &PlatformInfo) -> PathBuf {
        self.versions
            .join(version)
            .join(format!("natives-{}-{}", platform.name, platform.arch))
    }

    pub fn get_version_root<P: AsRef<Path>>(&self, version: P) -> PathBuf {
        self.versions.join(version)
    }

    pub fn get_version_json<P: AsRef<Path> + Display>(&self, version: P) -> PathBuf {
        self.get_version_root(&version)
            .join(format!("{version}.json"))
    }

    pub fn get_version_jar<P: AsRef<Path> + Display>(
        &self,
        version: P,
        r#type: Option<&str>,
    ) -> PathBuf {
        if r#type == Some("client") || r#type.is_none() {
            self.get_version_root(&version)
                .join(format!("{version}.jar"))
        } else {
            self.get_version_root(&version)
                .join(format!("{version}-{}.jar", r#type.unwrap()))
        }
    }

    pub fn get_version_all<P: AsRef<Path> + Display>(&self, version: P) -> Vec<PathBuf> {
        vec![
            self.versions.join(&version),
            self.versions.join(&version).join(format!("{version}.json")),
            self.versions.join(&version).join(format!("{version}.jar")),
        ]
    }

    pub fn get_resource_pack<P: AsRef<Path>>(&self, file_name: P) -> PathBuf {
        self.resourcepacks.join(file_name)
    }

    pub fn get_mod<P: AsRef<Path>>(&self, file_name: P) -> PathBuf {
        self.mods.join(file_name)
    }

    pub fn get_log<P: AsRef<Path>>(&self, file_name: P) -> PathBuf {
        self.logs.join(file_name)
    }

    pub fn get_library_by_path<P: AsRef<Path>>(&self, library_path: P) -> PathBuf {
        self.libraries.join(library_path)
    }

    pub fn get_assets_index(&self, version_assets: &str) -> PathBuf {
        self.assets
            .join("indexes")
            .join(format!("{version_assets}.json"))
    }

    pub fn get_log_config<P: AsRef<Path>>(&self, file: P) -> PathBuf {
        self.assets.join("log_configs").join(file)
    }
}

pub fn get_path(path: &PathBuf) -> String {
    match path.to_str() {
        None => panic!("New path is noe a valid UTF-8 sequence!"),
        Some(s) => String::from(s),
    }
}

#[tokio::test]
async fn test() {
    let minecraft = MinecraftLocation::new("test");
    println!("{:?}", minecraft.get_natives_root("1.19.4", &PlatformInfo::new().await));
}
