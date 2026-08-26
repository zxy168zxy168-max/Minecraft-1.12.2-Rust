use std::collections::BTreeMap;
use std::io::{self, Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt};
use uuid::Uuid;

use crate::net::minecraft::nbt::NBTBase::{
    emptyCompoundMap, readJavaUtf, writeJavaUtf, NBTBase, TAG_COMPOUND, TAG_END,
};
use crate::net::minecraft::nbt::NBTTagList::NBTTagList;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct NBTTagCompound {
    tagMap: BTreeMap<String, NBTBase>,
}

impl NBTTagCompound {
    pub fn new() -> Self {
        Self {
            tagMap: emptyCompoundMap(),
        }
    }
    pub const fn getId(&self) -> u8 {
        TAG_COMPOUND
    }

    pub fn setTag(&mut self, key: impl Into<String>, value: NBTBase) {
        self.tagMap.insert(key.into(), value);
    }
    pub fn setByte(&mut self, key: impl Into<String>, value: i8) {
        self.setTag(key, NBTBase::Byte(value));
    }
    pub fn setBoolean(&mut self, key: impl Into<String>, value: bool) {
        self.setByte(key, if value { 1 } else { 0 });
    }
    pub fn setShort(&mut self, key: impl Into<String>, value: i16) {
        self.setTag(key, NBTBase::Short(value));
    }
    pub fn setInteger(&mut self, key: impl Into<String>, value: i32) {
        self.setTag(key, NBTBase::Int(value));
    }
    pub fn setLong(&mut self, key: impl Into<String>, value: i64) {
        self.setTag(key, NBTBase::Long(value));
    }
    /// MCP `NBTTagCompound#setUniqueId`: UUID is stored as signed Most/Least longs.
    pub fn setUniqueId(&mut self, key: &str, value: Uuid) {
        let raw = value.as_u128();
        self.setLong(format!("{key}Most"), (raw >> 64) as u64 as i64);
        self.setLong(format!("{key}Least"), raw as u64 as i64);
    }
    pub fn setFloat(&mut self, key: impl Into<String>, value: f32) {
        self.setTag(key, NBTBase::Float(value));
    }
    pub fn setDouble(&mut self, key: impl Into<String>, value: f64) {
        self.setTag(key, NBTBase::Double(value));
    }
    pub fn setString(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.setTag(key, NBTBase::String(value.into()));
    }
    pub fn setByteArray(&mut self, key: impl Into<String>, value: Vec<u8>) {
        self.setTag(key, NBTBase::ByteArray(value));
    }
    pub fn setIntArray(&mut self, key: impl Into<String>, value: Vec<i32>) {
        self.setTag(key, NBTBase::IntArray(value));
    }
    pub fn setLongArray(&mut self, key: impl Into<String>, value: Vec<i64>) {
        self.setTag(key, NBTBase::LongArray(value));
    }
    pub fn setCompoundTag(&mut self, key: impl Into<String>, value: NBTTagCompound) {
        self.setTag(key, NBTBase::Compound(value));
    }
    pub fn setTagList(&mut self, key: impl Into<String>, value: NBTTagList) {
        self.setTag(key, NBTBase::List(value));
    }

    pub fn getTag(&self, key: &str) -> Option<&NBTBase> {
        self.tagMap.get(key)
    }
    pub fn getTagId(&self, key: &str) -> u8 {
        self.getTag(key).map(NBTBase::getId).unwrap_or(TAG_END)
    }
    pub fn hasKey(&self, key: &str) -> bool {
        self.tagMap.contains_key(key)
    }
    pub fn hasKeyWithType(&self, key: &str, tagType: u8) -> bool {
        let actual = self.getTagId(key);
        actual == tagType || (tagType == 99 && matches!(actual, 1..=6))
    }

