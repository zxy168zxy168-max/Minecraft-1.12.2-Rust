use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumFacing::{Axis, AxisDirection, EnumFacing};

/// Client animation state from MCP 1.12.2 `TileEntityShulkerBox`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationStatus {
    Closed,
    Opening,
    Opened,
    Closing,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileEntityShulkerBox {
    pub pos: BlockPos,
    animationStatus: AnimationStatus,
    progress: f32,
    progressOld: f32,
    colorMetadata: i32,
    numPlayersUsing: i32,
    pushEntitiesThisTick: bool,
}

impl TileEntityShulkerBox {
    pub fn new(pos: BlockPos, colorMetadata: i32) -> Self {
        Self {
            pos,
            animationStatus: AnimationStatus::Closed,
            progress: 0.0,
            progressOld: 0.0,
            colorMetadata: colorMetadata.clamp(0, 15),
            numPlayersUsing: 0,
            pushEntitiesThisTick: false,
        }
    }

    pub fn fromNbt(tag: &NBTTagCompound, colorMetadata: i32) -> Option<Self> {
        let id = tag.getString("id");
        if !id.is_empty() && id != "minecraft:shulker_box" && id != "ShulkerBox" {
            return None;
        }
        Some(Self::new(
            BlockPos::new(
                tag.getInteger("x"),
                tag.getInteger("y"),
                tag.getInteger("z"),
            ),
            colorMetadata,
        ))
    }

    /// MCP `TileEntityShulkerBox#receiveClientEvent` event 1.
    pub fn receiveClientEvent(&mut self, id: i32, eventType: i32) -> bool {
        if id != 1 {
            return false;
        }
        self.numPlayersUsing = eventType;
        if eventType == 0 {
            self.animationStatus = AnimationStatus::Closing;
        } else if eventType == 1 {
            self.animationStatus = AnimationStatus::Opening;
        }
        true
    }

    /// MCP `TileEntityShulkerBox#update` plus `func_190583_o`.
    ///
    /// `pushEntitiesThisTick` records the exact source call sites of
    /// `func_190589_G`: every non-final OPENING tick, the final OPENING tick,
    /// and every non-final CLOSING tick. `WorldClient` then applies the source
    /// displacement through ordinary entity collision movement after the tile
    /// entity update, matching `World#updateEntities` ordering.
    pub fn update(&mut self) {
        self.pushEntitiesThisTick = false;
        self.progressOld = self.progress;
        match self.animationStatus {
            AnimationStatus::Closed => self.progress = 0.0,
            AnimationStatus::Opening => {
                self.progress += 0.1;
                if self.progress >= 1.0 {
                    // MCP calls func_190589_G before switching to OPENED.
                    self.pushEntitiesThisTick = true;
                    self.progress = 1.0;
                    self.animationStatus = AnimationStatus::Opened;
                }
            }
            AnimationStatus::Closing => {
                self.progress -= 0.1;
                if self.progress <= 0.0 {
                    self.progress = 0.0;
                    self.animationStatus = AnimationStatus::Closed;
                }
            }
            AnimationStatus::Opened => self.progress = 1.0,
        }

        if matches!(
            self.animationStatus,
            AnimationStatus::Opening | AnimationStatus::Closing
        ) {
            self.pushEntitiesThisTick = true;
        }
    }

    pub fn interpolatedProgress(&self, partialTicks: f32) -> f32 {
        self.progressOld + (self.progress - self.progressOld) * partialTicks.clamp(0.0, 1.0)
    }

    pub const fn colorMetadata(&self) -> i32 {
        self.colorMetadata
    }

    pub const fn animationStatus(&self) -> AnimationStatus {
        self.animationStatus
    }

    pub const fn pushesEntitiesThisTick(&self) -> bool {
        self.pushEntitiesThisTick
    }

    /// MCP `TileEntityShulkerBox#func_190587_b`.
    pub fn collisionBoundingBox(&self, facing: EnumFacing) -> AxisAlignedBB {
        let (x, y, z) = facing.offsets();
        let extension = 0.5 * self.progress as f64;
        AxisAlignedBB::from_block(BlockPos::new(0, 0, 0)).add_coord(
            extension * x as f64,
            extension * y as f64,
            extension * z as f64,
        )
    }

