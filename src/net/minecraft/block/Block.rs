use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::block::BlockMiningData::{BLOCK_HARDNESS, TOOL_NOT_REQUIRED};
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// Registry identity used by protocol 340. MCP `Block.registerBlocks` stores
/// block states as `(block registry id << 4) | metadata`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Block {
    registryId: u16,
    registryName: &'static str,
}

impl Block {
    pub const fn new(registryId: u16, registryName: &'static str) -> Self {
        Self {
            registryId,
            registryName,
        }
    }
    pub const fn getIdFromBlock(block: Block) -> i32 {
        block.registryId as i32
    }
    pub fn getBlockById(id: i32) -> Block {
        let id = id.clamp(0, 255) as usize;
        match BLOCK_NAMES[id] {
            Some(name) => Block::new(id as u16, name),
            None => AIR,
        }
    }
    pub fn getBlockFromName(name: &str) -> Option<Block> {
        let location = ResourceLocation::parse(name);
        if location.getNamespace() == "minecraft" {
            if let Some(block) = BLOCK_NAMES.iter().enumerate().find_map(|(id, candidate)| {
                candidate
                    .filter(|candidate| *candidate == location.getPath())
                    .map(|candidate| Block::new(id as u16, candidate))
            }) {
                return Some(block);
            }
        }
        // MCP `Block#getBlockFromName`: after the registry-name lookup fails,
        // legacy numeric strings are parsed as registry IDs. Adventure-mode
        // CanDestroy/CanPlaceOn NBT still accepts this form in 1.12.2.
        let id = name.parse::<usize>().ok()?;
        let registryName = BLOCK_NAMES.get(id).and_then(|entry| *entry)?;
        Some(Block::new(id as u16, registryName))
    }
    pub fn getStateById(id: i32) -> IBlockState {
        IBlockState::fromGlobalStateId(id)
    }
    pub const fn getStateId(state: IBlockState) -> i32 {
        state.getGlobalStateId()
    }
    pub fn getRegistryName(self) -> ResourceLocation {
        ResourceLocation::new("minecraft", self.registryName)
    }
    pub const fn getRegistryPath(self) -> &'static str {
        self.registryName
    }
    pub const fn isAir(self) -> bool {
        self.registryId == 0
    }

    /// Registry bridge for MCP `IBlockState#getMaterial().isSolid()`.  The
    /// default Material implementation is solid; only blocks constructed from
    /// MaterialTransparent, MaterialLiquid, MaterialLogic, MaterialPortal or
    /// the circuit-style non-solid materials are listed here.
    pub const fn materialIsSolid(self) -> bool {
        !matches!(
            self.registryId as i32,
            0 | 6 | 8..=11 | 27 | 28 | 31 | 32 | 37..=40 | 50 | 51 | 55 | 59
                | 65 | 66 | 69 | 75..=78 | 83 | 90 | 93 | 94 | 104..=106 | 111
                | 115 | 119 | 127 | 131 | 132 | 140..=144 | 149 | 150 | 157
                | 171 | 175 | 198 | 207 | 209 | 217
        )
    }

    /// MCP `Material#blocksMovement` for the vanilla material identity behind
    /// this registered block. `Material.WEB` is the one 1.12.2 material whose
    /// default solidity does not imply movement blocking.
    pub const fn materialBlocksMovement(self) -> bool {
        self.materialIsSolid() && self.registryId != 30
    }

    /// Source-owned client prediction for 1.12.2 blocks whose
    /// `onBlockActivated` returns true on the remote client without requiring
    /// unsynchronised TileEntity contents or permission state. This prevents a
    /// held ItemBlock from being treated as the successful branch when vanilla
    /// would consume the click by opening or toggling the target block.
    /// Conditional handlers such as cake, cauldron, flower pot, TNT, fence
    /// leash knots and command/structure blocks remain with their concrete
    /// class ports rather than being guessed. Jukebox conditional ownership is
    /// delegated to `BlockJukebox`.
    pub const fn predictsActivationSuccess(self) -> bool {
        matches!(
            self.registryId as i32,
            // Container / dedicated GUI blocks.
            23 | 54 | 58 | 61 | 62 | 116 | 117 | 130 | 138 | 145 | 146 | 154 | 158
                | 219..=234
                // Client-side unconditional activation branches.
                | 25 | 26 | 63 | 64 | 68 | 69 | 73 | 74 | 77 | 93 | 94 | 96
                | 107 | 122 | 143 | 149 | 150 | 151 | 178 | 183..=187 | 193..=197
        )
    }

    /// MCP `Block#getTickRandomly` for the vanilla 1.12.2 registry.
    ///
    /// The ids are derived from the concrete registered block classes whose
    /// constructors (or inherited constructors such as `BlockBush`/`BlockLiquid`)
    /// call `setTickRandomly(true)`.  This is used by
    /// `ExtendedBlockStorage#removeInvalidBlocks` to rebuild `tickRefCount`
    /// exactly like the Java client/server chunk container.
    pub const fn getTickRandomly(self) -> bool {
        matches!(
            self.registryId as i32,
            2 | 6 | 8..=11 | 18 | 28 | 31 | 32 | 37..=40 | 50 | 51 | 59 | 60
                | 70 | 72..=81 | 83 | 86 | 90..=92 | 104..=106 | 110 | 111
                | 115 | 127 | 131 | 132 | 141..=143 | 147 | 148 | 161 | 171
                | 175 | 200 | 207 | 212 | 213
        )
    }

    /// Exact MCP 1.12.2 default-state `Block.blockHardness` for protocol IDs.
    pub const fn getBlockHardness(self) -> f32 {
        BLOCK_HARDNESS[self.registryId as usize]
    }

    /// Exact MCP `IBlockState.getMaterial().isToolNotRequired()` result used by
    /// `InventoryPlayer.canHarvestBlock`.
    pub const fn isToolNotRequired(self) -> bool {
        TOOL_NOT_REQUIRED[self.registryId as usize]
    }

    /// MCP `Block#getLightOpacity(IBlockState)` default/vanilla constructor values.
    /// The base constructor uses 255 for an opaque default cube and 0 otherwise;
    /// the explicit 1.12.2 constructor/registerBlocks overrides are listed here.
    pub const fn getLightOpacity(self) -> i32 {
        match self.registryId as i32 {
            8 | 9 | 79 | 212 => 3,     // water, ice, frosted ice
            18 | 30 | 161 => 1,        // leaves/web/leaves2
            78 | 116 | 145 | 171 => 0, // snow layer, enchant table, anvil, carpet
            43
            | 44
            | 53
            | 60
            | 67
            | 108
            | 109
            | 114
            | 125
            | 126
            | 128
            | 134..=136
            | 156
            | 163
            | 164
            | 180..=182
            | 203..=205
            | 208 => 255,
            _ => {
                if self.isOpaqueCube() {
                    255
                } else {
                    0
                }
            }
        }
    }

    /// Protocol-ID bridge for the vanilla default-state `isOpaqueCube` result.
    /// Dynamic actual-state exceptions remain with their concrete block ports.
    pub const fn isOpaqueCube(self) -> bool {
        !matches!(
            self.registryId as i32,
            0 | 6 | 8..=11 | 18 | 20 | 26 | 27 | 28 | 30..=32 | 37..=40 | 44 | 50 | 51
                | 53 | 54 | 55 | 59 | 60 | 63..=72 | 75..=79 | 83 | 85 | 90 | 92..=96
                | 101 | 102 | 104..=111 | 113..=120 | 126 | 127 | 130..=132 | 134..=140
                | 141..=150 | 151 | 154 | 156 | 157 | 160 | 161 | 163 | 164 | 167 | 171
                | 175..=178 | 180 | 182..=205 | 207 | 209 | 217
        )
    }

    /// MCP `Block.registerBlocks` sets `useNeighborBrightness` for stairs,
    /// slabs, farmland, grass paths, translucent blocks and blocks with zero
    /// light opacity. For the protocol-only registry currently ported, these
    /// are the non-air states whose default state is not an opaque cube.
    pub const fn useNeighborBrightness(self) -> bool {
        if self.isAir() {
            false
        } else {
            self.isSlab()
                || matches!(
                    self.registryId as i32,
                    53 | 60 | 67 | 108 | 109 | 114 | 128 | 134
                        ..=136 | 156 | 163 | 164 | 180 | 203 | 208
                )
                || !self.isOpaqueCube()
        }
    }

    pub const fn isSlab(self) -> bool {
        matches!(
            self.registryId as i32,
            43 | 44 | 125 | 126 | 181 | 182 | 204 | 205
        )
    }

    /// Exact block-family exclusion in MCP `Block.func_193384_b`.
    pub const fn func_193384_b(self) -> bool {
        matches!(
            self.registryId as i32,
            18 | 79 | 89 | 95 | 96 | 118 | 138 | 161 | 167 | 169 | 219..=234 | 20
        )
    }

    /// Exact extension of `func_193384_b` used by attachable blocks.
    pub const fn func_193382_c(self) -> bool {
        self.func_193384_b() || matches!(self.registryId as i32, 29 | 33 | 34)
    }

    /// MCP `Block#canProvidePower` registry bridge used by ladder support.
    pub const fn canProvidePower(self) -> bool {
        matches!(
            self.registryId as i32,
            28 | 55 | 69 | 70 | 72 | 75 | 76 | 77 | 93 | 94 | 131 | 143 | 146..=152 | 178 | 218
        )
    }

    /// MCP `Block#isFullyOpaque` plus the vanilla overrides whose result is
    /// metadata-dependent. This is the backing contract for
    /// `IBlockState#isTopSolid` in placement rules.
    pub const fn isFullyOpaque(self, state: IBlockState) -> bool {
        let meta = state.getMetadata();
        match self.registryId as i32 {
            29 | 33 => meta & 8 == 0 || meta & 7 == 0, // BlockPistonBase
            34 => meta & 7 == 1,                       // BlockPistonExtension
            43 | 125 | 181 | 204 => true,              // double slabs
            44 | 126 | 182 | 205 => meta & 8 != 0,     // top single slabs
            53 | 67 | 108 | 109 | 114 | 128 | 134..=136 | 156 | 163 | 164 | 180 | 203 => {
                meta & 4 != 0
            } // top stairs
            78 => meta & 7 == 7,                       // eight snow layers
            154 => true,                               // hopper override
            _ => self.isOpaqueCube(),
        }
    }

    /// Collision boxes in block-local coordinates. This batch ports the
    /// vanilla default full-block path and metadata-only shapes needed by the
    /// first walking/physics loop. Neighbour-derived actual-state collision
    /// (fences, panes, walls, gates, doors and inner/outer stair corners) is
    /// intentionally left explicit rather than replaced by invented cubes.
    pub fn getCollisionBoxes(self, state: IBlockState) -> Vec<AxisAlignedBB> {
        let id = self.registryId as i32;
        let meta = state.getMetadata();
        let full = || vec![AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)];
        let one = |min_x, min_y, min_z, max_x, max_y, max_z| {
            vec![AxisAlignedBB::new(min_x, min_y, min_z, max_x, max_y, max_z)]
        };

        match id {
            // MCP classes returning NULL_AABB for entity collision.
            0
            | 6
            | 8..=11
            | 27
            | 28
            | 30..=32
            | 36..=40
            | 50
            | 51
            | 55
            | 59
            | 63
            | 66
            | 68
            | 69
            | 75..=77
            | 83
            | 90
            | 104..=106
            | 115
            | 119
            | 127
            | 131
            | 132
            | 141..=143
            | 157
            | 175..=177
            | 193..=197
            | 207
            | 209
            | 217 => Vec::new(),

            26 => one(0.0, 0.0, 0.0, 1.0, 0.5625, 1.0), // BlockBed

            // Single slabs. Double slabs retain the default full cube.
            44 | 126 | 182 | 205 => {
                if meta & 8 != 0 {
                    one(0.0, 0.5, 0.0, 1.0, 1.0, 1.0)
                } else {
                    one(0.0, 0.0, 0.0, 1.0, 0.5, 1.0)
                }
            }

            // Base straight stair shape from BlockStairs. The neighbour-driven
            // inner/outer quarter and eighth boxes are not yet applied.
            53 | 67 | 108 | 109 | 114 | 128 | 134..=136 | 156 | 163 | 164 | 180 | 203 => {
                let top = meta & 4 != 0;
                let mut boxes = if top {
                    vec![AxisAlignedBB::new(0.0, 0.5, 0.0, 1.0, 1.0, 1.0)]
                } else {
                    vec![AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 0.5, 1.0)]
                };
                let (min_y, max_y) = if top { (0.0, 0.5) } else { (0.5, 1.0) };
                boxes.push(match meta & 3 {
                    0 => AxisAlignedBB::new(0.5, min_y, 0.0, 1.0, max_y, 1.0),
                    1 => AxisAlignedBB::new(0.0, min_y, 0.0, 0.5, max_y, 1.0),
                    2 => AxisAlignedBB::new(0.0, min_y, 0.5, 1.0, max_y, 1.0),
                    _ => AxisAlignedBB::new(0.0, min_y, 0.0, 1.0, max_y, 0.5),
                });
                boxes
            }

            54 | 130 | 146 => one(0.0625, 0.0, 0.0625, 0.9375, 0.875, 0.9375),
            60 | 208 => one(0.0, 0.0, 0.0, 1.0, 0.9375, 1.0),

            65 => match meta {
                // BlockLadder#getStateFromMeta
                0..=2 => one(0.0, 0.0, 0.8125, 1.0, 1.0, 1.0),
                3 => one(0.0, 0.0, 0.0, 1.0, 1.0, 0.1875),
                4 => one(0.8125, 0.0, 0.0, 1.0, 1.0, 1.0),
                _ => one(0.0, 0.0, 0.0, 0.1875, 1.0, 1.0),
            },

            70 | 72 => {
                // BlockPressurePlate
                let height = if meta == 0 { 0.0625 } else { 0.03125 };
                one(0.0625, 0.0, 0.0625, 0.9375, height, 0.9375)
            }
            78 => one(0.0, 0.0, 0.0, 1.0, (meta & 7) as f64 * 0.125, 1.0),
            81 => one(0.0625, 0.0, 0.0625, 0.9375, 0.9375, 0.9375),
            88 => one(0.0, 0.0, 0.0, 1.0, 0.875, 1.0),
            92 => {
                let bites = (meta & 7).min(6) as f64;
                one(0.0625 + bites * 0.125, 0.0, 0.0625, 0.9375, 0.5, 0.9375)
            }
            93 | 94 | 149 | 150 => one(0.0, 0.0, 0.0, 1.0, 0.125, 1.0),

            96 | 167 => {
                // BlockTrapDoor legacy metadata
                if meta & 4 != 0 {
                    match meta & 3 {
                        0 => one(0.0, 0.0, 0.8125, 1.0, 1.0, 1.0),
                        1 => one(0.0, 0.0, 0.0, 1.0, 1.0, 0.1875),
                        2 => one(0.8125, 0.0, 0.0, 1.0, 1.0, 1.0),
                        _ => one(0.0, 0.0, 0.0, 0.1875, 1.0, 1.0),
                    }
                } else if meta & 8 != 0 {
                    one(0.0, 0.8125, 0.0, 1.0, 1.0, 1.0)
                } else {
                    one(0.0, 0.0, 0.0, 1.0, 0.1875, 1.0)
                }
            }

            111 => one(0.0625, 0.0, 0.0625, 0.9375, 0.09375, 0.9375),
            116 => one(0.0, 0.0, 0.0, 1.0, 0.75, 1.0),
            117 => vec![
                AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 0.125, 1.0),
                AxisAlignedBB::new(0.4375, 0.0, 0.4375, 0.5625, 0.875, 0.5625),
            ],
            118 => vec![
                AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 0.3125, 1.0),
                AxisAlignedBB::new(0.0, 0.0, 0.0, 0.125, 1.0, 1.0),
                AxisAlignedBB::new(0.875, 0.0, 0.0, 1.0, 1.0, 1.0),
                AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 0.125),
                AxisAlignedBB::new(0.0, 0.0, 0.875, 1.0, 1.0, 1.0),
            ],
            140 => one(0.3125, 0.0, 0.3125, 0.6875, 0.375, 0.6875),
            145 => {
                if meta & 1 == 0 {
                    one(0.0, 0.0, 0.125, 1.0, 1.0, 0.875)
                } else {
                    one(0.125, 0.0, 0.0, 0.875, 1.0, 1.0)
                }
            }
            147 | 148 => {
                let height = if meta == 0 { 0.0625 } else { 0.03125 };
                one(0.0625, 0.0, 0.0625, 0.9375, height, 0.9375)
            }
            151 | 178 => one(0.0, 0.0, 0.0, 1.0, 0.375, 1.0),
            154 => vec![
                AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 0.625, 1.0),
                AxisAlignedBB::new(0.0, 0.0, 0.0, 0.125, 1.0, 1.0),
                AxisAlignedBB::new(0.875, 0.0, 0.0, 1.0, 1.0, 1.0),
                AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 0.125),
                AxisAlignedBB::new(0.0, 0.0, 0.875, 1.0, 1.0, 1.0),
            ],
            171 => one(0.0, 0.0, 0.0, 1.0, 0.0625, 1.0),
            198 => match (meta & 7) % 6 {
                // BlockEndRod / EnumFacing.getFront
                2 | 3 => one(0.375, 0.375, 0.0, 0.625, 0.625, 1.0),
                4 | 5 => one(0.0, 0.375, 0.375, 1.0, 0.625, 0.625),
                _ => one(0.375, 0.0, 0.375, 0.625, 1.0, 0.625),
            },

            // Actual-state / tile-entity dependent shapes are not guessed.
            34
            | 64
            | 71
            | 85
            | 101
            | 102
            | 107
            | 113
            | 120
            | 122
            | 139
            | 144
            | 160
            | 183..=192
            | 199..=200 => Vec::new(),

            _ => full(),
        }
    }

    /// MCP `IBlockState.getLightValue` for the vanilla registry entries that
    /// emit light independently of their received chunk light arrays.
    pub const fn getLightValue(self) -> u8 {
        match self.registryId as i32 {
            10 | 11 | 51 | 89 | 91 | 119 | 124 | 138 | 169 | 209 => 15,
            50 | 198 => 14,
            62 => 13,
            90 => 11,
            74 | 150 => 9,
            76 | 130 => 7,
            213 => 3,
            39 | 40 | 117 | 120 | 122 => 1,
            _ => 0,
        }
    }

    pub const fn getSlipperiness(self) -> f32 {
        match self.registryId {
            79 | 174 | 212 => 0.98,
            165 => 0.8,
            _ => 0.6,
        }
    }
}

