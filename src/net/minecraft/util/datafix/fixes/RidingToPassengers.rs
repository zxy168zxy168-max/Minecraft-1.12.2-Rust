use crate::net::minecraft::nbt::NBTBase::NBTBase;
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::nbt::NBTTagList::NBTTagList;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;

/// MCP 1.12.2 `RidingToPassengers` (DataVersion 135).
pub struct RidingToPassengers;
impl IFixableData for RidingToPassengers {
    fn getFixVersion(&self) -> i32 { 135 }
    fn fixTagCompound(&self, mut compound: NBTTagCompound) -> NBTTagCompound {
        while compound.hasKeyWithType("Riding", 10) {
            let mut vehicle = compound.getCompoundTag("Riding");
            compound.removeTag("Riding");
            let mut passengers = NBTTagList::new();
            passengers.appendTag(NBTBase::Compound(compound));
            vehicle.setTagList("Passengers", passengers);
            compound = vehicle;
        }
        compound
    }
}
