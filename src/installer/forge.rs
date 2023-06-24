use serde_json::Value;
use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::PathBuf,
};
use tokio::fs;
use zip::ZipArchive;

use crate::{utils::{
    folder::{get_path, MinecraftLocation},
}, core::version::Artifact};

pub struct ForgeVersion {
    pub installer: ForgeVersionInstaller,
    pub universal: ForgeVersionUniversal,

    /// The Minecraft version
    pub mcversion: String,

    /// The forge Version
    pub version: String,
    pub r#type: ForgeVersionType,
}

pub struct ForgeVersionInstaller {
    pub md5: String,
    pub sha1: String,

    /// The url path to concat with forge maven
    pub path: String,
}

pub struct ForgeVersionUniversal {
    pub md5: String,
    pub sha1: String,

    /// The url path to concat with forge maven
    pub path: String,
}

pub enum ForgeVersionType {
    Buggy(String),
    Recommended(String),
    Common(String),
    Latest(String),
}

#[derive(Debug)]
/// All the useful entries in forge installer jar
pub struct ForgeInstallerEntries {
    /// maven/net/minecraftforge/forge/${forgeVersion}/forge-${forgeVersion}-universal.jar
    pub forge_jar: Option<Vec<u8>>,

    /// maven/net/minecraftforge/forge/${forgeVersion}/forge-${forgeVersion}-universal.jar
    pub forge_universal_jar: Option<Vec<u8>>,

    /// data/client.lzma
    pub client_lzma: Option<Vec<u8>>,

    /// data/server.lzma
    pub server_lzma: Option<Vec<u8>>,

    /// install_profile.json
    pub install_profile_json: Option<Vec<u8>>,

    /// version.json
    pub version_json: Option<Vec<u8>>,

    /// forge-${forgeVersion}-universal.jar
    pub legacy_universal_jar: Option<Vec<u8>>,

    /// data/run.sh
    pub run_sh: Option<Vec<u8>>,

    /// data/run.bat
    pub run_bat: Option<Vec<u8>>,

    /// data/unix_args.txt
    pub unix_args: Option<Vec<u8>>,

    /// data/user_jvm_args.txt
    pub user_jvm_args: Option<Vec<u8>>,

    /// data/win_args.txt
    pub win_args: Option<Vec<u8>>,
}

pub struct RequiredVersion {
    pub installer: Option<RequiredVersionInstaller>,
    pub mcversion: String,
    pub version: String,
}

pub struct RequiredVersionInstaller {
    pub sha1: Option<String>,
    /// The url path to concat with forge maven
    pub path: String,
}

const DEFAULT_FORGE_MAVEN: &str = "http://files.minecraftforge.net/maven";

pub struct InstallForgeOptions {
    /// The alterative maven host to download library. It will try to use these host from the `[0]` to the `[maven.length - 1]`
    pub maven_host: Option<Vec<String>>,

    /// Control how many libraries download task should run at the same time.
    /// It will override the `maxConcurrencyOption` if this is presented.
    ///
    /// This will be ignored if you have your own downloader assigned.
    pub libraries_download_concurrency: Option<i32>,

    /// When you want to install a version over another one.
    ///
    /// Like, you want to install liteloader over a forge version.
    /// You should fill this with that forge version id.
    pub inherits_from: Option<String>,

    /// Override the newly installed version id.
    ///
    /// If this is absent, the installed version id will be either generated or provided by installer.
    pub version_id: Option<String>,

    /// New forge (>=1.13) require java to install. Can be a executor or java executable path.
    pub java: Option<String>,
}

pub enum ForgeType {
    New,
    Legacy,
    Bad,
}

struct DecompressionTask {
    zip: ZipArchive<File>,
    entry: String,
    to: PathBuf,
}

async fn download_forge_installer(
    forge_version: &str,
    required_version: RequiredVersion,
    minecraft: MinecraftLocation,
    _options: Option<InstallForgeOptions>,
) -> String {
    let path = if let Some(installer) = &required_version.installer {
        String::from(&installer.path)
    } else {
        format!(
            "net/minecraftforge/forge/{}/forge-{}-installer.jar",
            forge_version, forge_version
        )
    };
    let forge_maven_path = path.replace("/maven", "").replace("maven", "");
    let sha1 = match &required_version.installer {
        Some(installer) => match &installer.sha1 {
            Some(sha1) => String::from(sha1),
            _ => "".to_string(),
        },
        _ => "".to_string(),
    };
    let library = Artifact {
        path: format!(
            "net/minecraftforge/forge/{}/forge-{}-installer.jar",
            forge_version, forge_version
        ),
        size: 0,
        sha1,
        url: format!("{}/{}", DEFAULT_FORGE_MAVEN, forge_maven_path),
    };
    let file_path = get_path(&minecraft.root.join(library.path.clone()));
    println!("{}", library.url);
    // download(Download {
    //     url: library.url,
    //     file: file_path.clone(),
    // })
    // .await;
    file_path
}

async fn read_zipfiles_to_bytes(
    zip: &mut ZipArchive<File>,
    entries: Vec<String>,
) -> HashMap<String, Vec<u8>> {
    // todo: 异步执行这个函数
    let mut result = HashMap::new();
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).unwrap();
        let entry_name = entry.name().to_owned();
        println!("{}", entry_name);
        for target_file in entries.clone() {
            println!("{}", target_file);
            if entry_name.starts_with(&target_file) {
                let mut data = Vec::new();
                entry.read_to_end(&mut data).unwrap();
                result.insert(target_file, data);
            }
        }
    }

    result
}

