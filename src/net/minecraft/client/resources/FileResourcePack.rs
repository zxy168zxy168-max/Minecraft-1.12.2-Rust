use std::{
    collections::HashSet,
    fmt,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use zip::ZipArchive;

/// File-backed resource pack equivalent to MCP 1.12.2 `FileResourcePack`.
///
/// MCP keeps one `ZipFile` open for the lifetime of the pack. The Rust
/// equivalent shares one mutex-protected `ZipArchive<File>` across clones, so
/// a reload does not reopen the ZIP and reparse its central directory for
/// every PNG, JSON or OGG lookup.
#[derive(Clone)]
pub struct FileResourcePack {
    archive_path: PathBuf,
    entries: HashSet<String>,
    archive: Arc<Mutex<ZipArchive<File>>>,
}

impl fmt::Debug for FileResourcePack {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FileResourcePack")
            .field("archive_path", &self.archive_path)
            .field("entries", &self.entries.len())
            .finish_non_exhaustive()
    }
}

impl FileResourcePack {
    pub fn new(archive_path: impl Into<PathBuf>) -> io::Result<Self> {
        let archive_path = archive_path.into();
        let file = File::open(&archive_path)?;
        let mut archive = ZipArchive::new(file).map_err(zip_error)?;
        let mut entries = HashSet::with_capacity(archive.len());
        for index in 0..archive.len() {
            let entry = archive.by_index(index).map_err(zip_error)?;
            if !entry.is_dir() {
                entries.insert(entry.name().to_owned());
            }
        }
        Ok(Self {
            archive_path,
            entries,
            archive: Arc::new(Mutex::new(archive)),
        })
    }

    pub fn contains(&self, location: &ResourceLocation) -> bool {
        self.entries.contains(&resource_name(location))
    }

    pub fn read(&self, location: &ResourceLocation) -> io::Result<Vec<u8>> {
        self.read_name(&resource_name(location))
    }

    pub fn contains_name(&self, name: &str) -> bool {
        self.entries.contains(name)
    }

    pub fn read_name(&self, name: &str) -> io::Result<Vec<u8>> {
        let mut archive = self.archive.lock().map_err(|_| {
            io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "resource-pack ZIP lock poisoned: {}",
                    self.archive_path.display()
                ),
            )
        })?;
        let mut entry = archive.by_name(name).map_err(zip_error)?;
        let mut bytes = Vec::with_capacity(entry.size().min(usize::MAX as u64) as usize);
        entry.read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    pub fn archive_path(&self) -> &Path {
        &self.archive_path
    }

    pub fn getResourceDomains(&self) -> HashSet<String> {
        let mut domains = HashSet::new();
        for entry in &self.entries {
            let Some(remainder) = entry.strip_prefix("assets/") else {
                continue;
            };
            let Some((namespace, _)) = remainder.split_once('/') else {
                continue;
            };
            if namespace.is_empty() {
                continue;
            }
            if namespace == namespace.to_ascii_lowercase() {
                domains.insert(namespace.to_owned());
            } else {
                log::warn!(
                    "ResourcePack: ignored non-lowercase namespace: {} in {}",
                    namespace,
                    self.archive_path.display(),
                );
            }
        }
        domains
    }
}

fn resource_name(location: &ResourceLocation) -> String {
    format!("assets/{}/{}", location.getNamespace(), location.getPath())
}

fn zip_error(error: zip::result::ZipError) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    fn temp_zip() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("mc112-resource-pack-{unique}.zip"))
    }

    #[test]
    fn reads_standard_java_resource_pack_layout_without_reopening_zip() {
        let path = temp_zip();
        let file = File::create(&path).unwrap();
        let mut writer = ZipWriter::new(file);
        writer
            .start_file("pack.mcmeta", SimpleFileOptions::default())
            .unwrap();
        writer
            .write_all(br#"{"pack":{"pack_format":3,"description":"test"}}"#)
            .unwrap();
        writer
            .start_file(
                "assets/minecraft/test/value.txt",
                SimpleFileOptions::default(),
            )
            .unwrap();
        writer.write_all(b"zip-pack").unwrap();
        writer.finish().unwrap();

        let pack = FileResourcePack::new(&path).unwrap();
        let location = ResourceLocation::new("minecraft", "test/value.txt");
        assert!(pack.contains(&location));
        assert_eq!(pack.read(&location).unwrap(), b"zip-pack");
        assert!(pack.contains_name("pack.mcmeta"));
        let cloned = pack.clone();
        assert!(Arc::ptr_eq(&pack.archive, &cloned.archive));
        assert_eq!(cloned.read(&location).unwrap(), b"zip-pack");
        let _ = std::fs::remove_file(path);
    }
}
