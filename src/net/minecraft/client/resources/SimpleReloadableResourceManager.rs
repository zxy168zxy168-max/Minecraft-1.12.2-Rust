use std::{
    collections::HashSet,
    fs,
    io,
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::net::minecraft::client::resources::FileResourcePack::FileResourcePack;
use crate::net::minecraft::client::resources::FolderResourcePack::DirectoryResourcePack;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

#[derive(Debug, Clone)]
pub struct ResourceBytes {
    pub pack_name: String,
    pub location: ResourceLocation,
    pub bytes: Vec<u8>,
    pub metadata: Option<Vec<u8>>,
}

#[derive(Debug, Error)]
pub enum ResourceManagerError {
    #[error("invalid relative resource path: {0}")]
    InvalidPath(ResourceLocation),
    #[error("resource not found: {0}")]
    NotFound(ResourceLocation),
    #[error("failed opening resource pack {path}: {source}")]
    OpenPack {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed reading resource {location} from {path}: {source}")]
    Read {
        location: ResourceLocation,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

/// Ordered resource-pack manager matching `SimpleReloadableResourceManager`
/// plus `FallbackResourceManager` precedence. Packs added later override
/// earlier packs for `get_resource`; `get_all_resources` retains insertion
/// order exactly as Minecraft 1.12.2 does during reload.
#[derive(Debug, Clone, Default)]
pub struct ResourceManager {
    packs: Vec<NamedResourcePack>,
}

#[derive(Debug, Clone)]
struct NamedResourcePack {
    name: String,
    pack: ResourcePackSource,
}

#[derive(Debug, Clone)]
enum ResourcePackSource {
    Directory(DirectoryResourcePack),
    Zip(FileResourcePack),
}

impl ResourcePackSource {
    fn contains(&self, location: &ResourceLocation) -> bool {
        match self {
            Self::Directory(pack) => pack.contains(location),
            Self::Zip(pack) => pack.contains(location),
        }
    }

    fn read(&self, location: &ResourceLocation) -> io::Result<Vec<u8>> {
        match self {
            Self::Directory(pack) => fs::read(pack.resolve(location)),
            Self::Zip(pack) => pack.read(location),
        }
    }

    fn source_path(&self, location: &ResourceLocation) -> PathBuf {
        match self {
            Self::Directory(pack) => pack.resolve(location),
            Self::Zip(pack) => pack.archive_path().to_path_buf(),
        }
    }

    fn resource_domains(&self) -> HashSet<String> {
        match self {
            Self::Directory(pack) => pack.getResourceDomains(),
            Self::Zip(pack) => pack.getResourceDomains(),
        }
    }
}

impl ResourceManager {
    pub fn new() -> Self { Self::default() }

    /// Adds a direct namespace root. The runtime asset root already has the
    /// shape `<root>/<namespace>/<path>`, while a folder resource pack passes
    /// its `<pack>/assets` directory here.
    pub fn add_directory_pack(&mut self, name: impl Into<String>, assets_root: impl Into<PathBuf>) {
        self.packs.push(NamedResourcePack {
            name: name.into(),
            pack: ResourcePackSource::Directory(DirectoryResourcePack::new(assets_root)),
        });
    }

    /// Adds a standard Java Edition `.zip` resource pack whose entries use
    /// `assets/<namespace>/<path>`.
    pub fn add_zip_pack(
        &mut self,
        name: impl Into<String>,
        archive_path: impl Into<PathBuf>,
    ) -> Result<(), ResourceManagerError> {
        let archive_path = archive_path.into();
        let pack = FileResourcePack::new(&archive_path).map_err(|source| {
            ResourceManagerError::OpenPack { path: archive_path.clone(), source }
        })?;
        self.packs.push(NamedResourcePack {
            name: name.into(),
            pack: ResourcePackSource::Zip(pack),
        });
        Ok(())
    }

    /// Reads the `{name}.mcmeta` metadata file from the root of every pack,
    /// in pack order (MCP `IResourcePack#getPackMetadata`, e.g. name "pack"
    /// reads `pack.mcmeta`). Packs without the file are skipped.
    pub fn read_pack_metadatas(&self, name: &str) -> Vec<Vec<u8>> {
        let mut sections = Vec::new();
        for pack in &self.packs {
            match &pack.pack {
                ResourcePackSource::Directory(directory) => {
                    // FolderResourcePack stores `<pack>/assets` as its namespace
                    // root, while IResourcePack#getPackMetadata reads the root
                    // `<pack>/{name}.mcmeta`. Runtime default assets may not have
                    // a parent metadata file; LanguageManager supplies the
                    // canonical en_us DefaultResourcePack fallback in that case.
                    let file = format!("{name}.mcmeta");
                    let path = directory.assets_root().parent()
                        .map(|root| root.join(&file))
                        .filter(|candidate| candidate.is_file())
                        .unwrap_or_else(|| directory.assets_root().join(&file));
                    if path.is_file() {
                        if let Ok(bytes) = fs::read(path) { sections.push(bytes); }
                    }
                }
                ResourcePackSource::Zip(zip) => {
                    if let Ok(bytes) = zip.read_name(&format!("{name}.mcmeta")) {
                        sections.push(bytes);
                    }
                }
            }
        }
        sections
    }

    pub fn pack_count(&self) -> usize { self.packs.len() }

    pub fn pack_names(&self) -> Vec<&str> {
        self.packs.iter().map(|pack| pack.name.as_str()).collect()
    }

    pub fn get_resource(&self, location: &ResourceLocation) -> Result<ResourceBytes, ResourceManagerError> {
        validate_path(location)?;
        let metadata_location = metadata_location(location);
        let mut metadata_source: Option<&NamedResourcePack> = None;

        for candidate in self.packs.iter().rev() {
            if metadata_source.is_none() && candidate.pack.contains(&metadata_location) {
                metadata_source = Some(candidate);
            }
            if candidate.pack.contains(location) {
                let bytes = read_resource(candidate, location)?;
                let metadata = metadata_source
                    .map(|pack| read_resource(pack, &metadata_location))
                    .transpose()?;
                return Ok(ResourceBytes {
                    pack_name: candidate.name.clone(),
                    location: location.clone(),
                    bytes,
                    metadata,
                });
            }
        }
        Err(ResourceManagerError::NotFound(location.clone()))
    }

    pub fn get_all_resources(&self, location: &ResourceLocation) -> Result<Vec<ResourceBytes>, ResourceManagerError> {
        validate_path(location)?;
        let metadata_location = metadata_location(location);
        let mut resources = Vec::new();
        for candidate in &self.packs {
            if !candidate.pack.contains(location) { continue; }
            resources.push(ResourceBytes {
                pack_name: candidate.name.clone(),
                location: location.clone(),
                bytes: read_resource(candidate, location)?,
                metadata: if candidate.pack.contains(&metadata_location) {
                    Some(read_resource(candidate, &metadata_location)?)
                } else {
                    None
                },
            });
        }
        if resources.is_empty() {
            Err(ResourceManagerError::NotFound(location.clone()))
        } else {
            Ok(resources)
        }
    }

    pub fn resource_exists(&self, location: &ResourceLocation) -> bool {
        validate_path(location).is_ok()
            && self.packs.iter().rev().any(|pack| pack.pack.contains(location))
    }


    /// MCP `IResourceManager#getResourceDomains`: union of all namespaces
    /// visible through the ordered fallback managers. Java resource packs may
    /// add their own domains; limiting reloads to `minecraft` would silently
    /// drop custom sounds, models and language resources.
    pub fn get_resource_domains(&self) -> HashSet<String> {
        self.packs.iter()
            .flat_map(|pack| pack.pack.resource_domains())
            .collect()
    }
}

fn metadata_location(location: &ResourceLocation) -> ResourceLocation {
    ResourceLocation::new(location.getNamespace(), format!("{}.mcmeta", location.getPath()))
}

fn validate_path(location: &ResourceLocation) -> Result<(), ResourceManagerError> {
    if location.getPath().contains("..") {
        Err(ResourceManagerError::InvalidPath(location.clone()))
    } else {
        Ok(())
    }
}

fn read_resource(
    pack: &NamedResourcePack,
    location: &ResourceLocation,
) -> Result<Vec<u8>, ResourceManagerError> {
    pack.pack.read(location).map_err(|source| ResourceManagerError::Read {
        location: location.clone(),
        path: pack.pack.source_path(location),
        source,
    })
}

pub fn directory_pack_assets_root(root: impl AsRef<Path>) -> PathBuf {
    root.as_ref().to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    fn temp_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("mc112-{label}-{unique}"));
        fs::create_dir_all(path.join("minecraft/test")).unwrap();
        path
    }

    #[test]
    fn later_pack_overrides_earlier_pack() {
        let base = temp_dir("base");
        let overlay = temp_dir("overlay");
        fs::write(base.join("minecraft/test/value.txt"), b"base").unwrap();
        fs::write(overlay.join("minecraft/test/value.txt"), b"overlay").unwrap();
        let mut manager = ResourceManager::new();
        manager.add_directory_pack("base", &base);
        manager.add_directory_pack("overlay", &overlay);
        let resource = manager.get_resource(&ResourceLocation::new("minecraft", "test/value.txt")).unwrap();
        assert_eq!(resource.bytes, b"overlay");
        let all = manager.get_all_resources(&ResourceLocation::new("minecraft", "test/value.txt")).unwrap();
        assert_eq!(
            all.iter().map(|resource| resource.bytes.as_slice()).collect::<Vec<_>>(),
            vec![b"base".as_slice(), b"overlay".as_slice()]
        );
        let _ = fs::remove_dir_all(base);
        let _ = fs::remove_dir_all(overlay);
    }

    #[test]
    fn zip_pack_uses_standard_assets_prefix_and_overrides_runtime() {
        let base = temp_dir("zip-base");
        fs::write(base.join("minecraft/test/value.txt"), b"base").unwrap();
        let zip_path = base.parent().unwrap().join(format!(
            "mc112-overlay-{}.zip",
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
        ));
        let file = fs::File::create(&zip_path).unwrap();
        let mut writer = ZipWriter::new(file);
        writer.start_file("assets/minecraft/test/value.txt", SimpleFileOptions::default()).unwrap();
        writer.write_all(b"zip-overlay").unwrap();
        writer.finish().unwrap();

        let mut manager = ResourceManager::new();
        manager.add_directory_pack("runtime", &base);
        manager.add_zip_pack("overlay.zip", &zip_path).unwrap();
        let resource = manager.get_resource(&ResourceLocation::new("minecraft", "test/value.txt")).unwrap();
        assert_eq!(resource.bytes, b"zip-overlay");
        assert_eq!(resource.pack_name, "overlay.zip");
        let _ = fs::remove_dir_all(base);
        let _ = fs::remove_file(zip_path);
    }

    #[test]
    fn resource_domains_union_java_pack_namespaces_and_ignore_uppercase() {
        let base = temp_dir("domains-base");
        fs::create_dir_all(base.join("custom_domain/sounds")).unwrap();
        fs::create_dir_all(base.join("UpperCase/sounds")).unwrap();
        fs::write(base.join("custom_domain/sounds.json"), b"{}").unwrap();

        let zip_path = base.parent().unwrap().join(format!(
            "mc112-domains-{}.zip",
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(),
        ));
        let file = fs::File::create(&zip_path).unwrap();
        let mut writer = ZipWriter::new(file);
        writer.start_file("assets/another-domain/sounds.json", SimpleFileOptions::default()).unwrap();
        writer.write_all(b"{}").unwrap();
        writer.start_file("assets/BadDomain/sounds.json", SimpleFileOptions::default()).unwrap();
        writer.write_all(b"{}").unwrap();
        writer.finish().unwrap();

        let mut manager = ResourceManager::new();
        manager.add_directory_pack("folder", &base);
        manager.add_zip_pack("domains.zip", &zip_path).unwrap();
        let domains = manager.get_resource_domains();
        assert!(domains.contains("minecraft"));
        assert!(domains.contains("custom_domain"));
        assert!(domains.contains("another-domain"));
        assert!(!domains.contains("UpperCase"));
        assert!(!domains.contains("BadDomain"));
        let _ = fs::remove_dir_all(base);
        let _ = fs::remove_file(zip_path);
    }

    #[test]
    fn rejects_parent_traversal() {
        let manager = ResourceManager::new();
        let result = manager.get_resource(&ResourceLocation::new("minecraft", "../secret"));
        assert!(matches!(result, Err(ResourceManagerError::InvalidPath(_))));
    }
}
