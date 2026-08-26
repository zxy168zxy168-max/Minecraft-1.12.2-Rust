use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fmt,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

use crate::net::optifine::shader::IShaderPack::IShaderPack;
use zip::ZipArchive;

pub struct ShaderPackZip {
    pub packFile: PathBuf,
    packZipFile: Option<ZipArchive<File>>,
    /// Empty for an ordinary OptiFine ZIP. GitHub's "Download ZIP" archives
    /// commonly wrap the pack in one repository directory.
    packRoot: String,
    packRootScanned: bool,
    /// ZIP central-directory index. OptiFine keeps one `ZipFile` open and does
    /// not rescan all entries for every `hasDirectory` call; this is the Rust
    /// equivalent for the `world-128..world128` probe.
    entryNames: HashSet<String>,
    directoryNames: HashSet<String>,
    /// Expanded source/include bytes are immutable during one selected-pack
    /// lifetime. Caching prevents the same shared include from being inflated
    /// once per program and once again per option scan.
    resourceCache: HashMap<String, Option<Vec<u8>>>,
    resourceInflates: usize,
    resourceCacheHits: usize,
}

impl fmt::Debug for ShaderPackZip {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ShaderPackZip")
            .field("packFile", &self.packFile)
            .field("packZipFileOpen", &self.packZipFile.is_some())
            .field("packRoot", &self.packRoot)
            .field("packRootScanned", &self.packRootScanned)
            .field("entryCount", &self.entryNames.len())
            .field("cachedResources", &self.resourceCache.len())
            .field("resourceInflates", &self.resourceInflates)
            .field("resourceCacheHits", &self.resourceCacheHits)
            .finish()
    }
}

impl ShaderPackZip {
    pub fn new(_name: impl AsRef<str>, file: impl Into<PathBuf>) -> Self {
        Self {
            packFile: file.into(),
            packZipFile: None,
            packRoot: String::new(),
            packRootScanned: false,
            entryNames: HashSet::new(),
            directoryNames: HashSet::new(),
            resourceCache: HashMap::new(),
            resourceInflates: 0,
            resourceCacheHits: 0,
        }
    }

    pub fn packFile(&self) -> &Path {
        &self.packFile
    }

    fn archive(&mut self) -> io::Result<&mut ZipArchive<File>> {
        if self.packZipFile.is_none() {
            let file = File::open(&self.packFile)?;
            self.packZipFile = Some(ZipArchive::new(file).map_err(zip_error)?);
        }
        Ok(self
            .packZipFile
            .as_mut()
            .expect("archive initialized above"))
    }

    fn ensurePackRoot(&mut self) -> io::Result<()> {
        if self.packRootScanned {
            return Ok(());
        }

        let (hasRootShaders, candidates, names) = {
            let archive = self.archive()?;
            let mut hasRootShaders = false;
            let mut candidates = BTreeSet::new();
            let mut names = HashSet::with_capacity(archive.len());
            for index in 0..archive.len() {
                let Ok(entry) = archive.by_index(index) else {
                    continue;
                };
                let name = normalizeArchiveName(entry.name());
                if name.is_empty() || name.starts_with("__MACOSX/") {
                    continue;
                }
                if name == "shaders" || name == "shaders/" || name.starts_with("shaders/") {
                    hasRootShaders = true;
                }
                if let Some(position) = name.find("/shaders/") {
                    candidates.insert(name[..position + 1].to_owned());
                } else if let Some(prefix) = name.strip_suffix("/shaders") {
                    candidates.insert(format!("{prefix}/"));
                } else if let Some(prefix) = name.strip_suffix("/shaders/") {
                    candidates.insert(format!("{prefix}/"));
                }
                names.insert(name);
            }
            (hasRootShaders, candidates, names)
        };

        self.packRoot = if hasRootShaders {
            String::new()
        } else if candidates.len() == 1 {
            candidates.into_iter().next().unwrap_or_default()
        } else {
            String::new()
        };
        self.entryNames = names;
        self.directoryNames = buildDirectoryIndex(&self.entryNames);
        self.packRootScanned = true;
        if !self.packRoot.is_empty() {
            log::info!(
                "OptiFine shader ZIP {:?} uses wrapped pack root {:?}",
                self.getName(),
                self.packRoot,
            );
        }
        Ok(())
    }

