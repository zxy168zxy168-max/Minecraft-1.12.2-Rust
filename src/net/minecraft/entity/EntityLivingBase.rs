use crate::net::minecraft::block::{BlockLadder, BlockTrapDoor, BlockVine};
use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::entity::Entity::Entity;
use crate::net::minecraft::util::math::BlockPos::BlockPos;

pub const LADDER_HORIZONTAL_LIMIT: f64 = 0.15000000596046448;
pub const LIQUID_JUMP_MOTION: f64 = 0.03999999910593033;
pub const LIQUID_WALL_EXIT_MOTION: f64 = 0.30000001192092896;

/// Direct port of MCP `EntityLivingBase#isOnLadder` and its private aligned
/// trapdoor bridge. Concrete player classes supply spectator state because the
/// base Java method checks `instanceof EntityPlayer`.
pub fn isOnLadder(world: &WorldClient, entity: &Entity, spectator: bool) -> bool {
    if spectator {
        return false;
    }
    let pos = BlockPos::new(
        entity.posX.floor() as i32,
        entity.boundingBox.min_y.floor() as i32,
        entity.posZ.floor() as i32,
    );
    let state = world.getBlockState(pos);
    if BlockLadder::isBlockLadder(state) || BlockVine::isBlockVine(state) {
        return true;
    }
    if BlockTrapDoor::isBlockTrapDoor(state) && BlockTrapDoor::isOpen(state) {
        let below = world.getBlockState(pos.down(1));
        return BlockLadder::isBlockLadder(below)
            && BlockLadder::facing(below) == BlockTrapDoor::facing(state);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::block::state::IBlockState::IBlockState;

    #[test]
    fn vine_at_feet_is_climbable_for_non_spectator() {
        let mut world = WorldClient::new(0);
        world
            .invalidateRegionAndSetBlock(
                BlockPos::new(0, 64, 0),
                IBlockState::fromGlobalStateId(106 << 4),
            )
            .unwrap();
        let mut entity = Entity::default();
        entity.setPosition(0.5, 64.0, 0.5);
        assert!(isOnLadder(&world, &entity, false));
        assert!(!isOnLadder(&world, &entity, true));
    }
}
