use std::cell::Cell;

use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::block::BlockPistonBase::BlockPistonBase;
use crate::net::minecraft::block::BlockPistonExtension::BlockPistonExtension;
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumFacing::EnumFacing;

thread_local! {
    /// MCP `TileEntityPiston.field_190613_i`. While an entity is being moved
    /// by a piston, that same moving shape is omitted from its collision query.
    static PISTON_PUSH_DIRECTION: Cell<Option<EnumFacing>> = const { Cell::new(None) };
}

/// Client movement/render port of MCP 1.12.2 `TileEntityPiston`.
#[derive(Debug, Clone, PartialEq)]
pub struct TileEntityPiston {
    pub pos: BlockPos,
    pub pistonState: IBlockState,
    pub pistonFacing: EnumFacing,
    pub progress: f32,
    pub lastProgress: f32,
    pub extending: bool,
    pub shouldHeadBeRendered: bool,
}

impl TileEntityPiston {
    pub fn fromNbt(tag: &NBTTagCompound) -> Option<Self> {
        let id = tag.getString("id");
        if !id.is_empty() && id != "minecraft:piston" && id != "Piston" {
            return None;
        }
        let facingIndex = tag.getInteger("facing").rem_euclid(6) as usize;
        let progress = tag.getFloat("progress").clamp(0.0, 1.0);
        Some(Self {
            pos: BlockPos::new(
                tag.getInteger("x"),
                tag.getInteger("y"),
                tag.getInteger("z"),
            ),
            pistonState: IBlockState::fromGlobalStateId(
                (tag.getInteger("blockId").clamp(0, 255) << 4) | (tag.getInteger("blockData") & 15),
            ),
            pistonFacing: EnumFacing::VALUES[facingIndex],
            progress,
            lastProgress: progress,
            extending: tag.getBoolean("extending"),
            shouldHeadBeRendered: tag.getBoolean("source"),
        })
    }

    /// Progress-only half of MCP `update`; WorldClient performs the entity
    /// sweep immediately before calling this, preserving vanilla tick order.
    pub fn update(&mut self) {
        self.lastProgress = self.progress;
        if self.lastProgress < 1.0 {
            self.progress = (self.progress + 0.5).min(1.0);
        }
    }

    pub fn nextProgress(&self) -> f32 {
        if self.progress >= 1.0 {
            1.0
        } else {
            (self.progress + 0.5).min(1.0)
        }
    }

    pub fn getProgress(&self, partialTicks: f32) -> f32 {
        let partial = partialTicks.min(1.0);
        self.lastProgress + (self.progress - self.lastProgress) * partial
    }

    pub fn getExtendedProgress(&self, progress: f32) -> f32 {
        if self.extending {
            progress - 1.0
        } else {
            1.0 - progress
        }
    }

    pub fn offset(&self, partialTicks: f32) -> [f32; 3] {
        let value = self.getExtendedProgress(self.getProgress(partialTicks));
        let (x, y, z) = self.pistonFacing.offsets();
        [x as f32 * value, y as f32 * value, z as f32 * value]
    }

    pub fn finished(&self) -> bool {
        self.progress >= 1.0 && self.lastProgress >= 1.0
    }

    pub fn movementDirection(&self) -> EnumFacing {
        if self.extending {
            self.pistonFacing
        } else {
            self.pistonFacing.opposite()
        }
    }

    pub fn isMovingSlimeBlock(&self) -> bool {
        self.pistonState.getBlockId() == 165
    }

    pub fn withPushDirection<T>(direction: EnumFacing, action: impl FnOnce() -> T) -> T {
        PISTON_PUSH_DIRECTION.with(|value| value.set(Some(direction)));
        let result = action();
        PISTON_PUSH_DIRECTION.with(|value| value.set(None));
        result
    }

    fn activePushDirection() -> Option<EnumFacing> {
        PISTON_PUSH_DIRECTION.with(Cell::get)
    }

