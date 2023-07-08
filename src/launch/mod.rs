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

//! A launcher for game
//!
//! This module contains the [`Launcher`] [`LauncherOptions`] struct, the [`launch`] function for
//! launching a game, and several error types that may result from
//! working with [`Launcher`].
//!
//! # Examples
//!
//! There are multiple ways to create a new [`LaunchOptions`] from a string literal:
//!
//! ```
//! use mgl_core::core::folder::MinecraftLocation;
//! use mgl_core::launch::LaunchOptions;
//!
//! async fn fn_name() {
//!     let version_id = "1.19.4";
//!     let minecraft = MinecraftLocation::new(".minecraft");
//!     let options = LaunchOptions::new("1.19.4", minecraft);
//! }
//! ```
//!
//! Then you can modify it with user custom options,
//! The step of creating default startup options is omitted here, in order to test through us here
//! assuming that the default options have been passed as parameters:
//!
//! ```
//! use mgl_core::launch::{GC, LaunchOptions};
//!
//! async fn fn_name2(default_options: &LaunchOptions) {
//!     let mut options = default_options.clone();
//!     options.game_profile.name = "Broken Deer".to_string();
//! }
//! ```
//!
//! Finally, you can use the [`LaunchOptions`] to build a [`Launcher`] instance, then launch the
//! game using [`Launcher::launch()`].
//!
//! ```
//! use mgl_core::core::folder::MinecraftLocation;
//! use mgl_core::launch::{Launcher, LaunchOptions};
//!
//!  async fn fn_name3(options: LaunchOptions) {
//!     let mut launcher = Launcher::from_options(options).await;
//!     launcher.launch().await;
//! }
//! ```

use std::collections::HashMap;
use std::env::vars;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use std::string::ToString;

use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;
use tokio::process::Command;

use crate::core::{DELIMITER, JavaExec, OsType, PlatformInfo};
use crate::core::folder::MinecraftLocation;
use crate::core::version::{ResolvedVersion, Version};

pub static DEFAULT_EXTRA_JVM_ARGS: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        "Xmx2G".to_string(),
        "-XX:+UnlockExperimentalVMOptions".to_string(),
        "-XX:+UseG1GC".to_string(),
        "-XX:G1NewSizePercent=20".to_string(),
        "-XX:G1ReservePercent=20".to_string(),
        "-XX:MaxGCPauseMillis=50".to_string(),
        "-XX:G1HeapRegionSize=32M".to_string(),
    ]
});

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
    pub min_memory: Option<u32>,

    /// Max memory, this will add a jvm flag -Xmx to the command result
    pub max_memory: Option<u32>,

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

    /// Native directory. It's .minecraft/versions/<version>/<version>-natives by default.
    ///
    /// You can replace this by your self.
    pub native_root: PathBuf,

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

    /// Support yushi's yggdrasil agent https://github.com/to2mbn/authlib-injector/wiki
    pub yggdrasil_agent: Option<YggdrasilAgent>,

    pub version_id: String,

    pub gc: GC,

    pub minecraft_location: MinecraftLocation,
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
            },
            access_token: uuid::Uuid::new_v4().to_string().replace('-', ""),
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
            min_memory: None,
            max_memory: Some(2048),
            server: None,
            width: 854,
            height: 480,
            fullscreen: false,
            extra_jvm_args: DEFAULT_EXTRA_JVM_ARGS.clone(),
            extra_mc_args: Vec::new(),
            is_demo: false,
            native_root: minecraft.get_natives_root(version_id),
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
        })
    }
}

/// launch arguments for launch
///
/// You can use `from_launch_options` to generate launch parameters and use `to_launch_command` to
/// convert to shell commands
pub struct LaunchArguments(Vec<String>);

const DEFAULT_GAME_ICON: &[u8] = include_bytes!("./assets/minecraft.icns");

