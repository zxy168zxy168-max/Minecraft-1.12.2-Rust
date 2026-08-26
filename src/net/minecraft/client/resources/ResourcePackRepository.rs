use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use thiserror::Error;

use crate::net::minecraft::client::resources::FileResourcePack::FileResourcePack;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourcePackKind {
    Folder,
    File,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourcePackEntry {
    pub resourcePackName: String,
    pub resourcePackFile: PathBuf,
    pub kind: ResourcePackKind,
    pub packFormat: i32,
    pub description: String,
    /// Root-level `pack.png`, retained for `ResourcePackListEntryFound`.
    pub iconBytes: Option<Vec<u8>>,
    pub iconLocation: ResourceLocation,
}

pub fn defaultPackIconLocation() -> ResourceLocation {
    ResourceLocation::new("minecraft", "dynamic/default_pack_icon.png")
}

pub fn defaultPackIconBytes() -> &'static [u8] {
    include_bytes!("default_pack.png")
}

impl ResourcePackEntry {
    pub const fn isCompatibleWith1122(&self) -> bool {
        self.packFormat == 3
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResourcePackRepository {
    repositoryEntriesAll: Vec<ResourcePackEntry>,
}

#[derive(Debug, Error)]
pub enum ResourcePackRepositoryError {
    #[error("failed creating resource-pack directory {path}: {source}")]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed listing resource-pack directory {path}: {source}")]
    ListDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

#[derive(Debug, Deserialize)]
struct PackMcmeta {
    pack: PackSection,
}

#[derive(Debug, Deserialize)]
struct PackSection {
    pack_format: i32,
    #[serde(default)]
    description: serde_json::Value,
}

impl ResourcePackRepository {
    /// MCP `fixDirResourcepacks` + `updateRepositoryEntriesAll` subset.
    pub fn scan(directory: impl Into<PathBuf>) -> Result<Self, ResourcePackRepositoryError> {
        let directory = directory.into();
        if directory.exists() && !directory.is_dir() {
            if let Err(source) =
                fs::remove_file(&directory).and_then(|_| fs::create_dir_all(&directory))
            {
                return Err(ResourcePackRepositoryError::CreateDirectory {
                    path: directory,
                    source,
                });
            }
        } else if !directory.exists() {
            fs::create_dir_all(&directory).map_err(|source| {
                ResourcePackRepositoryError::CreateDirectory {
                    path: directory.clone(),
                    source,
                }
            })?;
        }

        let entries = fs::read_dir(&directory).map_err(|source| {
            ResourcePackRepositoryError::ListDirectory {
                path: directory.clone(),
                source,
            }
        })?;
        let candidates = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                let isZip = path.is_file()
                    && path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.ends_with(".zip"));
                let isFolderPack = path.is_dir() && path.join("pack.mcmeta").is_file();
                isZip || isFolderPack
            })
            .collect::<Vec<_>>();

        // ResourcePackRepository#getResourcePackFiles preserves the directory
        // enumeration order returned by File#listFiles; do not introduce a
        // synthetic alphabetical order here.
        let repositoryEntriesAll = candidates
            .into_iter()
            .filter_map(|path| read_entry(path).ok())
            .collect();
        Ok(Self {
            repositoryEntriesAll,
        })
    }

    pub fn getRepositoryEntriesAll(&self) -> &[ResourcePackEntry] {
        &self.repositoryEntriesAll
    }

    pub fn findByName(&self, resourcePackName: &str) -> Option<&ResourcePackEntry> {
        self.repositoryEntriesAll
            .iter()
            .find(|entry| entry.resourcePackName == resourcePackName)
    }

    #[cfg(test)]
    pub fn fromEntriesForTest(entries: Vec<ResourcePackEntry>) -> Self {
        Self {
            repositoryEntriesAll: entries,
        }
    }
}

fn read_entry(path: PathBuf) -> io::Result<ResourcePackEntry> {
    let (kind, metadataBytes, iconBytes) = if path.is_dir() {
        (
            ResourcePackKind::Folder,
            fs::read(path.join("pack.mcmeta"))?,
            fs::read(path.join("pack.png")).ok(),
        )
    } else {
        let pack = FileResourcePack::new(&path)?;
        (
            ResourcePackKind::File,
            pack.read_name("pack.mcmeta")?,
            pack.read_name("pack.png").ok(),
        )
    };
    let metadata: PackMcmeta = serde_json::from_slice(&metadataBytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let resourcePackName = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "resource-pack name is not valid UTF-8",
            )
        })?
        .to_owned();
    let iconLocation = resource_pack_icon_location(&path);
    Ok(ResourcePackEntry {
        resourcePackName,
        resourcePackFile: path,
        kind,
        packFormat: metadata.pack.pack_format,
        description: description_text(&metadata.pack.description),
        iconBytes,
        iconLocation,
    })
}

fn resource_pack_icon_location(path: &Path) -> ResourceLocation {
    // Stable internal key corresponding to Entry#locationTexturePackIcon.
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in path.to_string_lossy().as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    ResourceLocation::new("minecraft", format!("resourcepackicons/{hash:016x}.png"))
}

fn description_text(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Object(object) => object
            .get("text")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        _ => String::new(),
    }
}

pub fn folder_assets_root(pack_root: &Path) -> PathBuf {
    pack_root.join("assets")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("mc112-resource-repository-{unique}"))
    }

    #[test]
    fn scans_pack_metadata_and_retains_pack_icon() {
        let root = temp_dir();
        let valid = root.join("Vanilla Test");
        fs::create_dir_all(valid.join("assets/minecraft/textures")).unwrap();
        fs::write(
            valid.join("pack.mcmeta"),
            br#"{"pack":{"pack_format":3,"description":"1.12.2 pack"}}"#,
        )
        .unwrap();
        fs::write(valid.join("pack.png"), b"retained-verbatim").unwrap();
        fs::create_dir_all(root.join("Not A Pack")).unwrap();

        let repository = ResourcePackRepository::scan(&root).unwrap();
        assert_eq!(repository.getRepositoryEntriesAll().len(), 1);
        let entry = &repository.getRepositoryEntriesAll()[0];
        assert_eq!(entry.resourcePackName, "Vanilla Test");
        assert_eq!(entry.packFormat, 3);
        assert!(entry.isCompatibleWith1122());
        assert_eq!(
            entry.iconBytes.as_deref(),
            Some(b"retained-verbatim".as_slice())
        );
        assert_eq!(entry.iconLocation, resource_pack_icon_location(&valid));
        let _ = fs::remove_dir_all(root);
    }
}
