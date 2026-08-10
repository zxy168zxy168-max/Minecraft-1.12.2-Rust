use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use flate2::{read::{GzDecoder, ZlibDecoder}, write::ZlibEncoder, Compression};

const SECTOR_BYTES: usize = 4096;
const SECTOR_INTS: usize = SECTOR_BYTES / 4;
const HEADER_BYTES: usize = SECTOR_BYTES * 2;
const EMPTY_SECTOR: [u8; SECTOR_BYTES] = [0; SECTOR_BYTES];

/// Rust equivalent of MCP 1.12.2 `RegionFile` (`.mca`).
///
/// The on-disk contract is unchanged: sector 0 is the 1024-entry location
/// table, sector 1 is the timestamp table, chunk payloads start at sector 2,
/// and payload compression byte 1/2 means gzip/zlib respectively. New writes
/// use type 2 exactly like 1.12.2.
#[derive(Debug)]
pub struct RegionFile {
    fileName: PathBuf,
    dataFile: File,
    offsets: [i32; SECTOR_INTS],
    chunkTimestamps: [i32; SECTOR_INTS],
    sectorFree: Vec<bool>,
    sizeDelta: i32,
    lastModified: u64,
}

impl RegionFile {
    pub fn new(fileNameIn: impl AsRef<Path>) -> io::Result<Self> {
        let fileName = fileNameIn.as_ref().to_path_buf();
        if let Some(parent) = fileName.parent() { fs::create_dir_all(parent)?; }
        let lastModified = fileName.metadata().ok().and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_millis() as u64).unwrap_or(0);
        let mut dataFile = OpenOptions::new().read(true).write(true).create(true).open(&fileName)?;
        let mut sizeDelta = 0_i32;
        let mut length = dataFile.metadata()?.len();
        if length < SECTOR_BYTES as u64 {
            dataFile.seek(SeekFrom::Start(0))?;
            dataFile.write_all(&EMPTY_SECTOR)?;
            dataFile.write_all(&EMPTY_SECTOR)?;
            sizeDelta += HEADER_BYTES as i32;
            length = HEADER_BYTES as u64;
        }
        let remainder = (length as usize) & (SECTOR_BYTES - 1);
        if remainder != 0 {
            // Pad to the next complete 4 KiB sector. This is the intended
            // RegionFile invariant before the location table is interpreted.
            let padding = SECTOR_BYTES - remainder;
            dataFile.seek(SeekFrom::End(0))?;
            dataFile.write_all(&EMPTY_SECTOR[..padding])?;
            sizeDelta += padding as i32;
            length += padding as u64;
        }
        dataFile.flush()?;

        let sectorCount = (length as usize) / SECTOR_BYTES;
        let mut sectorFree = vec![true; sectorCount.max(2)];
        sectorFree[0] = false;
        sectorFree[1] = false;
        let mut offsets = [0_i32; SECTOR_INTS];
        let mut chunkTimestamps = [0_i32; SECTOR_INTS];

        dataFile.seek(SeekFrom::Start(0))?;
        for entry in &mut offsets {
            *entry = dataFile.read_i32::<BigEndian>()?;
            let sectorNumber = (*entry >> 8) as usize;
            let sectorLength = (*entry & 255) as usize;
            if *entry != 0 && sectorNumber + sectorLength <= sectorFree.len() {
                for sector in sectorNumber..sectorNumber + sectorLength {
                    sectorFree[sector] = false;
                }
            }
        }
        for entry in &mut chunkTimestamps {
            *entry = dataFile.read_i32::<BigEndian>()?;
        }

