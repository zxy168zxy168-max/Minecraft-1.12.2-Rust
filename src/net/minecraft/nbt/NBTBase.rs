use std::collections::BTreeMap;
use std::io::{self, Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::nbt::NBTTagList::NBTTagList;

pub const TAG_END: u8 = 0;
pub const TAG_BYTE: u8 = 1;
pub const TAG_SHORT: u8 = 2;
pub const TAG_INT: u8 = 3;
pub const TAG_LONG: u8 = 4;
pub const TAG_FLOAT: u8 = 5;
pub const TAG_DOUBLE: u8 = 6;
pub const TAG_BYTE_ARRAY: u8 = 7;
pub const TAG_STRING: u8 = 8;
pub const TAG_LIST: u8 = 9;
pub const TAG_COMPOUND: u8 = 10;
pub const TAG_INT_ARRAY: u8 = 11;
pub const TAG_LONG_ARRAY: u8 = 12;

#[derive(Debug, Clone, PartialEq)]
pub enum NBTBase {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<u8>),
    String(String),
    List(NBTTagList),
    Compound(NBTTagCompound),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl NBTBase {
    pub const fn getId(&self) -> u8 {
        match self {
            Self::End => TAG_END,
            Self::Byte(_) => TAG_BYTE,
            Self::Short(_) => TAG_SHORT,
            Self::Int(_) => TAG_INT,
            Self::Long(_) => TAG_LONG,
            Self::Float(_) => TAG_FLOAT,
            Self::Double(_) => TAG_DOUBLE,
            Self::ByteArray(_) => TAG_BYTE_ARRAY,
            Self::String(_) => TAG_STRING,
            Self::List(_) => TAG_LIST,
            Self::Compound(_) => TAG_COMPOUND,
            Self::IntArray(_) => TAG_INT_ARRAY,
            Self::LongArray(_) => TAG_LONG_ARRAY,
        }
    }

    pub fn writePayload<W: Write>(&self, output: &mut W) -> io::Result<()> {
        match self {
            Self::End => Ok(()),
            Self::Byte(value) => output.write_i8(*value),
            Self::Short(value) => output.write_i16::<BigEndian>(*value),
            Self::Int(value) => output.write_i32::<BigEndian>(*value),
            Self::Long(value) => output.write_i64::<BigEndian>(*value),
            Self::Float(value) => output.write_f32::<BigEndian>(*value),
            Self::Double(value) => output.write_f64::<BigEndian>(*value),
            Self::ByteArray(values) => {
                writeLength(output, values.len())?;
                output.write_all(values)
            }
            Self::String(value) => writeJavaUtf(output, value),
            Self::List(value) => value.write(output),
            Self::Compound(value) => value.write(output),
            Self::IntArray(values) => {
                writeLength(output, values.len())?;
                for value in values {
                    output.write_i32::<BigEndian>(*value)?;
                }
                Ok(())
            }
            Self::LongArray(values) => {
                writeLength(output, values.len())?;
                for value in values {
                    output.write_i64::<BigEndian>(*value)?;
                }
                Ok(())
            }
        }
    }

    pub fn readPayload<R: Read>(tagId: u8, input: &mut R, depth: usize) -> io::Result<Self> {
        if depth > 512 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Tried to read NBT tag with too high complexity, depth > 512",
            ));
        }
        Ok(match tagId {
            TAG_END => Self::End,
            TAG_BYTE => Self::Byte(input.read_i8()?),
            TAG_SHORT => Self::Short(input.read_i16::<BigEndian>()?),
            TAG_INT => Self::Int(input.read_i32::<BigEndian>()?),
            TAG_LONG => Self::Long(input.read_i64::<BigEndian>()?),
            TAG_FLOAT => Self::Float(input.read_f32::<BigEndian>()?),
            TAG_DOUBLE => Self::Double(input.read_f64::<BigEndian>()?),
            TAG_BYTE_ARRAY => {
                let length = readLength(input)?;
                let mut values = vec![0_u8; length];
                input.read_exact(&mut values)?;
                Self::ByteArray(values)
            }
            TAG_STRING => Self::String(readJavaUtf(input)?),
            TAG_LIST => Self::List(NBTTagList::read(input, depth + 1)?),
            TAG_COMPOUND => Self::Compound(NBTTagCompound::read(input, depth + 1)?),
            TAG_INT_ARRAY => {
                let length = readLength(input)?;
                let mut values = Vec::with_capacity(length);
                for _ in 0..length {
                    values.push(input.read_i32::<BigEndian>()?);
                }
                Self::IntArray(values)
            }
            TAG_LONG_ARRAY => {
                let length = readLength(input)?;
                let mut values = Vec::with_capacity(length);
                for _ in 0..length {
                    values.push(input.read_i64::<BigEndian>()?);
                }
                Self::LongArray(values)
            }
            other => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid NBT tag id {other}"),
                ))
            }
        })
    }
}

