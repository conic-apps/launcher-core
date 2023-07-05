pub mod folder;
pub mod task;
pub mod version;

use tokio::process::Command;

pub struct PlatformInfo {
    pub arch: String,
    pub name: String,
    pub version: String,
}

impl PlatformInfo {
    /// get platform information
    pub async fn new() -> Self {
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
            version: {
                #[cfg(windows)]
                {
                    let mut command = Command::new("C:\\Windows\\System32\\cmd.exe");
                    command.args(&["/C", r#"powershell -c [System.Environment]::OSVersion.Version"#]);
                    let output = command.output().await.unwrap();
                    let stdout = String::from_utf8(output.stdout).unwrap();
                
                    let regex = Regex::new(r"\s+").unwrap();
                    regex.replace_all(&stdout, ".").to_string()
                }
                #[cfg(not(windows))]
                {
                    let mut command = Command::new("uname");
                    command.args(&["-r"]);
                    let output = command.output().await.unwrap();
                    String::from_utf8(output.stdout).unwrap()
                }
            },
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
