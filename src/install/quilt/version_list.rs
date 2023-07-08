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

use super::{DEFAULT_META_URL, QuiltArtifactVersion};

pub async fn get_quilt_version_list(remote: Option<String>) -> Vec<QuiltArtifactVersion> {
    let remote = match remote {
        None => DEFAULT_META_URL.to_string(),
        Some(remote) => remote,
    };
    let url = format!("{remote}/v3/versions/loader");
    let response = reqwest::get(url).await.unwrap();
    response.json().await.unwrap()
}

#[tokio::test]
async fn test() {
    let version_list = get_quilt_version_list(None).await;
    println!("{:#?}", version_list);
}

#[tokio::test]
async fn test1() {
    let version_list = get_quilt_version_list(Some("https://meta.quiltmc.org".to_string())).await;
    println!("{:#?}", version_list);
}
