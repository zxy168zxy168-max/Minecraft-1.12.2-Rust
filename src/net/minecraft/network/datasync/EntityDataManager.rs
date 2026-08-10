use std::collections::BTreeMap;

use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::network::datasync::DataSerializers::{readValue, DataValue};
use crate::net::minecraft::network::PacketBuffer::{read_u8, read_var_i32, CodecError};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EntityDataManager {
    entries: BTreeMap<u8, DataValue>,
}

impl EntityDataManager {
    pub fn readEntries(input: &mut &[u8]) -> Result<Vec<(u8, DataValue)>, CodecError> {
        let mut result = Vec::new();
        loop {
            let id = read_u8(input)?;
            if id == 0xFF { break; }
            let serializer = read_var_i32(input)?;
            result.push((id, readValue(serializer, input)?));
        }
        Ok(result)
    }

    pub fn setEntryValues(&mut self, values: impl IntoIterator<Item = (u8, DataValue)>) {
        for (id, value) in values { self.entries.insert(id, value); }
    }

    pub fn get(&self, id: u8) -> Option<&DataValue> { self.entries.get(&id) }

    pub fn setByte(&mut self, id: u8, value: i8) {
        self.entries.insert(id, DataValue::Byte(value));
    }

    pub fn setVarInt(&mut self, id: u8, value: i32) {
        self.entries.insert(id, DataValue::VarInt(value));
    }

    pub fn setFloat(&mut self, id: u8, value: f32) {
        self.entries.insert(id, DataValue::Float(value));
    }

    pub fn setBoolean(&mut self, id: u8, value: bool) {
        self.entries.insert(id, DataValue::Boolean(value));
    }

    pub fn byte(&self, id: u8, fallback: i8) -> i8 {
        match self.entries.get(&id) { Some(DataValue::Byte(value)) => *value, _ => fallback }
    }

    pub fn varInt(&self, id: u8, fallback: i32) -> i32 {
        match self.entries.get(&id) { Some(DataValue::VarInt(value)) => *value, _ => fallback }
    }

    pub fn float(&self, id: u8, fallback: f32) -> f32 {
        match self.entries.get(&id) { Some(DataValue::Float(value)) => *value, _ => fallback }
    }

    pub fn boolean(&self, id: u8, fallback: bool) -> bool {
        match self.entries.get(&id) { Some(DataValue::Boolean(value)) => *value, _ => fallback }
    }

    pub fn string(&self, id: u8) -> Option<&str> {
        match self.entries.get(&id) { Some(DataValue::String(value)) => Some(value.as_str()), _ => None }
    }

    pub fn rotations(&self, id: u8, fallback: [f32; 3]) -> [f32; 3] {
        match self.entries.get(&id) { Some(DataValue::Rotations(value)) => *value, _ => fallback }
    }

    pub fn facing(&self, id: u8, fallback: i32) -> i32 {
        match self.entries.get(&id) { Some(DataValue::Facing(value)) => *value, _ => fallback }
    }

    pub fn optionalBlockPos(&self, id: u8) -> Option<crate::net::minecraft::util::math::BlockPos::BlockPos> {
        match self.entries.get(&id) {
            Some(DataValue::OptionalBlockPos(value)) => *value,
            _ => None,
        }
    }

    pub fn optionalBlockState(&self, id: u8) -> Option<i32> {
        match self.entries.get(&id) {
            Some(DataValue::OptionalBlockState(value)) => *value,
            _ => None,
        }
    }

    pub fn itemStack(&self, id: u8) -> Option<&ItemStack> {
        match self.entries.get(&id) { Some(DataValue::ItemStack(value)) => Some(value), _ => None }
    }
}
