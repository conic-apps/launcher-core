use tokio::fs::{create_dir_all, self};

use crate::core::{version::LibraryInfo, folder::MinecraftLocation};

use super::{*, install_profile::InstallProfileLegacy};

pub(super) async fn install_legacy_forge_from_zip(
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
