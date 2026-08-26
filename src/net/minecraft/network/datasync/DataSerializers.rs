use uuid::Uuid;

use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::network::PacketBuffer::{
    read_bool, read_f32_be, read_i64_be, read_nbt_compound, read_string, read_u8, read_uuid,
    read_var_i32, CodecError,
};
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::text::ITextComponent::ITextComponent;

#[derive(Debug, Clone, PartialEq)]
pub enum DataValue {
    Byte(i8),
    VarInt(i32),
    Float(f32),
    String(String),
    TextComponent(ITextComponent),
    ItemStack(ItemStack),
    Boolean(bool),
    Rotations([f32; 3]),
    BlockPos(BlockPos),
    OptionalBlockPos(Option<BlockPos>),
    Facing(i32),
    OptionalUuid(Option<Uuid>),
    OptionalBlockState(Option<i32>),
    Compound(Option<NBTTagCompound>),
}

pub fn readValue(serializerId: i32, input: &mut &[u8]) -> Result<DataValue, CodecError> {
    Ok(match serializerId {
        0 => DataValue::Byte(read_u8(input)? as i8),
        1 => DataValue::VarInt(read_var_i32(input)?),
        2 => DataValue::Float(read_f32_be(input)?),
        3 => DataValue::String(read_string(input, 32767)?),
        4 => {
            let json = read_string(input, 32767)?;
            DataValue::TextComponent(
                ITextComponent::fromJsonLenient(&json)
                    .map_err(|error| CodecError::InvalidData(error.to_string()))?,
            )
        }
        5 => DataValue::ItemStack(ItemStack::readFromBuffer(input)?),
        6 => DataValue::Boolean(read_bool(input)?),
        7 => DataValue::Rotations([
            read_f32_be(input)?,
            read_f32_be(input)?,
            read_f32_be(input)?,
        ]),
        8 => DataValue::BlockPos(unpack_block_pos(read_i64_be(input)?)),
        9 => DataValue::OptionalBlockPos(if read_bool(input)? {
            Some(unpack_block_pos(read_i64_be(input)?))
        } else {
            None
        }),
        10 => DataValue::Facing(read_var_i32(input)?),
        11 => DataValue::OptionalUuid(if read_bool(input)? {
            Some(read_uuid(input)?)
        } else {
            None
        }),
        12 => {
            let state = read_var_i32(input)?;
            DataValue::OptionalBlockState((state != 0).then_some(state))
        }
        13 => DataValue::Compound(read_nbt_compound(input)?),
        id => {
            return Err(CodecError::InvalidData(format!(
                "unknown entity data serializer {id}"
            )))
        }
    })
}

fn unpack_block_pos(value: i64) -> BlockPos {
    let x = (value >> 38) as i32;
    let y = (value & 0xFFF) as i32;
    let z = ((value << 26) >> 38) as i32;
    BlockPos::new(x, y, z)
}
