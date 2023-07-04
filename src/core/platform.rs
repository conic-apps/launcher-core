//! A module for platform information

use uname::uname;

// todo: 静态...

/// get platform information including `name`, `version`, `arch`
/// 
/// # Example
/// 
/// basic usage:
/// 
/// ```rust
/// use mgl_core::core::platform::PlatformInfo;
/// 
/// let info = PlatformInfo::new();
/// println!("{:#?}", info.name);
#[derive(Debug)]
pub struct PlatformInfo {
    pub name: String,
    pub version: String,
    pub arch: String,
}

impl PlatformInfo {
    /// get platform information
    pub fn new() -> Self {
        let info = uname().unwrap();
        Self {
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
