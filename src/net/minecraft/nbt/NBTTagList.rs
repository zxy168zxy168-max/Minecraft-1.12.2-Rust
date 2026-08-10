use std::io::{self, Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::net::minecraft::nbt::NBTBase::{NBTBase, TAG_END};
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct NBTTagList {
    tagList: Vec<NBTBase>,
    tagType: u8,
}

impl NBTTagList {
    pub fn new() -> Self { Self::default() }

    pub fn appendTag(&mut self, nbt: NBTBase) {
        if nbt.getId() == TAG_END { return; }
        if self.tagType == TAG_END { self.tagType = nbt.getId(); }
        if self.tagType == nbt.getId() { self.tagList.push(nbt); }
    }

    pub fn set(&mut self, index: usize, nbt: NBTBase) {
        if nbt.getId() == TAG_END || index >= self.tagList.len() { return; }
        if self.tagType == TAG_END { self.tagType = nbt.getId(); }
        if self.tagType == nbt.getId() { self.tagList[index] = nbt; }
    }

    pub fn removeTag(&mut self, index: usize) -> Option<NBTBase> {
        if index >= self.tagList.len() { return None; }
        let removed = self.tagList.remove(index);
        if self.tagList.is_empty() { self.tagType = TAG_END; }
        Some(removed)
    }

    pub fn hasNoTags(&self) -> bool { self.tagList.is_empty() }
    pub fn tagCount(&self) -> usize { self.tagList.len() }
    pub const fn getTagType(&self) -> u8 { self.tagType }
    pub fn tags(&self) -> &[NBTBase] { &self.tagList }

    pub fn getCompoundTagAt(&self, index: usize) -> NBTTagCompound {
        match self.tagList.get(index) {
            Some(NBTBase::Compound(value)) => value.clone(),
            _ => NBTTagCompound::new(),
        }
    }

    pub fn getStringTagAt(&self, index: usize) -> String {
        match self.tagList.get(index) {
            Some(NBTBase::String(value)) => value.clone(),
            Some(value) => format!("{value:?}"),
            None => String::new(),
        }
    }

    /// MCP `NBTTagList#getDoubleAt`. Numeric list access returns zero for a
    /// missing/wrong-type element instead of failing the whole entity load.
    pub fn getDoubleAt(&self, index: usize) -> f64 {
        match self.tagList.get(index) {
            Some(NBTBase::Double(value)) => *value,
            Some(NBTBase::Float(value)) => *value as f64,
            Some(NBTBase::Long(value)) => *value as f64,
            Some(NBTBase::Int(value)) => *value as f64,
            Some(NBTBase::Short(value)) => *value as f64,
            Some(NBTBase::Byte(value)) => *value as f64,
            _ => 0.0,
        }
    }

    /// MCP `NBTTagList#getFloatAt`.
    pub fn getFloatAt(&self, index: usize) -> f32 {
        match self.tagList.get(index) {
            Some(NBTBase::Float(value)) => *value,
            Some(NBTBase::Double(value)) => *value as f32,
            Some(NBTBase::Long(value)) => *value as f32,
            Some(NBTBase::Int(value)) => *value as f32,
            Some(NBTBase::Short(value)) => *value as f32,
            Some(NBTBase::Byte(value)) => *value as f32,
            _ => 0.0,
        }
    }

    pub(crate) fn write<W: Write>(&self, output: &mut W) -> io::Result<()> {
        let tagType = self.tagList.first().map(NBTBase::getId).unwrap_or(TAG_END);
        output.write_u8(tagType)?;
        output.write_i32::<BigEndian>(i32::try_from(self.tagList.len()).map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "NBT list too large"))?)?;
        for tag in &self.tagList { tag.writePayload(output)?; }
        Ok(())
    }

    pub(crate) fn read<R: Read>(input: &mut R, depth: usize) -> io::Result<Self> {
        let tagType = input.read_u8()?;
        let length = input.read_i32::<BigEndian>()?;
        if length < 0 { return Err(io::Error::new(io::ErrorKind::InvalidData, "Negative NBT list length")); }
        if tagType == TAG_END && length > 0 { return Err(io::Error::new(io::ErrorKind::InvalidData, "Missing type on ListTag")); }
        let mut tagList = Vec::with_capacity(length as usize);
        for _ in 0..length { tagList.push(NBTBase::readPayload(tagType, input, depth + 1)?); }
        Ok(Self { tagList, tagType })
    }
}
