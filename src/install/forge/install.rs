use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
    str::FromStr,
    thread,
    time::Duration,
};

use reqwest::Response;
use zip::ZipArchive;

use crate::{
    core::{folder::MinecraftLocation, version::Artifact},
    install::forge::{
        install_profile::{InstallProfile, InstallProfileLegacy},
        legacy_install::install_legacy_forge_from_zip,
        new_install::unpack_forge_installer,
    },
    utils::{
        download::{download, Download},
        unzip::filter_entries,
    },
};

use super::*;

const DEFAULT_FORGE_MAVEN: &str = "http://files.minecraftforge.net/maven";

// todo: 使用 Steve-xmh/forge-install-bootstrapper 修复新版forge安装

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
    let file_path = minecraft
        .get_library_by_path(&library.path)
        .to_str()
        .unwrap()
        .to_string();
    let response = download(Download {
        url: library.url,
        file: file_path.clone(),
        sha1: None,
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
