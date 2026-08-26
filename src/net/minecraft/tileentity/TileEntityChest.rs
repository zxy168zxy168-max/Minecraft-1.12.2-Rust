use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::math::BlockPos::BlockPos;

/// Client animation subset of MCP 1.12.2 `TileEntityChest`.
#[derive(Debug, Clone, PartialEq)]
pub struct TileEntityChest {
    pub pos: BlockPos,
    pub lidAngle: f32,
    pub prevLidAngle: f32,
    pub numPlayersUsing: i32,
}

impl TileEntityChest {
    pub fn new(pos: BlockPos) -> Self {
        Self {
            pos,
            lidAngle: 0.0,
            prevLidAngle: 0.0,
            numPlayersUsing: 0,
        }
    }

    pub fn fromNbt(tag: &NBTTagCompound) -> Option<Self> {
        let id = tag.getString("id");
        if !matches!(
            id.as_str(),
            "minecraft:chest" | "Chest" | "minecraft:trapped_chest"
        ) {
            return None;
        }
        Some(Self::new(BlockPos::new(
            tag.getInteger("x"),
            tag.getInteger("y"),
            tag.getInteger("z"),
        )))
    }

    pub fn receiveClientEvent(&mut self, id: i32, eventType: i32) -> bool {
        if id != 1 {
            return false;
        }
        self.numPlayersUsing = eventType;
        true
    }

    pub fn update(&mut self) {
        self.prevLidAngle = self.lidAngle;
        if self.numPlayersUsing == 0 && self.lidAngle > 0.0
            || self.numPlayersUsing > 0 && self.lidAngle < 1.0
        {
            if self.numPlayersUsing > 0 {
                self.lidAngle += 0.1;
            } else {
                self.lidAngle -= 0.1;
            }
            self.lidAngle = self.lidAngle.clamp(0.0, 1.0);
        }
    }

    pub fn interpolatedLidAngle(&self, partialTicks: f32) -> f32 {
        let angle =
            self.prevLidAngle + (self.lidAngle - self.prevLidAngle) * partialTicks.clamp(0.0, 1.0);
        let inverse = 1.0 - angle;
        1.0 - inverse * inverse * inverse
    }
}
