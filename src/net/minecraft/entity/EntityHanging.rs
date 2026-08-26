use crate::net::minecraft::entity::Entity::Entity;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumFacing::{Axis, EnumFacing};

/// Geometry-bearing subset of MCP 1.12.2 `EntityHanging`.
///
/// The server owns surface validation and break/drop behavior. The client still
/// needs the exact hanging anchor, facing, position, yaw and thin collision box
/// for rendering, selection and interpolation.
pub struct EntityHanging;

impl EntityHanging {
    pub const DEFAULT_WIDTH: f32 = 0.5;
    pub const DEFAULT_HEIGHT: f32 = 0.5;
    pub const WALL_OFFSET: f64 = 0.46875;

    pub fn updateFacingWithBoundingBox(
        entity: &mut Entity,
        hangingPosition: BlockPos,
        facingDirection: EnumFacing,
        widthPixels: i32,
        heightPixels: i32,
    ) {
        assert!(matches!(facingDirection.axis(), Axis::X | Axis::Z));
        entity.rotationYaw = facingDirection
            .horizontalIndex()
            .expect("horizontal facing") as f32
            * 90.0;
        entity.prevRotationYaw = entity.rotationYaw;
        Self::updateBoundingBox(
            entity,
            hangingPosition,
            facingDirection,
            widthPixels,
            heightPixels,
        );
    }

    pub fn updateBoundingBox(
        entity: &mut Entity,
        hangingPosition: BlockPos,
        facingDirection: EnumFacing,
        widthPixels: i32,
        heightPixels: i32,
    ) {
        let (frontX, _, frontZ) = facingDirection.offsets();
        let side = facingDirection.rotateYCCW();
        let (sideX, _, sideZ) = side.offsets();
        let horizontalOffset = Self::offs(widthPixels);
        let verticalOffset = Self::offs(heightPixels);

        let mut x = hangingPosition.x as f64 + 0.5;
        let mut y = hangingPosition.y as f64 + 0.5;
        let mut z = hangingPosition.z as f64 + 0.5;
        x -= frontX as f64 * Self::WALL_OFFSET;
        z -= frontZ as f64 * Self::WALL_OFFSET;
        y += verticalOffset;
        x += horizontalOffset * sideX as f64;
        z += horizontalOffset * sideZ as f64;

        entity.posX = x;
        entity.posY = y;
        entity.posZ = z;
        entity.prevPosX = x;
        entity.prevPosY = y;
        entity.prevPosZ = z;

        let mut halfX = widthPixels as f64;
        let halfY = heightPixels as f64;
        let mut halfZ = widthPixels as f64;
        if facingDirection.axis() == Axis::Z {
            halfZ = 1.0;
        } else {
            halfX = 1.0;
        }
        let halfX = halfX / 32.0;
        let halfY = halfY / 32.0;
        let halfZ = halfZ / 32.0;
        entity.boundingBox = AxisAlignedBB::new(
            x - halfX,
            y - halfY,
            z - halfZ,
            x + halfX,
            y + halfY,
            z + halfZ,
        );
    }

    const fn offs(pixels: i32) -> f64 {
        if pixels % 32 == 0 {
            0.5
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_block_north_painting_is_one_block_wide_and_one_sixteenth_deep() {
        let mut entity = Entity::default();
        EntityHanging::updateFacingWithBoundingBox(
            &mut entity,
            BlockPos::new(10, 64, 20),
            EnumFacing::North,
            16,
            16,
        );
        assert!((entity.posX - 10.5).abs() < 1.0e-9);
        assert!((entity.posY - 64.5).abs() < 1.0e-9);
        assert!((entity.posZ - 20.96875).abs() < 1.0e-9);
        assert!((entity.boundingBox.max_x - entity.boundingBox.min_x - 1.0).abs() < 1.0e-9);
        assert!((entity.boundingBox.max_z - entity.boundingBox.min_z - 0.0625).abs() < 1.0e-9);
        assert_eq!(entity.rotationYaw, 180.0);
    }

    #[test]
    fn even_two_block_width_uses_half_block_side_offset() {
        let mut entity = Entity::default();
        EntityHanging::updateFacingWithBoundingBox(
            &mut entity,
            BlockPos::new(0, 0, 0),
            EnumFacing::South,
            32,
            16,
        );
        // SOUTH.rotateYCCW() is EAST.
        assert!((entity.posX - 1.0).abs() < 1.0e-9);
        assert_eq!(entity.rotationYaw, 0.0);
    }
}
