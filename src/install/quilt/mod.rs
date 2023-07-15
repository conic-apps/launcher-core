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

use serde::{Deserialize, Serialize};

pub mod install;
pub mod version_list;

const DEFAULT_META_URL: &str = "https://meta.quiltmc.org";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QuiltArtifactVersion {
    separator: String,
    build: u32,

    /// e.g. "org.quiltmc.quilt-loader:0.16.1"
    maven: String,
    version: String,
}
