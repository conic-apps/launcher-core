use crate::utils;
use serde::{Deserialize, Serialize};

static DEFAULT_EXTRA_JVM_ARGS: [&str; 7] = [
    "Xmx2G",
    "-XX:+UnlockExperimentalVMOptions",
    "-XX:+UseG1GC",
    "-XX:G1NewSizePercent=20",
    "-XX:G1ReservePercent=20",
    "-XX:MaxGCPauseMillis=50",
    "-XX:G1HeapRegionSize=32M",
];

pub struct GameProfile {
    name: String,
}

pub enum UserType {
    Mojang,
    Legacy,
}

pub enum Properties {
    Highest,
    High,
    Medium,
    Low,
    Lowest,
}

pub enum OSName {
    Osx,
    Linux,
    Windows,
    Unknown,
}

pub struct Server {
    ip: String,
    prot: i32,
}

pub struct LaunchOption {
    /// User selected game profile.
    ///
    /// For game display name & uuid
    pub game_profile: GameProfile,

    pub access_token: String,
    pub user_type: UserType,
    pub properties: Properties,
    pub launcher_name: String,
    pub launcher_brand: String,

    /// Overwrite the version name of the current version.
    ///
    /// If this is absent, it will use version name from resolved version.
    version_name: String,

    /// Overwrite the version type of the current version.
    ///
    /// If this is absent, it will use version type from resolved version.
    ///  
    /// Some people use this to show fantastic message on the welcome screen.
    version_type: String,

    /// The full path of launched game icon
    ///
    /// Currently, this only supported on MacOS
    game_icon: String,

    /// The launched game name
    ///
    /// Currently, this only supported on MacOS.
    gane_name: String,

    /// The path of parent directory of saves/logs/configs/mods/resourcepacks
    game_path: String,

    /// The path of parent directory of assets/libraries
    resource_path: String,

    /// The java executable file path.
    ///
    /// Not the java home directory!
    java_path: String,

    /// Min memory, this will add a jvm flag -XMS to the command result
    min_memory: i32,

    /// Max memory, this will add a jvm flag -Xmx to the command result
    max_memory: i32,

    /// The version of launched Minecraft. Can be either resolved version or version string
    version: String,

    /// Directly launch to a server.
    server: Server,

    /// window widOSNameth
    width: i32,

    /// window height
    height: i32,

    fullscreen: bool,

    /// Extra jvm options. This will append after to generated options.
    ///
    /// If this is empty, the `DEFAULT_EXTRA_JVM_ARGS` will be used.
    extra_jvm_args: Vec<String>,

    /// Extra program arguments. This will append after to generated options.
    extra_mc_args: Vec<String>,

    is_demo: bool,

    /// Native directory. It's .minecraft/versions/<version>/<version>-natives by default.
    ///
    /// You can replace this by your self.
    native_root: String,

    // Todo: yggdrasilAgent
    /// Add `-Dfml.ignoreInvalidMinecraftCertificates=true` to jvm argument
    ignore_invalid_minecraft_certificates: bool,

    /// Add `-Dfml.ignorePatchDiscrepancies=true` to jvm argument
    ignore_patch_discrepancies: bool,

    /// Add extra classpaths
    extra_class_paths: Vec<String>,
}
#[derive(Serialize, Deserialize)]
struct Test {
    article: String,
    author: String,
}
