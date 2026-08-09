use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::block::BlockLiquid::LiquidMaterial;
use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::util::math::BlockPos::BlockPos;

pub const BUCKET: i16 = 325;
pub const WATER_BUCKET: i16 = 326;
pub const LAVA_BUCKET: i16 = 327;

/// Client-side fill prediction of MCP 1.12.2 `ItemBucket#onItemRightClick`
/// for the empty bucket. World mutation and the resulting held item remain
/// authoritative on the server (`fillBucket`/`setBlockState` run there);
/// the client plays the fill sound and removes the source block locally.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BucketFill {
    /// Filled bucket item id (`WATER_BUCKET` / `LAVA_BUCKET`).
    pub bucket: i16,
    /// `SoundEvents.ITEM_BUCKET_FILL` / `ITEM_BUCKET_FILL_LAVA` name.
    pub sound: &'static str,
    /// The liquid source the client removes (prediction, server overwrites).
    pub source: BlockPos,
}

pub struct ItemBucket;

impl ItemBucket {
    pub const fn isBucket(stack: &ItemStack) -> bool {
        matches!(stack.itemId, BUCKET | WATER_BUCKET | LAVA_BUCKET)
    }

    /// `ItemBucket#onItemRightClick` fill branch: a level-0 water or lava
    /// block under the ray trace fills the matching bucket.
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