fn writeLength<W: Write>(output: &mut W, length: usize) -> io::Result<()> {
    let length = i32::try_from(length)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "NBT collection is too large"))?;
    output.write_i32::<BigEndian>(length)
}

fn readLength<R: Read>(input: &mut R) -> io::Result<usize> {
    let length = input.read_i32::<BigEndian>()?;
    if length < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Negative NBT collection length",
        ));
    }
    Ok(length as usize)
}

/// DataInputStream/DataOutputStream modified UTF-8 used by the NBT format.
pub fn writeJavaUtf<W: Write>(output: &mut W, value: &str) -> io::Result<()> {
    let mut encoded = Vec::with_capacity(value.len());
    for unit in value.encode_utf16() {
        match unit {
            0x0001..=0x007F => encoded.push(unit as u8),
            0x0000 | 0x0080..=0x07FF => {
                encoded.push((0xC0 | ((unit >> 6) & 0x1F)) as u8);
                encoded.push((0x80 | (unit & 0x3F)) as u8);
            }
            _ => {
                encoded.push((0xE0 | ((unit >> 12) & 0x0F)) as u8);
                encoded.push((0x80 | ((unit >> 6) & 0x3F)) as u8);
                encoded.push((0x80 | (unit & 0x3F)) as u8);
            }
        }
    }
    let length = u16::try_from(encoded.len()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "encoded string too long: more than 65535 bytes",
        )
    })?;
    output.write_u16::<BigEndian>(length)?;
    output.write_all(&encoded)
}

pub fn readJavaUtf<R: Read>(input: &mut R) -> io::Result<String> {
    let length = input.read_u16::<BigEndian>()? as usize;
    let mut bytes = vec![0_u8; length];
    input.read_exact(&mut bytes)?;
    let mut units = Vec::<u16>::with_capacity(length);
    let mut index = 0;
    while index < bytes.len() {
        let first = bytes[index];
        if first & 0x80 == 0 {
            if first == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "NUL byte is invalid in modified UTF-8",
                ));
            }
            units.push(first as u16);
            index += 1;
        } else if first & 0xE0 == 0xC0 {
            if index + 1 >= bytes.len() {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "truncated modified UTF-8 sequence",
                ));
            }
            let second = bytes[index + 1];
            if second & 0xC0 != 0x80 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid modified UTF-8 continuation byte",
                ));
            }
            units.push((((first & 0x1F) as u16) << 6) | ((second & 0x3F) as u16));
            index += 2;
        } else if first & 0xF0 == 0xE0 {
            if index + 2 >= bytes.len() {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "truncated modified UTF-8 sequence",
                ));
            }
            let second = bytes[index + 1];
            let third = bytes[index + 2];
            if second & 0xC0 != 0x80 || third & 0xC0 != 0x80 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid modified UTF-8 continuation byte",
                ));
            }
            units.push(
                (((first & 0x0F) as u16) << 12)
                    | (((second & 0x3F) as u16) << 6)
                    | ((third & 0x3F) as u16),
            );
            index += 3;
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid modified UTF-8 leading byte",
            ));
        }
    }
    String::from_utf16(&units).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid UTF-16 sequence in modified UTF-8 string",
        )
    })
}

pub(crate) fn emptyCompoundMap() -> BTreeMap<String, NBTBase> {
    BTreeMap::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn java_modified_utf_roundtrip_includes_nul_and_supplementary_character() {
        let value = "A\0中😀";
        let mut bytes = Vec::new();
        writeJavaUtf(&mut bytes, value).unwrap();
        assert_eq!(u16::from_be_bytes([bytes[0], bytes[1]]), 12);
        assert_eq!(bytes[2], b'A');
        assert_eq!(&bytes[3..5], &[0xC0, 0x80]);
        assert_eq!(readJavaUtf(&mut bytes.as_slice()).unwrap(), value);
    }
}
