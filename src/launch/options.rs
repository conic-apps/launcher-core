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

use std::{path::{PathBuf, Path}, collections::HashMap};

use anyhow::Result;
use serde_json::Value;

use crate::core::{version::Version, folder::MinecraftLocation};

#[derive(Debug, Clone)]
pub struct GameProfile {
    pub name: String,
    pub uuid: String,
}

#[derive(Debug, Clone)]
pub enum UserType {
    Mojang,
    Legacy,
}

#[derive(Debug, Clone)]
pub struct Server {
    pub ip: String,
    pub port: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct YggdrasilAgent {
    /// The jar file path of the authlib-injector
    pub jar: PathBuf,

    /// The auth server host
    pub server: String,

    /// The prefetched base64
    pub prefetched: Option<String>,
}

/// Game process priority, invalid on windows
#[derive(Debug, Clone)]
pub enum ProcessPriority {
    High,
    AboveNormal,
    Normal,
    BelowNormal,
    LOW,
}

/// User custom jvm gc
#[derive(Debug, Clone)]
pub enum GC {
    Serial,
    Parallel,
    ParallelOld,
    G1,
    Z,
}

#[derive(Debug, Clone)]
/// Launch options for game
pub struct LaunchOptions {
    /// User selected game profile.
    ///
    /// For game display name & uuid
    pub game_profile: GameProfile,

    pub access_token: String,
    pub user_type: UserType,
    pub properties: String,
    pub launcher_name: String,
    pub launcher_version: String,

    /// Overwrite the version name of the current version.
    ///
    /// If this is absent, it will use version name from resolved version.
    pub version_name: Option<String>,

    /// Overwrite the version type of the current version.
    ///
    /// If this is absent, it will use version type from resolved version.
    ///  
    /// Some people use this to show fantastic message on the welcome screen.
    pub version_type: Option<String>,

    /// The full path of launched game icon
    ///
    /// Currently, this only supported on MacOS
    pub game_icon: Option<PathBuf>,

    /// The launched game name
    ///
    /// Currently, this only supported on MacOS.
    pub game_name: String,

    /// The path of parent directory of `saves` / `logs` / `configs` / `mods` / `resourcepacks`
    ///
    /// If None, will be generated using the version_id passed in at startup
    ///
    /// ### WARN: If it is not an absolute path, the related operation will return `Err()`
    pub game_path: PathBuf,

    /// The path of parent directory of `assets` / `libraries`, like `.minecraft` folder
    pub resource_path: PathBuf,

    /// The java executable file path.
    ///
    /// Not the java home directory!
    pub java_path: PathBuf,

    /// Min memory, this will add a jvm flag -XMS to the command result
    pub min_memory: u32,

    /// Max memory, this will add a jvm flag -Xmx to the command result
    pub max_memory: u32,

    /// Directly launch to a server.
    pub server: Option<Server>,

    /// window width
    pub width: u32,

    /// window height
    pub height: u32,

    pub fullscreen: bool,

    /// User custom additional java virtual machine command line arguments.
    ///
    /// If this is empty, the `DEFAULT_EXTRA_JVM_ARGS` will be used.
    pub extra_jvm_args: Vec<String>,

    /// User custom additional minecraft command line arguments.
    pub extra_mc_args: Vec<String>,

    pub is_demo: bool,

    // Todo: yggdrasilAgent
    /// Add `-Dfml.ignoreInvalidMinecraftCertificates=true` to jvm argument
    pub ignore_invalid_minecraft_certificates: bool,

    /// Add `-Dfml.ignorePatchDiscrepancies=true` to jvm argument
    pub ignore_patch_discrepancies: bool,

    /// Add extra classpath
    pub extra_class_paths: Option<Vec<String>>,

    /// The path of parent directory of `<version_id>.jar` and `<version_id>.json`,
    ///
    /// default is `versions/{version_id}`
    ///
    /// ### WARN: If you have not saved `version.jar` and `version.json` to the default location, please modify this after creating the Launcher, otherwise related operations will return Err()
    pub version_root: PathBuf,

    /// The version of launched Minecraft. Can be either resolved version or version string
    pub version: Version,

    /// Enable features. Not really in used...
    pub features: HashMap<String, Value>,

    /// Game process priority, invalid on windows
    pub process_priority: ProcessPriority,

    /// Support yushi's yggdrasil agent <https://github.com/to2mbn/authlib-injector/wiki>
    pub yggdrasil_agent: Option<YggdrasilAgent>,

    pub version_id: String,

    pub gc: GC,

    pub minecraft_location: MinecraftLocation,

    pub native_path: PathBuf,
}

impl LaunchOptions {
    /// spawn an instance with default launch options
    pub async fn new(version_id: &str, minecraft: MinecraftLocation) -> Result<Self> {
        let version_json_path = minecraft.get_version_json(version_id);
        let raw_version_json = tokio::fs::read_to_string(version_json_path).await?;
        let version_json: Version = serde_json::from_str((&raw_version_json).as_ref())?;

        Ok(Self {
            game_profile: GameProfile {
                name: "Steve".to_string(),
                uuid: uuid::Uuid::new_v4().to_string().replace('-', ""),
                // uuid: "100062fc9db949789bccc0f781cc0cad".to_string()
            },
            access_token: uuid::Uuid::new_v4().to_string().replace('-', ""),
            // access_token: "eyJraWQiOiJhYzg0YSIsImFsZyI6IkhTMjU2In0.eyJ4dWlkIjoiMjUzNTQyNzc5NTA3MzUxOCIsImFnZyI6IkFkdWx0Iiwic3ViIjoiNThjZDc4MzQtMzZjMi00YjFmLThkMjUtYTdhMmUwMDE2Y2E5IiwiYXV0aCI6IlhCT1giLCJucyI6ImRlZmF1bHQiLCJyb2xlcyI6W10sImlzcyI6ImF1dGhlbnRpY2F0aW9uIiwiZmxhZ3MiOlsidHdvZmFjdG9yYXV0aCIsIm9yZGVyc18yMDIyIl0sInBsYXRmb3JtIjoiVU5LTk9XTiIsInl1aWQiOiIzZGFlYzJmZjMxMjYwMjFhNzk3YWJjNDJiYzU4MDIzMSIsIm5iZiI6MTY4ODkwNjQ2MywiZXhwIjoxNjg4OTkyODYzLCJpYXQiOjE2ODg5MDY0NjN9.BL3S2hA94toLOzEIv048oemlEumiKHR59CtuCFKb6_w".to_string(),
            user_type: UserType::Mojang,
            properties: "{}".to_string(),
            launcher_name: "Magical_Launcher".to_string(),
            launcher_version: "0.0.1".to_string(),
            version_name: None,
            version_type: None,
            game_icon: None,
            game_name: "Minecraft".to_string(),
            game_path: minecraft.get_version_root(version_id),
            version_root: minecraft.get_version_root(version_id),
            resource_path: minecraft.root.clone(),
            java_path: Path::new("java").to_path_buf(),
            min_memory: 128,
            max_memory: 2048,
            server: None,
            width: 854,
            height: 480,
            fullscreen: false,
            extra_jvm_args: vec![],
            extra_mc_args: Vec::new(),
            is_demo: false,
            ignore_invalid_minecraft_certificates: false,
            ignore_patch_discrepancies: false,
            extra_class_paths: None,
            version: version_json,
            features: HashMap::new(),
            yggdrasil_agent: None,
            process_priority: ProcessPriority::Normal,
            version_id: version_id.to_string(),
            gc: GC::G1,
            minecraft_location: minecraft.clone(),
            native_path: MinecraftLocation::get_natives_root(),
        })
    }
}
