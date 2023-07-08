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

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use zip::ZipArchive;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JarsEntry {
    file: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FabricModMixinObject {
    pub config: String,
    pub environment: String,
}

/// Corresponds to the <mod_pack>/`fabric.mod.json` file in the module archive
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FabricModMetadata {
    /* Required */
    pub schema_version: u8,
    pub id: String,
    pub version: String,

    /* Mod loading */
    pub provides: Option<Vec<String>>,
    pub environment: Option<String>,
    pub entrypoints: Option<HashMap<String, Vec<String>>>,
    pub jars: Option<Vec<JarsEntry>>,
    pub language_adapters: Option<HashMap<String, String>>,
    pub mixins: Value,

    /* Dependency resolution */
    pub depends: Option<HashMap<String, String>>,
    pub recommends: Option<HashMap<String, String>>,
    pub suggests: Option<HashMap<String, String>>,
    pub breaks: Option<HashMap<String, String>>,
    pub conflicts: Option<HashMap<String, String>>,

    /* Metadata */
    pub name: Option<String>,
    pub description: Option<String>,
    pub contact: Option<HashMap<String, Value>>,
    pub authors: Option<Vec<String>>,
    pub contributors: Option<Vec<Value>>,
    pub license: Option<Value>,
    pub icon: Option<String>,

    /* Custom fields */
    pub custom: Option<HashMap<String, Value>>,
}

impl FabricModMetadata {
    pub fn from_file<P: AsRef<Path>>(file: P) -> Result<Self> {
        let mod_file = File::open(file).unwrap();
        let mut mod_file_archive = ZipArchive::new(mod_file).unwrap();
        let mod_json = mod_file_archive.by_name("fabric.mod.json")?;
        Ok(serde_json::from_reader(mod_json)?)
    }

    pub fn parse(&self) -> ResolvedFabricModMetadata {
        let name = match self.name.to_owned() {
            Some(v) => v,
            None => self.id.to_owned(),
        };
        let description = match self.description.to_owned() {
            Some(v) => v,
            None => "".to_string(),
        };
        let mut minecraft_depend = None;
        let mut fabric_loader_depend = None;
        let mut java_depend = None;
        if let Some(depends) = self.depends.to_owned() {
            for depend in depends {
                match depend.0.as_str() {
                    "minecraft" => minecraft_depend = Some(depend.1),
                    "fabricloader" => fabric_loader_depend = Some(depend.1),
                    "java" => java_depend = Some(depend.1),
                    _ => (),
                };
            }
        }
        let license = if let Some(license) = self.license.to_owned() {
            if license.is_string() {
                Some(vec![license
                    .as_str()
                    .unwrap()
                    .to_string()])
            } else if license.is_array() {
                Some(
                    license
                        .as_array()
                        .unwrap()
                        .into_iter()
                        .map(|v| v.as_str().unwrap().to_string())
                        .collect::<Vec<String>>(),
                )
            } else{
                None
            }
        } else {
            None
        };
        ResolvedFabricModMetadata {
            name,
            description,
            depends: ResolvedFabricDepends {
                minecraft: minecraft_depend,
                fabric_loader: fabric_loader_depend,
                java: java_depend,
            },
            authors: self.authors.to_owned(),
            license,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedFabricDepends {
    pub minecraft: Option<String>,
    pub fabric_loader: Option<String>,
    pub java: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedFabricModMetadata {
    pub name: String,
    pub description: String,
    pub depends: ResolvedFabricDepends,
    pub authors: Option<Vec<String>>,
    pub license: Option<Vec<String>>,
}

#[test]
fn test() {
    let file = "mock/fabric-mod.jar";
    let a = FabricModMetadata::from_file(file).unwrap();
    println!("{:#?}", a);
    let b = a.parse();
    println!("{:#?}", b);
    assert_eq!(b.name, "Carpet Mod".to_string());
}