    fn archiveName(&mut self, resource: &str) -> io::Result<String> {
        self.ensurePackRoot()?;
        let relative = resource.strip_prefix('/').unwrap_or(resource);
        Ok(format!("{}{}", self.packRoot, relative).replace('\\', "/"))
    }

    #[cfg(test)]
    fn ioStats(&self) -> (usize, usize) {
        (self.resourceInflates, self.resourceCacheHits)
    }
}

impl IShaderPack for ShaderPackZip {
    fn getName(&self) -> &str {
        self.packFile
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
    }

    fn getResourceAsStream(&mut self, resName: &str) -> io::Result<Option<Vec<u8>>> {
        let archiveName = match self.archiveName(resName) {
            Ok(value) => value,
            Err(_) => return Ok(None),
        };
        if let Some(cached) = self.resourceCache.get(&archiveName) {
            self.resourceCacheHits += 1;
            return Ok(cached.clone());
        }
        if !self.entryNames.contains(&archiveName) {
            self.resourceCache.insert(archiveName, None);
            return Ok(None);
        }

        let bytes = {
            let archive = match self.archive() {
                Ok(value) => value,
                Err(_) => return Ok(None),
            };
            // Keep the `ZipFile` temporary out of the block's tail expression;
            // otherwise its borrow may be considered live across the archive
            // borrow on older stable compilers.
            let result = match archive.by_name(&archiveName) {
                Ok(mut entry) => {
                    let mut bytes =
                        Vec::with_capacity(entry.size().min(usize::MAX as u64) as usize);
                    entry.read_to_end(&mut bytes).ok().map(|_| bytes)
                }
                Err(_) => None,
            };
            result
        };
        let Some(bytes) = bytes else {
            self.resourceCache.insert(archiveName, None);
            return Ok(None);
        };
        self.resourceInflates += 1;
        self.resourceCache.insert(archiveName, Some(bytes.clone()));
        Ok(Some(bytes))
    }

    fn hasDirectory(&mut self, resName: &str) -> bool {
        let Ok(archiveName) = self.archiveName(resName) else {
            return false;
        };
        self.directoryNames
            .contains(archiveName.trim_end_matches('/'))
    }

    fn close(&mut self) {
        self.packZipFile = None;
        // The index and immutable resource cache remain valid for this selected
        // pack object, matching the persistent lifetime of Java `ZipFile`.
    }
}

fn normalizeArchiveName(name: &str) -> String {
    name.replace('\\', "/").trim_start_matches("./").to_owned()
}

fn buildDirectoryIndex(entries: &HashSet<String>) -> HashSet<String> {
    let mut directories = HashSet::new();
    for name in entries {
        let normalized = name.trim_end_matches('/');
        if name.ends_with('/') && !normalized.is_empty() {
            directories.insert(normalized.to_owned());
        }
        for (index, character) in normalized.char_indices() {
            if character == '/' && index > 0 {
                directories.insert(normalized[..index].to_owned());
            }
        }
    }
    directories
}

