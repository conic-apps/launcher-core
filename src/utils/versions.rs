use crate::utils::platform::PlatformInfo;
use regex::Regex;
use serde_json as JSON;

#[derive(Debug)]
pub struct AssetIndexDownload {
    pub id: String,
    pub sha1: String,
    pub size: i64,
    pub total_size: i64,
    pub url: String,
}

#[derive(Debug)]
pub struct Download {
    pub sha1: String,
    pub size: i64,
    pub url: String,
}

#[derive(Debug)]
pub struct Artifact {
    pub name: String,
    pub path: String,
    pub sha1: String,
    pub size: i64,
    pub url: String,
}

#[derive(Debug)]
pub struct Logging {
    pub argument: String,
    pub id: String,
    pub sha1: String,
    pub size: i64,
    pub url: String,
    pub logging_type: String,
}

#[derive(Debug)]
pub enum VersionType {
    Release,
    Snapshot,
    OldAlpha,
    OldBeta,
    Unknown,
}

#[derive(Debug)]
pub struct ResolvedArguments {
    pub game: String,
    pub jvm: String,
}

#[derive(Debug)]
pub struct Version {
    pub arguments: ResolvedArguments,
    pub asset_index: AssetIndexDownload,
    pub assets: String,
    pub compliance_level: i64,
    pub client: Download,
    pub id: String,
    pub java_version: i64,
    pub libraries: Vec<Artifact>,
    pub logging: Logging,
    pub main_class: String,
    pub release_time: String,
    pub time: String,
    pub version_type: VersionType,
    pub root: JSON::Value,
}