impl LaunchArguments {
    pub async fn from_launch_options(
        launch_options: LaunchOptions,
        version: ResolvedVersion,
    ) -> Result<Self> {
        // todo: if launch_options.game_path.is_absolute() { return Err(); }
        let platform = PlatformInfo::new().await;
        let minecraft = MinecraftLocation::new(&launch_options.resource_path);

        let game_icon = match launch_options.game_icon {
            Some(icon_path) => icon_path,
            None => {
                let icon_path = minecraft.assets.join("minecraft.icns");
                tokio::fs::write(&icon_path, DEFAULT_GAME_ICON).await?;
                icon_path
            }
        };

        let mut command_arguments = Vec::new();

        command_arguments.push(format!(
            "-Dminecraft.client.jar={version_jar}",
            version_jar = launch_options
                .version_root
                .join(&launch_options.version_id)
                .to_string_lossy()
        ));

        if platform.name == "osx" {
            command_arguments.push(format!(
                "-Xdock:name={game_name}",
                game_name = launch_options.game_name
            ));
            command_arguments.push(format!(
                "-Xdock:icon={game_icon}",
                game_icon = game_icon.to_string_lossy()
            ));
        }

        if let Some(min_memory) = launch_options.min_memory {
            command_arguments.push(format!("-Xms{min_memory}M"));
        }
        if let Some(max_memory) = launch_options.max_memory {
            command_arguments.push(format!("-Xmx{max_memory}M"));
        }

        if launch_options.ignore_invalid_minecraft_certificates {
            command_arguments.push("-Dfml.ignoreInvalidMinecraftCertificates=true".to_string());
        }
        if launch_options.ignore_patch_discrepancies {
            command_arguments.push("-Dfml.ignorePatchDiscrepancies=true".to_string());
        }

        match launch_options.gc {
            GC::G1 => {
                command_arguments.extend([
                    "-XX:+UseG1GC".to_string(),
                    "-XX:+UnlockExperimentalVMOptions".to_string(),
                    "-XX:G1NewSizePercent=20".to_string(),
                    "-XX:G1ReservePercent=20".to_string(),
                    "-XX:MaxGCPauseMillis=50".to_string(),
                    "-XX:G1HeapRegionSize=16M".to_string(),
                ]);
            }
            GC::Parallel => {
                command_arguments.extend([
                    "-XX:+UseParallelGC".to_string(),
                    format!(
                        "-XX:ParallelGCThreads={num}",
                        num = num_cpus::get_physical()
                    ),
                ]);
            }
            GC::ParallelOld => {
                command_arguments.push("-XX:+UseParallelOldGC".to_string());
            }
            GC::Serial => {
                command_arguments.push("-XX:+UseSerialGC".to_string());
            }
            GC::Z => {
                command_arguments.push("-XX:+UseZGC".to_string());
            }
        }

        if let Some(ygg) = launch_options.yggdrasil_agent {
            command_arguments.push(format!(
                "-javaagent:{jar}={server}",
                jar = ygg.jar.to_string_lossy(),
                server = ygg.server
            ));
            command_arguments.push("-Dauthlibinjector.side=client".to_string());
            if let Some(prefetched) = ygg.prefetched {
                command_arguments.push(format!(
                    "-Dauthlibinjector.yggdrasil.prefetched={prefetched}"
                ));
            }
        }

        command_arguments.extend([
            "-Xverify:none".to_string(),
            "-XX:MaxInlineSize=420".to_string(),
            "-XX:-UseAdaptiveSizePolicy".to_string(),
            "-XX:-OmitStackTraceInFastThrow".to_string(),
            "-XX:-DontCompileHugeMethods".to_string(),
            "-Xss:1m".to_string(),
            "-Xmn128m".to_string(),
            "-Djava.rmi.server.useCodebaseOnly=true".to_string(),
            "-Dcom.sun.jndi.rmi.object.trustURLCodebase=false".to_string(),
            "-Dcom.sun.jndi.cosnaming.object.trustURLCodebase=false".to_string(),
            "-Dlog4j2.formatMsgNoLookups=true".to_string(),
        ]); // todo: test the jvm args
        // todo: support proxy

        let mut jvm_options: HashMap<&str, String> = HashMap::new();
        jvm_options.insert(
            "natives_directory",
            launch_options.native_root.to_string_lossy().to_string(),
        );
        jvm_options.insert("launcher_name", launch_options.launcher_name);
        jvm_options.insert("launcher_version", launch_options.launcher_version);
        jvm_options.insert(
            "classpath",
            resolve_classpath(&version, &minecraft, launch_options.extra_class_paths),
        );

        let mut jvm_arguments = version.arguments.clone().unwrap().jvm;
        if let Some(logging) = version.logging {
            if let Some(client) = logging.get("client") {
                let argument = &client.argument;
                let file_path = minecraft.get_log_config(&client.file.id);
                if tokio::fs::try_exists(&file_path).await? {
                    jvm_arguments.push(
                        argument.replace("${path}", &file_path.to_string_lossy().to_string()),
                    );
                }
            }
        }

        command_arguments.extend(
            jvm_arguments
                .iter()
                .map(|arg| format(arg, jvm_options.clone())),
        );
        command_arguments.extend(launch_options.extra_jvm_args);

        command_arguments.push(version.main_class);

        let mut game_options = HashMap::with_capacity(13);

        let assets_dir = launch_options.resource_path.join("assets");
        game_options.insert(
            "version_name",
            match launch_options.version_name {
                Some(v) => v,
                None => version.id,
            },
        );
        game_options.insert(
            "version_type",
            match launch_options.version_type {
                Some(v) => v,
                None => version.version_type,
            },
        );
        game_options.insert("assets_root", assets_dir.to_string_lossy().to_string());
        game_options.insert(
            "game_assets",
            assets_dir
                .join("virtual")
                .join(&version.assets)
                .to_string_lossy()
                .to_string(),
        );
        game_options.insert("assets_index_name", version.assets);
        game_options.insert(
            "game_directory",
            launch_options.game_path.to_string_lossy().to_string(),
        );
        game_options.insert("auth_player_name", launch_options.game_profile.name);
        game_options.insert("auth_uuid", launch_options.game_profile.uuid);
        game_options.insert("auth_access_token", launch_options.access_token);
        game_options.insert("user_properties", launch_options.properties);
        game_options.insert(
            "user_type",
            match launch_options.user_type {
                UserType::Mojang => "mojang".to_string(),
                UserType::Legacy => "legacy".to_string(),
            },
        );
        game_options.insert("resolution_width", launch_options.width.to_string());
        game_options.insert("resolution_height", launch_options.height.to_string());

        command_arguments.extend(
            version
                .arguments
                .unwrap()
                .game
                .iter()
                .map(|arg| format(arg, game_options.clone())),
        );
        command_arguments.extend(launch_options.extra_mc_args);
        if let Some(server) = launch_options.server {
            command_arguments.extend(vec!["--server".to_string(), server.ip]);
            if let Some(port) = server.port {
                command_arguments.extend(vec!["--port".to_string(), port.to_string()])
            }
        }
        if launch_options.fullscreen {
            command_arguments.push("--fullscreen".to_string());
        }
        let no_width_arguments = None
            == command_arguments
            .iter()
            .find(|v| v == &&"--width".to_string());
        if no_width_arguments && !launch_options.fullscreen {
            command_arguments.extend(vec![
                "--width".to_string(),
                launch_options.width.to_string(),
                "--height".to_string(),
                launch_options.height.to_string(),
            ]);
        }

        Ok(LaunchArguments(command_arguments))
    }

