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

use std::collections::HashMap;

use serde_json::Value;

pub mod fabric;
pub mod forge;
pub mod quilt;

pub trait Parse {
    fn parse(self) -> ResolvedMod;
}

#[derive(Debug, Clone)]
pub struct ResolvedMod {
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub depends: ResolvedDepends,
    pub authors: Vec<ResolvedAuthorInfo>,
    pub license: Option<Vec<String>>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedDepends {
    pub minecraft: Option<Value>,
    pub java: Option<Value>,
    pub mod_loader: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct ResolvedAuthorInfo {
    pub name: String,
    pub contact: Option<HashMap<String, String>>,
}