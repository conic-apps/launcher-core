// use serde_json::Value;
// use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};
// use tokio::fs;
// use zip::ZipArchive;

// use crate::{
//     core::version::Artifact,
//     utils::folder::{get_path, MinecraftLocation},
// };

// use super::profile::{InstallProfile, InstallProfileData};

use std::{
    collections::HashMap,
    fs::File,
    io::{self, Cursor, Read},
    path::PathBuf,
    str::FromStr,
    thread,
    time::Duration,
};

use reqwest::Response;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::fs::{self, create_dir_all};
use zip::ZipArchive;

use crate::{
    core::version::{Artifact, LibraryInfo, Version},
    installer::profile::{InstallProfile, InstallProfileLegacy},
    utils::{
        download::{download, Download},
        folder::{get_path, MinecraftLocation},
        unzip::{decompression_files, filter_entries, Entry},
    },
};

use super::profile::InstallProfileData;

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

/// All the useful entries in forge installer jar
pub struct ForgeInstallerEntries {
    /// maven/net/minecraftforge/forge/${forgeVersion}/forge-${forgeVersion}-universal.jar
    pub forge_jar: Option<Entry>,

    /// maven/net/minecraftforge/forge/${forgeVersion}/forge-${forgeVersion}-universal.jar
    pub forge_universal_jar: Option<Entry>,

    /// data/client.lzma
    pub client_lzma: Option<Entry>,

    /// data/server.lzma
    pub server_lzma: Option<Entry>,
    /// install_profile.json
    pub install_profile_json: Option<Entry>,

    /// version.json
    pub version_json: Option<Entry>,

    /// forge-${forgeVersion}-universal.jar
    pub legacy_universal_jar: Option<Entry>,

    /// data/run.sh
    pub run_sh: Option<Entry>,

    /// data/run.bat
    pub run_bat: Option<Entry>,

    /// data/unix_args.txt
    pub unix_args: Option<Entry>,

    /// data/user_jvm_args.txt
    pub user_jvm_args: Option<Entry>,

    /// data/win_args.txt
    pub win_args: Option<Entry>,
}

pub struct ForgeInstallerEntriesPatten {
    /// maven/net/minecraftforge/forge/${forgeVersion}/forge-${forgeVersion}-universal.jar
    pub forge_jar: Option<Entry>,

    /// maven/net/minecraftforge/forge/${forgeVersion}/forge-${forgeVersion}-universal.jar
    pub forge_universal_jar: Option<Entry>,

    /// data/client.lzma
    pub client_lzma: Option<Entry>,

    /// data/server.lzma
    pub server_lzma: Option<Entry>,
    /// install_profile.json
    pub install_profile_json: Entry,

    /// version.json
    pub version_json: Entry,

    /// forge-${forgeVersion}-universal.jar
    pub legacy_universal_jar: Option<Entry>,

    /// data/run.sh
    pub run_sh: Option<Entry>,

    /// data/run.bat
    pub run_bat: Option<Entry>,

    /// data/unix_args.txt
    pub unix_args: Option<Entry>,

    /// data/user_jvm_args.txt
    pub user_jvm_args: Option<Entry>,

    /// data/win_args.txt
    pub win_args: Option<Entry>,
}

pub struct ForgeLegacyInstallerEntriesPatten {
    /// install_profile.json
    pub install_profile_json: Entry,

    /// forge-${forgeVersion}-universal.jar
    pub legacy_universal_jar: Entry,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
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

async fn download_forge_installer(
    forge_version: &str,
    required_version: RequiredVersion,
    minecraft: &MinecraftLocation,
    _options: &Option<InstallForgeOptions>,
) -> (String, Response) {
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
            _ => String::new(),
        },
        _ => String::new(),
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
    let file_path = minecraft.get_library_by_path(&library.path).to_str().unwrap().to_string();
    let response = download(Download {
        url: library.url,
        file: file_path.clone(),
    })
    .await;
    (file_path, response)
}

