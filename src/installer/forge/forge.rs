









// #[tokio::test]
// async fn test() {
//     let version_list = ForgeVersionList::from_mcversion("1.19.4").await;
//     println!("{:#?}", version_list);
// }

// #[tokio::test]
// async fn test2() {
//     install_forge(
//         RequiredVersion {
//             installer: None,
//             mcversion: "1.19.4".to_string(),
//             version: "45.1.0".to_string(),
//         },
//         MinecraftLocation::new("test"),
//         None,
//     )
//     .await;
// }

// #[tokio::test]
// async fn test1() {
//     install_forge(
//         RequiredVersion {
//             installer: None,
//             mcversion: "1.7.10".to_string(),
//             version: "10.13.4.1614".to_string(),
//         },
//         MinecraftLocation::new("test"),
//         None,
//     )
//     .await;
// }

// #[tokio::test]
// async fn test3() {
//     install_forge(
//         RequiredVersion {
//             installer: None,
//             mcversion: "1.19.4".to_string(),
//             version: "45.1.0".to_string(),
//         },
//         MinecraftLocation::new("test"),
//         Some(InstallForgeOptions {
//             maven_host: None,
//             libraries_download_concurrency: None,
//             inherits_from: None,
//             version_id: Some("123".to_string()),
//             java: None,
//         }),
//     )
//     .await;
// }