impl Version {
    /**
     * Parse vanilla version.json file.
     */
    pub fn new(version_json: &str) -> Version {
        let root: JSON::Value = JSON::from_str(version_json).unwrap();
        let platform_info = PlatformInfo::get();
        Version {
            asset_index: AssetIndexDownload {
                id: String::from(root["assetIndex"]["id"].as_str().unwrap()),
                sha1: String::from(root["assetIndex"]["sha1"].as_str().unwrap()),
                size: root["assetIndex"]["size"].as_i64().unwrap(),
                total_size: root["assetIndex"]["totalSize"].as_i64().unwrap(),
                url: String::from(root["assetIndex"]["url"].as_str().unwrap()),
            },
            compliance_level: root["complianceLevel"].as_i64().unwrap(),
            assets: String::from(root["assets"].as_str().unwrap()),
            client: Download {
                sha1: root["downloads"]["client"]["sha1"]
                    .as_str()
                    .unwrap()
                    .to_string(),
                size: root["downloads"]["client"]["size"].as_i64().unwrap(),
                url: root["downloads"]["client"]["url"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            },
            id: String::from(root["id"].as_str().unwrap()),
            java_version: root["javaVersion"]["majorVersion"].as_i64().unwrap(),
            logging: Logging {
                argument: String::from(root["logging"]["client"]["argument"].as_str().unwrap()),
                id: String::from(root["logging"]["client"]["file"]["id"].as_str().unwrap()),
                sha1: String::from(root["logging"]["client"]["file"]["sha1"].as_str().unwrap()),
                size: root["logging"]["client"]["file"]["size"].as_i64().unwrap(),
                url: String::from(root["logging"]["client"]["file"]["url"].as_str().unwrap()),
                logging_type: String::from(root["logging"]["client"]["type"].as_str().unwrap()),
            },
            main_class: String::from(root["mainClass"].as_str().unwrap()),
            release_time: String::from(root["releaseTime"].as_str().unwrap()),
            time: String::from(root["time"].as_str().unwrap()),
            version_type: match root["type"].as_str().unwrap() {
                "old_alpha" => VersionType::OldAlpha,
                "old_beta" => VersionType::OldBeta,
                "snapshot" => VersionType::Snapshot,
                "release" => VersionType::Release,
                _ => VersionType::Unknown,
            },
            libraries: resolve_libraries(root["libraries"].as_array().unwrap(), &platform_info),
            arguments: resolve_arguments(
                Arguments {
                    game: root["arguments"]["game"].as_array().unwrap().to_vec(),
                    jvm: root["arguments"]["jvm"].as_array().unwrap().to_vec(),
                },
                &platform_info,
            ),
            root,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct LatestVersion {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct VersionInfo {
    pub id: String,
    pub r#type: String,
    pub url: String,
    pub time: String,
    pub releaseTime: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct VersionManifest {
    pub latest: LatestVersion,
    pub versions: Vec<VersionInfo>,
}

impl VersionManifest {
    pub async fn new() -> VersionManifest {
        let response = reqwest::get("https://piston-meta.mojang.com/mc/game/version_manifest.json")
            .await
            .unwrap();
        response.json::<VersionManifest>().await.unwrap()
    }
}

/// Check if all the rules in Rule[] are acceptable in certain OS platform and features.
fn check_allowed(rules: &Vec<JSON::Value>, platform: &PlatformInfo) -> bool {
    // by default it's allowed
    if rules.len() == 0 {
        return true;
    }
    // else it's disallow by default
    let mut allow = false;
    for rule in rules {
        let action = rule["action"].as_str().unwrap() == "allow";
        // apply by default
        let mut apply = true;
        let os = rule["os"].as_object();
        if os != None {
            // don't apply by default if has os rule
            apply = false;
            let version = rule["os"]["version"].as_str();
            let name = rule["os"]["name"].as_str();
            if name != None {
                if name.unwrap() == platform.name {
                    if version == None {
                        apply = true
                    } else {
                        let version = version.unwrap();
                        println!("{}", version);
                        if Regex::is_match(
                            &Regex::new(version).unwrap(),
                            &PlatformInfo::get().version,
                        ) {
                            apply = true
                        }
                    }
                }
            }
        }
        // todo: check `features`
        if apply {
            allow = action;
        }
    }
    allow
}

struct Arguments {
    game: Vec<JSON::Value>,
    jvm: Vec<JSON::Value>,
}

fn resolve_arguments(arguments: Arguments, platform: &PlatformInfo) -> ResolvedArguments {
    fn resolve(arguments: Vec<JSON::Value>, platform: &PlatformInfo) -> String {
        let mut result = String::from("");
        for argument in arguments {
            if argument.is_string() {
                result.insert_str(result.len(), argument.as_str().unwrap());
                result.insert_str(result.len(), " ");
                continue;
            }
            let rules = argument["rules"].as_array();
            if rules != None {
                if !check_allowed(rules.unwrap(), platform) {
                    continue;
                }
            }
            if argument["value"].is_string() {
                result.insert_str(result.len(), argument["value"].as_str().unwrap());
                result.insert_str(result.len(), " ");
                continue;
            }
            if argument["value"].is_array() {
                let values = argument["value"].as_array().unwrap();
                for value in values {
                    result.insert_str(result.len(), value.as_str().unwrap());
                    result.insert_str(result.len(), " ");
                }
            }
        }
        result
    }
    ResolvedArguments {
        game: resolve(arguments.game, &platform),
        jvm: resolve(arguments.jvm, &platform),
    }
}

fn resolve_libraries(libraries: &Vec<JSON::Value>, platform: &PlatformInfo) -> Vec<Artifact> {
    let mut result: Vec<Artifact> = Vec::new();
    for library in libraries {
        let rules = library["rules"].as_array();
        // check rules
        if rules != None {
            if !check_allowed(rules.unwrap(), &platform) {
                continue;
            }
        }
        result.push(Artifact {
            name: library["name"].as_str().unwrap().to_string(),
            path: library["downloads"]["artifact"]["path"]
                .as_str()
                .unwrap()
                .to_string(),
            sha1: library["downloads"]["artifact"]["sha1"]
                .as_str()
                .unwrap()
                .to_string(),
            size: library["downloads"]["artifact"]["size"].as_i64().unwrap(),
            url: library["downloads"]["artifact"]["url"]
                .as_str()
                .unwrap()
                .to_string(),
        })
    }
    result
}