    /// Rust equivalent of `TileEntityPiston#func_190609_a`, in block-local
    /// coordinates. The fixed shortened piston base is retained while a source
    /// piston retracts; the moving shape is suppressed only for an entity being
    /// moved in the exact same direction by this piston.
    pub fn collisionBoxesLocal(&self) -> Vec<AxisAlignedBB> {
        let mut boxes = Vec::new();
        if !self.extending && self.shouldHeadBeRendered {
            let metadata = self.pistonFacing.index() | 8;
            boxes.push(BlockPistonBase::getBoundingBox(
                IBlockState::fromGlobalStateId((self.pistonState.getBlockId() << 4) | metadata),
            ));
        }

        let push_direction = Self::activePushDirection();
        if self.progress >= 1.0 || push_direction != Some(self.movementDirection()) {
            let extended = self.getExtendedProgress(self.progress) as f64;
            let (x, y, z) = self.pistonFacing.offsets();
            let offset = (
                x as f64 * extended,
                y as f64 * extended,
                z as f64 * extended,
            );
            let queryBoxes = if self.shouldHeadBeRendered {
                let short = self.extending != (1.0 - self.progress < 0.25);
                BlockPistonExtension::collisionBoxesForFacing(self.pistonFacing, short)
            } else {
                self.movingShapeBoxes()
            };
            boxes.extend(
                queryBoxes
                    .into_iter()
                    .map(|bounds| bounds.offset(offset.0, offset.1, offset.2)),
            );
        }
        boxes
    }

    /// Swept union used by `moveCollidedEntities` to select candidate entities.
    pub fn sweptEntityBounds(&self, next_progress: f32) -> Option<AxisAlignedBB> {
        let boxes = self.movingShapeBoxes();
        if boxes.is_empty() {
            return None;
        }
        let mut union = AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        for bounds in &boxes {
            union = union.union(*bounds);
        }
        let current = self.toCurrentWorldBox(union);
        let delta = (next_progress - self.progress) as f64;
        Some(current.union(Self::sweepStrip(current, self.movementDirection(), delta)))
    }

    /// Maximum overlap distance among all collision components, corresponding
    /// to the inner loop of `TileEntityPiston#moveCollidedEntities`.
    pub fn primaryPushDistance(&self, entity_box: AxisAlignedBB, next_progress: f32) -> f64 {
        let delta = (next_progress - self.progress) as f64;
        if delta <= 0.0 {
            return 0.0;
        }
        let direction = self.movementDirection();
        let mut distance: f64 = 0.0;
        for bounds in self.movingShapeBoxes() {
            let swept = Self::sweepStrip(self.toCurrentWorldBox(bounds), direction, delta);
            if swept.intersects(entity_box) {
                distance = distance.max(Self::overlapDistance(swept, direction, entity_box));
                if distance >= delta {
                    break;
                }
            }
        }
        if distance > 0.0 {
            distance.min(delta) + 0.01
        } else {
            0.0
        }
    }

    /// Retraction-only correction from MCP `func_190605_a`. This prevents an
    /// entity from remaining embedded in the piston base when the moving head
    /// retracts through the source block.
    pub fn retractionCorrectionDistance(&self, entity_box: AxisAlignedBB, delta: f64) -> f64 {
        if self.extending || !self.shouldHeadBeRendered {
            return 0.0;
        }
        let full = AxisAlignedBB::from_block(self.pos);
        if !entity_box.intersects(full) {
            return 0.0;
        }
        let outward = self.movementDirection().opposite();
        let direct = Self::overlapDistance(full, outward, entity_box) + 0.01;
        let intersection = entity_box.intersection(full);
        let clipped = Self::overlapDistance(full, outward, intersection) + 0.01;
        if (direct - clipped).abs() < 0.01 {
            direct.min(delta) + 0.01
        } else {
            0.0
        }
    }

