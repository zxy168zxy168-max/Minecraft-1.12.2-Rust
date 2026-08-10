use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;

pub struct PaintingDirection;
impl IFixableData for PaintingDirection {
    fn getFixVersion(&self) -> i32 { 111 }
    fn fixTagCompound(&self, mut compound: NBTTagCompound) -> NBTTagCompound {
        let id = compound.getString("id");
        let painting = id == "Painting"; let frame = id == "ItemFrame";
        if (painting || frame) && !compound.hasKeyWithType("Facing", 99) {
            let horizontal = if compound.hasKeyWithType("Direction", 99) {
                let h = ((compound.getByte("Direction") as i32 % 4).abs()) as u8;
                // EnumFacing.getHorizontal: 0=SOUTH,1=WEST,2=NORTH,3=EAST.
                const OFFSETS: [(i32,i32,i32);4] = [(0,0,1),(-1,0,0),(0,0,-1),(1,0,0)];
                let (dx,dy,dz)=OFFSETS[h as usize];
                compound.setInteger("TileX", compound.getInteger("TileX").wrapping_add(dx));
                compound.setInteger("TileY", compound.getInteger("TileY").wrapping_add(dy));
                compound.setInteger("TileZ", compound.getInteger("TileZ").wrapping_add(dz));
                compound.removeTag("Direction");
                if frame && compound.hasKeyWithType("ItemRotation",99) { compound.setByte("ItemRotation", compound.getByte("ItemRotation").wrapping_mul(2)); }
                h
            } else {
                let h = ((compound.getByte("Dir") as i32 % 4).abs()) as u8; compound.removeTag("Dir"); h
            };
            compound.setByte("Facing", horizontal as i8);
        }
        compound
    }
}
