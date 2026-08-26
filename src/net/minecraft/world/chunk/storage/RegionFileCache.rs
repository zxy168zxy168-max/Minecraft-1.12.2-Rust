use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use crate::net::minecraft::world::chunk::storage::RegionFile::RegionFile;

pub type SharedRegionFile = Arc<Mutex<RegionFile>>;

fn cache() -> &'static Mutex<HashMap<PathBuf, SharedRegionFile>> {
    static REGIONS_BY_FILE: OnceLock<Mutex<HashMap<PathBuf, SharedRegionFile>>> = OnceLock::new();
    REGIONS_BY_FILE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn region_path(worldDir: &Path, chunkX: i32, chunkZ: i32) -> PathBuf {
    worldDir
        .join("region")
        .join(format!("r.{}.{}.mca", chunkX >> 5, chunkZ >> 5))
}

/// MCP 1.12.2 `RegionFileCache` with the same 256-file eviction ceiling.
pub fn createOrLoadRegionFile(
    worldDir: impl AsRef<Path>,
    chunkX: i32,
    chunkZ: i32,
) -> io::Result<SharedRegionFile> {
    let worldDir = worldDir.as_ref();
    let path = region_path(worldDir, chunkX, chunkZ);
    let mut regions = cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(region) = regions.get(&path) {
        return Ok(Arc::clone(region));
    }
    std::fs::create_dir_all(worldDir.join("region"))?;
    if regions.len() >= 256 {
        for region in regions.values() {
            if let Ok(mut region) = region.lock() {
                let _ = region.close();
            }
        }
        regions.clear();
    }
    let region = Arc::new(Mutex::new(RegionFile::new(&path)?));
    regions.insert(path, Arc::clone(&region));
    Ok(region)
}

/// MCP `func_191065_b`: return no region when neither region directory nor
/// target file exists; do not create a new file for existence checks.
pub fn getExistingRegionFile(
    worldDir: impl AsRef<Path>,
    chunkX: i32,
    chunkZ: i32,
) -> io::Result<Option<SharedRegionFile>> {
    let worldDir = worldDir.as_ref();
    let path = region_path(worldDir, chunkX, chunkZ);
    let mut regions = cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(region) = regions.get(&path) {
        return Ok(Some(Arc::clone(region)));
    }
    if !worldDir.join("region").exists() || !path.exists() {
        return Ok(None);
    }
    if regions.len() >= 256 {
        for region in regions.values() {
            if let Ok(mut region) = region.lock() {
                let _ = region.close();
            }
        }
        regions.clear();
    }
    let region = Arc::new(Mutex::new(RegionFile::new(&path)?));
    regions.insert(path, Arc::clone(&region));
    Ok(Some(region))
}

pub fn clearRegionFileReferences() {
    let mut regions = cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    for region in regions.values() {
        if let Ok(mut region) = region.lock() {
            let _ = region.close();
        }
    }
    regions.clear();
}

pub fn readChunkData(
    worldDir: impl AsRef<Path>,
    chunkX: i32,
    chunkZ: i32,
) -> io::Result<Option<Vec<u8>>> {
    let region = createOrLoadRegionFile(worldDir, chunkX, chunkZ)?;
    let result = region
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .readChunkData(chunkX & 31, chunkZ & 31)?;
    Ok(result)
}

pub fn writeChunkData(
    worldDir: impl AsRef<Path>,
    chunkX: i32,
    chunkZ: i32,
    data: &[u8],
) -> io::Result<()> {
    let region = createOrLoadRegionFile(worldDir, chunkX, chunkZ)?;
    region
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .writeChunkData(chunkX & 31, chunkZ & 31, data)?;
    Ok(())
}

pub fn isChunkSaved(worldDir: impl AsRef<Path>, chunkX: i32, chunkZ: i32) -> io::Result<bool> {
    let Some(region) = getExistingRegionFile(worldDir, chunkX, chunkZ)? else {
        return Ok(false);
    };
    let saved = region
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .isChunkSaved(chunkX & 31, chunkZ & 31);
    Ok(saved)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_maps_absolute_chunk_coordinates_to_region_local_slots() {
        clearRegionFileReferences();
        let root = std::env::temp_dir().join(format!("mc1122-region-cache-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        writeChunkData(&root, 33, -1, b"region cache payload").unwrap();
        assert!(root.join("region/r.1.-1.mca").is_file());
        assert!(isChunkSaved(&root, 33, -1).unwrap());
        assert_eq!(
            readChunkData(&root, 33, -1).unwrap().unwrap(),
            b"region cache payload"
        );
        clearRegionFileReferences();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn existence_probe_does_not_create_missing_region() {
        clearRegionFileReferences();
        let root = std::env::temp_dir().join(format!("mc1122-region-probe-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        assert!(!isChunkSaved(&root, 0, 0).unwrap());
        assert!(!root.join("region").exists());
        let _ = std::fs::remove_dir_all(root);
    }
}