    /// `ItemBucket#tryPlaceContainedLiquid` client check: the destination
    /// block (`pos.offset(side)` unless the target itself is replaceable and
    /// the side is UP) must be air, a non-solid material, or a replaceable
    /// block so the liquid can be placed. Returns the destination and the
    /// empty sound the client plays there (vanilla plays it at `posIn` with
    /// `SoundCategory.BLOCKS`, 1.0 / 1.0).
    pub fn predictEmpty(
        world: &WorldClient,
        pos: BlockPos,
        sideHit: EnumFacing,
        bucket: i16,
    ) -> Option<BucketEmpty> {
        let target = world.getBlockState(pos);
        // MCP `onItemRightClick` flag1: the hit block's isReplaceable decides
        // whether the destination is the hit block itself (side UP) or the
        // neighbouring block.
        let destination = if isReplaceable(target) && sideHit == EnumFacing::Up {
            pos
        } else {
            pos.offset(sideHit, 1)
        };
        let dest = world.getBlockState(destination);
        // MCP `tryPlaceContainedLiquid`: `!isAirBlock && !flag && !flag1`
        // fails, with flag = `!material.isSolid()` and flag1 = the
        // destination block's isReplaceable.
        let placeable = dest.getBlockId() == 0
            || !dest.getBlock().materialIsSolid()
            || isReplaceable(dest);
        if placeable {
            Some(BucketEmpty {
                destination,
                sound: if bucket == LAVA_BUCKET {
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

/// Client-side result of MCP `ItemBucket#onItemRightClick` full-bucket
/// branch: where the liquid lands and the empty sound played there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BucketEmpty {
    /// Destination block (`blockpos1`) the liquid is placed at.
    pub destination: BlockPos,
    /// `SoundEvents.ITEM_BUCKET_EMPTY` / `ITEM_BUCKET_EMPTY_LAVA` name.
    pub sound: &'static str,
}

/// MCP 1.12.2 `Block#isReplaceable` overrides: BlockAir, BlockTallGrass,
/// BlockDeadBush, BlockSnow, BlockVine and BlockDoublePlant return true.
pub const fn isReplaceable(state: IBlockState) -> bool {
    matches!(state.getBlockId(), 0 | 31 | 32 | 78 | 106 | 175)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::util::math::BlockPos::BlockPos;

    fn state(id: i32) -> IBlockState { IBlockState::fromGlobalStateId(id) }

    #[test]
    fn empty_bucket_fills_still_water_and_lava() {
        let mut world = WorldClient::new(0);
        let water = BlockPos::new(0, 60, 0);
        world.invalidateRegionAndSetBlock(water, state(8 << 4)).unwrap();
        let fill = ItemBucket::predictFill(Some((water, world.getBlockState(water)))).unwrap();
        assert_eq!(fill.bucket, WATER_BUCKET);
        assert_eq!(fill.sound, "item.bucket.fill");
        assert_eq!(fill.source, water);

        let lava = BlockPos::new(5, 60, 0);
        world.invalidateRegionAndSetBlock(lava, state(10 << 4)).unwrap();
        let fill = ItemBucket::predictFill(Some((lava, world.getBlockState(lava)))).unwrap();
        assert_eq!(fill.bucket, LAVA_BUCKET);
        assert_eq!(fill.sound, "item.bucket.fill_lava");
    }

    #[test]
    fn empty_bucket_refuses_flowing_liquid_and_solid_blocks() {
        let mut world = WorldClient::new(0);
        let flowing = BlockPos::new(0, 60, 0);
        // Flowing water block id 8 with level 3 metadata.
        world.invalidateRegionAndSetBlock(flowing, IBlockState::fromGlobalStateId(8 << 4 | 3)).unwrap();
        assert!(ItemBucket::predictFill(Some((flowing, world.getBlockState(flowing)))).is_none());
        let stone = BlockPos::new(5, 60, 0);
        world.invalidateRegionAndSetBlock(stone, state(1 << 4)).unwrap();
        assert!(ItemBucket::predictFill(Some((stone, world.getBlockState(stone)))).is_none());
    }

    #[test]
    fn empty_bucket_places_only_into_air_non_solid_or_replaceable_destination() {
        let mut world = WorldClient::new(0);
        let solid = BlockPos::new(0, 60, 0);
        world.invalidateRegionAndSetBlock(solid, state(1 << 4)).unwrap();
        // Side hit on solid: destination = pos.up, which is air -> placeable.
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
        // Target itself air + UP side: destination = the air block itself.
        let air = BlockPos::new(0, 61, 0);
        assert_eq!(
            ItemBucket::predictEmpty(&world, air, EnumFacing::Up, WATER_BUCKET)
                .unwrap()
                .destination,
            air,
        );
        // Destination occupied by a solid: air target with DOWN side hits the
        // block below as destination -> not placeable.
        let below = BlockPos::new(0, 60, 0);
        world.invalidateRegionAndSetBlock(below, state(1 << 4)).unwrap();
        assert!(ItemBucket::predictEmpty(&world, air, EnumFacing::Down, WATER_BUCKET).is_none());
    }

    #[test]
    fn empty_bucket_places_into_replaceable_target_and_non_solid_material() {
        let mut world = WorldClient::new(0);
        // Tall grass (id 31) is replaceable: with UP side the destination is
        // the grass cell itself, which is non-solid -> placeable.
        let grass = BlockPos::new(0, 60, 0);
        world.invalidateRegionAndSetBlock(grass, state(31 << 4)).unwrap();
        let empty = ItemBucket::predictEmpty(&world, grass, EnumFacing::Up, WATER_BUCKET)
            .expect("replaceable target with UP side places into itself");
        assert_eq!(empty.destination, grass);
        // Grass under a solid: DOWN side hits the solid below -> not placeable.
        let below = BlockPos::new(0, 59, 0);
        world.invalidateRegionAndSetBlock(below, state(1 << 4)).unwrap();
        assert!(ItemBucket::predictEmpty(&world, grass, EnumFacing::Down, WATER_BUCKET).is_none());
        // A non-solid destination material (e.g. torch, id 50, material
        // CIRCUITS) is placeable even though it is neither air nor
        // replaceable: solid target with UP side, torch occupying pos.up.
        let base = BlockPos::new(5, 60, 0);
        world.invalidateRegionAndSetBlock(base, state(1 << 4)).unwrap();
        let torch = base.up(1);
        world.invalidateRegionAndSetBlock(torch, state(50 << 4)).unwrap();
        let empty = ItemBucket::predictEmpty(&world, base, EnumFacing::Up, WATER_BUCKET)
            .expect("non-solid destination material accepts the liquid");
        assert_eq!(empty.destination, torch);
    }
}