    pub fn getByte(&self, key: &str) -> i8 {
        match self.getTag(key) {
            Some(NBTBase::Byte(v)) => *v,
            Some(NBTBase::Short(v)) => *v as i8,
            Some(NBTBase::Int(v)) => *v as i8,
            Some(NBTBase::Long(v)) => *v as i8,
            Some(NBTBase::Float(v)) => *v as i8,
            Some(NBTBase::Double(v)) => *v as i8,
            _ => 0,
        }
    }
    pub fn getShort(&self, key: &str) -> i16 {
        match self.getTag(key) {
            Some(NBTBase::Byte(v)) => *v as i16,
            Some(NBTBase::Short(v)) => *v,
            Some(NBTBase::Int(v)) => *v as i16,
            Some(NBTBase::Long(v)) => *v as i16,
            Some(NBTBase::Float(v)) => *v as i16,
            Some(NBTBase::Double(v)) => *v as i16,
            _ => 0,
        }
    }
    pub fn getBoolean(&self, key: &str) -> bool {
        self.getByte(key) != 0
    }
    pub fn getInteger(&self, key: &str) -> i32 {
        match self.getTag(key) {
            Some(NBTBase::Byte(v)) => *v as i32,
            Some(NBTBase::Short(v)) => *v as i32,
            Some(NBTBase::Int(v)) => *v,
            Some(NBTBase::Long(v)) => *v as i32,
            Some(NBTBase::Float(v)) => *v as i32,
            Some(NBTBase::Double(v)) => *v as i32,
            _ => 0,
        }
    }
    pub fn getLong(&self, key: &str) -> i64 {
        match self.getTag(key) {
            Some(NBTBase::Byte(v)) => *v as i64,
            Some(NBTBase::Short(v)) => *v as i64,
            Some(NBTBase::Int(v)) => *v as i64,
            Some(NBTBase::Long(v)) => *v,
            Some(NBTBase::Float(v)) => *v as i64,
            Some(NBTBase::Double(v)) => *v as i64,
            _ => 0,
        }
    }
    /// MCP `NBTTagCompound#getUniqueId`.
    pub fn getUniqueId(&self, key: &str) -> Uuid {
        let most = self.getLong(&format!("{key}Most")) as u64 as u128;
        let least = self.getLong(&format!("{key}Least")) as u64 as u128;
        Uuid::from_u128((most << 64) | least)
    }
    pub fn hasUniqueId(&self, key: &str) -> bool {
        self.hasKeyWithType(&format!("{key}Most"), 99)
            && self.hasKeyWithType(&format!("{key}Least"), 99)
    }
    pub fn getFloat(&self, key: &str) -> f32 {
        match self.getTag(key) {
            Some(NBTBase::Byte(v)) => *v as f32,
            Some(NBTBase::Short(v)) => *v as f32,
            Some(NBTBase::Int(v)) => *v as f32,
            Some(NBTBase::Long(v)) => *v as f32,
            Some(NBTBase::Float(v)) => *v,
            Some(NBTBase::Double(v)) => *v as f32,
            _ => 0.0,
        }
    }
    pub fn getDouble(&self, key: &str) -> f64 {
        match self.getTag(key) {
            Some(NBTBase::Byte(v)) => *v as f64,
            Some(NBTBase::Short(v)) => *v as f64,
            Some(NBTBase::Int(v)) => *v as f64,
            Some(NBTBase::Long(v)) => *v as f64,
            Some(NBTBase::Float(v)) => *v as f64,
            Some(NBTBase::Double(v)) => *v,
            _ => 0.0,
        }
    }
    pub fn getByteArray(&self, key: &str) -> Vec<u8> {
        match self.getTag(key) {
            Some(NBTBase::ByteArray(value)) => value.clone(),
            _ => Vec::new(),
        }
    }
    pub fn getIntArray(&self, key: &str) -> Vec<i32> {
        match self.getTag(key) {
            Some(NBTBase::IntArray(value)) => value.clone(),
            _ => Vec::new(),
        }
    }
    pub fn getLongArray(&self, key: &str) -> Vec<i64> {
        match self.getTag(key) {
            Some(NBTBase::LongArray(value)) => value.clone(),
            _ => Vec::new(),
        }
    }
    pub fn getString(&self, key: &str) -> String {
        match self.getTag(key) {
            Some(NBTBase::String(v)) => v.clone(),
            _ => String::new(),
        }
    }
    pub fn getCompoundTag(&self, key: &str) -> NBTTagCompound {
        match self.getTag(key) {
            Some(NBTBase::Compound(v)) => v.clone(),
            _ => Self::new(),
        }
    }
    pub fn getTagList(&self, key: &str, expectedType: u8) -> NBTTagList {
        match self.getTag(key) {
            Some(NBTBase::List(v)) if v.hasNoTags() || v.getTagType() == expectedType => v.clone(),
            _ => NBTTagList::new(),
        }
    }
    pub fn getKeySet(&self) -> impl Iterator<Item = &String> {
        self.tagMap.keys()
    }
    pub fn removeTag(&mut self, key: &str) {
        self.tagMap.remove(key);
    }
    pub fn hasNoTags(&self) -> bool {
        self.tagMap.is_empty()
    }

    pub(crate) fn write<W: Write>(&self, output: &mut W) -> io::Result<()> {
        for (name, tag) in &self.tagMap {
            output.write_u8(tag.getId())?;
            writeJavaUtf(output, name)?;
            tag.writePayload(output)?;
        }
        output.write_u8(TAG_END)
    }

    pub(crate) fn read<R: Read>(input: &mut R, depth: usize) -> io::Result<Self> {
        if depth > 512 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Tried to read NBT tag with too high complexity, depth > 512",
            ));
        }
        let mut result = Self::new();
        loop {
            let tagId = input.read_u8()?;
            if tagId == TAG_END {
                break;
            }
            let name = readJavaUtf(input)?;
            let value = NBTBase::readPayload(tagId, input, depth + 1)?;
            result.tagMap.insert(name, value);
        }
        Ok(result)
    }
}
