use crate::net::minecraft::util::math::BlockPos::BlockPos;

/// MCP 1.12.2 `TileEntityEnderChest` lid state.
#[derive(Debug, Clone, PartialEq)]
pub struct TileEntityEnderChest {
    pub pos: BlockPos,
    pub lidAngle: f32,
    pub prevLidAngle: f32,
    pub numPlayersUsing: i32,
}

impl TileEntityEnderChest {
    pub fn new(pos: BlockPos) -> Self {
        Self {
            pos,
            lidAngle: 0.0,
            prevLidAngle: 0.0,
            numPlayersUsing: 0,
        }
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
        if self.numPlayersUsing > 0 && self.lidAngle < 1.0 {
            self.lidAngle += 0.1;
        } else if self.numPlayersUsing == 0 && self.lidAngle > 0.0 {
            self.lidAngle -= 0.1;
        }
        self.lidAngle = self.lidAngle.clamp(0.0, 1.0);
    }
    pub fn interpolatedLidAngle(&self, partialTicks: f32) -> f32 {
        let angle =
            self.prevLidAngle + (self.lidAngle - self.prevLidAngle) * partialTicks.clamp(0.0, 1.0);
        let inverse = 1.0 - angle;
        1.0 - inverse * inverse * inverse
    }
}
