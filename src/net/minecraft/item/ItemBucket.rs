use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::block::BlockLiquid::LiquidMaterial;
use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumFacing::EnumFacing;

pub const BUCKET: i16 = 325;
pub const WATER_BUCKET: i16 = 326;
pub const LAVA_BUCKET: i16 = 327;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BucketFill {
    pub bucket: i16,
    pub sound: &'static str,
    pub source: BlockPos,
}

pub struct ItemBucket;

impl ItemBucket {
    pub const fn isBucket(stack: &ItemStack) -> bool {
        matches!(stack.itemId, BUCKET | WATER_BUCKET | LAVA_BUCKET)
    }

    pub fn predictFill(target: Option<(BlockPos, IBlockState)>) -> Option<BucketFill> {
        let (pos, state) = target?;
        if state.getMetadata() & 15 != 0 {
            return None;
        }
        match LiquidMaterial::fromState(state) {
            Some(LiquidMaterial::Water) => Some(BucketFill {
                bucket: WATER_BUCKET,
                sound: "item.bucket.fill",
                source: pos,
            }),
            Some(LiquidMaterial::Lava) => Some(BucketFill {
                bucket: LAVA_BUCKET,
                sound: "item.bucket.fill_lava",
                source: pos,
            }),
            None => None,
        }
    }

