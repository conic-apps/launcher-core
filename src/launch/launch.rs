/*
 * Magical Launcher Core
 * Copyright (C) 2023 Broken-Deer <old_driver__@outlook.com> and contributors
 *
 * This program is free software, you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::process::ExitStatus;

use anyhow::Result;

use crate::{core::{folder::MinecraftLocation, JavaExec, PlatformInfo, task::TaskEventListeners}, install::install};

use super::{argument::LaunchArguments, options::LaunchOptions};

/// All game launcher
///
/// Use `Launcher::new` to spawn an instance with minimal launch options
pub struct Launcher {
    pub launch_options: LaunchOptions,
    pub minecraft: MinecraftLocation,

    /// Whether to check game integrity before launching
    pub check_game_integrity: bool,

    pub exit_status: Option<ExitStatus>,

    pub java: JavaExec,
}

impl Launcher {
    /// spawn an instance with default launch options
    pub async fn new(
        version_id: &str,
        minecraft: MinecraftLocation,
        java: JavaExec,
    ) -> Result<Self> {
        let launch_options = LaunchOptions::new(version_id, minecraft.clone()).await?;
        Ok(Self {
            launch_options,
            minecraft,
            check_game_integrity: true,
            exit_status: None,
            java,
        })
    }

    /// spawn an instance with custom launch options
    pub async fn from_options(launch_options: LaunchOptions, java: JavaExec) -> Self {
        Self {
            minecraft: launch_options.minecraft_location.clone(),
            launch_options,
            check_game_integrity: true,
            exit_status: None,
            java,
        }
    }

    /// launch game.
    ///
    // /// Note: this function will block the current thread when game running
    pub async fn launch(&mut self) -> Result<()> {
        let platform = PlatformInfo::new().await;
        let options = self.launch_options.clone();
        let version = self
            .launch_options
            .version
            .parse(&self.minecraft, &platform)
            .await?;
        let mut command = LaunchArguments::from_launch_options(options.clone(), version.clone())
            .await?
            .to_async_command(self.java.clone(), options, &platform)
            .await?;
        let mut child = command.spawn()?;
        self.exit_status = Some(child.wait().await?);
        Ok(())
    }
}

#[tokio::test]
async fn test() {
    let a = MinecraftLocation::new("/home/brokendeer/桌面/magical-launcher-core/test");
    install("1.20.1", a.clone(), TaskEventListeners::new()).await.unwrap();
    let options = LaunchOptions::new("1.20.1", a).await.unwrap();
    let mut b = Launcher::from_options(options, JavaExec::new("/usr/lib64/jvm/java-17-openjdk-17").await).await;
    b.launch().await.unwrap();
}