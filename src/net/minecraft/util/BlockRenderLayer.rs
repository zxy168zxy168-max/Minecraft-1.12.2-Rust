/// MCP 1.12.2 `net.minecraft.util.BlockRenderLayer`.
///
/// The layer belongs to the block implementation rather than the baked model.
/// Until the complete `Block` registry hierarchy is ported, `forBlockId` is a
/// source-traceable bridge built from the vanilla block classes that override
/// `Block#getBlockLayer()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BlockRenderLayer {
    Solid,
    CutoutMipped,
    Cutout,
    Translucent,
}

impl BlockRenderLayer {
    pub const VALUES: [Self; 4] = [
        Self::Solid,
        Self::CutoutMipped,
        Self::Cutout,
        Self::Translucent,
    ];

    pub const fn index(self) -> usize {
        match self {
            Self::Solid => 0,
            Self::CutoutMipped => 1,
            Self::Cutout => 2,
            Self::Translucent => 3,
        }
    }

    /// Source bridge for protocol-340 numerical block IDs.
    ///
    /// This follows the corresponding MCP block classes:
    /// `Block` (SOLID), `BlockGrass`, `BlockLeaves`, `BlockPane`,
    /// `BlockHopper`, `BlockTripWireHook`, `BlockBush`, `BlockGlass`,
    /// `BlockLiquid`, `BlockIce`, `BlockStainedGlass`,
    /// `BlockStainedGlassPane`, `BlockSlime`, and the other explicit CUTOUT
    /// overrides. It must be replaced by virtual `Block#getBlockLayer()` once
    /// the block registry is ported; callers must not infer a layer from model
    /// shape or texture alpha.
    pub fn forBlockId(blockId: i32, fancyGraphics: bool) -> Self {
        match blockId {
            // BlockGrass
            2 => Self::CutoutMipped,

            // BlockLeaves: fancy => CUTOUT_MIPPED, fast => SOLID.
            18 | 161 if fancyGraphics => Self::CutoutMipped,

            // BlockPane, BlockHopper, BlockTripWireHook.
            101 | 102 | 131 | 154 => Self::CutoutMipped,

            // BlockLiquid (water), BlockPortal, BlockIce/FrostedIce,
            // BlockStainedGlass/Pane, BlockTripWire and BlockSlime.
            8 | 9 | 79 | 90 | 95 | 132 | 160 | 165 | 212 => Self::Translucent,

            // BlockRailBase.
            27 | 28 | 66 | 157 => Self::Cutout,

            // BlockBush and direct subclasses used by the 1.12.2 registry.
            6 | 31 | 32 | 37 | 38 | 39 | 40 | 59 | 83 | 104 | 105 | 111 | 115 | 141 | 142 | 175
            | 207 => Self::Cutout,

            // Other vanilla classes with explicit CUTOUT getBlockLayer().
            // This list is the protocol-340 registry mapping of the MCP classes
            // enumerated above; blocks that merely have partial geometry remain
            // SOLID unless their class actually overrides getBlockLayer().
            20 | 26 | 30 | 50 | 51 | 52 | 55 | 64 | 65 | 71 | 75 | 76 | 81 | 92 | 93 | 94 | 96
            | 106 | 117 | 127 | 138 | 140 | 149 | 150 | 167 | 193 | 194 | 195 | 196 | 197 | 198
            | 199 | 200 => Self::Cutout,

            // Block#getBlockLayer default, including lava.
            _ => Self::Solid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_layer_overrides_remain_distinct() {
        assert_eq!(
            BlockRenderLayer::forBlockId(1, true),
            BlockRenderLayer::Solid
        );
        assert_eq!(
            BlockRenderLayer::forBlockId(2, true),
            BlockRenderLayer::CutoutMipped
        );
        assert_eq!(
            BlockRenderLayer::forBlockId(20, true),
            BlockRenderLayer::Cutout
        );
        assert_eq!(
            BlockRenderLayer::forBlockId(26, true),
            BlockRenderLayer::Cutout
        );
        assert_eq!(
            BlockRenderLayer::forBlockId(78, true),
            BlockRenderLayer::Solid
        );
        assert_eq!(
            BlockRenderLayer::forBlockId(143, true),
            BlockRenderLayer::Solid
        );
        assert_eq!(
            BlockRenderLayer::forBlockId(95, true),
            BlockRenderLayer::Translucent
        );
        assert_eq!(
            BlockRenderLayer::forBlockId(111, true),
            BlockRenderLayer::Cutout
        );
    }

    #[test]
    fn leaves_follow_fancy_graphics_like_block_leaves() {
        assert_eq!(
            BlockRenderLayer::forBlockId(18, true),
            BlockRenderLayer::CutoutMipped
        );
        assert_eq!(
            BlockRenderLayer::forBlockId(18, false),
            BlockRenderLayer::Solid
        );
    }
}
