use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct DirectoryResourcePack {
    assets_root: PathBuf,
}

impl DirectoryResourcePack {
    pub fn new(assets_root: impl Into<PathBuf>) -> Self {
        Self {
            assets_root: assets_root.into(),
        }
    }

    pub fn resolve(&self, location: &ResourceLocation) -> PathBuf {
        self.assets_root
            .join(location.getNamespace())
            .join(location.getPath())
    }

    pub fn contains(&self, location: &ResourceLocation) -> bool {
        self.resolve(location).is_file()
    }

    pub fn assets_root(&self) -> &Path {
        &self.assets_root
    }

    pub fn getResourceDomains(&self) -> HashSet<String> {
        let Ok(entries) = fs::read_dir(&self.assets_root) else {
            return HashSet::new();
        };
        let mut domains = HashSet::new();
        for entry in entries
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
        {
            let Ok(namespace) = entry.file_name().into_string() else {
                continue;
            };
            if namespace == namespace.to_ascii_lowercase() {
                domains.insert(namespace);
            } else {
                log::warn!(
                    "ResourcePack: ignored non-lowercase namespace: {} in {}",
                    namespace,
                    self.assets_root.display(),
                );
            }
        }
        domains
    }
}
