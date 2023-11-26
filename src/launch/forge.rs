/*
 * Amethyst Launcher Core
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

use std::{
    io::{BufRead, BufReader},
    process::{ExitStatus, Stdio},
    sync::{Arc, Mutex},
    thread,
};

use anyhow::Result;
use async_trait::async_trait;

use crate::core::{folder::MinecraftLocation, JavaExec, PlatformInfo};

use super::{argument::LaunchArguments, launch::Launch, options::LaunchOptions};

pub struct ForgeLauncher {
    pub launch_options: LaunchOptions,
    pub minecraft: MinecraftLocation,

    /// Whether to check game integrity before launching
    pub check_game_integrity: bool,

    pub exit_status: Option<ExitStatus>,

    pub java: JavaExec,
}

#[async_trait]
impl Launch for ForgeLauncher {
    async fn new(version_id: &str, minecraft: MinecraftLocation, java: JavaExec) -> Result<Self> {
        let launch_options = LaunchOptions::new(version_id, &minecraft).await?;
        Ok(Self {
            launch_options,
            minecraft,
            check_game_integrity: true,
            exit_status: None,
            java,
        })
    }

    fn from_options(launch_options: LaunchOptions, java: JavaExec) -> Self {
        Self {
            minecraft: launch_options.minecraft_location.clone(),
            launch_options,
            check_game_integrity: true,
            exit_status: None,
            java,
        }
    }

    async fn launch(
        &mut self,
        on_start: Option<Box<dyn FnMut() + Send>>,
        on_stdout: Option<Box<dyn FnMut(String) + Send>>,
        on_stderr: Option<Box<dyn FnMut(String) + Send>>,
        on_exit: Option<Box<dyn FnMut(i32) + Send>>,
    ) -> Result<()> {
        let mut on_start = match on_start {
            None => Box::new(|| {}),
            Some(on_start) => on_start,
        };
        let on_stdout = match on_stdout {
            None => Box::new(|_| {}),
            Some(on_stdout) => on_stdout,
        };
        let on_stderr = match on_stderr {
            None => Box::new(|_| {}),
            Some(on_stderr) => on_stderr,
        };
        let mut on_exit = match on_exit {
            None => Box::new(|_| {}),
            Some(on_exit) => on_exit,
        };

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

        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let output = child.stdout.take().unwrap();
        let error = child.stderr.take().unwrap();

        let on_stdout = Arc::new(Mutex::new(on_stdout));
        let on_stderr = Arc::new(Mutex::new(on_stderr));

        let should_terminate = Arc::new(Mutex::new(false));

        let _thread1 = {
            let should_terminate = should_terminate.clone();
            thread::spawn(move || {
                let mut output = BufReader::new(output);
                let mut buf = String::new();
                while !*should_terminate.lock().unwrap() {
                    if let Ok(_) = output.read_line(&mut buf) {
                        if buf.len() > 0 {
                            on_stdout.lock().unwrap()(buf.clone());
                        }
                        buf.clear();
                    }
                }
            })
        };
        let _thread2 = {
            let should_terminate = should_terminate.clone();
            thread::spawn(move || {
                let mut error = BufReader::new(error);
                let mut buf = String::new();
                while !*should_terminate.lock().unwrap() {
                    if let Ok(_) = error.read_line(&mut buf) {
                        if buf.len() > 0 {
                            on_stderr.lock().unwrap()(buf.clone());
                        }
                        buf.clear();
                    }
                }
            })
        };

        loop {
            on_start();
            if let Ok(Some(v)) = child.try_wait() {
                self.exit_status = Some(v);
                on_exit(v.code().unwrap_or(0));
                *should_terminate.lock().unwrap() = true;
                break;
            }
        }

        Ok(())
    }
}
