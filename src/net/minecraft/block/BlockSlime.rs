use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::entity::Entity::Entity;

pub const BLOCK_ID: i32 = 165;
pub const SLIPPERINESS: f32 = 0.8;

pub const fn isBlockSlime(state: IBlockState) -> bool {
    state.getBlockId() == BLOCK_ID
}

/// MCP `BlockSlime#onLanded`. `livingBase` is the Rust equivalent of the
/// source `instanceof EntityLivingBase` branch.
pub fn onLanded(entity: &mut Entity, livingBase: bool) {
    if entity.sneaking {
        entity.motionY = 0.0;
    } else if entity.motionY < 0.0 {
        entity.motionY = -entity.motionY;
        if !livingBase {
            entity.motionY *= 0.8;
        }
    }
}

/// MCP `BlockSlime#onEntityWalk`.
pub fn onEntityWalk(entity: &mut Entity) {
    if entity.motionY.abs() < 0.1 && !entity.sneaking {
        let factor = 0.4 + entity.motionY.abs() * 0.2;
        entity.motionX *= factor;
        entity.motionZ *= factor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn living_entity_bounces_without_non_living_attenuation() {
        let mut entity = Entity::default();
        entity.motionY = -0.8;
        onLanded(&mut entity, true);
        assert!((entity.motionY - 0.8).abs() < 1.0e-9);
    }

    #[test]
    fn sneaking_entity_does_not_bounce() {
        let mut entity = Entity::default();
        entity.sneaking = true;
        entity.motionY = -0.8;
        onLanded(&mut entity, true);
        assert_eq!(entity.motionY, 0.0);
    }
}
