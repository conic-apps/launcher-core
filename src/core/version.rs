use std::{collections::HashMap, fs::read_to_string, path::PathBuf};

use regex::Regex;
use serde::Deserialize;
use serde_json::Value;

use crate::utils::folder::MinecraftLocation;

use super::platform::PlatformInfo;

#[derive(Debug, Clone, Deserialize)]
pub struct Download {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndex {
    // pub sha1: String,
    pub size: u64,
    pub url: String,
    pub id: String,
    pub total_size: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Artifact {
    pub sha1: String,
    pub size: u64,
    pub url: String,
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingFile {
    pub size: u64,
    pub url: String,
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NormalLibrary {
    pub name: String,
    pub downloads: HashMap<String, Artifact>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    pub action: String,
    pub os: Option<Platform>,
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Extract {
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NativeLibrary {
    pub name: String,
    pub downloads: HashMap<String, Artifact>,
    pub classifiers: HashMap<String, Artifact>,
    pub rules: Vec<Rule>,
    pub extract: Extract,
    pub natives: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlatformSpecificLibrary {
    pub name: String,
    pub downloads: HashMap<String, Artifact>,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LegacyLibrary {
    pub name: String,
    pub url: Option<String>,
    pub clientreq: Option<bool>,
    pub serverreq: Option<bool>,
    pub checksums: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum Library {
    Normal(NormalLibrary),
    Native(NativeLibrary),
    PlatformSpecific(PlatformSpecificLibrary),
    Legacy(LegacyLibrary),
}

#[derive(Debug, Clone, Deserialize)]
pub enum LaunchArgument {
    String(String),
    Object(serde_json::map::Map<String, serde_json::Value>),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Platform {
    pub name: String,
    pub version: Option<String>,
    // Add other platform properties if needed
}

#[derive(Debug, Clone, Deserialize)]
pub struct Arguments {
    game: Option<Vec<serde_json::Value>>,
    jvm: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Logging {
    pub file: Download,
    pub argument: String,
    pub r#type: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    pub component: String,
    pub major_version: i32,
}

#[derive(Debug, Clone)]
pub struct ResolvedVersion {
    /// The id of the version, should be identical to the version folder.
    pub id: String,
    pub arguments: Option<ResolvedArguments>,

    /// The main class full qualified name.
    pub main_class: String,
    pub asset_index: Option<AssetIndex>,

    /// The asset index id of this version. Should be something like `1.14`, `1.12`.
    pub assets: String,
    pub downloads: Option<HashMap<String, Download>>,
    pub libraries: ResolvedLibraries,
    pub minimum_launcher_version: i32,
    pub release_time: String,
    pub time: String,
    pub version_type: String,
    pub logging: Option<HashMap<String, Logging>>,

    /// Recommended java version.
    pub java_version: JavaVersion,

    /// The minecraft version of this version.
    pub minecraft_version: String,

    /// The version inheritances of this whole resolved version.
    ///
    /// The first element is this version, and the last element is the root Minecraft version.
    /// The dependencies of \[\<a\>, \<b\>, \<c\>\] should be \<a\> -> \<b\> -> \<c\>, where c is a Minecraft version.
    pub inheritances: Vec<String>,

    /// All array of json file paths.
    ///
    /// It's the chain of inherits json path. The root json will be the last element of the array.
    /// The first element is the user provided version.
    pub path_chain: Vec<PathBuf>,
}

/// The raw json format provided by Minecraft. Also the namespace of version operation.
///
/// Use `parse` to parse a Minecraft version json on the disk, and see the detail info of the version.
///
/// With `ResolvedVersion`, you can use the resolved version to launch the game.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub id: String,
    pub time: Option<String>,
    pub r#type: Option<String>,
    pub release_time: Option<String>,
    pub inherits_from: Option<String>,
    pub minimum_launcher_version: Option<i32>,
    pub minecraft_arguments: Option<String>,
    pub arguments: Option<Arguments>,
    pub main_class: Option<String>,
    pub libraries: Option<Vec<serde_json::Value>>,
    pub jar: Option<String>,
    pub asset_index: Option<AssetIndex>,
    pub assets: Option<String>,
    pub downloads: Option<HashMap<String, Download>>,
    pub client: Option<String>,
    pub server: Option<String>,
    pub logging: Option<HashMap<String, Logging>>,
    pub java_version: Option<JavaVersion>,
    pub client_version: Option<String>,
}

impl Version {
    pub fn new(raw: &str) -> Result<Version, serde_json::Error> {
        serde_json::from_str(raw)
    }

    pub fn get(
        minecraft: MinecraftLocation,
        version_name: &str,
    ) -> Result<Version, std::io::Error> {
        let versions_folder = minecraft.versions;
        let path = versions_folder
            .join(version_name)
            .join(format!("{}.json", version_name));

        let raw = read_to_string(path)?;
        let version: Version = serde_json::from_str(&raw)?;
        Ok(version)
    }

    pub fn parse(&self, minecraft: MinecraftLocation) -> ResolvedVersion {
        let mut inherits_from = self.inherits_from.clone();
        let versions_folder = minecraft.versions;
        let mut versions = Vec::new();
        let mut inheritances = Vec::new();
        let mut path_chain = Vec::new();
        versions.push(self.clone());
        while let Some(_) = inherits_from {
            let inherits_from_unwrap = inherits_from.unwrap();
            inheritances.push(inherits_from_unwrap.clone());

            let path = versions_folder
                .join(inherits_from_unwrap.clone())
                .join(format!("{}.json", inherits_from_unwrap.clone()));
            path_chain.push(path.clone());
            let version_json = read_to_string(path).unwrap();
            let version_json: Version = serde_json::from_str(&version_json).unwrap();

            versions.push(version_json.clone());
            inherits_from = version_json.inherits_from;
        }

        let mut assets = "".to_string();
        let mut minimum_launcher_version = 0;
        let mut game_args = Vec::new();
        let mut jvm_args = Vec::new();
        let mut release_time = "".to_string();
        let mut time = "".to_string();
        let mut version_type = "".to_string();
        let mut logging = HashMap::new();
        let mut main_class = "".to_string();
        let mut assets_index = AssetIndex {
            size: 0,
            url: "".to_string(),
            id: "".to_string(),
            total_size: 0,
        };
        let mut java_version = JavaVersion {
            component: "jre-legacy".to_string(),
            major_version: 8,
        };
        let mut libraries_raw = Vec::new();
        let mut downloads = HashMap::new();

        while versions.len() != 0 {
            let version = versions.pop().unwrap();
            println!("{}", version.id);
            minimum_launcher_version = std::cmp::max(
                version.minimum_launcher_version.unwrap_or(0),
                minimum_launcher_version,
            );

            if let Some(arguments) = version.arguments {
                if let Some(mut game) = arguments.game {
                    game_args.append(&mut game);
                }
                if let Some(mut jvm) = arguments.jvm {
                    jvm_args.append(&mut jvm);
                }
            }

            release_time = version.release_time.unwrap_or(release_time);
            time = version.time.unwrap_or(time);
            logging = version.logging.unwrap_or(logging);
            assets = version.assets.unwrap_or(assets);
            version_type = version.r#type.unwrap_or(version_type);
            main_class = version.main_class.unwrap_or(main_class);
            assets_index = version.asset_index.unwrap_or(assets_index);
            java_version = version.java_version.unwrap_or(java_version);

            if let Some(mut libraries) = version.libraries {
                libraries_raw.append(&mut libraries);
            }
            downloads = version.downloads.unwrap_or(downloads);
        }

        if main_class == ""
            || assets_index
                == (AssetIndex {
                    size: 0,
                    url: "".to_string(),
                    id: "".to_string(),
                    total_size: 0,
                })
            || downloads.len() == 0
        {
            panic!("Bad Version JSON");
        }
        ResolvedVersion {
            id: self.id.clone(),
            arguments: Some(ResolvedArguments {
                game: resolve_arguments(game_args),
                jvm: resolve_arguments(jvm_args),
            }),
            main_class,
            asset_index: self.asset_index.clone(),
            assets: self.assets.clone().unwrap_or("".to_string()),
            downloads: self.downloads.clone(),
            libraries: resolve_libraries(libraries_raw),
            minimum_launcher_version,
            release_time,
            time,
            version_type,
            logging: self.logging.clone(),
            java_version: self.java_version.clone().unwrap_or(JavaVersion {
                component: "jre-legacy".to_string(),
                major_version: 8,
            }),
            minecraft_version: self.client_version.clone().unwrap_or(self.id.clone()),
            inheritances,
            path_chain,
        }
    }
}

#[test]
fn test() {
    let a = Version::get(
        MinecraftLocation::new("test"),
        "1.7.10-Forge10.13.4.1614-1.7.10",
    )
    .unwrap();
    let b = a.parse(MinecraftLocation::new("test"));
    println!("{:#?}", b)
}

#[derive(Debug, Clone)]
pub struct ResolvedArguments {
    pub game: String,
    pub jvm: String,
}

#[derive(Debug, Clone)]
pub struct ResolvedLibraries(Vec<Artifact>);

fn resolve_arguments(arguments: Vec<Value>) -> String {
    let platform = &PlatformInfo::get();
    let mut result = String::new();
    for argument in arguments {
        if argument.is_string() {
            result.push_str(&format!("{} ", argument.as_str().unwrap()));
            continue;
        }
        if !argument.is_object() {
            continue;
        }
        let rules = argument["rules"].as_array();
        if let Some(rules) = rules {
            if !check_allowed(rules.clone(), platform) {
                continue;
            };
        }
        if argument["value"].is_string() {
            result.push_str(&format!("{} ", argument["value"].as_str().unwrap_or("")));
            continue;
        }
        if argument["value"].is_array() {
            let values = argument["value"].as_array().unwrap_or(&vec![]).clone();
            for value in values {
                result.push_str(&format!("{} ", value.as_str().unwrap()));
            }
        }
    }
    result
}

fn resolve_libraries(libraries: Vec<Value>) -> ResolvedLibraries {
    let platform = PlatformInfo::get();
    let mut result: Vec<Artifact> = Vec::new();
    for library in libraries {
        let rules = library["rules"].as_array();
        // check rules
        if let Some(rules) = rules {
            if !check_allowed(rules.clone(), &platform) {
                continue;
            }
        }
        if library["downloads"]["artifact"].is_object() {
            result.push(serde_json::from_value(library["downloads"]["artifact"].clone()).unwrap());
            continue;
        }
        let name = library["name"].as_str();
        if name == None {
            continue;
        }
        let name: Vec<&str> = name.unwrap().split(":").collect();
        if name.len() != 3 {
            continue;
        }
        let package = name.get(0).unwrap().replace(".", "/");
        let version = name.get(2).unwrap();
        let name = name.get(1).unwrap();

        let url;
        if let Some(url_) = library["url"].as_str() {
            url = url_;
        } else {
            url = "http://files.minecraftforge.net/maven/"
        }
        let path = format!("{package}/{name}/{version}/{name}-{version}.jar");
        result.push(Artifact {
            sha1: "".to_string(),
            size: 0,
            url: format!("{url}{path}"),
            path,
        });
    }
    ResolvedLibraries(result)
}

/// Check if all the rules in Rule[] are acceptable in certain OS platform and features.
fn check_allowed(rules: Vec<Value>, platform: &PlatformInfo) -> bool {
    // by default it's allowed
    if rules.is_empty() {
        return true;
    }
    // else it's disallow by default
    let mut allow = false;
    for rule in rules {
        let action = rule["action"].as_str().unwrap() == "allow";
        // apply by default
        let os = rule["os"].clone();
        if !os.is_object() {
            allow = action;
            continue;
        }
        // don't apply by default if has os rule
        if !os["name"].is_string() {
            allow = action;
            continue;
        }
        if platform.name != os["name"].as_str().unwrap() {
            continue;
        }
        if !os["version"].is_string() {
            allow = action;
            continue;
        }
        let version = os["version"].as_str().unwrap();
        if Regex::is_match(&Regex::new(version).unwrap(), &platform.version) {
            allow = action;
        }
        // todo: check `features`
    }
    allow
}