pub const AIR: Block = Block::new(0, "air");

pub static BLOCK_NAMES: [Option<&'static str>; 256] = [
    Some("air"),
    Some("stone"),
    Some("grass"),
    Some("dirt"),
    Some("cobblestone"),
    Some("planks"),
    Some("sapling"),
    Some("bedrock"),
    Some("flowing_water"),
    Some("water"),
    Some("flowing_lava"),
    Some("lava"),
    Some("sand"),
    Some("gravel"),
    Some("gold_ore"),
    Some("iron_ore"),
    Some("coal_ore"),
    Some("log"),
    Some("leaves"),
    Some("sponge"),
    Some("glass"),
    Some("lapis_ore"),
    Some("lapis_block"),
    Some("dispenser"),
    Some("sandstone"),
    Some("noteblock"),
    Some("bed"),
    Some("golden_rail"),
    Some("detector_rail"),
    Some("sticky_piston"),
    Some("web"),
    Some("tallgrass"),
    Some("deadbush"),
    Some("piston"),
    Some("piston_head"),
    Some("wool"),
    Some("piston_extension"),
    Some("yellow_flower"),
    Some("red_flower"),
    Some("brown_mushroom"),
    Some("red_mushroom"),
    Some("gold_block"),
    Some("iron_block"),
    Some("double_stone_slab"),
    Some("stone_slab"),
    Some("brick_block"),
    Some("tnt"),
    Some("bookshelf"),
    Some("mossy_cobblestone"),
    Some("obsidian"),
    Some("torch"),
    Some("fire"),
    Some("mob_spawner"),
    Some("oak_stairs"),
    Some("chest"),
    Some("redstone_wire"),
    Some("diamond_ore"),
    Some("diamond_block"),
    Some("crafting_table"),
    Some("wheat"),
    Some("farmland"),
    Some("furnace"),
    Some("lit_furnace"),
    Some("standing_sign"),
    Some("wooden_door"),
    Some("ladder"),
    Some("rail"),
    Some("stone_stairs"),
    Some("wall_sign"),
    Some("lever"),
    Some("stone_pressure_plate"),
    Some("iron_door"),
    Some("wooden_pressure_plate"),
    Some("redstone_ore"),
    Some("lit_redstone_ore"),
    Some("unlit_redstone_torch"),
    Some("redstone_torch"),
    Some("stone_button"),
    Some("snow_layer"),
    Some("ice"),
    Some("snow"),
    Some("cactus"),
    Some("clay"),
    Some("reeds"),
    Some("jukebox"),
    Some("fence"),
    Some("pumpkin"),
    Some("netherrack"),
    Some("soul_sand"),
    Some("glowstone"),
    Some("portal"),
    Some("lit_pumpkin"),
    Some("cake"),
    Some("unpowered_repeater"),
    Some("powered_repeater"),
    Some("stained_glass"),
    Some("trapdoor"),
    Some("monster_egg"),
    Some("stonebrick"),
    Some("brown_mushroom_block"),
    Some("red_mushroom_block"),
    Some("iron_bars"),
    Some("glass_pane"),
    Some("melon_block"),
    Some("pumpkin_stem"),
    Some("melon_stem"),
    Some("vine"),
    Some("fence_gate"),
    Some("brick_stairs"),
    Some("stone_brick_stairs"),
    Some("mycelium"),
    Some("waterlily"),
    Some("nether_brick"),
    Some("nether_brick_fence"),
    Some("nether_brick_stairs"),
    Some("nether_wart"),
    Some("enchanting_table"),
    Some("brewing_stand"),
    Some("cauldron"),
    Some("end_portal"),
    Some("end_portal_frame"),
    Some("end_stone"),
    Some("dragon_egg"),
    Some("redstone_lamp"),
    Some("lit_redstone_lamp"),
    Some("double_wooden_slab"),
    Some("wooden_slab"),
    Some("cocoa"),
    Some("sandstone_stairs"),
    Some("emerald_ore"),
    Some("ender_chest"),
    Some("tripwire_hook"),
    Some("tripwire"),
    Some("emerald_block"),
    Some("spruce_stairs"),
    Some("birch_stairs"),
    Some("jungle_stairs"),
    Some("command_block"),
    Some("beacon"),
    Some("cobblestone_wall"),
    Some("flower_pot"),
    Some("carrots"),
    Some("potatoes"),
    Some("wooden_button"),
    Some("skull"),
    Some("anvil"),
    Some("trapped_chest"),
    Some("light_weighted_pressure_plate"),
    Some("heavy_weighted_pressure_plate"),
    Some("unpowered_comparator"),
    Some("powered_comparator"),
    Some("daylight_detector"),
    Some("redstone_block"),
    Some("quartz_ore"),
    Some("hopper"),
    Some("quartz_block"),
    Some("quartz_stairs"),
    Some("activator_rail"),
    Some("dropper"),
    Some("stained_hardened_clay"),
    Some("stained_glass_pane"),
    Some("leaves2"),
    Some("log2"),
    Some("acacia_stairs"),
    Some("dark_oak_stairs"),
    Some("slime"),
    Some("barrier"),
    Some("iron_trapdoor"),
    Some("prismarine"),
    Some("sea_lantern"),
    Some("hay_block"),
    Some("carpet"),
    Some("hardened_clay"),
    Some("coal_block"),
    Some("packed_ice"),
    Some("double_plant"),
    Some("standing_banner"),
    Some("wall_banner"),
    Some("daylight_detector_inverted"),
    Some("red_sandstone"),
    Some("red_sandstone_stairs"),
    Some("double_stone_slab2"),
    Some("stone_slab2"),
    Some("spruce_fence_gate"),
    Some("birch_fence_gate"),
    Some("jungle_fence_gate"),
    Some("dark_oak_fence_gate"),
    Some("acacia_fence_gate"),
    Some("spruce_fence"),
    Some("birch_fence"),
    Some("jungle_fence"),
    Some("dark_oak_fence"),
    Some("acacia_fence"),
    Some("spruce_door"),
    Some("birch_door"),
    Some("jungle_door"),
    Some("acacia_door"),
    Some("dark_oak_door"),
    Some("end_rod"),
    Some("chorus_plant"),
    Some("chorus_flower"),
    Some("purpur_block"),
    Some("purpur_pillar"),
    Some("purpur_stairs"),
    Some("purpur_double_slab"),
    Some("purpur_slab"),
    Some("end_bricks"),
    Some("beetroots"),
    Some("grass_path"),
    Some("end_gateway"),
    Some("repeating_command_block"),
    Some("chain_command_block"),
    Some("frosted_ice"),
    Some("magma"),
    Some("nether_wart_block"),
    Some("red_nether_brick"),
    Some("bone_block"),
    Some("structure_void"),
    Some("observer"),
    Some("white_shulker_box"),
    Some("orange_shulker_box"),
    Some("magenta_shulker_box"),
    Some("light_blue_shulker_box"),
    Some("yellow_shulker_box"),
    Some("lime_shulker_box"),
    Some("pink_shulker_box"),
    Some("gray_shulker_box"),
    Some("silver_shulker_box"),
    Some("cyan_shulker_box"),
    Some("purple_shulker_box"),
    Some("blue_shulker_box"),
    Some("brown_shulker_box"),
    Some("green_shulker_box"),
    Some("red_shulker_box"),
    Some("black_shulker_box"),
    Some("white_glazed_terracotta"),
    Some("orange_glazed_terracotta"),
    Some("magenta_glazed_terracotta"),
    Some("light_blue_glazed_terracotta"),
    Some("yellow_glazed_terracotta"),
    Some("lime_glazed_terracotta"),
    Some("pink_glazed_terracotta"),
    Some("gray_glazed_terracotta"),
    Some("silver_glazed_terracotta"),
    Some("cyan_glazed_terracotta"),
    Some("purple_glazed_terracotta"),
    Some("blue_glazed_terracotta"),
    Some("brown_glazed_terracotta"),
    Some("green_glazed_terracotta"),
    Some("red_glazed_terracotta"),
    Some("black_glazed_terracotta"),
    Some("concrete"),
    Some("concrete_powder"),
    None,
    None,
    Some("structure_block"),
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registry_ids_match_mcp_registration() {
        assert_eq!(Block::getBlockById(1).getRegistryPath(), "stone");
        assert_eq!(
            Block::getBlockById(255).getRegistryPath(),
            "structure_block"
        );
        assert_eq!(
            Block::getBlockFromName("minecraft:grass").map(Block::getIdFromBlock),
            Some(2)
        );
        assert_eq!(
            Block::getBlockFromName("2").map(Block::getIdFromBlock),
            Some(2)
        );
        assert!(Block::getBlockFromName("999").is_none());
    }
    #[test]
    fn state_id_is_block_id_shifted_by_four_plus_meta() {
        let state = Block::getStateById((17 << 4) | 5);
        assert_eq!(state.getBlockId(), 17);
        assert_eq!(state.getMetadata(), 5);
        assert_eq!(Block::getStateId(state), (17 << 4) | 5);
    }
    #[test]
    fn random_tick_registry_matches_vanilla_registered_classes() {
        let expected = [
            2, 6, 8, 9, 10, 11, 18, 28, 31, 32, 37, 38, 39, 40, 50, 51, 59, 60, 70, 72, 73, 74, 75,
            76, 77, 78, 79, 80, 81, 83, 86, 90, 91, 92, 104, 105, 106, 110, 111, 115, 127, 131,
            132, 141, 142, 143, 147, 148, 161, 171, 175, 200, 207, 212, 213,
        ];
        for id in 0..=255 {
            assert_eq!(
                Block::getBlockById(id).getTickRandomly(),
                expected.contains(&id),
                "block id {id}"
            );
        }
    }
    #[test]
    fn unconditional_client_activations_consume_itemblock_clicks() {
        for id in [
            23, 25, 26, 54, 64, 69, 77, 96, 107, 122, 130, 143, 149, 151, 219, 234,
        ] {
            assert!(
                Block::getBlockById(id).predictsActivationSuccess(),
                "block id {id}"
            );
        }
        for id in [1, 46, 84, 92, 118, 140, 255] {
            assert!(
                !Block::getBlockById(id).predictsActivationSuccess(),
                "conditional/non-activating id {id}"
            );
        }
    }
}
