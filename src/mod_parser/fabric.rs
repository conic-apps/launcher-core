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
use std::ffi::OsStr;
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
    pub mixins: Option<Value>,

    /* Dependency resolution */
    pub depends: Option<HashMap<String, Value>>,
    pub recommends: Option<HashMap<String, String>>,
    pub suggests: Option<HashMap<String, String>>,
    pub breaks: Option<HashMap<String, String>>,
    pub conflicts: Option<HashMap<String, String>>,

    /* Metadata */
    pub name: Option<String>,
    pub description: Option<String>,
    pub contact: Option<HashMap<String, Value>>,
    pub authors: Option<Vec<Value>>,
    pub contributors: Option<Vec<Value>>,
    pub license: Option<Value>,
    pub icon: Option<String>,

    /* Custom fields */
    pub custom: Option<HashMap<String, Value>>,
}

impl FabricModMetadata {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mod_file = File::open(path).unwrap();
        let mut mod_file_archive = ZipArchive::new(mod_file).unwrap();
        Self::from_zip_archive(&mut mod_file_archive)
    }
    pub fn from_zip_archive(archive: &mut ZipArchive<File>) -> Result<Self> {
        let mod_json = archive.by_name("fabric.mod.json").unwrap();
        Ok(serde_json::from_reader(mod_json).unwrap())
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
                Some(vec![license.as_str().unwrap().to_string()])
            } else if license.is_array() {
                Some(
                    license
                        .as_array()
                        .unwrap()
                        .into_iter()
                        .map(|v| v.as_str().unwrap().to_string())
                        .collect::<Vec<String>>(),
                )
            } else {
                None
            }
        } else {
            None
        };
        // let authors = self.authors.iter().map(|author_info| {
        //     if author_info {

        //     }
        // });
        let mut parsed_authors = None;
        if let Some(authors) = self.authors.to_owned() {
            parsed_authors = Some(
                authors
                    .iter()
                    .map(|author_info| {
                        let author_info = author_info.to_owned();
                        match author_info {
                            Value::String(v) => ResolvedAuthorInfo {
                                name: v,
                                contact: None,
                            },
                            Value::Object(v) => ResolvedAuthorInfo {
                                name: match v["name"].as_str() {
                                    Some(v) => v.to_string(),
                                    None => "".to_string(),
                                },
                                contact: serde_json::from_value(v["contact"].clone()).unwrap(),
                            },
                            _ => ResolvedAuthorInfo {
                                name: "".to_string(),
                                contact: None,
                            },
                        }
                    })
                    .collect::<Vec<ResolvedAuthorInfo>>(),
            );
        }
        ResolvedFabricModMetadata {
            name,
            description,
            depends: ResolvedFabricDepends {
                minecraft: minecraft_depend,
                fabric_loader: fabric_loader_depend,
                java: java_depend,
            },
            authors: parsed_authors,
            license,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedFabricDepends {
    pub minecraft: Option<Value>,
    pub fabric_loader: Option<Value>,
    pub java: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct ResolvedAuthorInfo {
    pub name: String,
    pub contact: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone)]
pub struct ResolvedFabricModMetadata {
    pub name: String,
    pub description: String,
    pub depends: ResolvedFabricDepends,
    pub authors: Option<Vec<ResolvedAuthorInfo>>,
    pub license: Option<Vec<String>>,
}

pub fn parse_folder<S: AsRef<OsStr> + ?Sized>(
    folder: &S,
) -> Result<Vec<ResolvedFabricModMetadata>> {
    let folder = Path::new(folder).to_path_buf();
    let entries = folder.read_dir().unwrap();
    let mut result = Vec::new();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        println!("{:?}", path);
        result.push(FabricModMetadata::from_path(path).unwrap().parse());
    }
    Ok(result)
}

#[test]
fn test() {
    let file = "mock/fabric-mod.jar";
    let a = FabricModMetadata::from_path(file).unwrap();
    println!("{:#?}", a);
    let b = a.parse();
    println!("{:#?}", b);
    assert_eq!(b.name, "Carpet Mod".to_string());
}

#[test]
fn test2() {
    let folder = "mock/fabricmod";
    let a = parse_folder(folder).unwrap();
    println!("{:#?}", a.len());
}