    /// MCP `TileEntityShulkerBox#func_190588_c`, already offset to world
    /// coordinates. This is only the newly occupied lid sweep, not the full
    /// expanded box.
    pub fn sweptPushBox(&self, facing: EnumFacing) -> AxisAlignedBB {
        let (x, y, z) = facing.opposite().offsets();
        self.collisionBoundingBox(facing)
            .contract_directional(x as f64, y as f64, z as f64)
            .offset(self.pos.x as f64, self.pos.y as f64, self.pos.z as f64)
    }

    /// Exact displacement calculation from MCP `func_190589_G` for one
    /// ordinary (non-IGNORE push reaction) entity bounding box.
    pub fn pushDisplacement(
        &self,
        facing: EnumFacing,
        entityBounds: AxisAlignedBB,
    ) -> Option<[f64; 3]> {
        if !self.pushEntitiesThisTick {
            return None;
        }
        let sweep = self.sweptPushBox(facing);
        if !sweep.intersects(entityBounds) {
            return None;
        }

        let mut displacement = [0.0_f64; 3];
        match facing.axis() {
            Axis::X => {
                displacement[0] = if facing.axis_direction() == AxisDirection::Positive {
                    sweep.max_x - entityBounds.min_x
                } else {
                    entityBounds.max_x - sweep.min_x
                } + 0.01;
            }
            Axis::Y => {
                displacement[1] = if facing.axis_direction() == AxisDirection::Positive {
                    sweep.max_y - entityBounds.min_y
                } else {
                    entityBounds.max_y - sweep.min_y
                } + 0.01;
            }
            Axis::Z => {
                displacement[2] = if facing.axis_direction() == AxisDirection::Positive {
                    sweep.max_z - entityBounds.min_z
                } else {
                    entityBounds.max_z - sweep.min_z
                } + 0.01;
            }
        }

        let (x, y, z) = facing.offsets();
        Some([
            displacement[0] * x as f64,
            displacement[1] * y as f64,
            displacement[2] * z as f64,
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_event_drives_the_vanilla_ten_tick_open_close_curve() {
        let mut tile = TileEntityShulkerBox::new(BlockPos::new(1, 2, 3), 10);
        assert!(tile.receiveClientEvent(1, 1));
        for _ in 0..10 {
            tile.update();
        }
        assert_eq!(tile.animationStatus(), AnimationStatus::Opened);
        assert!((tile.interpolatedProgress(1.0) - 1.0).abs() < f32::EPSILON);
        assert!(tile.receiveClientEvent(1, 0));
        for _ in 0..10 {
            tile.update();
        }
        assert_eq!(tile.animationStatus(), AnimationStatus::Closed);
        assert!(tile.interpolatedProgress(1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn opening_up_pushes_an_entity_standing_on_the_lid_by_source_overlap() {
        let mut tile = TileEntityShulkerBox::new(BlockPos::new(4, 10, -2), 0);
        assert!(tile.receiveClientEvent(1, 1));
        tile.update();
        let player = AxisAlignedBB::new(4.2, 11.0, -1.8, 4.8, 12.8, -1.2);
        let displacement = tile.pushDisplacement(EnumFacing::Up, player).unwrap();
        // First lid step reaches y=11.05; MCP adds 0.01 after overlap.
        assert!((displacement[1] - 0.06).abs() < 1.0e-9);
        assert_eq!(displacement[0], 0.0);
        assert_eq!(displacement[2], 0.0);
    }

    #[test]
    fn final_closing_tick_does_not_run_the_push_sweep() {
        let mut tile = TileEntityShulkerBox::new(BlockPos::new(0, 0, 0), 0);
        tile.receiveClientEvent(1, 1);
        for _ in 0..10 {
            tile.update();
        }
        tile.receiveClientEvent(1, 0);
        for _ in 0..9 {
            tile.update();
        }
        assert!(tile.pushesEntitiesThisTick());
        tile.update();
        assert_eq!(tile.animationStatus(), AnimationStatus::Closed);
        assert!(!tile.pushesEntitiesThisTick());
    }
}
