use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::entity::Entity::Entity;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;

pub const BLOCK_ID: i32 = 30;

pub const fn isBlockWeb(state: IBlockState) -> bool {
    state.getBlockId() == BLOCK_ID
}

pub const FULL_BLOCK_AABB: AxisAlignedBB = AxisAlignedBB {
    min_x: 0.0,
    min_y: 0.0,
    min_z: 0.0,
    max_x: 1.0,
    max_y: 1.0,
    max_z: 1.0,
};

/// MCP `BlockWeb#getCollisionBoundingBox` returns `NULL_AABB`, while the
/// inherited selected bounding box remains the full block cube.
pub const fn getBoundingBox() -> AxisAlignedBB {
    FULL_BLOCK_AABB
}

pub fn onEntityCollidedWithBlock(entity: &mut Entity) {
    entity.setInWeb();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_has_selection_box_but_no_collision_box_contract() {
        let bounds = getBoundingBox();
        assert_eq!((bounds.min_x, bounds.min_y, bounds.min_z), (0.0, 0.0, 0.0));
        assert_eq!((bounds.max_x, bounds.max_y, bounds.max_z), (1.0, 1.0, 1.0));
    }
}
