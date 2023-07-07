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

use super::*;

impl FabricArtifacts {
    /// get fabric artifacts
    pub async fn new() -> Self {
        reqwest::get("https://meta.fabricmc.net/v2/versions")
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }
}

impl YarnArtifactList {
    /// get yarn artifacts
    pub async fn new() -> Self {
        reqwest::get("https://meta.fabricmc.net/v2/versions/yarn")
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }
    /// get the yarn of the specified minecraft version
    pub async fn from_mcversion(mcversion: &str) -> Self {
        reqwest::get(format!(
            "https://meta.fabricmc.net/v2/versions/yarn/{}",
            mcversion
        ))
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
    }
}

impl LoaderArtifactList {
    /// get loader artifacts
    pub async fn new() -> Self {
        reqwest::get("https://meta.fabricmc.net/v2/versions/loader")
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }
    /// get the loader of the specified minecraft version
    pub async fn from_mcversion(mcversion: &str) -> Self {
        reqwest::get(format!(
            "https://meta.fabricmc.net/v2/versions/loader/{}",
            mcversion
        ))
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
    }
}

impl FabricLoaderArtifact {
    /// get fabric loader artifact
    pub async fn new(mcversion: &str, loader: &str) -> Self {
        reqwest::get(format!(
            "https://meta.fabricmc.net/v2/versions/loader/{}/{}",
            mcversion, loader
        ))
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
    }
}
