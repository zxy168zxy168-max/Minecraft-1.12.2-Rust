use uuid::Uuid;

use crate::com::mojang::authlib::properties::Property::Property;
use crate::com::mojang::authlib::GameProfile::GameProfile;
use crate::net::minecraft::nbt::NBTBase::TAG_COMPOUND;
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::math::BlockPos::BlockPos;

/// Client-side data required by MCP 1.12.2 `TileEntitySkull` and
/// `TileEntitySkullRenderer`. Profile completion remains the responsibility of
/// the future SkinManager/session-service port; the packet/chunk NBT is kept
/// exactly rather than discarded.
#[derive(Debug, Clone, PartialEq)]
pub struct TileEntitySkull {
    pub pos: BlockPos,
    skullType: i32,
    skullRotation: i32,
    playerProfile: Option<GameProfile>,
    dragonAnimatedTicks: i32,
    dragonAnimated: bool,
}

impl TileEntitySkull {
    pub fn new(pos: BlockPos) -> Self {
        Self {
            pos,
            skullType: 0,
            skullRotation: 0,
            playerProfile: None,
            dragonAnimatedTicks: 0,
            dragonAnimated: false,
        }
    }

    pub fn fromNbt(tag: &NBTTagCompound) -> Option<Self> {
        let id = tag.getString("id");
        if !matches!(id.as_str(), "minecraft:skull" | "Skull") {
            return None;
        }
        let pos = BlockPos::new(
            tag.getInteger("x"),
            tag.getInteger("y"),
            tag.getInteger("z"),
        );
        let mut result = Self::new(pos);
        result.readFromNBT(tag);
        Some(result)
    }

    /// `TileEntitySkull#readFromNBT` plus `NBTUtil#readGameProfileFromNBT`.
    pub fn readFromNBT(&mut self, tag: &NBTTagCompound) {
        self.skullType = tag.getByte("SkullType") as u8 as i32;
        self.skullRotation = tag.getByte("Rot") as u8 as i32;
        self.playerProfile = if self.skullType == 3 {
            if tag.hasKeyWithType("Owner", TAG_COMPOUND) {
                readGameProfileFromNBT(&tag.getCompoundTag("Owner"))
            } else {
                let extra = tag.getString("ExtraType");
                (!extra.is_empty()).then(|| GameProfile::new(None, extra))
            }
        } else {
            None
        };
    }

    /// Public MCP `NBTUtil#readGameProfileFromNBT` bridge shared by world
    /// skulls and `LayerCustomHead` item NBT. Keeping one parser prevents the
    /// two rendering paths from disagreeing about signed texture properties.
    pub fn readGameProfileFromNBT(tag: &NBTTagCompound) -> Option<GameProfile> {
        readGameProfileFromNBT(tag)
    }

    pub const fn getSkullType(&self) -> i32 {
        self.skullType
    }
    pub const fn getSkullRotation(&self) -> i32 {
        self.skullRotation
    }
    pub fn getPlayerProfile(&self) -> Option<&GameProfile> {
        self.playerProfile.as_ref()
    }
    pub fn getAnimationProgress(&self, partialTicks: f32) -> f32 {
        if self.dragonAnimated {
            self.dragonAnimatedTicks as f32 + partialTicks
        } else {
            self.dragonAnimatedTicks as f32
        }
    }

    pub fn setDragonPowered(&mut self, powered: bool) {
        self.dragonAnimated = powered;
    }

    pub fn tick(&mut self) {
        if self.skullType == 5 && self.dragonAnimated {
            self.dragonAnimatedTicks = self.dragonAnimatedTicks.wrapping_add(1);
        }
    }
}

pub fn readGameProfileFromNBT(tag: &NBTTagCompound) -> Option<GameProfile> {
    let name = tag.getString("Name");
    let id = Uuid::parse_str(&tag.getString("Id")).ok();
    if name.is_empty() && id.is_none() {
        return None;
    }
    let mut profile = GameProfile::new(id, name);
    if tag.hasKeyWithType("Properties", TAG_COMPOUND) {
        let properties = tag.getCompoundTag("Properties");
        for property_name in properties.getKeySet() {
            let list = properties.getTagList(property_name, TAG_COMPOUND);
            for index in 0..list.tagCount() {
                let property = list.getCompoundTagAt(index);
                let value = property.getString("Value");
                if value.is_empty() {
                    continue;
                }
                let signature = property
                    .hasKey("Signature")
                    .then(|| property.getString("Signature"));
                profile.addProperty(Property::new(property_name.clone(), value, signature));
            }
        }
    }
    Some(profile)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::nbt::NBTBase::NBTBase;
    use crate::net::minecraft::nbt::NBTTagList::NBTTagList;

    #[test]
    fn chunk_nbt_restores_type_rotation_and_owner() {
        let mut texture = NBTTagCompound::new();
        texture.setString("Value", "base64");
        texture.setString("Signature", "signed");
        let mut list = NBTTagList::new();
        list.appendTag(NBTBase::Compound(texture));
        let mut properties = NBTTagCompound::new();
        properties.setTagList("textures", list);
        let mut owner = NBTTagCompound::new();
        owner.setString("Name", "Alex");
        owner.setString("Id", "ec561538-f3fd-461d-aff5-086b22154bce");
        owner.setCompoundTag("Properties", properties);
        let mut tag = NBTTagCompound::new();
        tag.setString("id", "minecraft:skull");
        tag.setInteger("x", 2);
        tag.setInteger("y", 64);
        tag.setInteger("z", -3);
        tag.setByte("SkullType", 3);
        tag.setByte("Rot", 12);
        tag.setCompoundTag("Owner", owner);
        let skull = TileEntitySkull::fromNbt(&tag).unwrap();
        assert_eq!(skull.pos, BlockPos::new(2, 64, -3));
        assert_eq!(skull.getSkullType(), 3);
        assert_eq!(skull.getSkullRotation(), 12);
        let profile = skull.getPlayerProfile().unwrap();
        assert_eq!(profile.getName(), "Alex");
        assert_eq!(profile.getProperties().len(), 1);
    }
}