fn zip_error(error: zip::result::ZipError) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::Write,
        time::{SystemTime, UNIX_EPOCH},
    };
    use zip::{write::SimpleFileOptions, ZipWriter};

    fn temporaryZip(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("mc112-{name}-{unique}.zip"))
    }

    #[test]
    fn lazily_reads_and_closes_optifine_zip_pack() {
        let path = temporaryZip("shader-pack");
        let file = File::create(&path).unwrap();
        let mut writer = ZipWriter::new(file);
        writer
            .add_directory("shaders/", SimpleFileOptions::default())
            .unwrap();
        writer
            .start_file("shaders/gbuffers_basic.fsh", SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"zip-pack").unwrap();
        writer.finish().unwrap();

        let mut pack = ShaderPackZip::new("ignored", &path);
        assert!(pack.packZipFile.is_none());
        assert_eq!(
            pack.getResourceAsStream("/shaders/gbuffers_basic.fsh")
                .unwrap(),
            Some(b"zip-pack".to_vec())
        );
        assert!(pack.packZipFile.is_some());
        assert!(pack.hasDirectory("/shaders/"));
        pack.close();
        assert!(pack.packZipFile.is_none());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn caches_repeated_include_reads_without_reinflating_zip_entries() {
        let path = temporaryZip("shader-cache");
        let file = File::create(&path).unwrap();
        let mut writer = ZipWriter::new(file);
        writer
            .start_file("shaders/lib/common.glsl", SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"shared include").unwrap();
        writer.finish().unwrap();

        let mut pack = ShaderPackZip::new("ignored", &path);
        for _ in 0..8 {
            assert_eq!(
                pack.getResourceAsStream("/shaders/lib/common.glsl")
                    .unwrap(),
                Some(b"shared include".to_vec()),
            );
        }
        assert_eq!(pack.ioStats(), (1, 7));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn indexes_world_directories_once_for_optifine_dimension_probe() {
        let path = temporaryZip("shader-world-index");
        let file = File::create(&path).unwrap();
        let mut writer = ZipWriter::new(file);
        writer
            .start_file(
                "shaders/world-1/composite.fsh",
                SimpleFileOptions::default(),
            )
            .unwrap();
        writer.write_all(b"world minus one").unwrap();
        writer.finish().unwrap();

        let mut pack = ShaderPackZip::new("ignored", &path);
        assert!(pack.hasDirectory("/shaders/world-1"));
        assert!(!pack.hasDirectory("/shaders/world1"));
        assert_eq!(pack.entryNames.len(), 1);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn discovers_a_single_wrapped_github_repository_root() {
        let path = temporaryZip("wrapped-shader-pack");
        let file = File::create(&path).unwrap();
        let mut writer = ZipWriter::new(file);
        writer
            .start_file(
                "Example-main/shaders/gbuffers_terrain.vsh",
                SimpleFileOptions::default(),
            )
            .unwrap();
        writer.write_all(b"wrapped-pack").unwrap();
        writer.finish().unwrap();

        let mut pack = ShaderPackZip::new("ignored", &path);
        assert!(pack.hasDirectory("/shaders"));
        assert_eq!(
            pack.getResourceAsStream("/shaders/gbuffers_terrain.vsh")
                .unwrap(),
            Some(b"wrapped-pack".to_vec())
        );
        assert_eq!(pack.packRoot, "Example-main/");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn ignores_macos_metadata_when_discovering_wrapped_root() {
        let path = temporaryZip("wrapped-macos-shader-pack");
        let file = File::create(&path).unwrap();
        let mut writer = ZipWriter::new(file);
        writer
            .start_file(
                "Example-main/shaders/gbuffers_terrain.vsh",
                SimpleFileOptions::default(),
            )
            .unwrap();
        writer.write_all(b"wrapped-pack").unwrap();
        writer
            .start_file(
                "__MACOSX/Example-main/shaders/._gbuffers_terrain.vsh",
                SimpleFileOptions::default(),
            )
            .unwrap();
        writer.write_all(b"metadata").unwrap();
        writer.finish().unwrap();

        let mut pack = ShaderPackZip::new("ignored", &path);
        assert!(pack.hasDirectory("/shaders"));
        assert_eq!(pack.packRoot, "Example-main/");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn does_not_guess_between_multiple_wrapped_shader_roots() {
        let path = temporaryZip("ambiguous-shader-pack");
        let file = File::create(&path).unwrap();
        let mut writer = ZipWriter::new(file);
        for root in ["A-main", "B-main"] {
            writer
                .start_file(
                    format!("{root}/shaders/gbuffers_basic.vsh"),
                    SimpleFileOptions::default(),
                )
                .unwrap();
            writer.write_all(root.as_bytes()).unwrap();
        }
        writer.finish().unwrap();

        let mut pack = ShaderPackZip::new("ignored", &path);
        assert!(!pack.hasDirectory("/shaders"));
        assert_eq!(
            pack.getResourceAsStream("/shaders/gbuffers_basic.vsh")
                .unwrap(),
            None
        );
        let _ = std::fs::remove_file(path);
    }
}