    pub fn predictEmpty(
        world: &WorldClient,
        pos: BlockPos,
        sideHit: EnumFacing,
        bucket: i16,
    ) -> Option<BucketEmpty> {
        let target = world.getBlockState(pos);
        let destination = if isReplaceable(world, pos, target) && sideHit == EnumFacing::Up {
            pos
        } else {
            pos.offset(sideHit, 1)
        };
        let dest = world.getBlockState(destination);
        let placeable = dest.getBlockId() == 0
            || !dest.getBlock().materialIsSolid()
            || isReplaceable(world, destination, dest);
        if placeable {
            let vaporizesWater = bucket == WATER_BUCKET && world.getProvider().doesWaterVaporize();
            Some(BucketEmpty {
                destination,
                vaporizesWater,
                sound: if vaporizesWater {
                    "block.fire.extinguish"
                } else if bucket == LAVA_BUCKET {
                    "item.bucket.empty_lava"
                } else {
                    "item.bucket.empty"
                },
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BucketEmpty {
    pub destination: BlockPos,
    /// MCP `WorldProvider#doesWaterVaporize` + FLOWING_WATER branch.
    /// In the Nether the bucket succeeds but no water block is placed.
    pub vaporizesWater: bool,
    pub sound: &'static str,
}

/// MCP 1.12.2 `Block#isReplaceable` overrides used by `ItemBucket`.
/// `BlockSnow` is replaceable only at one layer; `BlockDoublePlant` only for
/// GRASS/FERN and must resolve the lower half's VARIANT through actual state.
pub fn isReplaceable(world: &WorldClient, pos: BlockPos, state: IBlockState) -> bool {
    match state.getBlockId() {
        0 | 31 | 32 | 106 => true,
        78 => state.getMetadata() & 7 == 0,
        175 => {
            let lower = if state.getMetadata() & 8 != 0 {
                world.getBlockState(pos.down(1))
            } else {
                state
            };
            lower.getBlockId() == 175 && matches!(lower.getMetadata() & 7, 2 | 3)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(id: i32) -> IBlockState {
        IBlockState::fromGlobalStateId(id)
    }

    #[test]
    fn empty_bucket_fills_still_water_and_lava() {
        let mut world = WorldClient::new(0);
        let water = BlockPos::new(0, 60, 0);
        world
            .invalidateRegionAndSetBlock(water, state(8 << 4))
            .unwrap();
        let fill = ItemBucket::predictFill(Some((water, world.getBlockState(water)))).unwrap();
        assert_eq!(fill.bucket, WATER_BUCKET);
        assert_eq!(fill.sound, "item.bucket.fill");
        assert_eq!(fill.source, water);

        let lava = BlockPos::new(5, 60, 0);
        world
            .invalidateRegionAndSetBlock(lava, state(10 << 4))
            .unwrap();
        let fill = ItemBucket::predictFill(Some((lava, world.getBlockState(lava)))).unwrap();
        assert_eq!(fill.bucket, LAVA_BUCKET);
        assert_eq!(fill.sound, "item.bucket.fill_lava");
    }

    #[test]
    fn empty_bucket_refuses_flowing_liquid_and_solid_blocks() {
        let mut world = WorldClient::new(0);
        let flowing = BlockPos::new(0, 60, 0);
        world
            .invalidateRegionAndSetBlock(flowing, IBlockState::fromGlobalStateId(8 << 4 | 3))
            .unwrap();
        assert!(ItemBucket::predictFill(Some((flowing, world.getBlockState(flowing)))).is_none());
        let stone = BlockPos::new(5, 60, 0);
        world
            .invalidateRegionAndSetBlock(stone, state(1 << 4))
            .unwrap();
        assert!(ItemBucket::predictFill(Some((stone, world.getBlockState(stone)))).is_none());
    }

    #[test]
    fn empty_bucket_places_only_into_air_non_solid_or_replaceable_destination() {
        let mut world = WorldClient::new(0);
        let solid = BlockPos::new(0, 60, 0);
        world
            .invalidateRegionAndSetBlock(solid, state(1 << 4))
            .unwrap();
        let empty = ItemBucket::predictEmpty(&world, solid, EnumFacing::Up, WATER_BUCKET)
            .expect("solid target with UP side places into the air above");
        assert_eq!(empty.destination, solid.up(1));
        assert_eq!(empty.sound, "item.bucket.empty");
        assert_eq!(
            ItemBucket::predictEmpty(&world, solid, EnumFacing::Up, LAVA_BUCKET)
                .unwrap()
                .sound,
            "item.bucket.empty_lava",
        );
        let air = BlockPos::new(0, 61, 0);
        assert_eq!(
            ItemBucket::predictEmpty(&world, air, EnumFacing::Up, WATER_BUCKET)
                .unwrap()
                .destination,
            air,
        );
        let below = BlockPos::new(0, 60, 0);
        world
            .invalidateRegionAndSetBlock(below, state(1 << 4))
            .unwrap();
        assert!(ItemBucket::predictEmpty(&world, air, EnumFacing::Down, WATER_BUCKET).is_none());
    }

    #[test]
    fn empty_bucket_places_into_replaceable_target_and_non_solid_material() {
        let mut world = WorldClient::new(0);
        let grass = BlockPos::new(0, 60, 0);
        world
            .invalidateRegionAndSetBlock(grass, state(31 << 4))
            .unwrap();
        let empty = ItemBucket::predictEmpty(&world, grass, EnumFacing::Up, WATER_BUCKET)
            .expect("replaceable target with UP side places into itself");
        assert_eq!(empty.destination, grass);
        let below = BlockPos::new(0, 59, 0);
        world
            .invalidateRegionAndSetBlock(below, state(1 << 4))
            .unwrap();
        assert!(ItemBucket::predictEmpty(&world, grass, EnumFacing::Down, WATER_BUCKET).is_none());
        let base = BlockPos::new(5, 60, 0);
        world
            .invalidateRegionAndSetBlock(base, state(1 << 4))
            .unwrap();
        let torch = base.up(1);
        world
            .invalidateRegionAndSetBlock(torch, state(50 << 4))
            .unwrap();
        let empty = ItemBucket::predictEmpty(&world, base, EnumFacing::Up, WATER_BUCKET)
            .expect("non-solid destination material accepts the liquid");
        assert_eq!(empty.destination, torch);
    }
    #[test]
    fn nether_water_bucket_vaporizes_instead_of_placing_water() {
        let world = WorldClient::new(-1);
        let base = BlockPos::new(0, 60, 0);
        let empty = ItemBucket::predictEmpty(&world, base, EnumFacing::Up, WATER_BUCKET)
            .expect("air in the Nether accepts the bucket action");
        assert!(empty.vaporizesWater);
        assert_eq!(empty.sound, "block.fire.extinguish");

        let overworld = WorldClient::new(0);
        let normal = ItemBucket::predictEmpty(&overworld, base, EnumFacing::Up, WATER_BUCKET)
            .expect("air in the Overworld accepts water");
        assert!(!normal.vaporizesWater);
        assert_eq!(normal.sound, "item.bucket.empty");
    }

    #[test]
    fn replaceability_matches_snow_layers_and_double_plant_actual_variant() {
        let mut world = WorldClient::new(0);
        let snow = BlockPos::new(0, 60, 0);
        world
            .invalidateRegionAndSetBlock(snow, IBlockState::fromGlobalStateId(78 << 4))
            .unwrap();
        assert!(isReplaceable(&world, snow, world.getBlockState(snow)));
        world
            .invalidateRegionAndSetBlock(snow, IBlockState::fromGlobalStateId((78 << 4) | 1))
            .unwrap();
        assert!(!isReplaceable(&world, snow, world.getBlockState(snow)));

        let lower = BlockPos::new(4, 60, 0);
        let upper = lower.up(1);
        world
            .invalidateRegionAndSetBlock(lower, IBlockState::fromGlobalStateId((175 << 4) | 2))
            .unwrap();
        world
            .invalidateRegionAndSetBlock(upper, IBlockState::fromGlobalStateId((175 << 4) | 8))
            .unwrap();
        assert!(isReplaceable(&world, lower, world.getBlockState(lower)));
        assert!(isReplaceable(&world, upper, world.getBlockState(upper)));

        world
            .invalidateRegionAndSetBlock(lower, IBlockState::fromGlobalStateId((175 << 4) | 0))
            .unwrap();
        assert!(!isReplaceable(&world, lower, world.getBlockState(lower)));
        assert!(!isReplaceable(&world, upper, world.getBlockState(upper)));
    }
}
