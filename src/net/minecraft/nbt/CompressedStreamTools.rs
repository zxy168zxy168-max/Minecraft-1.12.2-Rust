use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;

use byteorder::{ReadBytesExt, WriteBytesExt};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};

use crate::net::minecraft::nbt::NBTBase::{
    readJavaUtf, writeJavaUtf, NBTBase, TAG_COMPOUND, TAG_END,
};
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;

pub fn read(fileIn: impl AsRef<Path>) -> io::Result<Option<NBTTagCompound>> {
    let path = fileIn.as_ref();
    if !path.exists() {
        return Ok(None);
    }
    let mut input = BufReader::new(File::open(path)?);
    readRoot(&mut input).map(Some)
}

pub fn write(compound: &NBTTagCompound, fileIn: impl AsRef<Path>) -> io::Result<()> {
    let path = fileIn.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut output = BufWriter::new(File::create(path)?);
    writeRoot(compound, &mut output)?;
    output.flush()
}

pub fn safeWrite(compound: &NBTTagCompound, fileIn: impl AsRef<Path>) -> io::Result<()> {
    let path = fileIn.as_ref();
    let temporary = path.with_file_name(format!(
        "{}_tmp",
        path.file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("servers.dat")
    ));
    if temporary.exists() {
        fs::remove_file(&temporary)?;
    }
    write(compound, &temporary)?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(temporary, path)
}

pub fn readCompressed<R: Read>(input: R) -> io::Result<NBTTagCompound> {
    let mut decoder = BufReader::new(GzDecoder::new(input));
    readRoot(&mut decoder)
}

pub fn writeCompressed<W: Write>(compound: &NBTTagCompound, output: W) -> io::Result<()> {
    let mut encoder = GzEncoder::new(output, Compression::default());
    writeRoot(compound, &mut encoder)?;
    encoder.finish().map(|_| ())
}

pub fn readRoot<R: Read>(input: &mut R) -> io::Result<NBTTagCompound> {
    let tagId = input.read_u8()?;
    if tagId == TAG_END {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Root tag must be a named compound tag",
        ));
    }
    let _name = readJavaUtf(input)?;
    match NBTBase::readPayload(tagId, input, 0)? {
        NBTBase::Compound(compound) => Ok(compound),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Root tag must be a named compound tag",
        )),
    }
}

pub fn writeRoot<W: Write>(compound: &NBTTagCompound, output: &mut W) -> io::Result<()> {
    output.write_u8(TAG_COMPOUND)?;
    writeJavaUtf(output, "")?;
    compound.write(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::nbt::NBTBase::NBTBase;
    use crate::net::minecraft::nbt::NBTTagList::NBTTagList;

    #[test]
    fn servers_dat_shape_roundtrips_without_compression() {
        let mut server = NBTTagCompound::new();
        server.setString("name", "Local");
        server.setString("ip", "127.0.0.1:25565");
        server.setBoolean("acceptTextures", true);
        let mut list = NBTTagList::new();
        list.appendTag(NBTBase::Compound(server));
        let mut root = NBTTagCompound::new();
        root.setTagList("servers", list);
        let mut bytes = Vec::new();
        writeRoot(&root, &mut bytes).unwrap();
        let decoded = readRoot(&mut bytes.as_slice()).unwrap();
        assert_eq!(decoded, root);
    }
}