async fn walk_forge_installer_entries(
    mut zip: ZipArchive<File>,
    forge_version: &str,
) -> ForgeInstallerEntries {
    let entries = vec![
        format!(
            "maven/net/minecraftforge/forge/{}/forge-{}.jar",
            forge_version, forge_version
        ),
        format!(
            "maven/net/minecraftforge/forge/{}/forge-{}-universal.jar",
            forge_version, forge_version
        ),
        "data/client.lzma".to_string(),
        "data/server.lzma".to_string(),
        "install_profile.json".to_string(),
        "version.json".to_string(),
        format!("forge-{}-universal.jar", forge_version),
        "data/run.sh".to_string(),
        "data/run.bat".to_string(),
        "data/unix_args.txt".to_string(),
        "data/unix_jvm_args".to_string(),
        "data/win_args".to_string(),
    ];
    let content = read_zipfiles_to_bytes(&mut zip, entries.clone()).await;
    let get_content = move |index: usize| -> Option<Vec<u8>> {
        content.get(&entries.get(index).unwrap().clone()).cloned()
    };
    ForgeInstallerEntries {
        forge_jar: get_content(0),
        forge_universal_jar: get_content(1),
        client_lzma: get_content(2),
        server_lzma: get_content(3),
        install_profile_json: get_content(4),
        version_json: get_content(5),
        legacy_universal_jar: get_content(6),
        run_sh: get_content(7),
        run_bat: get_content(8),
        unix_args: get_content(9),
        user_jvm_args: get_content(10),
        win_args: get_content(11),
    }
}

async fn unpack_forge_installer(
    zip: ZipArchive<File>,
    entries: ForgeInstallerEntries,
    forge_version: &str,
    minecraft: MinecraftLocation,
    jar_path: PathBuf,
    // profile: InstallProfile,
    options: Option<InstallForgeOptions>,
) {
    let version_json_raw = entries.version_json.unwrap();
    let mut version_json: Value =
        serde_json::from_str(&String::from_utf8(version_json_raw).unwrap()).unwrap();

    // apply override for inheritsFrom
    if let Some(options) = options {
        if let Some(id) = options.version_id {
            version_json["id"] = Value::String(id);
        }
        if let Some(inherits_from) = options.inherits_from {
            version_json["inheritsFrom"] = Value::String(inherits_from);
        }
    }

    // resolve all the required paths
    let root_path = minecraft.root;

    let version_json_path =
        root_path.join(format!("{}.json", version_json["id"].as_str().unwrap()));
    let install_json_path = root_path.join("install_profile.json");

    let data_root = jar_path.parent().unwrap().to_path_buf();

    let mut decompression_tasks = Vec::new();

    let unpack_data =
        |entry: String, zip: ZipArchive<File>, decompression_tasks: &mut Vec<DecompressionTask>| {
            decompression_tasks.push(DecompressionTask {
                zip,
                entry: entry.clone(),
                to: PathBuf::new().join(entry.replace("data/", "")),
            });
        };

    fs::create_dir_all(version_json_path.parent().unwrap())
        .await
        .unwrap();

    if let Some(_) = entries.forge_universal_jar {
        decompression_tasks.push(DecompressionTask {
            zip,
            entry: format!(
                "maven/net/minecraftforge/forge/{}/forge-{}-universal.jar",
                forge_version, forge_version
            ),
            to: minecraft.libraries.clone().join(format!(
                "maven/net/minecraftforge/forge/{}/forge-{}-universal.jar",
                forge_version, forge_version
            )),
        })
    }

    
}

pub async fn install_forge(
    version: RequiredVersion,
    minecraft: MinecraftLocation,
    options: Option<InstallForgeOptions>,
) {
    let mcversion: Vec<_> = version.mcversion.split(".").collect();
    let minor = *mcversion.get(1).unwrap();
    let minor_version = minor.parse::<i32>().unwrap();

    let forge_version = if minor_version >= 7 && minor_version <= 8 {
        format!(
            "{}-{}-{}",
            version.mcversion, version.version, version.mcversion
        )
    } else {
        format!("{}-{}", version.mcversion, version.version)
    };

    let installer_jar_path =
        download_forge_installer(&forge_version, version, minecraft, options).await;
    println!("{}", installer_jar_path);

    // thread::sleep(tokio::time::Duration::from_secs(1));

    let installer_jar = File::open(installer_jar_path).unwrap();
    let installer_jar_ziparchive = zip::ZipArchive::new(installer_jar).unwrap();
    println!("{}", installer_jar_ziparchive.len());

    let entries = walk_forge_installer_entries(installer_jar_ziparchive, &forge_version).await;
    let install_profile_json = match &entries.install_profile_json {
        None => panic!("Bad forge installer jar!"),
        Some(data) => String::from_utf8(data.to_vec()).unwrap(),
    };
    let profile: Value = serde_json::from_str(&install_profile_json).unwrap();
    let forge_type = if let Some(_) = &entries.install_profile_json {
        if let Some(_) = entries.version_json {
            ForgeType::New
        } else if let Some(_) = &entries.legacy_universal_jar {
            ForgeType::Legacy
        } else {
            ForgeType::Bad
        }
    } else {
        ForgeType::Bad
    };
    match forge_type {
        ForgeType::New => {
            // let version_id = unpack_forge_installer(zip, entries);
        }
        ForgeType::Legacy => (),
        ForgeType::Bad => panic!("Bad forge installer jar!"),
    }
}

#[tokio::test]
async fn test() {
    install_forge(
        RequiredVersion {
            installer: None,
            mcversion: "1.19.4".to_string(),
            version: "45.1.0".to_string(),
        },
        MinecraftLocation::new("test"),
        None,
    )
    .await;
}
