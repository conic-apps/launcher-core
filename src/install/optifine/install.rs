use std::{ffi::OsStr, fmt::Display, path::Path};

use tokio::{fs, io::AsyncWriteExt};

use crate::{
    install::optifine::DEFAULT_META_URL,
    utils::download::{download, Download}, core::folder::MinecraftLocation,
};

use super::InstallOptifineOptions;

const OPTIFINE_INSTALL_HELPER: &[u8] = include_bytes!("./optifine-installer.jar");

/// Download forge installer
pub async fn download_optifine_installer<P, D>(
    mcversion: &str,
    optifine_type: &str,
    optifine_patch: &str,
    dest_path: P,
    remote: Option<D>,
) where
    P: AsRef<Path> + AsRef<OsStr>,
    D: Display,
{
    let url = match remote {
        None => format!("{DEFAULT_META_URL}/{mcversion}/{optifine_type}/{optifine_patch}"),
        Some(remote) => format!("{remote}/{mcversion}/{optifine_type}/{optifine_patch}"),
    };
    download(Download {
        url,
        file: dest_path,
        sha1: None,
    })
    .await;
}

/// Install optifine
///
/// referenced from [Sharp Craft Launcher](https://github.com/Steve-xmh/scl/blob/main/scl-core/src/download/optifine.rs)
///
/// #### Note:
///
/// if you need to install as mod, use download_optifine_install function
pub async fn install_optifine(
    minecraft: MinecraftLocation,
    version_name: &str,
    mcversion: &str,
    optifine_type: &str,
    optifine_patch: &str,
    java_executable_path: &str,
    options: Option<InstallOptifineOptions>,
) {
    let options = match options {
        None => InstallOptifineOptions {
            use_forge_tweaker: None,
            inherits_from: None,
            version_id: None,
            remote: None,
        },
        Some(options) => options,
    };
    let full_path = minecraft.get_library_by_path(format!("net/optifine/{mcversion}-{optifine_type}-{optifine_patch}/Optifine-{mcversion}-{optifine_type}-{optifine_patch}.jar"));
    let full_path = full_path.to_str().unwrap();

    download_optifine_installer(
        mcversion,
        optifine_type,
        optifine_patch,
        full_path,
        options.remote,
    )
    .await;

    let installer_path = minecraft
        .get_library_by_path("net/stevexmh/optifine-installer/0.0.0/optifine-installer.jar");
    let installer_path = installer_path.to_str().unwrap();

    fs::create_dir_all(Path::new(&installer_path).parent().unwrap())
        .await
        .unwrap();

    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(installer_path)
        .await
        .unwrap();
    file.write_all(OPTIFINE_INSTALL_HELPER).await.unwrap();
    file.flush().await.unwrap();
    file.sync_all().await.unwrap();

    // #[cfg(not(windows))]
    let mut command = tokio::process::Command::new(java_executable_path);

    // // #[cfg(windows)]
    // let mut command = {
    //     use tokio::process::windows::CommandExt;
    //     let mut command = tokio::process::Command::new(java_executable_path);
    //     command.creation_flags(0x08000000);
    //     command
    // };

    command.args(&[
        "-cp",
        &format!("{installer_path}:{full_path}"),
        "net.stevexmh.OptifineInstaller",
        minecraft.root.to_str().unwrap(),
        version_name,
    ]);

    command.status().await.unwrap();
}

#[tokio::test]
async fn test() {
    // install(
    //     "1.19.4",
    //     MinecraftLocation::new("test"),
    //     EventListeners::new(),
    // )
    // .await;
    // install_optifine(
    //     MinecraftLocation::new("test"),
    //     "1.19.4-optifine",
    //     "1.19.4",
    //     "HD_U",
    //     "I3",
    //     "java",
    //     None,
    // )
    // .await;
}

//     let options = match options {
//         None => InstallOptifineOptions {
//             use_forge_tweaker: None,
//             inherits_from: None,
//             version_id: None,
//         },
//         Some(options) => options,
//     };

//     // progress: 0%

//     let mut zip = ZipArchive::new(File::open(installer_path).unwrap()).unwrap();
//     let entries = Entry::from_zip_archive(&mut zip);
//     let record = Entry::get_entries_record(entries);

//     // progress: 10%

//     let entry = record
//         .get("net/optifine/Config.class")
//         .or_else(|| record.get("Config.class"))
//         .or_else(|| record.get("notch/net/optifine/Config.class"));
//     if let None = entry {
//         panic!("Bad Optifine!");
//     }
//     let entry = entry.unwrap();

//     let launch_wrapper_version_entry = record.get("launchwrapper-of.txt");
//     let launch_wrapper_version = match launch_wrapper_version_entry {
//         None => None,
//         Some(entry) => Some(entry.content.clone()),
//     };

//     // progress: 15%

//     let visiter =
// }