    fn movingShapeBoxes(&self) -> Vec<AxisAlignedBB> {
        if !self.extending && self.shouldHeadBeRendered {
            BlockPistonExtension::collisionBoxesForFacing(self.pistonFacing, false)
        } else {
            self.pistonState
                .getBlock()
                .getCollisionBoxes(self.pistonState)
        }
    }

    fn toCurrentWorldBox(&self, bounds: AxisAlignedBB) -> AxisAlignedBB {
        let extended = self.getExtendedProgress(self.progress) as f64;
        let (x, y, z) = self.pistonFacing.offsets();
        bounds.offset(
            self.pos.x as f64 + extended * x as f64,
            self.pos.y as f64 + extended * y as f64,
            self.pos.z as f64 + extended * z as f64,
        )
    }

    fn sweepStrip(bounds: AxisAlignedBB, direction: EnumFacing, delta: f64) -> AxisAlignedBB {
        let sign = match direction.axis_direction() {
            crate::net::minecraft::util::EnumFacing::AxisDirection::Positive => 1.0,
            crate::net::minecraft::util::EnumFacing::AxisDirection::Negative => -1.0,
        };
        let amount = delta * sign;
        let low = amount.min(0.0);
        let high = amount.max(0.0);
        match direction {
            EnumFacing::West => AxisAlignedBB::new(
                bounds.min_x + low,
                bounds.min_y,
                bounds.min_z,
                bounds.min_x + high,
                bounds.max_y,
                bounds.max_z,
            ),
            EnumFacing::East => AxisAlignedBB::new(
                bounds.max_x + low,
                bounds.min_y,
                bounds.min_z,
                bounds.max_x + high,
                bounds.max_y,
                bounds.max_z,
            ),
            EnumFacing::Down => AxisAlignedBB::new(
                bounds.min_x,
                bounds.min_y + low,
                bounds.min_z,
                bounds.max_x,
                bounds.min_y + high,
                bounds.max_z,
            ),
            EnumFacing::Up => AxisAlignedBB::new(
                bounds.min_x,
                bounds.max_y + low,
                bounds.min_z,
                bounds.max_x,
                bounds.max_y + high,
                bounds.max_z,
            ),
            EnumFacing::North => AxisAlignedBB::new(
                bounds.min_x,
                bounds.min_y,
                bounds.min_z + low,
                bounds.max_x,
                bounds.max_y,
                bounds.min_z + high,
            ),
            EnumFacing::South => AxisAlignedBB::new(
                bounds.min_x,
                bounds.min_y,
                bounds.max_z + low,
                bounds.max_x,
                bounds.max_y,
                bounds.max_z + high,
            ),
        }
    }

    fn overlapDistance(moving: AxisAlignedBB, direction: EnumFacing, entity: AxisAlignedBB) -> f64 {
        match direction {
            EnumFacing::East => moving.max_x - entity.min_x,
            EnumFacing::West => entity.max_x - moving.min_x,
            EnumFacing::Up => moving.max_y - entity.min_y,
            EnumFacing::Down => entity.max_y - moving.min_y,
            EnumFacing::South => moving.max_z - entity.min_z,
            EnumFacing::North => entity.max_z - moving.min_z,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_progress_matches_mcp_direction() {
        let piston = TileEntityPiston {
            pos: BlockPos::new(0, 0, 0),
            pistonState: IBlockState::fromGlobalStateId(1 << 4),
            pistonFacing: EnumFacing::East,
            progress: 0.5,
            lastProgress: 0.0,
            extending: true,
            shouldHeadBeRendered: false,
        };
        assert_eq!(piston.getExtendedProgress(0.5), -0.5);
        assert_eq!(piston.movementDirection(), EnumFacing::East);
    }

    #[test]
    fn piston_head_has_plate_and_arm() {
        assert_eq!(
            BlockPistonExtension::collisionBoxesForFacing(EnumFacing::Up, false).len(),
            2
        );
    }
}