async fn walk_forge_installer_entries<R: Read + io::Seek>(
    mut zip: ZipArchive<R>,
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
    let filted_entries = filter_entries(&mut zip, &entries);
    let get_content = move |index: usize| -> Option<Entry> {
        match filted_entries.get(entries.clone().get(index).unwrap()) {
            None => None,
            Some(value) => Some(value.clone()),
        }
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

async fn unpack_forge_installer<R: Read + io::Seek>(
    zip: &mut ZipArchive<R>,
    entries: ForgeInstallerEntries,
    forge_version: &str,
    minecraft: MinecraftLocation,
    jar_path: PathBuf,
    profile: InstallProfile,
    options: Option<InstallForgeOptions>,
) -> String {
    let version_json_raw = entries.version_json.unwrap().content;
    let mut version_json: Value =
        serde_json::from_str(&String::from_utf8(version_json_raw).unwrap()).unwrap();

    //  apply override for inheritsFrom
    if let Some(options) = options {
        if let Some(id) = options.version_id {
            version_json["id"] = Value::String(id);
        }
        if let Some(inherits_from) = options.inherits_from {
            version_json["inheritsFrom"] = Value::String(inherits_from);
        }
    }

    //   resolve all the required paths
    let root_path = minecraft.root.clone();

    let version_json_path =
        root_path.join(format!("{}.json", version_json["id"].as_str().unwrap()));
    let install_json_path = root_path.join("install_profile.json");

    let data_root = jar_path.parent().unwrap().to_path_buf();

    let mut decompression_tasks: Vec<(String, PathBuf)> = Vec::new();

    create_dir_all(version_json_path.parent().unwrap())
        .await
        .unwrap();

    if let Some(_) = entries.forge_universal_jar {
        decompression_tasks.push((
            format!(
                "maven/net/minecraftforge/forge/{}/forge-{}-universal.jar",
                forge_version, forge_version
            ),
            minecraft.libraries.clone().join(format!(
                "maven/net/minecraftforge/forge/{}/forge-{}-universal.jar",
                forge_version, forge_version
            )),
        ));
    }
    let mut profile_data;
    if let Some(h) = profile.data.clone() {
        profile_data = h;
    } else {
        profile_data = HashMap::new();
    }

    let installer_maven = format!("net.minecraftforge:forge:{forge_version}:installer");
    let profile_data_installer = InstallProfileData {
        client: Some(format!("[{installer_maven}]")),
        server: Some(format!("[{installer_maven}]")),
    };
    profile_data.insert("INSTALLER".to_string(), profile_data_installer);

    let path = &format!("net/minecraftforge/forge/{forge_version}/forge-{forge_version}.jar");
    if let Some(server_lzma) = entries.server_lzma {
        // forge version and mavens, compatible with twitch api
        let server_maven = format!("net.minecraftforge:forge:{forge_version}:serverdata@lzma");
        // override forge bin patch location
        profile_data.insert(
            "BINPATCH".to_string(),
            InstallProfileData {
                client: None,
                server: Some(format!("[{server_maven}]")),
            },
        );

        let server_bin_path = minecraft.libraries.join(path);
        decompression_tasks.push((server_lzma.name.clone(), server_bin_path));
    }

    if let Some(client_lzma) = entries.client_lzma {
        //forge version and mavens, compatible with twitch api
        let client_maven = format!("net.minecraftforge:forge:{forge_version}:clientdata@lzma");
        //override forge bin patch location
        let mut server = String::new();
        let binpatch = profile_data.get("BINPATCH");
        if let Some(b) = binpatch {
            if let Some(s) = b.server.clone() {
                server = s;
            }
        }
        profile_data.insert(
            "BINPATCH".to_string(),
            InstallProfileData {
                client: Some(format!("[{client_maven}]]")),
                server: Some(server),
            },
        );

        let client_bin_path = minecraft.libraries.join(format!(
            "net/minecraftforge/forge/{forge_version}/forge-{forge_version}.jar"
        ));
        decompression_tasks.push((client_lzma.name.clone(), client_bin_path));
    }

    if let Some(forge_jar) = entries.forge_jar {
        let file_name = entries.forge_universal_jar.unwrap().name;
        fs::write(
            minecraft.get_library_by_path(&file_name[file_name.find('/').unwrap() + 1..]),
            forge_jar.content,
        )
        .await
        .unwrap();
    }

    let unpack_data = |entry: Entry| async {
        let path = data_root.clone().join(entry.name);
        create_dir_all(path.parent().unwrap()).await.unwrap();
        fs::write(path, entry.content).await.unwrap();
    };

    if let Some(run_bat) = entries.run_bat {
        unpack_data(run_bat).await;
    }
    if let Some(run_sh) = entries.run_sh {
        unpack_data(run_sh).await;
    }
    if let Some(win_args) = entries.win_args {
        unpack_data(win_args).await;
    }
    if let Some(unix_args) = entries.unix_args {
        unpack_data(unix_args).await;
    }
    if let Some(unix_jvm_args) = entries.user_jvm_args {
        unpack_data(unix_jvm_args).await;
    }

    create_dir_all(install_json_path.parent().unwrap())
        .await
        .unwrap();
    fs::write(
        install_json_path,
        serde_json::to_string_pretty(&profile).unwrap(),
    )
    .await
    .unwrap();

    create_dir_all(version_json_path.parent().unwrap())
        .await
        .unwrap();
    fs::write(
        version_json_path,
        serde_json::to_string_pretty(&version_json).unwrap(),
    )
    .await
    .unwrap();

    decompression_files(zip, decompression_tasks).await;

    Version::from_value(version_json).unwrap().id
}

async fn install_legacy_forge_from_zip(
    entries: ForgeLegacyInstallerEntriesPatten,
    profile: InstallProfileLegacy,
    minecraft: MinecraftLocation,
    options: Option<InstallForgeOptions>,
) {
    let options = match options {
        Some(options) => options,
        None => InstallForgeOptions {
            maven_host: None,
            libraries_download_concurrency: None,
            inherits_from: None,
            version_id: None,
            java: None,
        },
    };
    let mut version_json = profile.version_info.unwrap();

    // apply override for inheritsFrom
    version_json.id = options.version_id.unwrap_or(version_json.id);
    version_json.inherits_from = match options.inherits_from {
        None => version_json.inherits_from,
        Some(inherits_from) => Some(inherits_from),
    };

    let root_path = minecraft.get_version_root(&version_json.id);
    let version_json_path = root_path.join(format!("{}.json", version_json.id));

    create_dir_all(&version_json_path.parent().unwrap())
        .await
        .unwrap();
    let library = version_json.libraries.clone().unwrap();
    let library = library
        .iter()
        .find(|l| {
            l["name"]
                .as_str()
                .unwrap()
                .starts_with("net.minecraftforge:forge")
        })
        .unwrap();
    let library = LibraryInfo::from_value(library);

    fs::write(
        version_json_path,
        serde_json::to_string_pretty(&version_json).unwrap(),
    )
    .await
    .unwrap();

    create_dir_all(
        minecraft
            .get_library_by_path(&library.path)
            .parent()
            .unwrap(),
    )
    .await
    .unwrap();
    fs::write(
        minecraft.get_library_by_path(&library.path),
        entries.legacy_universal_jar.content,
    )
    .await
    .unwrap();
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

    let (installer_jar_path, _installer_jar) =
        download_forge_installer(&forge_version, version, &minecraft, &options).await;
    println!("{}", installer_jar_path);
    thread::sleep(Duration::from_secs(1));
    let installer_jar = ZipArchive::new(File::open(&installer_jar_path).unwrap()).unwrap();

    let entries = walk_forge_installer_entries(installer_jar, &forge_version).await;
    let mut installer_jar = ZipArchive::new(File::open(&installer_jar_path).unwrap()).unwrap();

    let install_profile_json = match &entries.install_profile_json {
        None => panic!("Bad forge installer jar!"),
        Some(data) => String::from_utf8(data.content.clone()).unwrap(),
    };
    println!("{}", install_profile_json);
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
            let profile: InstallProfile = serde_json::from_str(&install_profile_json).unwrap();
            let _version_id = unpack_forge_installer(
                &mut installer_jar,
                entries,
                &forge_version,
                minecraft,
                PathBuf::from_str(&installer_jar_path).unwrap(),
                profile,
                options,
            )
            .await;
        }
        ForgeType::Legacy => {
            let profile: InstallProfileLegacy =
                serde_json::from_str(&install_profile_json).unwrap();
            let entries = ForgeLegacyInstallerEntriesPatten {
                install_profile_json: entries.install_profile_json.unwrap(),
                legacy_universal_jar: entries.legacy_universal_jar.unwrap(),
            };
            install_legacy_forge_from_zip(entries, profile, minecraft, options).await;
        }
        ForgeType::Bad => panic!("Bad forge installer jar!"),
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ForgeVersionListItem {
    pub _id: String,
    pub build: u32,
    pub __v: u32,
    pub version: String,
    pub modified: String,
    pub mcversion: String,
    pub files: Vec<ForgeInstallerFile>,
    pub branch: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ForgeInstallerFile {
    pub format: String,
    pub category: String,
    pub hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ForgeVersionList(Vec<ForgeVersionListItem>);

impl ForgeVersionList {
    pub async fn new() -> Self {
        reqwest::get("https://bmclapi2.bangbang93.com/forge/list/0")
            .await
            .unwrap()
            .json::<Self>()
            .await
            .unwrap()
    }

    pub async fn from_mcversion(mcversion: &str) -> Self {
        reqwest::get(format!(
            "https://bmclapi2.bangbang93.com/forge/minecraft/{mcversion}"
        ))
        .await
        .unwrap()
        .json::<Self>()
        .await
        .unwrap()
    }
}

#[tokio::test]
async fn test() {
    let version_list = ForgeVersionList::from_mcversion("1.19.4").await;
    println!("{:#?}", version_list);
}

#[tokio::test]
async fn test2() {
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

#[tokio::test]
async fn test1() {
    install_forge(
        RequiredVersion {
            installer: None,
            mcversion: "1.7.10".to_string(),
            version: "10.13.4.1614".to_string(),
        },
        MinecraftLocation::new("test"),
        None,
    )
    .await;
}

#[tokio::test]
async fn test3() {
    install_forge(
        RequiredVersion {
            installer: None,
            mcversion: "1.19.4".to_string(),
            version: "45.1.0".to_string(),
        },
        MinecraftLocation::new("test"),
        Some(InstallForgeOptions {
            maven_host: None,
            libraries_download_concurrency: None,
            inherits_from: None,
            version_id: Some("123".to_string()),
            java: None,
        }),
    )
    .await;
}
