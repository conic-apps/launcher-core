use uname::uname;
#[derive(Debug)]
pub struct PlatformInfo {
    pub name: String,
    pub version: String,
    pub arch: String,
}
impl PlatformInfo {
    pub fn get() -> PlatformInfo {
        let info = uname().unwrap();
        PlatformInfo {
            name: if cfg!(target_os = "windows") {
                "windows"
            } else if cfg!(target_os = "linux") {
                "linux"
            } else if cfg!(target_os = "macos") {
                "osx"
            } else {
                "unknown"
            }
            .to_string(),
            version: info.version,
            arch: if cfg!(target_arch = "x86_64") {
                "x64"
            } else if cfg!(target_arch = "x86") {
                "x86"
            } else if cfg!(target_arch = "mips") {
                "mips"
            } else if cfg!(target_arch = "powerpc") {
                "powerpc"
            } else if cfg!(target_arch = "powerpc64") {
                "powerpc64"
            } else if cfg!(target_arch = "arm") {
                "arm"
            } else if cfg!(target_arch = "aarch64") {
                "aarch64"
            } else {
                "unknown"
            }
            .to_string(),
        }
    }
}

#[test]
fn test() {
    println!("{:#?}", PlatformInfo::get());
}