        Ok(Self { fileName, dataFile, offsets, chunkTimestamps, sectorFree, sizeDelta, lastModified })
    }

    /// MCP `getChunkDataInputStream`, returned as decompressed bytes so Rust
    /// callers can pass a slice directly to `CompressedStreamTools::readRoot`.
    pub fn readChunkData(&mut self, x: i32, z: i32) -> io::Result<Option<Vec<u8>>> {
        if Self::outOfBounds(x, z) { return Ok(None); }
        let offset = self.getOffset(x, z);
        if offset == 0 { return Ok(None); }
        let sectorNumber = (offset >> 8) as usize;
        let sectorCount = (offset & 255) as usize;
        if sectorNumber + sectorCount > self.sectorFree.len() { return Ok(None); }
        self.dataFile.seek(SeekFrom::Start((sectorNumber * SECTOR_BYTES) as u64))?;
        let length = self.dataFile.read_i32::<BigEndian>()?;
        if length <= 0 || length as usize > SECTOR_BYTES * sectorCount { return Ok(None); }
        let compression = self.dataFile.read_u8()?;
        let payloadLength = length as usize - 1;
        let mut compressed = vec![0_u8; payloadLength];
        self.dataFile.read_exact(&mut compressed)?;
        let mut output = Vec::new();
        match compression {
            1 => GzDecoder::new(compressed.as_slice()).read_to_end(&mut output)?,
            2 => ZlibDecoder::new(compressed.as_slice()).read_to_end(&mut output)?,
            _ => return Ok(None),
        };
        Ok(Some(output))
    }

    /// MCP `getChunkDataOutputStream` + `ChunkBuffer#close`: callers provide
    /// the uncompressed NBT stream, RegionFile writes zlib type 2.
    pub fn writeChunkData(&mut self, x: i32, z: i32, data: &[u8]) -> io::Result<bool> {
        if Self::outOfBounds(x, z) { return Ok(false); }
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        let compressed = encoder.finish()?;
        self.writeCompressed(x, z, &compressed)?;
        Ok(true)
    }

    fn writeCompressed(&mut self, x: i32, z: i32, data: &[u8]) -> io::Result<()> {
        let oldOffset = self.getOffset(x, z);
        let mut sectorNumber = (oldOffset >> 8) as usize;
        let oldSectorCount = (oldOffset & 255) as usize;
        let sectorsNeeded = (data.len() + 5) / SECTOR_BYTES + 1;
        if sectorsNeeded >= 256 { return Ok(()); }

        if sectorNumber != 0 && oldSectorCount == sectorsNeeded {
            self.writeSectorPayload(sectorNumber, data)?;
        } else {
            if sectorNumber != 0 {
                for sector in sectorNumber..sectorNumber.saturating_add(oldSectorCount).min(self.sectorFree.len()) {
                    self.sectorFree[sector] = true;
                }
            }

            let mut runStart = None;
            let mut runLength = 0_usize;
            for (index, free) in self.sectorFree.iter().copied().enumerate() {
                if free {
                    if runStart.is_none() { runStart = Some(index); runLength = 1; }
                    else { runLength += 1; }
                    if runLength >= sectorsNeeded { break; }
                } else {
                    runStart = None;
                    runLength = 0;
                }
            }

            if let Some(start) = runStart.filter(|_| runLength >= sectorsNeeded) {
                self.setOffset(x, z, ((start << 8) | sectorsNeeded) as i32)?;
                for sector in start..start + sectorsNeeded { self.sectorFree[sector] = false; }
                self.writeSectorPayload(start, data)?;
            } else {
                self.dataFile.seek(SeekFrom::End(0))?;
                sectorNumber = self.sectorFree.len();
                for _ in 0..sectorsNeeded {
                    self.dataFile.write_all(&EMPTY_SECTOR)?;
                    self.sectorFree.push(false);
                }
                self.sizeDelta += (SECTOR_BYTES * sectorsNeeded) as i32;
                self.writeSectorPayload(sectorNumber, data)?;
                self.setOffset(x, z, ((sectorNumber << 8) | sectorsNeeded) as i32)?;
            }
        }
        self.setChunkTimestamp(x, z, current_time_seconds())?;
        self.dataFile.flush()?;
        Ok(())
    }

    fn writeSectorPayload(&mut self, sectorNumber: usize, data: &[u8]) -> io::Result<()> {
        self.dataFile.seek(SeekFrom::Start((sectorNumber * SECTOR_BYTES) as u64))?;
        self.dataFile.write_i32::<BigEndian>((data.len() + 1) as i32)?;
        self.dataFile.write_u8(2)?;
        self.dataFile.write_all(data)
    }

    const fn outOfBounds(x: i32, z: i32) -> bool { x < 0 || x >= 32 || z < 0 || z >= 32 }
    fn getOffset(&self, x: i32, z: i32) -> i32 { self.offsets[(x + z * 32) as usize] }
    pub fn isChunkSaved(&self, x: i32, z: i32) -> bool {
        !Self::outOfBounds(x, z) && self.getOffset(x, z) != 0
    }
    fn setOffset(&mut self, x: i32, z: i32, offset: i32) -> io::Result<()> {
        let index = (x + z * 32) as usize;
        self.offsets[index] = offset;
        self.dataFile.seek(SeekFrom::Start((index * 4) as u64))?;
        self.dataFile.write_i32::<BigEndian>(offset)
    }
    fn setChunkTimestamp(&mut self, x: i32, z: i32, timestamp: i32) -> io::Result<()> {
        let index = (x + z * 32) as usize;
        self.chunkTimestamps[index] = timestamp;
        self.dataFile.seek(SeekFrom::Start((SECTOR_BYTES + index * 4) as u64))?;
        self.dataFile.write_i32::<BigEndian>(timestamp)
    }

    pub const fn getSizeDelta(&self) -> i32 { self.sizeDelta }
    pub const fn getLastModified(&self) -> u64 { self.lastModified }
    pub fn getFileName(&self) -> &Path { &self.fileName }
    pub fn close(&mut self) -> io::Result<()> { self.dataFile.flush() }
}

fn current_time_seconds() -> i32 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_region_has_two_header_sectors_and_roundtrips_zlib_chunks() {
        let root = std::env::temp_dir().join(format!("mc1122-region-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let path = root.join("r.0.0.mca");
        let mut region = RegionFile::new(&path).unwrap();
        assert!(region.getSizeDelta() >= 8192);
        assert_eq!(fs::metadata(&path).unwrap().len() % 4096, 0);
        assert!(!region.isChunkSaved(3, 5));
        let data = b"uncompressed NBT-shaped payload used by RegionFile";
        assert!(region.writeChunkData(3, 5, data).unwrap());
        assert!(region.isChunkSaved(3, 5));
        assert_eq!(region.readChunkData(3, 5).unwrap().unwrap(), data);
        region.close().unwrap();
        drop(region);
        let mut reopened = RegionFile::new(&path).unwrap();
        assert_eq!(reopened.readChunkData(3, 5).unwrap().unwrap(), data);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn chunk_coordinates_are_region_local() {
        let root = std::env::temp_dir().join(format!("mc1122-region-bounds-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let mut region = RegionFile::new(root.join("r.0.0.mca")).unwrap();
        assert!(!region.writeChunkData(-1, 0, b"x").unwrap());
        assert!(!region.writeChunkData(32, 0, b"x").unwrap());
        assert!(region.readChunkData(0, 32).unwrap().is_none());
        let _ = fs::remove_dir_all(root);
    }
}