    /// spawn a command instance, you can use this to launch the game
    pub fn to_async_command(
        &self,
        java_exec: JavaExec,
        launch_options: LaunchOptions,
        platform: &PlatformInfo,
    ) -> Command {
        let mut command = match platform.os_type {
            OsType::Windows => {
                let vars = vars().find(|v| v.0 == "PATH").unwrap();

                let path_vars = vars.1.as_str().split(";").collect::<Vec<&str>>(); // todo: test it in windows
                let powershell_folder = PathBuf::from(
                    path_vars
                        .into_iter()
                        .find(|v| v.to_lowercase().contains("powershell"))
                        .unwrap(),
                );
                let powershell_exec = powershell_folder
                    .join("powershell.exe")
                    .to_string_lossy()
                    .to_string();
                Command::new(powershell_exec)
            }
            _ => Command::new("nice"),
        };

        if let OsType::Windows = platform.os_type {
            command.arg("-c");
        }

        if platform.os_type != OsType::Windows {
            match launch_options.process_priority {
                ProcessPriority::High => {
                    command.args(&["-n", "0"]);
                }
                ProcessPriority::AboveNormal => {
                    command.args(&["-n", "5"]);
                }
                ProcessPriority::Normal => (), // nothing to do
                ProcessPriority::BelowNormal => {
                    command.args(["-n", "15"]);
                }
                ProcessPriority::LOW => {
                    command.args(["-n", "19"]);
                }
            };
        }
        // todo(after java exec): add -Dfile.encoding=encoding.name() and other
        let launch_options = self.0.join(" ").to_string();
        command.arg(format!(
            "{java} {launch_options}",
            java = java_exec.binary.to_string_lossy().to_string()
        ));
        command
    }
}

fn resolve_classpath(
    version: &ResolvedVersion,
    minecraft: &MinecraftLocation,
    extra_class_paths: Option<Vec<String>>,
) -> String {
    let mut classpath = version
        .libraries
        .iter()
        .filter(|lib| !lib.is_native_library)
        .map(|lib| {
            minecraft
                .get_library_by_path(lib.artifact.path.clone())
                .to_string_lossy()
                .to_string()
        })
        .collect::<Vec<String>>();

    classpath.push(
        minecraft
            .get_version_jar(version.id.clone(), None)
            .to_str()
            .unwrap()
            .to_string(),
    );

    if let Some(extra_class_paths) = extra_class_paths {
        classpath.extend(extra_class_paths);
    }
    classpath.join(DELIMITER)
}

fn format(template: &str, args: HashMap<&str, String>) -> String {
    let regex = Regex::new(r"\$\{(.*?)}").unwrap();

    regex
        .replace_all(&template, |caps: &regex::Captures| {
            let key = String::from(&caps[1]);
            let value = args.get(&caps[1]).unwrap_or(&key);
            value.to_string()
        })
        .to_string()
}

/// All game launcher
///
/// Use `Launcher::new` to spawn an instance with minimal launch options
pub struct Launcher {
    pub launch_options: LaunchOptions,
    pub minecraft: MinecraftLocation,

    /// Whether to check game integrity before launching
    pub check_game_integrity: bool,

    pub exit_status: Option<ExitStatus>,
}

impl Launcher {
    /// spawn an instance with default launch options
    pub async fn new(version_id: &str, minecraft: MinecraftLocation) -> Result<Self> {
        let launch_options = LaunchOptions::new(version_id, minecraft.clone()).await?;
        Ok(Self {
            launch_options,
            minecraft,
            check_game_integrity: true,
            exit_status: None,
        })
    }

    /// spawn an instance with custom launch options
    pub async fn from_options(launch_options: LaunchOptions) -> Self {
        Self {
            minecraft: launch_options.minecraft_location.clone(),
            launch_options,
            check_game_integrity: true,
            exit_status: None,
        }
    }

    /// launch game.
    ///
    // /// Note: this function will block the current thread when game running
    pub async fn launch(&mut self) -> Result<()> {
        let platform = PlatformInfo::new().await;
        let options = self.launch_options.clone();
        let version = self
            .launch_options
            .version
            .parse(&self.minecraft, &platform)
            .await?;
        let mut command = LaunchArguments::from_launch_options(options.clone(), version.clone())
            .await?
            .to_async_command(
                JavaExec::new(&"/usr/lib64/jvm/java-17-openjdk-17").await,
                options,
                &platform,
            );
        let mut child = command.spawn()?;
        self.exit_status = Some(child.wait().await?);
        Ok(())
    }
}
