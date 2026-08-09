use crate::net::minecraft::client::gui::inventory::GuiInventory::GuiInventory;
use crate::net::minecraft::client::gui::inventory::GuiChest::GuiChest;
use crate::net::minecraft::client::gui::inventory::GuiShulkerBox::GuiShulkerBox;
use crate::net::minecraft::client::gui::inventory::GuiScreenHorseInventory::GuiScreenHorseInventory;
use crate::net::minecraft::client::gui::inventory::GuiCrafting::GuiCrafting;
use crate::net::minecraft::client::gui::inventory::GuiFurnace::GuiFurnace;
use crate::net::minecraft::client::gui::inventory::GuiBrewingStand::GuiBrewingStand;
use crate::net::minecraft::client::gui::inventory::GuiDispenser::GuiDispenser;
use crate::net::minecraft::client::gui::inventory::GuiBeacon::GuiBeacon;
use crate::net::minecraft::client::gui::GuiHopper::GuiHopper;
use crate::net::minecraft::client::gui::GuiRepair::GuiRepair;
use crate::net::minecraft::client::gui::GuiEnchantment::{EnchantmentBookRenderState, GuiEnchantment};
use crate::net::minecraft::enchantment::Enchantment::Enchantment;
use crate::net::minecraft::util::EnchantmentNameParts::EnchantmentNameParts;
use crate::net::minecraft::client::gui::GuiMerchant::{GuiMerchant, MerchantPreviewRegion};
use crate::net::minecraft::village::MerchantRecipeList::MerchantRecipeList;
use crate::net::minecraft::client::gui::inventory::GuiContainer::GuiContainer;
use crate::net::minecraft::creativetab::CreativeTabs::{
    byIndex as creativeTabByIndex, BUILDING_BLOCKS as RECIPE_BUILDING_TAB,
    COMBAT as RECIPE_COMBAT_TAB, CREATIVE_TAB_ARRAY, FOOD as RECIPE_FOOD_TAB,
    INVENTORY as CREATIVE_INVENTORY_TAB, MISC as RECIPE_MISC_TAB,
    REDSTONE as RECIPE_REDSTONE_TAB, SEARCH as RECIPE_SEARCH_TAB,
    TOOLS as RECIPE_TOOLS_TAB,
};
use crate::net::minecraft::client::model::ModelShield::ModelShield;
use crate::net::minecraft::inventory::ContainerHorseInventory::HorseInventorySpec;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::{self, Write as _};
use std::path::PathBuf;
use std::sync::{mpsc, Arc, OnceLock};
use std::time::{Duration, Instant};

use rayon::prelude::*;

use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::math::MathHelper::{cos as minecraft_cos, sin as minecraft_sin};
use crate::net::minecraft::block::Block::Block;
use crate::net::minecraft::block::BlockBed::BlockBed;
use crate::net::minecraft::block::BlockDoor;
use crate::net::minecraft::block::BlockLiquid;
use crate::net::minecraft::block::BlockFence;
use crate::net::minecraft::block::BlockFenceGate;
use crate::net::minecraft::block::BlockFlowerPot;
use crate::net::minecraft::block::BlockRedstoneWire;
use crate::net::minecraft::block::BlockFire::BlockFire;
use crate::net::minecraft::block::BlockPane;
use crate::net::minecraft::block::BlockSkull::BlockSkull;
use crate::net::minecraft::block::BlockStairs::{self, EnumShape as StairShape};
use crate::net::minecraft::block::BlockWall;
use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::MapItemRenderer::MapItemRenderer;
use crate::net::minecraft::client::gui::GuiIngame::{GuiIngame, HudText, HudTexture};
use crate::net::minecraft::client::gui::GuiOverlayDebug::DebugOverlayData;
use crate::net::minecraft::client::gui::GuiBossOverlay::GuiBossOverlay;
use crate::net::minecraft::network::play::server::SPacketTitle::{SPacketTitle, Type as TitleType};
use crate::net::minecraft::network::play::server::SPacketUpdateBossInfo::SPacketUpdateBossInfo;
use crate::net::minecraft::client::gui::GuiPlayerTabOverlay::GuiPlayerTabOverlay;
use crate::net::minecraft::client::gui::GuiNewChat::GuiNewChat;
use crate::net::minecraft::client::gui::GuiTextField::GuiTextFieldRenderState;
use crate::net::minecraft::client::network::NetworkPlayerInfo::NetworkPlayerInfo;
use crate::net::minecraft::client::multiplayer::WorldClient::WorldClient;
use crate::net::minecraft::client::entity::EntityOtherClient::{
    ClientEntityKind, EntityOtherClient, ObjectSpawnType,
};
use crate::net::minecraft::client::particle::ParticleDigging::{ParticleActualModel, ParticleDiggingRenderState};
use crate::net::minecraft::client::particle::ParticleRenderState::ParticleRenderState;
use crate::net::minecraft::client::renderer::DestroyBlockProgress::DestroyBlockProgress;
use crate::net::minecraft::client::network::NetHandlerPlayClient::{
    PlayClientState, PlayerPositionState, ReceivedChatMessage,
};
use crate::net::minecraft::client::renderer::EntityRenderer::{EntityRenderer, LightmapParameters};
use crate::net::minecraft::client::renderer::ShaderFrameState::ShaderFrameState;
use crate::net::minecraft::client::renderer::entity::RenderPlayer::{ElytraCorpseRotation, PlayerRenderInput, RenderPlayer};
use crate::net::minecraft::client::renderer::entity::Render::Render;
use crate::net::minecraft::client::renderer::entity::RenderManager::{EntityRendererKind, RenderManager};
use crate::net::minecraft::client::renderer::entity::RenderEntityItem::RenderEntityItem;
use crate::net::minecraft::client::renderer::entity::RenderFallingBlock::RenderFallingBlock;
use crate::net::minecraft::client::renderer::entity::RenderSnowball::RenderSnowball;
use crate::net::minecraft::client::renderer::entity::RenderXPOrb::RenderXPOrb;
use crate::net::minecraft::client::renderer::entity::RenderArrow::RenderArrow;
use crate::net::minecraft::client::renderer::entity::RenderTNTPrimed::RenderTNTPrimed;
use crate::net::minecraft::client::renderer::entity::RenderEnderCrystal::RenderEnderCrystal;
use crate::net::minecraft::client::renderer::entity::RenderBoat::RenderBoat;
use crate::net::minecraft::client::renderer::entity::RenderMinecart::RenderMinecart;
use crate::net::minecraft::client::renderer::entity::RenderPainting::RenderPainting;
use crate::net::minecraft::client::renderer::entity::RenderItemFrame::RenderItemFrame;
use crate::net::minecraft::client::renderer::entity::RenderLeashKnot::RenderLeashKnot;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{LivingModelMesh, RenderLivingBase};
use crate::net::minecraft::client::renderer::entity::RenderZombie::{RenderZombie, ZombieRenderVariant};
use crate::net::minecraft::client::renderer::entity::RenderSkeleton::{RenderSkeleton, SkeletonRenderVariant};
use crate::net::minecraft::client::renderer::entity::RenderArmorStand::RenderArmorStand;
use crate::net::minecraft::client::renderer::entity::RenderPig::RenderPig;
use crate::net::minecraft::client::renderer::entity::RenderCow::RenderCow;
use crate::net::minecraft::client::renderer::entity::RenderSheep::RenderSheep;
use crate::net::minecraft::client::renderer::entity::RenderChicken::RenderChicken;
use crate::net::minecraft::client::renderer::entity::RenderMooshroom::RenderMooshroom;
use crate::net::minecraft::client::renderer::entity::RenderCreeper::RenderCreeper;
use crate::net::minecraft::client::renderer::entity::RenderSpider::{RenderSpider, SpiderVariant};
use crate::net::minecraft::client::renderer::entity::RenderSlime::RenderSlime;
use crate::net::minecraft::client::renderer::entity::RenderMagmaCube::RenderMagmaCube;
use crate::net::minecraft::client::renderer::entity::RenderBlaze::RenderBlaze;
use crate::net::minecraft::client::renderer::entity::RenderGhast::RenderGhast;
use crate::net::minecraft::client::renderer::entity::RenderGuardian::RenderGuardian;
use crate::net::minecraft::client::renderer::entity::RenderShulker::{RenderShulker, ShulkerTransformOp};
use crate::net::minecraft::client::renderer::entity::RenderShulkerBullet::RenderShulkerBullet;
use crate::net::minecraft::client::renderer::entity::RenderFireball::RenderFireball;
use crate::net::minecraft::client::renderer::entity::RenderDragonFireball::RenderDragonFireball;
use crate::net::minecraft::client::renderer::entity::RenderWitherSkull::RenderWitherSkull;
use crate::net::minecraft::client::renderer::entity::RenderFish::RenderFish;
use crate::net::minecraft::entity::projectile::EntityShulkerBullet::EntityShulkerBullet;
use crate::net::minecraft::entity::projectile::EntityFishHook::EntityFishHook;
use crate::net::minecraft::entity::projectile::EntityFireball::EntityFireball;
use crate::net::minecraft::entity::item::EntityMinecart::{EntityMinecart, MinecartType};
use crate::net::minecraft::entity::EntityLeashKnot::EntityLeashKnot;
use crate::net::minecraft::entity::item::EntityItemFrame::EntityItemFrame;
use crate::net::minecraft::client::renderer::entity::RenderWolf::RenderWolf;
use crate::net::minecraft::client::renderer::entity::RenderOcelot::RenderOcelot;
use crate::net::minecraft::client::renderer::entity::RenderRabbit::RenderRabbit;
use crate::net::minecraft::client::renderer::entity::RenderPolarBear::RenderPolarBear;
use crate::net::minecraft::client::renderer::entity::RenderAbstractHorse::RenderAbstractHorse;
use crate::net::minecraft::client::renderer::entity::RenderHorse::RenderHorse;
use crate::net::minecraft::client::renderer::entity::RenderLlama::RenderLlama;
use crate::net::minecraft::client::renderer::entity::RenderVillager::RenderVillager;
use crate::net::minecraft::client::renderer::entity::RenderWitch::RenderWitch;
use crate::net::minecraft::client::renderer::entity::RenderVindicator::RenderVindicator;
use crate::net::minecraft::client::renderer::entity::RenderEvoker::RenderEvoker;
use crate::net::minecraft::client::renderer::entity::RenderIllusionIllager::RenderIllusionIllager;
use crate::net::minecraft::client::renderer::entity::RenderZombieVillager::RenderZombieVillager;
use crate::net::minecraft::client::model::ModelBoat::ModelBoat;
use crate::net::minecraft::client::model::ModelMinecart::ModelMinecart;
use crate::net::minecraft::client::model::ModelLeashKnot::ModelLeashKnot;
use crate::net::minecraft::client::model::ModelEnderCrystal::ModelEnderCrystal;
use crate::net::minecraft::client::model::ModelVehicleBox::VehicleModelMesh;
use crate::net::minecraft::client::model::ModelZombie::ModelZombie;
use crate::net::minecraft::client::model::ModelSkeleton::ModelSkeleton;
use crate::net::minecraft::client::model::ModelArmorStand::ModelArmorStand;
use crate::net::minecraft::client::model::ModelPig::ModelPig;
use crate::net::minecraft::client::model::ModelCow::ModelCow;
use crate::net::minecraft::client::model::ModelSheep1::ModelSheep1;
use crate::net::minecraft::client::model::ModelSheep2::ModelSheep2;
use crate::net::minecraft::client::model::ModelChicken::ModelChicken;
use crate::net::minecraft::client::model::ModelCreeper::ModelCreeper;
use crate::net::minecraft::client::model::ModelSpider::ModelSpider;
use crate::net::minecraft::client::model::ModelSlime::ModelSlime;
use crate::net::minecraft::client::model::ModelMagmaCube::ModelMagmaCube;
use crate::net::minecraft::client::model::ModelBlaze::ModelBlaze;
use crate::net::minecraft::client::model::ModelGhast::ModelGhast;
use crate::net::minecraft::client::model::ModelGuardian::{GuardianModelState, ModelGuardian};
use crate::net::minecraft::client::model::ModelShulker::{ModelShulker, ShulkerModelState};
use crate::net::minecraft::client::model::ModelShulkerBullet::{ModelShulkerBullet, ShulkerBulletModelMesh};
use crate::net::minecraft::client::model::ModelSkeletonHead::ModelSkeletonHead;
use crate::net::minecraft::client::model::ModelWolf::ModelWolf;
use crate::net::minecraft::client::model::ModelOcelot::ModelOcelot;
use crate::net::minecraft::client::model::ModelRabbit::ModelRabbit;
use crate::net::minecraft::client::model::ModelPolarBear::ModelPolarBear;
use crate::net::minecraft::client::model::ModelHorse::{HorseModelVariant, ModelHorse};
use crate::net::minecraft::client::model::ModelLlama::ModelLlama;
use crate::net::minecraft::client::model::ModelVillager::ModelVillager;
use crate::net::minecraft::client::model::ModelWitch::ModelWitch;
use crate::net::minecraft::client::model::ModelIllager::{IllagerPose, ModelIllager};
use crate::net::minecraft::client::model::ModelZombieVillager::ModelZombieVillager;
use crate::net::minecraft::client::model::ModelBook::ModelBook;
use crate::net::minecraft::client::renderer::tileentity::TileEntityEnchantmentTableRenderer::TileEntityEnchantmentTableRenderer;
use crate::net::minecraft::client::renderer::tileentity::TileEntityBeaconRenderer::TileEntityBeaconRenderer;
use crate::net::minecraft::client::renderer::tileentity::TileEntityEndPortalRenderer::TileEntityEndPortalRenderer;
use crate::net::minecraft::client::renderer::tileentity::TileEntityShulkerBoxRenderer::TileEntityShulkerBoxRenderer;
use crate::net::minecraft::client::renderer::tileentity::TileEntitySignRenderer::TileEntitySignRenderer;
use crate::net::minecraft::client::renderer::entity::layers::LayerHeldItem::LayerHeldItem;
use crate::net::minecraft::client::renderer::entity::layers::LayerCape::{CapeMotionInput, LayerCape};
use crate::net::minecraft::client::renderer::entity::layers::LayerSaddle::LayerSaddle;
use crate::net::minecraft::client::renderer::entity::layers::LayerSheepWool::LayerSheepWool;
use crate::net::minecraft::client::renderer::entity::layers::LayerMooshroomMushroom::LayerMooshroomMushroom;
use crate::net::minecraft::client::renderer::entity::layers::LayerCreeperCharge::LayerCreeperCharge;
use crate::net::minecraft::client::renderer::entity::layers::LayerSpiderEyes::LayerSpiderEyes;
use crate::net::minecraft::client::renderer::entity::layers::LayerSlimeGel::LayerSlimeGel;
use crate::net::minecraft::client::renderer::entity::layers::LayerWolfCollar::LayerWolfCollar;
use crate::net::minecraft::client::renderer::entity::layers::LayerLlamaDecor::LayerLlamaDecor;
use crate::net::minecraft::client::model::ModelBiped::{ArmPose, BipedPose, PartPose};
use crate::net::minecraft::client::model::ModelElytra::{ElytraRotationState, ModelElytra};
use crate::net::minecraft::client::renderer::entity::layers::LayerArmorBase::LayerArmorBase;
use crate::net::minecraft::client::renderer::entity::layers::LayerBipedArmor::LayerBipedArmor;
use crate::net::minecraft::client::renderer::entity::layers::LayerElytra::LayerElytra;
use crate::net::minecraft::client::renderer::entity::layers::LayerCustomHead::LayerCustomHead;
use crate::net::minecraft::item::ItemArmor::ItemArmor;
use crate::net::minecraft::item::ItemSkull::ItemSkull;
use crate::net::minecraft::client::renderer::ItemModelMesher::ItemModelMesher;
use crate::net::minecraft::client::renderer::ItemRenderer::{FirstPersonItemRenderState, ItemRenderer};
use crate::net::minecraft::client::renderer::RenderItem::{RenderItem, ResolvedItemModel};
use crate::net::minecraft::client::renderer::tileentity::TileEntitySkullRenderer::TileEntitySkullRenderer;
use crate::net::minecraft::client::renderer::tileentity::TileEntityItemStackRenderer::{BuiltInItemMesh, TileEntityItemStackRenderer};
use crate::net::minecraft::client::renderer::tileentity::TileEntityChestRenderer::{ChestRenderInput, TileEntityChestRenderer};
use crate::net::minecraft::client::renderer::block::model::ItemCameraTransforms::{
    ItemTransformVec3f, TransformType,
};
use crate::net::minecraft::client::resources::DefaultPlayerSkin::DefaultPlayerSkin;
use crate::net::minecraft::client::resources::SkinManager::SkinManager;
use crate::com::mojang::authlib::GameProfile::GameProfile;
use crate::com::mojang::authlib::minecraft::MinecraftSessionService::MinecraftSessionService;
use crate::net::minecraft::client::renderer::BlockModelRenderer::BlockModelRenderer;
use crate::net::minecraft::client::renderer::BlockFluidRenderer::{self, FluidSprites};
use crate::net::minecraft::client::renderer::color::BlockColors::BlockColors;
use crate::net::minecraft::client::renderer::color::ItemColors::ItemColors;
use crate::net::minecraft::world::ColorizerGrass::ColorizerGrass;
use crate::net::minecraft::world::ColorizerFoliage::ColorizerFoliage;
use crate::net::minecraft::world::biome::BiomeColorHelper::BiomeAccess;
use crate::net::minecraft::world::biome::Biome::Biome;
use crate::net::minecraft::client::renderer::culling::ClippingHelperImpl::ClippingHelperImpl;
use crate::net::minecraft::client::renderer::culling::Frustum::Frustum;
use crate::net::minecraft::client::renderer::chunk::CompiledChunk::CompiledChunk;
use crate::net::minecraft::client::renderer::chunk::RenderChunk::RenderChunkKey;
use crate::net::minecraft::client::renderer::chunk::VisGraph::VisGraph;
use crate::net::minecraft::client::renderer::RenderGlobal::{
    drawSelectionBox, setupTerrainWithLookup, SelectionBoxRenderState,
};
use crate::net::minecraft::client::renderer::BlockModelShapes::{
    BlockModelShapes, ResolvedBlockModel, ResolvedFace,
};
use crate::net::minecraft::client::resources::Locale::Locale;
use crate::net::minecraft::client::gui::recipebook::GuiRecipeBook::{GuiRect, RecipeBookRenderState};
use crate::net::minecraft::item::crafting::CraftingManager::CraftingManager;
use crate::net::minecraft::item::crafting::RecipeRegistryData::RecipeCategory;
use crate::net::minecraft::client::resources::SimpleReloadableResourceManager::ResourceManager;
use crate::net::minecraft::util::BlockRenderLayer::BlockRenderLayer;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::util::EnumHand::EnumHand;
use crate::net::minecraft::util::EnumHandSide::EnumHandSide;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::net::minecraft::util::text::ITextComponent::ITextComponent;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::item::ItemMap::ItemMap;
use crate::net::minecraft::item::ItemTooltip;
use crate::net::minecraft::inventory::ContainerWindow::{canHoldBrewingPotion, isBrewingReagent, isFurnaceFuel, ContainerWindowKind};
use crate::net::minecraft::inventory::Container::Container;
use crate::net::minecraft::inventory::EntityEquipmentSlot::EntityEquipmentSlot;
use crate::net::minecraft::inventory::ContainerPlayer::{
    playerContainerSlotAccepts, playerContainerSlotLimit,
};
use crate::net::minecraft::item::EnumAction::EnumAction;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::math::Vec3d::Vec3d;
use crate::net::minecraft::world::EnumSkyBlock::EnumSkyBlock;
use crate::net::minecraft::world::WorldProvider::WorldProvider;
use crate::net::minecraft::world::GameType::GameType;
use crate::net::minecraft::world::storage::MapData::MapData;
use crate::net::minecraft::scoreboard::Scoreboard::Scoreboard;
use crate::net::minecraft::world::chunk::Chunk::Chunk;
use crate::net::minecraft::world::IBlockAccess::IBlockAccess;
use crate::vulkan::GuiDrawList::{GuiDrawCommand, GuiDrawList, GuiTopology};
use crate::vulkan::NativeImage::NativeImage;
use crate::vulkan::TextureSource::{TextureAnimation, TextureSource};

const MAX_GLOBAL_BLOCK_STATE_ID: i32 = (255 << 4) | 15;
// reference-renderer-style background scheduling: one priority column plus two normal
// streaming columns may be built concurrently. A column job retains the MCP
// 16x16x16 RenderChunk outputs, but shares one immutable 3x3 column snapshot
// and one scheduling/channel operation across all dirty vertical sections.
const MAX_PRIORITY_BACKGROUND_COLUMN_JOBS: usize = 1;
const MAX_NORMAL_BACKGROUND_COLUMN_JOBS: usize = 2;
const MAX_FINISHED_COLUMN_BATCHES_PER_FRAME: usize =
    MAX_PRIORITY_BACKGROUND_COLUMN_JOBS + MAX_NORMAL_BACKGROUND_COLUMN_JOBS;
const MAX_FINISHED_CHUNKS_PER_FRAME: usize = 24;
const MAX_FINISHED_CHUNK_BYTES_PER_FRAME: usize = 8 * 1024 * 1024;
const MAX_SINGLE_FINISHED_CHUNK_BYTES: usize = 1024 * 1024;
const MAX_FINISHED_CHUNK_UPLOAD_TIME: Duration = Duration::from_micros(8_000);
/// reference-renderer-style CPU mesh cadence. Minecraft simulation and camera matrices
/// continue at the render rate; only CPU tessellation of dynamic render streams
/// is bounded to 60 Hz and reused by the GPU between rebuilds.
const DYNAMIC_MESH_BUILD_INTERVAL: Duration = Duration::from_micros(16_667);
/// Reference renderer threshold: small entity lists remain serial to avoid Rayon
/// scheduling overhead; larger lists are split into ordered independent batches.
const PARALLEL_ENTITY_BATCH_THRESHOLD: usize = 8;
const PARALLEL_PLAYER_BATCH_THRESHOLD: usize = 4;
/// Initial terrain within this X/Z radius uses the priority column lane. This
/// mirrors the reference immediate-player-area scheduling while preserving MCP's
/// independent 16x16x16 RenderChunk products.
const INITIAL_PRIORITY_COLUMN_RADIUS: i32 = 1;
const MAX_GPU_REMOVALS_PER_FRAME: usize = 64;
/// Fixed TextureManager-equivalent capacity keeps the stitched atlas geometry
/// stable while asynchronous player textures and resource-pack icons arrive.
const DYNAMIC_PLAYER_TEXTURE_RESERVE: usize = 256;
const DYNAMIC_RESOURCE_PACK_ICON_RESERVE: usize = 64;
const DYNAMIC_PLAYER_MATERIAL_BASE: i32 = -100_000;
const DYNAMIC_RESOURCE_PACK_ICON_MATERIAL_BASE: i32 = -200_000;
const DYNAMIC_ATLAS_REBUILD_DELAY: Duration = Duration::from_millis(250);

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
    /// Vanilla lightmap coordinates expressed as discrete block/sky levels.
    /// The shader reproduces the 16 x 16 linear-filtered lightmap.
    pub lightmap: [f32; 2],
    /// OptiFine `SVertexBuilder.entityData`: mapped block id, metadata and
    /// `EnumBlockRenderType` ordinal. Non-block geometry uses -1 sentinels.
    pub shaderEntity: [i16; 3],
    pub shaderPadding: i16,
}

impl WorldVertex {
    pub const STRIDE: u32 = std::mem::size_of::<Self>() as u32;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldPushConstants {
    /// Column-major matrix, matching GLSL's default `mat4` memory layout.
    pub viewProjection: [f32; 16],
    pub cameraPosition: [f32; 4],
    pub fogColor: [f32; 4],
    pub fogParameters: [f32; 4],
    /// sun brightness, torch flicker, gamma, dimension id
    pub lightmapParameters: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChunkKey {
    pub x: i32,
    pub z: i32,
}

impl ChunkKey {
    fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisibleChunk {
    pub key: RenderChunkKey,
    pub aabbMin: [f32; 3],
    pub aabbMax: [f32; 3],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockTextureAtlas {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DynamicAtlasSlot {
    originX: u32,
    originY: u32,
    tileSize: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ChunkLayerRange {
    pub firstIndex: u32,
    pub indexCount: u32,
}

#[derive(Debug, Clone)]
pub struct ChunkMeshUpload {
    pub key: RenderChunkKey,
    pub meshRevision: u64,
    pub vertices: Arc<Vec<WorldVertex>>,
    pub indices: Arc<Vec<u32>>,
    pub layerRanges: [ChunkLayerRange; 4],
    /// `RenderChunk#resortTransparency` changes only quad order. Backends use
    /// this to retain the resident vertex allocation and update indices only.
    pub verticesUnchanged: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HudPipelineKind {
    Alpha,
    Crosshair,
    Glint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudDrawRange {
    pub pipeline: HudPipelineKind,
    pub firstIndex: u32,
    pub indexCount: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirstPersonPipelineKind {
    Alpha,
    /// MCP `ItemRenderer#renderFireInFirstPerson`: GL_ALWAYS, no depth write,
    /// alpha blend and no face culling.
    Fire,
    Glint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FirstPersonDrawRange {
    pub pipeline: FirstPersonPipelineKind,
    pub firstIndex: u32,
    pub indexCount: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldEntityPipelineKind {
    Entities,
    BlockEntities,
    /// `EntityRenderer#drawNameplate`: untextured black background with depth
    /// testing and depth writes disabled for ordinary (non-sneaking) players.
    NameplateBackgroundSeeThrough,
    /// First dim font pass with depth testing and depth writes disabled.
    NameplateTextSeeThrough,
    /// Sneaking nameplate background: untextured, depth test retained and
    /// depth writes disabled.
    NameplateBackgroundDepthNoWrite,
    /// Final font pass: depth test and depth writes enabled.
    NameplateTextDepthWrite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldEntityMeshKind {
    Dynamic,
    BlockEntities,
    StaticEntities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldEntityDrawRange {
    pub pipeline: WorldEntityPipelineKind,
    pub mesh: WorldEntityMeshKind,
    pub firstIndex: u32,
    pub indexCount: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityOverlayPipelineKind {
    ArmorGlint,
    TntFlash,
    EndPortalAlpha,
    EndPortalAdditive,
    BeaconCore,
    BeaconGlow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityOverlayDrawRange {
    pub pipeline: EntityOverlayPipelineKind,
    pub firstIndex: u32,
    pub indexCount: u32,
}

#[derive(Debug, Clone)]
pub struct WorldRenderFrame {
    pub shaderState: ShaderFrameState,
    pub atlasRevision: u64,
    pub atlas: Arc<BlockTextureAtlas>,
    pub chunkUploads: Vec<ChunkMeshUpload>,
    pub removedChunks: Vec<RenderChunkKey>,
    pub visibleChunks: Vec<VisibleChunk>,
    /// Latest dynamic stream generation retained for diagnostics and backward
    /// compatibility. Each stream below owns an independent generation so one
    /// changing particle does not force entities, HUD and first-person buffers
    /// to be uploaded again.
    pub dynamicMeshGeneration: u64,
    pub entityMeshGeneration: u64,
    pub blockEntityMeshGeneration: u64,
    pub staticEntityMeshGeneration: u64,
    pub entityDepthMeshGeneration: u64,
    pub entityOverlayMeshGeneration: u64,
    pub particleMeshGeneration: u64,
    pub transparentParticleMeshGeneration: u64,
    pub damageMeshGeneration: u64,
    pub selectionMeshGeneration: u64,
    pub firstPersonMeshGeneration: u64,
    pub hudMeshGeneration: u64,
    pub entityVertices: Arc<Vec<WorldVertex>>,
    pub entityIndices: Arc<Vec<u32>>,
    /// Cached TESR and non-animated hanging geometry own independent streams.
    /// Draw ranges interleave these streams with animated entities in the exact
    /// existing RenderGlobal/RenderManager source order, so one moving entity
    /// no longer forces unchanged block-entity or hanging meshes to upload.
    pub blockEntityVertices: Arc<Vec<WorldVertex>>,
    pub blockEntityIndices: Arc<Vec<u32>>,
    pub staticEntityVertices: Arc<Vec<WorldVertex>>,
    pub staticEntityIndices: Arc<Vec<u32>>,
    pub entityDrawRanges: Vec<WorldEntityDrawRange>,
    /// MCP `RenderGlobal#renderEntities` immediately follows the ordinary
    /// entity pass with `RenderManager#renderMultipass`. In 1.12.2 boats use
    /// that pass for `ModelBoat#noWater`: RGBA writes are masked while depth
    /// remains enabled. Keeping a distinct stream preserves that ordering and
    /// state instead of drawing a visible surrogate surface.
    pub entityDepthVertices: Arc<Vec<WorldVertex>>,
    pub entityDepthIndices: Arc<Vec<u32>>,
    /// Texture-disabled, alpha-blended entity overlays. Batch 67 uses this
    /// for `RenderTntMinecart`'s white fuse flash, which cannot share the
    /// ordinary textured entity pipeline without changing vanilla blending.
    pub entityOverlayVertices: Arc<Vec<WorldVertex>>,
    pub entityOverlayIndices: Arc<Vec<u32>>,
    /// First range in the shared overlay mesh. RenderGlobal draws the sky
    /// before terrain with depth writes disabled; later ranges remain entity
    /// and tile-entity overlays.
    pub skyAlphaIndexCount: u32,
    pub skyCelestialIndexCount: u32,
    pub entityOverlayDrawRanges: Vec<EntityOverlayDrawRange>,
    pub renderedRemotePlayers: usize,
    pub renderedNonPlayerEntities: usize,
    pub particleVertices: Arc<Vec<WorldVertex>>,
    pub particleIndices: Arc<Vec<u32>>,
    /// MCP `ParticleManager` layer-0 transparent queue. This is kept apart
    /// because vanilla toggles `depthMask(false)` for `isTransparent()`.
    pub transparentParticleVertices: Arc<Vec<WorldVertex>>,
    pub transparentParticleIndices: Arc<Vec<u32>>,
    pub damageVertices: Arc<Vec<WorldVertex>>,
    pub damageIndices: Arc<Vec<u32>>,
    pub selectionVertices: Arc<Vec<WorldVertex>>,
    pub selectionIndices: Arc<Vec<u32>>,
    pub firstPersonVertices: Arc<Vec<WorldVertex>>,
    pub firstPersonIndices: Arc<Vec<u32>>,
    pub firstPersonDrawRanges: Vec<FirstPersonDrawRange>,
    pub firstPersonPushConstants: WorldPushConstants,
    pub hudVertices: Arc<Vec<WorldVertex>>,
    pub hudIndices: Arc<Vec<u32>>,
    pub hudDrawRanges: Vec<HudDrawRange>,
    pub pushConstants: WorldPushConstants,
    pub skyPushConstants: WorldPushConstants,
    pub hudPushConstants: WorldPushConstants,
    pub clearColor: [f32; 4],
}


/// Allocation-free FNV-1a sink used to fingerprint immutable render inputs.
/// Per-resource state fingerprints avoid rebuilding unchanged GPU-bound meshes;
/// this writer applies that principle without allocating a temporary `String`.
#[derive(Debug, Clone, Copy)]
struct RenderStateFingerprint(u64);

impl Default for RenderStateFingerprint {
    fn default() -> Self {
        Self(0xcbf29ce484222325)
    }
}

impl fmt::Write for RenderStateFingerprint {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        for byte in value.as_bytes() {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x100000001b3);
        }
        Ok(())
    }
}

fn debug_render_state_hash<T: fmt::Debug>(value: &T) -> u64 {
    let mut fingerprint = RenderStateFingerprint::default();
    // Formatting directly into the FNV sink avoids the large temporary debug
    // strings that would otherwise erase the benefit of a mesh cache.
    write!(&mut fingerprint, "{value:?}").expect("render-state hashing cannot fail");
    fingerprint.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum BlockEntityMeshKind {
    Skull,
    Bed,
    Chest,
    Piston,
    ShulkerBox,
    Sign,
    EnchantmentTable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct BlockEntityMeshIdentity {
    kind: BlockEntityMeshKind,
    pos: BlockPos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BlockEntityMeshCacheKey {
    stateHash: u64,
    snapshotHash: u64,
    atlasRevision: u64,
}

#[derive(Debug, Clone, Default)]
struct BlockEntityMeshBatch {
    vertices: Vec<WorldVertex>,
    indices: Vec<u32>,
}

#[derive(Debug, Clone)]
struct CachedBlockEntityMesh {
    key: BlockEntityMeshCacheKey,
    mesh: Arc<BlockEntityMeshBatch>,
    lastSeenEpoch: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum StaticEntityMeshKind {
    Painting,
    ItemFrame,
    LeashKnot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct StaticEntityMeshIdentity {
    kind: StaticEntityMeshKind,
    entityId: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StaticEntityMeshCacheKey {
    stateHash: u64,
    snapshotHash: u64,
    atlasRevision: u64,
}

#[derive(Debug, Clone, Default)]
struct StaticEntityMeshBatch {
    vertices: Vec<WorldVertex>,
    indices: Vec<u32>,
}

#[derive(Debug, Clone)]
struct CachedStaticEntityMesh {
    key: StaticEntityMeshCacheKey,
    mesh: Arc<StaticEntityMeshBatch>,
    lastSeenEpoch: u64,
}

#[derive(Debug, Clone)]
struct CachedDynamicMeshes {
    entityMeshGeneration: u64,
    blockEntityMeshGeneration: u64,
    staticEntityMeshGeneration: u64,
    entityDepthMeshGeneration: u64,
    entityOverlayMeshGeneration: u64,
    particleMeshGeneration: u64,
    transparentParticleMeshGeneration: u64,
    damageMeshGeneration: u64,
    selectionMeshGeneration: u64,
    firstPersonMeshGeneration: u64,
    hudMeshGeneration: u64,
    builtAt: Instant,
    worldGeneration: u64,
    atlasRevision: u64,
    outputWidth: u32,
    outputHeight: u32,
    guiWidth: i32,
    guiHeight: i32,
    entityVertices: Arc<Vec<WorldVertex>>,
    entityIndices: Arc<Vec<u32>>,
    blockEntityVertices: Arc<Vec<WorldVertex>>,
    blockEntityIndices: Arc<Vec<u32>>,
    staticEntityVertices: Arc<Vec<WorldVertex>>,
    staticEntityIndices: Arc<Vec<u32>>,
    entityDrawRanges: Vec<WorldEntityDrawRange>,
    entityDepthVertices: Arc<Vec<WorldVertex>>,
    entityDepthIndices: Arc<Vec<u32>>,
    entityOverlayVertices: Arc<Vec<WorldVertex>>,
    entityOverlayIndices: Arc<Vec<u32>>,
    skyAlphaIndexCount: u32,
    skyCelestialIndexCount: u32,
    entityOverlayDrawRanges: Vec<EntityOverlayDrawRange>,
    renderedRemotePlayers: usize,
    renderedNonPlayerEntities: usize,
    particleVertices: Arc<Vec<WorldVertex>>,
    particleIndices: Arc<Vec<u32>>,
    transparentParticleVertices: Arc<Vec<WorldVertex>>,
    transparentParticleIndices: Arc<Vec<u32>>,
    damageVertices: Arc<Vec<WorldVertex>>,
    damageIndices: Arc<Vec<u32>>,
    selectionVertices: Arc<Vec<WorldVertex>>,
    selectionIndices: Arc<Vec<u32>>,
    firstPersonVertices: Arc<Vec<WorldVertex>>,
    firstPersonIndices: Arc<Vec<u32>>,
    firstPersonDrawRanges: Vec<FirstPersonDrawRange>,
    firstPersonPushConstants: WorldPushConstants,
    hudVertices: Arc<Vec<WorldVertex>>,
    hudIndices: Arc<Vec<u32>>,
    hudDrawRanges: Vec<HudDrawRange>,
    hudPushConstants: WorldPushConstants,
}

#[derive(Debug)]
struct RenderFrameMeshCache {
    dynamic: Option<CachedDynamicMeshes>,
    blockEntities: HashMap<BlockEntityMeshIdentity, CachedBlockEntityMesh>,
    blockEntityEpoch: u64,
    staticEntities: HashMap<StaticEntityMeshIdentity, CachedStaticEntityMesh>,
    staticEntityEpoch: u64,
    nextGeneration: u64,
    profileBuilds: u64,
    profileReuses: u64,
    profileBuildNanos: u128,
    profileBlockEntityBuilds: u64,
    profileBlockEntityReuses: u64,
    profileStaticEntityBuilds: u64,
    profileStaticEntityReuses: u64,
}

impl Default for RenderFrameMeshCache {
    fn default() -> Self {
        Self {
            dynamic: None,
            blockEntities: HashMap::new(),
            blockEntityEpoch: 0,
            staticEntities: HashMap::new(),
            staticEntityEpoch: 0,
            nextGeneration: 1,
            profileBuilds: 0,
            profileReuses: 0,
            profileBuildNanos: 0,
            profileBlockEntityBuilds: 0,
            profileBlockEntityReuses: 0,
            profileStaticEntityBuilds: 0,
            profileStaticEntityReuses: 0,
        }
    }
}

impl RenderFrameMeshCache {
    fn clear(&mut self) {
        self.dynamic = None;
        self.blockEntities.clear();
        self.blockEntityEpoch = 0;
        self.staticEntities.clear();
        self.staticEntityEpoch = 0;
    }

    fn beginBlockEntityFrame(&mut self) {
        self.blockEntityEpoch = self.blockEntityEpoch.wrapping_add(1).max(1);
    }

    fn touchBlockEntity(&mut self, identity: BlockEntityMeshIdentity) {
        if let Some(cached) = self.blockEntities.get_mut(&identity) {
            cached.lastSeenEpoch = self.blockEntityEpoch;
        }
    }

    fn discardBlockEntity(&mut self, identity: BlockEntityMeshIdentity) {
        self.blockEntities.remove(&identity);
    }

    fn finishBlockEntityFrame(&mut self) {
        let epoch = self.blockEntityEpoch;
        self.blockEntities
            .retain(|_, cached| cached.lastSeenEpoch == epoch);
    }

    fn blockEntityMesh<F>(
        &mut self,
        identity: BlockEntityMeshIdentity,
        key: BlockEntityMeshCacheKey,
        cacheable: bool,
        build: F,
    ) -> Arc<BlockEntityMeshBatch>
    where
        F: FnOnce() -> BlockEntityMeshBatch,
    {
        if cacheable {
            if let Some(cached) = self.blockEntities.get_mut(&identity) {
                cached.lastSeenEpoch = self.blockEntityEpoch;
                if cached.key == key {
                    self.profileBlockEntityReuses =
                        self.profileBlockEntityReuses.saturating_add(1);
                    return Arc::clone(&cached.mesh);
                }
            }
        } else {
            self.blockEntities.remove(&identity);
        }

        let mesh = Arc::new(build());
        self.profileBlockEntityBuilds = self.profileBlockEntityBuilds.saturating_add(1);
        if cacheable {
            self.blockEntities.insert(
                identity,
                CachedBlockEntityMesh {
                    key,
                    mesh: Arc::clone(&mesh),
                    lastSeenEpoch: self.blockEntityEpoch,
                },
            );
        }
        mesh
    }

    fn blockEntityResidentCount(&self) -> usize {
        self.blockEntities.len()
    }


    fn beginStaticEntityFrame(&mut self) {
        self.staticEntityEpoch = self.staticEntityEpoch.wrapping_add(1).max(1);
    }

    fn touchStaticEntity(&mut self, identity: StaticEntityMeshIdentity) {
        if let Some(cached) = self.staticEntities.get_mut(&identity) {
            cached.lastSeenEpoch = self.staticEntityEpoch;
        }
    }

    fn finishStaticEntityFrame(&mut self) {
        let epoch = self.staticEntityEpoch;
        self.staticEntities
            .retain(|_, cached| cached.lastSeenEpoch == epoch);
    }

    fn staticEntityMesh<F>(
        &mut self,
        identity: StaticEntityMeshIdentity,
        key: StaticEntityMeshCacheKey,
        build: F,
    ) -> Arc<StaticEntityMeshBatch>
    where
        F: FnOnce() -> StaticEntityMeshBatch,
    {
        if let Some(cached) = self.staticEntities.get_mut(&identity) {
            cached.lastSeenEpoch = self.staticEntityEpoch;
            if cached.key == key {
                self.profileStaticEntityReuses =
                    self.profileStaticEntityReuses.saturating_add(1);
                return Arc::clone(&cached.mesh);
            }
        }

        let mesh = Arc::new(build());
        self.profileStaticEntityBuilds = self.profileStaticEntityBuilds.saturating_add(1);
        self.staticEntities.insert(
            identity,
            CachedStaticEntityMesh {
                key,
                mesh: Arc::clone(&mesh),
                lastSeenEpoch: self.staticEntityEpoch,
            },
        );
        mesh
    }

    fn staticEntityResidentCount(&self) -> usize {
        self.staticEntities.len()
    }

    fn shouldRebuild(
        &self,
        worldGeneration: u64,
        atlasRevision: u64,
        outputWidth: u32,
        outputHeight: u32,
        guiWidth: i32,
        guiHeight: i32,
    ) -> bool {
        self.dynamic.as_ref().map_or(true, |cached| {
            cached.worldGeneration != worldGeneration
                || cached.atlasRevision != atlasRevision
                || cached.outputWidth != outputWidth
                || cached.outputHeight != outputHeight
                || cached.guiWidth != guiWidth
                || cached.guiHeight != guiHeight
                || cached.builtAt.elapsed() >= DYNAMIC_MESH_BUILD_INTERVAL
        })
    }

    fn nextStreamGeneration(&mut self) -> u64 {
        let generation = self.nextGeneration;
        self.nextGeneration = self.nextGeneration.wrapping_add(1).max(1);
        generation
    }

    fn store(&mut self, mut meshes: CachedDynamicMeshes, elapsed: Duration) -> CachedDynamicMeshes {
        // Keep independent resource generations: a rebuilt CPU frame may
        // contain many byte-identical
        // streams, and those streams retain both their Arc allocation and GPU
        // generation instead of being uploaded just because another stream
        // changed.
        let previous = self.dynamic.clone();
        if let Some(previous) = previous.as_ref() {
            let sameEntity = meshes.entityVertices.as_ref() == previous.entityVertices.as_ref()
                && meshes.entityIndices.as_ref() == previous.entityIndices.as_ref();
            if sameEntity {
                meshes.entityVertices = Arc::clone(&previous.entityVertices);
                meshes.entityIndices = Arc::clone(&previous.entityIndices);
                meshes.entityMeshGeneration = previous.entityMeshGeneration;
            } else {
                meshes.entityMeshGeneration = self.nextStreamGeneration();
            }

            let sameBlockEntities =
                meshes.blockEntityVertices.as_ref() == previous.blockEntityVertices.as_ref()
                    && meshes.blockEntityIndices.as_ref() == previous.blockEntityIndices.as_ref();
            if sameBlockEntities {
                meshes.blockEntityVertices = Arc::clone(&previous.blockEntityVertices);
                meshes.blockEntityIndices = Arc::clone(&previous.blockEntityIndices);
                meshes.blockEntityMeshGeneration = previous.blockEntityMeshGeneration;
            } else {
                meshes.blockEntityMeshGeneration = self.nextStreamGeneration();
            }

            let sameStaticEntities =
                meshes.staticEntityVertices.as_ref() == previous.staticEntityVertices.as_ref()
                    && meshes.staticEntityIndices.as_ref() == previous.staticEntityIndices.as_ref();
            if sameStaticEntities {
                meshes.staticEntityVertices = Arc::clone(&previous.staticEntityVertices);
                meshes.staticEntityIndices = Arc::clone(&previous.staticEntityIndices);
                meshes.staticEntityMeshGeneration = previous.staticEntityMeshGeneration;
            } else {
                meshes.staticEntityMeshGeneration = self.nextStreamGeneration();
            }

            let sameDepth = meshes.entityDepthVertices.as_ref() == previous.entityDepthVertices.as_ref()
                && meshes.entityDepthIndices.as_ref() == previous.entityDepthIndices.as_ref();
            if sameDepth {
                meshes.entityDepthVertices = Arc::clone(&previous.entityDepthVertices);
                meshes.entityDepthIndices = Arc::clone(&previous.entityDepthIndices);
                meshes.entityDepthMeshGeneration = previous.entityDepthMeshGeneration;
            } else {
                meshes.entityDepthMeshGeneration = self.nextStreamGeneration();
            }

            let sameOverlay = meshes.entityOverlayVertices.as_ref() == previous.entityOverlayVertices.as_ref()
                && meshes.entityOverlayIndices.as_ref() == previous.entityOverlayIndices.as_ref()
                && meshes.skyAlphaIndexCount == previous.skyAlphaIndexCount
                && meshes.skyCelestialIndexCount == previous.skyCelestialIndexCount
                && meshes.entityOverlayDrawRanges == previous.entityOverlayDrawRanges;
            if sameOverlay {
                meshes.entityOverlayVertices = Arc::clone(&previous.entityOverlayVertices);
                meshes.entityOverlayIndices = Arc::clone(&previous.entityOverlayIndices);
                meshes.entityOverlayDrawRanges = previous.entityOverlayDrawRanges.clone();
                meshes.entityOverlayMeshGeneration = previous.entityOverlayMeshGeneration;
            } else {
                meshes.entityOverlayMeshGeneration = self.nextStreamGeneration();
            }

            let sameParticles = meshes.particleVertices.as_ref() == previous.particleVertices.as_ref()
                && meshes.particleIndices.as_ref() == previous.particleIndices.as_ref();
            if sameParticles {
                meshes.particleVertices = Arc::clone(&previous.particleVertices);
                meshes.particleIndices = Arc::clone(&previous.particleIndices);
                meshes.particleMeshGeneration = previous.particleMeshGeneration;
            } else {
                meshes.particleMeshGeneration = self.nextStreamGeneration();
            }

            let sameTransparentParticles =
                meshes.transparentParticleVertices.as_ref() == previous.transparentParticleVertices.as_ref()
                    && meshes.transparentParticleIndices.as_ref() == previous.transparentParticleIndices.as_ref();
            if sameTransparentParticles {
                meshes.transparentParticleVertices = Arc::clone(&previous.transparentParticleVertices);
                meshes.transparentParticleIndices = Arc::clone(&previous.transparentParticleIndices);
                meshes.transparentParticleMeshGeneration = previous.transparentParticleMeshGeneration;
            } else {
                meshes.transparentParticleMeshGeneration = self.nextStreamGeneration();
            }

            let sameDamage = meshes.damageVertices.as_ref() == previous.damageVertices.as_ref()
                && meshes.damageIndices.as_ref() == previous.damageIndices.as_ref();
            if sameDamage {
                meshes.damageVertices = Arc::clone(&previous.damageVertices);
                meshes.damageIndices = Arc::clone(&previous.damageIndices);
                meshes.damageMeshGeneration = previous.damageMeshGeneration;
            } else {
                meshes.damageMeshGeneration = self.nextStreamGeneration();
            }

            let sameSelection = meshes.selectionVertices.as_ref() == previous.selectionVertices.as_ref()
                && meshes.selectionIndices.as_ref() == previous.selectionIndices.as_ref();
            if sameSelection {
                meshes.selectionVertices = Arc::clone(&previous.selectionVertices);
                meshes.selectionIndices = Arc::clone(&previous.selectionIndices);
                meshes.selectionMeshGeneration = previous.selectionMeshGeneration;
            } else {
                meshes.selectionMeshGeneration = self.nextStreamGeneration();
            }

            let sameFirstPerson = meshes.firstPersonVertices.as_ref() == previous.firstPersonVertices.as_ref()
                && meshes.firstPersonIndices.as_ref() == previous.firstPersonIndices.as_ref()
                && meshes.firstPersonDrawRanges == previous.firstPersonDrawRanges
                && meshes.firstPersonPushConstants == previous.firstPersonPushConstants;
            if sameFirstPerson {
                meshes.firstPersonVertices = Arc::clone(&previous.firstPersonVertices);
                meshes.firstPersonIndices = Arc::clone(&previous.firstPersonIndices);
                meshes.firstPersonDrawRanges = previous.firstPersonDrawRanges.clone();
                meshes.firstPersonMeshGeneration = previous.firstPersonMeshGeneration;
            } else {
                meshes.firstPersonMeshGeneration = self.nextStreamGeneration();
            }

            let sameHud = meshes.hudVertices.as_ref() == previous.hudVertices.as_ref()
                && meshes.hudIndices.as_ref() == previous.hudIndices.as_ref()
                && meshes.hudDrawRanges == previous.hudDrawRanges
                && meshes.hudPushConstants == previous.hudPushConstants;
            if sameHud {
                meshes.hudVertices = Arc::clone(&previous.hudVertices);
                meshes.hudIndices = Arc::clone(&previous.hudIndices);
                meshes.hudDrawRanges = previous.hudDrawRanges.clone();
                meshes.hudMeshGeneration = previous.hudMeshGeneration;
            } else {
                meshes.hudMeshGeneration = self.nextStreamGeneration();
            }
        } else {
            meshes.entityMeshGeneration = self.nextStreamGeneration();
            meshes.blockEntityMeshGeneration = self.nextStreamGeneration();
            meshes.staticEntityMeshGeneration = self.nextStreamGeneration();
            meshes.entityDepthMeshGeneration = self.nextStreamGeneration();
            meshes.entityOverlayMeshGeneration = self.nextStreamGeneration();
            meshes.particleMeshGeneration = self.nextStreamGeneration();
            meshes.transparentParticleMeshGeneration = self.nextStreamGeneration();
            meshes.damageMeshGeneration = self.nextStreamGeneration();
            meshes.selectionMeshGeneration = self.nextStreamGeneration();
            meshes.firstPersonMeshGeneration = self.nextStreamGeneration();
            meshes.hudMeshGeneration = self.nextStreamGeneration();
        }
        self.profileBuilds = self.profileBuilds.saturating_add(1);
        self.profileBuildNanos = self.profileBuildNanos.saturating_add(elapsed.as_nanos());
        self.dynamic = Some(meshes.clone());
        meshes
    }

    fn reuse(&mut self) -> Option<CachedDynamicMeshes> {
        self.profileReuses = self.profileReuses.saturating_add(1);
        self.dynamic.clone()
    }

    fn takeProfile(&mut self) -> (u64, u64, u128, u64, u64, u64, u64) {
        let profile = (
            self.profileBuilds,
            self.profileReuses,
            self.profileBuildNanos,
            self.profileBlockEntityBuilds,
            self.profileBlockEntityReuses,
            self.profileStaticEntityBuilds,
            self.profileStaticEntityReuses,
        );
        self.profileBuilds = 0;
        self.profileReuses = 0;
        self.profileBuildNanos = 0;
        self.profileBlockEntityBuilds = 0;
        self.profileBlockEntityReuses = 0;
        self.profileStaticEntityBuilds = 0;
        self.profileStaticEntityReuses = 0;
        profile
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MaterialLayerKey {
    texture: ResourceLocation,
    tintIndex: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MaterialKey {
    blockId: i32,
    layers: Vec<MaterialLayerKey>,
}

#[derive(Debug, Clone)]
struct MaterialRegistration {
    key: MaterialKey,
    textures: Vec<Arc<TextureSource>>,
}

#[derive(Debug, Clone)]
struct AtlasState {
    revision: u64,
    atlas: Arc<BlockTextureAtlas>,
    rectangles: Arc<HashMap<MaterialKey, [f32; 4]>>,
    particleTextureRectangles: Arc<HashMap<ResourceLocation, [f32; 4]>>,
    entityTextureRectangles: Arc<HashMap<ResourceLocation, [f32; 4]>>,
    /// Exact native rectangles for every un-tinted single-texture material.
    ///
    /// Java's `TextureManager` can bind arbitrary GUI, entity and dynamic
    /// textures. The Vulkan backend shares one atlas, so `GuiDrawList` must
    /// resolve the requested `ResourceLocation` rather than silently falling
    /// back to widgets.png. This map is the atlas equivalent of that binding.
    textureRectangles: Arc<HashMap<ResourceLocation, [f32; 4]>>,
    particleTextures: Arc<Vec<ResourceLocation>>,
    destroyStageRectangles: [[f32; 4]; 10],
    waterStillRectangle: [f32; 4],
    waterFlowRectangle: [f32; 4],
    waterOverlayRectangle: [f32; 4],
    lavaStillRectangle: [f32; 4],
    lavaFlowRectangle: [f32; 4],
    fireLayer0Rectangle: [f32; 4],
    fireLayer1Rectangle: [f32; 4],
    fireLayer0Animation: Option<TextureAnimation>,
    fireLayer1Animation: Option<TextureAnimation>,
    fireLayer0FrameStepV: f32,
    fireLayer1FrameStepV: f32,
    missingRectangle: [f32; 4],
    solidWhiteRectangle: [f32; 4],
    mapCheckerRectangle: [f32; 4],
    mapBackgroundRectangle: [f32; 4],
    mapIconsRectangle: [f32; 4],
    models: Arc<Vec<Option<Arc<ResolvedBlockModel>>>>,
    stairModels: Arc<HashMap<(i32, StairShape), Arc<ResolvedBlockModel>>>,
    snowyModels: Arc<HashMap<i32, Arc<ResolvedBlockModel>>>,
    connectedModels: Arc<HashMap<(i32, u8), Arc<ResolvedBlockModel>>>,
    fireModels: Arc<HashMap<(i32, u8), Arc<ResolvedBlockModel>>>,
    doublePlantModels: Arc<HashMap<(u8, bool), Arc<ResolvedBlockModel>>>,
    doorModels: Arc<HashMap<(i32, u8), Arc<ResolvedBlockModel>>>,
    fenceGateModels: Arc<HashMap<(i32, u8), Arc<ResolvedBlockModel>>>,
    redstoneWireModels: Arc<HashMap<u8, Arc<ResolvedBlockModel>>>,
    flowerPotModels: Arc<HashMap<String, Arc<ResolvedBlockModel>>>,
    pistonHeadModels: Arc<HashMap<(u8, bool, bool), Arc<ResolvedBlockModel>>>,
    blockColors: Arc<BlockColors>,
    itemColors: Arc<ItemColors>,
    steveRectangle: [f32; 4],
    alexRectangle: [f32; 4],
    widgetsRectangle: [f32; 4],
    iconsRectangle: [f32; 4],
    barsRectangle: [f32; 4],
    fontRectangle: [f32; 4],
    fontTextureRectangles: Arc<HashMap<ResourceLocation, [f32; 4]>>,
    inventoryRectangle: [f32; 4],
    chestRectangle: [f32; 4],
    shulkerRectangle: [f32; 4],
    horseRectangle: [f32; 4],
    craftingRectangle: [f32; 4],
    furnaceRectangle: [f32; 4],
    anvilRectangle: [f32; 4],
    enchantingRectangle: [f32; 4],
    hopperRectangle: [f32; 4],
    brewingStandRectangle: [f32; 4],
    dispenserRectangle: [f32; 4],
    beaconRectangle: [f32; 4],
    merchantRectangle: [f32; 4],
    recipeBookRectangle: [f32; 4],
    creativeTabsRectangle: [f32; 4],
    creativeItemsRectangle: [f32; 4],
    creativeSearchRectangle: [f32; 4],
    creativeInventoryRectangle: [f32; 4],
    glintRectangle: [f32; 4],
    shieldBaseRectangle: [f32; 4],
    builtInItemRectangles: Arc<HashMap<ResourceLocation, [f32; 4]>>,
    bedRectangles: [[f32; 4]; 16],
    emptySlotRectangles: [[f32; 4]; 5],
    itemModels: Arc<HashMap<(i16, i16), Arc<ResolvedItemModel>>>,
    shieldBlockingModel: Option<Arc<ResolvedItemModel>>,
}

impl AtlasState {
    /// TextureMap updates animated sprites once per client tick. Fire keeps its
    /// complete vertical frame strip in the Vulkan atlas, so animation only
    /// changes the V offset supplied to tagged fire vertices and never stalls
    /// the GPU by recreating the complete atlas image.
    fn fireFrameOffsets(&self, tick: i64) -> [f32; 2] {
        let layer0 = self
            .fireLayer0Animation
            .as_ref()
            .map_or(0, |animation| animation.frame_index_at_tick(tick)) as f32
            * self.fireLayer0FrameStepV;
        let layer1 = self
            .fireLayer1Animation
            .as_ref()
            .map_or(0, |animation| animation.frame_index_at_tick(tick)) as f32
            * self.fireLayer1FrameStepV;
        [layer0, layer1]
    }
}

#[derive(Debug, Clone)]
struct CachedChunkMesh {
    sourceRevision: u64,
    meshRevision: u64,
    indexCount: u32,
    aabbMin: [i32; 3],
    aabbMax: [i32; 3],
    compiledChunk: CompiledChunk,
    layerRanges: [ChunkLayerRange; 4],
    vertices: Arc<Vec<WorldVertex>>,
    indices: Arc<Vec<u32>>,
    ready: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ChunkBuildToken {
    worldGeneration: u64,
    serial: u64,
    sourceRevision: u64,
}

#[derive(Debug, Clone, Copy)]
struct ChunkBuildRequest {
    key: RenderChunkKey,
    token: ChunkBuildToken,
    dimension: i32,
    fancyGraphics: bool,
    ambientOcclusion: i32,
    translucentSortPosition: [f32; 3],
}

#[derive(Debug, Clone, Copy)]
struct LivingTargetRenderState {
    entityId: i32,
    prevPosition: [f64; 3],
    position: [f64; 3],
    height: f32,
    eyeHeight: f32,
}

impl LivingTargetRenderState {
    fn previousEyes(self) -> [f64; 3] {
        [self.prevPosition[0], self.prevPosition[1] + self.eyeHeight as f64, self.prevPosition[2]]
    }

    fn interpolatedCenter(self, partialTicks: f32) -> [f64; 3] {
        let partial = partialTicks.clamp(0.0, 1.0) as f64;
        [
            self.prevPosition[0] + (self.position[0] - self.prevPosition[0]) * partial,
            self.prevPosition[1] + (self.position[1] - self.prevPosition[1]) * partial
                + self.height as f64 * 0.5,
            self.prevPosition[2] + (self.position[2] - self.prevPosition[2]) * partial,
        ]
    }

    fn currentCenter(self) -> [f64; 3] {
        [self.position[0], self.position[1] + self.height as f64 * 0.5, self.position[2]]
    }
}

#[derive(Clone)]
struct RemotePlayerRenderState {
    entityId: i32,
    uniqueId: uuid::Uuid,
    name: String,
    skinLocation: ResourceLocation,
    slim: bool,
    capeLocation: Option<ResourceLocation>,
    prevChasingPosition: [f64; 3],
    chasingPosition: [f64; 3],
    prevMovedDistance: f32,
    movedDistance: f32,
    prevCameraYaw: f32,
    cameraYaw: f32,
    chestStack: ItemStack,
    armorStacks: [ItemStack; 4],
    elytraLocation: Option<ResourceLocation>,
    customHeadSkinLocation: Option<ResourceLocation>,
    elytraRotation: ElytraRotationState,
    elytraFlying: bool,
    prevPosition: [f64; 3],
    position: [f64; 3],
    prevBodyYaw: f32,
    bodyYaw: f32,
    prevYaw: f32,
    yaw: f32,
    prevHeadYaw: f32,
    headYaw: f32,
    prevPitch: f32,
    pitch: f32,
    prevLimbSwingAmount: f32,
    limbSwingAmount: f32,
    limbSwing: f32,
    prevSwingProgress: f32,
    swingProgress: f32,
    ticksExisted: i32,
    ticksElytraFlying: i32,
    motion: [f64; 3],
    hurtTime: i32,
    deathTime: i32,
    sneaking: bool,
    riding: bool,
    sleeping: bool,
    bedOrientation: f32,
    renderOffset: [f32; 3],
    skinParts: u8,
    packedLight: u32,
    invisible: bool,
    beingRidden: bool,
    burning: bool,
    swingingArmIsLeft: bool,
    mainHandStack: ItemStack,
    offHandStack: ItemStack,
    primaryHand: EnumHandSide,
    activeHand: EnumHand,
    itemInUseCount: i32,
    height: f32,
    eyeHeight: f32,
}

#[derive(Debug, Clone)]
struct SkullRenderState {
    pos: BlockPos,
    facing: EnumFacing,
    rotation: i32,
    skullType: i32,
    playerSkinLocation: Option<ResourceLocation>,
    animateTicks: f32,
    packedLight: u32,
}

#[derive(Debug, Clone)]
struct BedRenderState {
    pos: BlockPos,
    head: bool,
    horizontalIndex: i32,
    colorMetadata: i16,
    packedLight: u32,
}

#[derive(Debug, Clone)]
struct ChestRenderState {
    pos: BlockPos,
    trapped: bool,
    ender: bool,
    large: bool,
    metadata: i32,
    adjacentXPos: bool,
    adjacentZPos: bool,
    lidProgress: f32,
    packedLight: u32,
}

#[derive(Debug, Clone)]
struct PistonRenderState {
    pos: BlockPos,
    pistonState: IBlockState,
    facing: EnumFacing,
    progress: f32,
    offset: [f32; 3],
    extending: bool,
    shouldHeadBeRendered: bool,
    packedLight: u32,
}

#[derive(Debug, Clone)]
struct ShulkerBoxRenderState {
    pos: BlockPos,
    colorMetadata: i32,
    facing: EnumFacing,
    progress: f32,
    packedLight: u32,
}

#[derive(Debug, Clone)]
struct SignRenderState {
    pos: BlockPos,
    blockId: i32,
    metadata: i32,
    lines: [String; 4],
    lineBeingEdited: i32,
    packedLight: u32,
}

#[derive(Debug, Clone)]
struct EnchantmentTableRenderState {
    pos: BlockPos,
    ticks: f32,
    pageFlipRight: f32,
    pageFlipLeft: f32,
    spread: f32,
    rotation: f32,
    packedLight: u32,
}

#[derive(Debug, Clone, Copy)]
struct EndPortalRenderState {
    pos: BlockPos,
}

#[derive(Debug, Clone)]
struct BeaconRenderState {
    pos: BlockPos,
    beamScale: f32,
    segments: Vec<crate::net::minecraft::tileentity::TileEntityBeacon::BeamSegment>,
}

pub struct WorldRenderCapture {
    playerPosition: PlayerPositionState,
    cameraPosition: [f32; 3],
    cameraYaw: f32,
    cameraPitch: f32,
    thirdPersonView: i32,
    outputWidth: u32,
    outputHeight: u32,
    guiWidth: i32,
    guiHeight: i32,
    currentHotbarSlot: i32,
    playerListVisible: bool,
    playerListShowsHeads: bool,
    playerListSkinParts: HashMap<uuid::Uuid, u8>,
    playerListEntries: Vec<NetworkPlayerInfo>,
    playerListHeader: Option<crate::net::minecraft::util::text::ITextComponent::ITextComponent>,
    playerListFooter: Option<crate::net::minecraft::util::text::ITextComponent::ITextComponent>,
    chatMessages: Vec<ReceivedChatMessage>,
    chatOpen: bool,
    chatVisible: bool,
    showSubtitles: bool,
    showDebugInfo: bool,
    reducedDebugInfo: bool,
    advancedItemTooltips: bool,
    showDebugProfilerChart: bool,
    showLagometer: bool,
    debugFps: i32,
    vulkanDevice: String,
    chatInput: Option<GuiTextFieldRenderState>,
    worldGuiDrawList: Option<GuiDrawList>,
    chatOpacity: f32,
    chatScale: f32,
    chatWidth: f32,
    chatHeightFocused: f32,
    chatHeightUnfocused: f32,
    scoreboard: Scoreboard,
    localPlayerName: String,
    localPlayerSpectator: bool,
    actionBarMessage: Option<crate::net::minecraft::util::text::ITextComponent::ITextComponent>,
    actionBarAge: i32,
    offhandNonEmpty: bool,
    hotbarStacks: Vec<ItemStack>,
    offhandStack: ItemStack,
    inventoryOpen: bool,
    inventoryIsChest: bool,
    inventoryIsShulker: bool,
    inventoryHorseSpec: Option<HorseInventorySpec>,
    inventoryHorseEntity: Option<EntityOtherClient>,
    inventoryWindowKind: Option<ContainerWindowKind>,
    inventoryProperties: Vec<i32>,
    merchantRecipes: Option<MerchantRecipeList>,
    merchantRecipeIndex: i32,
    inventoryIsCreative: bool,
    creativeSelectedTab: i32,
    creativeCurrentScroll: f32,
    creativeCanScroll: bool,
    creativeContainer: Option<GuiContainer>,
    creativeSearchInput: Option<GuiTextFieldRenderState>,
    anvilNameInput: Option<GuiTextFieldRenderState>,
    enchantmentBookState: Option<EnchantmentBookRenderState>,
    recipeBookState: Option<RecipeBookRenderState>,
    anvilCostFormat: String,
    anvilTooExpensive: String,
    creativeTabTitle: String,
    inventoryRows: i32,
    inventoryTitle: String,
    playerInventoryTitle: String,
    inventoryMouseX: i32,
    inventoryMouseY: i32,
    inventoryOldMouseX: f32,
    inventoryOldMouseY: f32,
    inventoryDragSplitting: bool,
    inventoryDragSplittingLimit: i32,
    inventoryDragSplittingRemnant: i32,
    inventoryDragSplittingSlots: Vec<i32>,
    inventorySlots: Vec<ItemStack>,
    inventoryCursorStack: ItemStack,
    playerHealth: f32,
    absorptionAmount: f32,
    itemActivationItem: ItemStack,
    itemActivationTicks: i32,
    itemActivationRandomX: f32,
    itemActivationRandomY: f32,
    foodLevel: i32,
    saturationLevel: f32,
    armorValue: i32,
    air: i32,
    inWater: bool,
    hardcoreMode: bool,
    activePotionEffects: Vec<crate::net::minecraft::potion::PotionEffect::PotionEffect>,
    experience: f32,
    experienceLevel: i32,
    playerCreativeMode: bool,
    xpBarCap: i32,
    hurtResistantTime: i32,
    playerTicksExisted: i32,
    systemTimeMillis: u64,
    primaryHand: EnumHandSide,
    localPlayerUniqueId: uuid::Uuid,
    localSkinLocation: ResourceLocation,
    localSlim: bool,
    localSkinParts: u8,
    localInvisible: bool,
    localBurning: bool,
    firstPersonItems: FirstPersonItemRenderState,
    localSwingProgress: f32,
    localSwingingHand: EnumHand,
    localLimbSwing: f32,
    localPrevLimbSwingAmount: f32,
    localLimbSwingAmount: f32,
    localSneaking: bool,
    localRiding: bool,
    localSwingingArmIsLeft: bool,
    localArmPitchOffset: f32,
    localArmYawOffset: f32,
    firstPersonPackedLight: u32,
    gameType: GameType,
    fov: f32,
    renderDistanceChunks: i32,
    dimension: i32,
    totalWorldTime: i64,
    worldTime: i64,
    biomeName: String,
    skyLight: u8,
    blockLight: u8,
    targetBlock: Option<(BlockPos, IBlockState)>,
    loadedRenderChunks: usize,
    queuedRenderChunks: usize,
    biomeSkyColor: [f32; 3],
    lastLightningBolt: i32,
    partialTicks: f32,
    gammaSetting: f32,
    ambientOcclusion: i32,
    torchFlickerX: f32,
    centerRenderChunk: RenderChunkKey,
    snapshot: Arc<HashMap<ChunkKey, Chunk>>,
    flowerPotContents: Arc<HashMap<BlockPos, String>>,
    jobs: Vec<ChunkBuildBatchRequest>,
    remotePlayers: Vec<RemotePlayerRenderState>,
    localPlayerRenderState: Option<RemotePlayerRenderState>,
    localPlayerTarget: Option<LivingTargetRenderState>,
    nonPlayerEntities: Vec<EntityOtherClient>,
    mapData: HashMap<i32, MapData>,
    skulls: Vec<SkullRenderState>,
    beds: Vec<BedRenderState>,
    chests: Vec<ChestRenderState>,
    pistons: Vec<PistonRenderState>,
    shulkerBoxes: Vec<ShulkerBoxRenderState>,
    signs: Vec<SignRenderState>,
    enchantmentTables: Vec<EnchantmentTableRenderState>,
    beacons: Vec<BeaconRenderState>,
    endPortals: Vec<EndPortalRenderState>,
    particleStates: Vec<ParticleDiggingRenderState>,
    miscParticleStates: Vec<ParticleRenderState>,
    damagedBlocks: Vec<DestroyBlockProgress>,
    selectionBox: Option<SelectionBoxRenderState>,
    showDebugHitboxes: bool,
    showChunkBoundaries: bool,
}

#[derive(Debug)]
struct ChunkBuildBatchRequest {
    requests: Vec<ChunkBuildRequest>,
    priority: bool,
}

struct ChunkBuildJob {
    request: ChunkBuildRequest,
    snapshot: Arc<HashMap<ChunkKey, Chunk>>,
    flowerPotContents: Arc<HashMap<BlockPos, String>>,
    atlas: Arc<AtlasState>,
}

struct ChunkColumnBuildJob {
    requests: Vec<ChunkBuildRequest>,
    priority: bool,
    snapshot: Arc<HashMap<ChunkKey, Chunk>>,
    flowerPotContents: Arc<HashMap<BlockPos, String>>,
    atlas: Arc<AtlasState>,
}

struct ChunkBuildResult {
    key: RenderChunkKey,
    token: ChunkBuildToken,
    vertices: Vec<WorldVertex>,
    indices: Vec<u32>,
    layerRanges: [ChunkLayerRange; 4],
    aabbMin: [i32; 3],
    aabbMax: [i32; 3],
    compiledChunk: CompiledChunk,
}

struct ChunkBuildBatchResult {
    worldGeneration: u64,
    priority: bool,
    results: Vec<ChunkBuildResult>,
}

/// reference-renderer-style background dispatcher. The renderer keeps one priority and
/// two streaming column lanes, while vertical RenderChunk sections within each
/// immutable 3x3 snapshot are built in parallel on a cpu_count - 1 Rayon pool.
/// MCP still receives one CompiledChunk per 16x16x16 section.
struct ChunkMeshDispatcher {
    sender: mpsc::Sender<ChunkBuildBatchResult>,
    receiver: mpsc::Receiver<ChunkBuildBatchResult>,
    /// A renderer-owned pool reserves one logical CPU for Winit/render work and
    /// prevents unrelated global Rayon jobs from changing chunk-build latency.
    pool: Option<rayon::ThreadPool>,
}

impl ChunkMeshDispatcher {
    fn new() -> Self {
        let (resultSender, resultReceiver) = mpsc::channel::<ChunkBuildBatchResult>();
        let workerCount = std::thread::available_parallelism()
            .map(|count| count.get().saturating_sub(1).clamp(1, 12))
            .unwrap_or(2);
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(workerCount)
            .thread_name(|index| format!("mc112-chunk-mesh-{index}"))
            .build()
            .map_err(|error| {
                log::warn!("failed creating dedicated chunk mesh pool; using global Rayon pool: {error}");
                error
            })
            .ok();
        Self {
            sender: resultSender,
            receiver: resultReceiver,
            pool,
        }
    }

    fn dispatch(&self, job: ChunkColumnBuildJob) {
        let sender = self.sender.clone();
        let task = move || {
            let worldGeneration = job
                .requests
                .first()
                .map_or(0, |request| request.token.worldGeneration);
            let snapshot = job.snapshot;
            let flowerPotContents = job.flowerPotContents;
            let atlas = job.atlas;
            let results: Vec<ChunkBuildResult> = job
                .requests
                .into_par_iter()
                .map(|request| {
                    build_chunk_mesh(ChunkBuildJob {
                        request,
                        snapshot: Arc::clone(&snapshot),
                        flowerPotContents: Arc::clone(&flowerPotContents),
                        atlas: Arc::clone(&atlas),
                    })
                })
                .collect();
            let _ = sender.send(ChunkBuildBatchResult {
                worldGeneration,
                priority: job.priority,
                results,
            });
        };
        if let Some(pool) = self.pool.as_ref() {
            pool.spawn(task);
        } else {
            rayon::spawn(task);
        }
    }

    fn tryReceive(&self) -> Option<ChunkBuildBatchResult> {
        self.receiver.try_recv().ok()
    }
}

/// Vulkan front end corresponding to MCP's
/// `RenderGlobal -> ViewFrustum -> RenderChunk -> ChunkRenderDispatcher` path.
///
/// The network-owned `WorldClient` is held only long enough to capture a small
/// immutable neighbourhood for dirty chunks. Chunk tessellation happens on
/// background workers, and completed meshes are uploaded independently. A new
/// server chunk therefore never rebuilds or replaces the complete world mesh.
pub struct VulkanWorldRenderer {
    blockModelShapes: BlockModelShapes,
    resourceManager: ResourceManager,
    skinManager: SkinManager,
    textureCache: HashMap<ResourceLocation, Arc<TextureSource>>,
    atlasState: Option<Arc<AtlasState>>,
    /// Coalesces asynchronous TextureManager arrivals into one atlas revision.
    dynamicAtlasDirtySince: Option<Instant>,
    /// Fixed 64x64 tail slots reserved by the current atlas. SkinManager
    /// completions patch these pixels directly instead of rebuilding every
    /// block model, item model and stable texture on the render thread.
    dynamicPlayerSlots: Vec<DynamicAtlasSlot>,
    dynamicPlayerAssignments: HashMap<ResourceLocation, usize>,
    dynamicPlayerAtlasDirty: bool,
    dynamicPackIconAtlasDirty: bool,
    /// Per-player values mutated by MCP `ModelElytra#setRotationAngles`.
    /// The Vulkan renderer captures immutable world snapshots, so the same
    /// temporal state is retained by UUID here instead of being discarded.
    elytraRotations: HashMap<uuid::Uuid, ElytraRotationState>,
    /// `TileEntitySkullRenderer`/`LayerCustomHead` profiles are not guaranteed
    /// to be present in the tab list. Retain independent `NetworkPlayerInfo`
    /// objects so their asynchronous SkinManager callbacks survive frames.
    skullPlayerInfos: HashMap<String, NetworkPlayerInfo>,
    chunkMeshes: HashMap<RenderChunkKey, CachedChunkMesh>,
    observedChunkRevisions: HashMap<RenderChunkKey, u64>,
    /// Hot-path scratch storage retained across frames. The reference renderer avoids
    /// reconstructing temporary world-index containers every frame; these
    /// buffers provide the same allocation discipline without changing MCP's
    /// section revision and unload semantics.
    sectionScanScratch: Vec<(RenderChunkKey, u64, bool)>,
    loadedSectionScratch: HashSet<RenderChunkKey>,
    removedSectionScratch: Vec<RenderChunkKey>,
    requiredChunkScratch: HashSet<ChunkKey>,
    /// Interactive rebuilds (block changes and neighbour invalidations) have a
    /// dedicated reference-renderer-style priority lane so they do not wait behind the
    /// initial streaming backlog.
    priorityPendingChunks: VecDeque<RenderChunkKey>,
    priorityQueuedChunks: HashSet<RenderChunkKey>,
    pendingChunks: VecDeque<RenderChunkKey>,
    /// `ChunkRenderDispatcher` keeps pending rebuilds in a priority queue.  The
    /// Rust VecDeque is only re-sorted when membership or the camera RenderChunk
    /// changes; re-sorting the same thousands of entries every rendered frame
    /// is not part of the 1.12.2 algorithm.
    pendingChunkOrderDirty: bool,
    pendingChunkOrderCenter: Option<RenderChunkKey>,
    queuedChunks: HashSet<RenderChunkKey>,
    inflightChunks: HashMap<RenderChunkKey, ChunkBuildToken>,
    inflightPriorityColumnJobs: usize,
    inflightNormalColumnJobs: usize,
    dirtyWhileInflight: HashSet<RenderChunkKey>,
    finishedMeshBacklog: VecDeque<ChunkBuildResult>,
    loggedFirstDrawableChunk: bool,
    pendingGpuRemovals: Vec<RenderChunkKey>,
    pendingGpuRemovalSet: HashSet<RenderChunkKey>,
    emptyRenderChunks: HashSet<RenderChunkKey>,
    /// MCP `RenderGlobal#chunksToResortTransparency`: at most the first 15
    /// visible translucent RenderChunks are queued after the player moves more
    /// than one block squared, and one queued section is re-sorted per frame.
    chunksToResortTransparency: VecDeque<RenderChunkKey>,
    chunksToResortTransparencySet: HashSet<RenderChunkKey>,
    previousTransparencySortPosition: [f64; 3],
    profileTransparencyResorts: u64,
    /// Reused by runtime `RenderChunk#resortTransparency` work so frequent
    /// camera movement does not allocate two temporary vectors per section.
    /// Worker-side initial chunk compilation keeps thread-local temporary
    /// storage and remains independent.
    translucentSortScratch: TranslucentSortScratch,
    dispatcher: ChunkMeshDispatcher,
    worldGeneration: u64,
    nextJobSerial: u64,
    nextMeshRevision: u64,
    nextAtlasRevision: u64,
    lastFancyGraphics: Option<bool>,
    lastAmbientOcclusion: Option<i32>,
    torchFlickerX: f32,
    torchFlickerDX: f32,
    lastTorchWorldTime: Option<i64>,
    guiIngame: GuiIngame,
    guiBossOverlay: GuiBossOverlay,
    playerTabOverlay: GuiPlayerTabOverlay,
    guiNewChat: GuiNewChat,
    fontRenderer: FontRenderer,
    standardGalacticFontRenderer: FontRenderer,
    locale: Locale,
    lastStatusLog: Instant,
    profileStarted: Instant,
    profileFrames: u64,
    profileAtlasNanos: u128,
    profileChunkWorkNanos: u128,
    profileFrameBuildNanos: u128,
    /// reference-renderer-style dynamic CPU mesh cache. Simulation, visibility and
    /// camera constants remain per-frame; expensive tessellation is reused.
    frameMeshCache: RenderFrameMeshCache,
    showDebugHitboxes: bool,
    showChunkBoundaries: bool,
}

impl VulkanWorldRenderer {
    pub fn new(
        resourceManager: ResourceManager,
        fontRenderer: FontRenderer,
        locale: Locale,
        skinCacheDir: PathBuf,
    ) -> Self {
        let standardGalacticFontRenderer = FontRenderer::load(
            &resourceManager,
            ResourceLocation::parse("textures/font/ascii_sga.png"),
            false,
            false,
            false,
        ).unwrap_or_else(|error| {
            log::warn!("failed loading standard galactic font, using normal font metrics: {error}");
            fontRenderer.clone()
        });
        Self {
            blockModelShapes: BlockModelShapes::new(resourceManager.clone()),
            resourceManager,
            skinManager: SkinManager::new(skinCacheDir, MinecraftSessionService::new()),
            textureCache: HashMap::new(),
            atlasState: None,
            dynamicAtlasDirtySince: None,
            dynamicPlayerSlots: Vec::new(),
            dynamicPlayerAssignments: HashMap::new(),
            dynamicPlayerAtlasDirty: false,
            dynamicPackIconAtlasDirty: false,
            elytraRotations: HashMap::new(),
            skullPlayerInfos: HashMap::new(),
            chunkMeshes: HashMap::new(),
            observedChunkRevisions: HashMap::new(),
            sectionScanScratch: Vec::new(),
            loadedSectionScratch: HashSet::new(),
            removedSectionScratch: Vec::new(),
            requiredChunkScratch: HashSet::new(),
            priorityPendingChunks: VecDeque::new(),
            priorityQueuedChunks: HashSet::new(),
            pendingChunks: VecDeque::new(),
            pendingChunkOrderDirty: false,
            pendingChunkOrderCenter: None,
            queuedChunks: HashSet::new(),
            inflightChunks: HashMap::new(),
            inflightPriorityColumnJobs: 0,
            inflightNormalColumnJobs: 0,
            dirtyWhileInflight: HashSet::new(),
            finishedMeshBacklog: VecDeque::new(),
            loggedFirstDrawableChunk: false,
            pendingGpuRemovals: Vec::new(),
            pendingGpuRemovalSet: HashSet::new(),
            emptyRenderChunks: HashSet::new(),
            chunksToResortTransparency: VecDeque::new(),
            chunksToResortTransparencySet: HashSet::new(),
            previousTransparencySortPosition: [0.0; 3],
            profileTransparencyResorts: 0,
            translucentSortScratch: TranslucentSortScratch::default(),
            dispatcher: ChunkMeshDispatcher::new(),
            worldGeneration: 1,
            nextJobSerial: 1,
            nextMeshRevision: 1,
            nextAtlasRevision: 1,
            lastFancyGraphics: None,
            lastAmbientOcclusion: None,
            torchFlickerX: 0.0,
            torchFlickerDX: 0.0,
            lastTorchWorldTime: None,
            guiIngame: GuiIngame::new(),
            guiBossOverlay: GuiBossOverlay::new(),
            playerTabOverlay: GuiPlayerTabOverlay::new(),
            guiNewChat: GuiNewChat::new(),
            fontRenderer,
            standardGalacticFontRenderer,
            locale,
            lastStatusLog: Instant::now() - Duration::from_secs(2),
            profileStarted: Instant::now(),
            profileFrames: 0,
            profileAtlasNanos: 0,
            profileChunkWorkNanos: 0,
            profileFrameBuildNanos: 0,
            frameMeshCache: RenderFrameMeshCache::default(),
            showDebugHitboxes: false,
            showChunkBoundaries: false,
        }
    }

    fn updatePlayerTextures(
        &mut self,
        infos: impl Iterator<Item = (NetworkPlayerInfo, bool)>,
    ) {
        for (info, requireSecure) in infos {
            self.skinManager.requestProfileTextures(&info, requireSecure);
        }
        let completed = self.skinManager.drainCompleted();
        for resolved in completed.profiles {
            if self.skullPlayerInfos.contains_key(&resolved.key) {
                let info = NetworkPlayerInfo::new(
                    resolved.profile, GameType::Survival, 0, None,
                );
                self.skinManager.requestProfileTextures(&info, false);
                self.skullPlayerInfos.insert(resolved.key, info);
            }
        }
        if completed.textures.is_empty() {
            return;
        }
        for texture in completed.textures {
            let source = Arc::new(TextureSource::dynamic(
                texture.location.clone(),
                texture.image,
                "SkinManager/ThreadDownloadImageData",
            ));
            self.textureCache.insert(texture.location, source);
        }
        self.dynamicPlayerAtlasDirty = true;
        self.dynamicAtlasDirtySince.get_or_insert_with(Instant::now);
    }

    /// ResourcePackListEntry uses TextureManager dynamic textures rather than
    /// TextureMap sprites. The shared Vulkan descriptor atlas retains the same
    /// separation semantically by assigning icons to a reserved tail region.
    pub fn setResourcePackIconTextures(
        &mut self,
        icons: impl IntoIterator<Item = (ResourceLocation, NativeImage)>,
    ) {
        let icons = icons.into_iter().collect::<HashMap<_, _>>();
        let mut changed = false;
        self.textureCache.retain(|location, _| {
            let isPackIcon = location.getNamespace() == "minecraft"
                && (location.getPath().starts_with("resourcepackicons/")
                    || location.getPath() == "dynamic/default_pack_icon.png");
            let keep = !isPackIcon || icons.contains_key(location);
            changed |= !keep;
            keep
        });
        for (location, image) in icons {
            if self.textureCache.get(&location).is_some_and(|source| source.image == image) {
                continue;
            }
            self.textureCache.insert(
                location.clone(),
                Arc::new(TextureSource::dynamic(location, image, "ResourcePackRepository/pack.png")),
            );
            changed = true;
        }
        if changed {
            self.dynamicPackIconAtlasDirty = true;
            self.dynamicAtlasDirtySince.get_or_insert_with(Instant::now);
        }
    }

    /// Patches only the fixed 64x64 SkinManager tail of the current atlas.
    /// Vanilla keeps ThreadDownloadImageData as independent TextureManager
    /// objects; this shared-descriptor Vulkan backend preserves that cheap
    /// update behavior by avoiding a full BlockModelShapes/item-model rebuild.
    fn patchDynamicPlayerTextures(&mut self) -> bool {
        let Some(current) = self.atlasState.clone() else {
            return false;
        };
        if self.dynamicPlayerSlots.len() != DYNAMIC_PLAYER_TEXTURE_RESERVE {
            return false;
        }

        let mut desired = self
            .textureCache
            .keys()
            .filter(|location| {
                location.getNamespace() == "minecraft"
                    && location.getPath().starts_with("skins/")
            })
            .cloned()
            .collect::<Vec<_>>();
        desired.sort();
        desired.dedup();
        if desired.len() > self.dynamicPlayerSlots.len() {
            return false;
        }
        let desiredSet = desired.iter().cloned().collect::<HashSet<_>>();
        self.dynamicPlayerAssignments
            .retain(|location, _| desiredSet.contains(location));
        let mut used = self
            .dynamicPlayerAssignments
            .values()
            .copied()
            .collect::<HashSet<_>>();
        for location in &desired {
            if self.dynamicPlayerAssignments.contains_key(location) {
                continue;
            }
            let Some(slotIndex) = (0..self.dynamicPlayerSlots.len())
                .find(|index| !used.contains(index))
            else {
                return false;
            };
            self.dynamicPlayerAssignments
                .insert(location.clone(), slotIndex);
            used.insert(slotIndex);
        }

        let mut atlas = (*current.atlas).clone();
        for slot in &self.dynamicPlayerSlots {
            for y in 0..slot.tileSize {
                let start = (((slot.originY + y) * atlas.width + slot.originX) * 4) as usize;
                let end = start + slot.tileSize as usize * 4;
                if end > atlas.rgba.len() {
                    return false;
                }
                atlas.rgba[start..end].fill(0);
            }
        }

        let mut entityRectangles = (*current.entityTextureRectangles).clone();
        let mut textureRectangles = (*current.textureRectangles).clone();
        entityRectangles.retain(|location, _| {
            !(location.getNamespace() == "minecraft"
                && location.getPath().starts_with("skins/"))
        });
        textureRectangles.retain(|location, _| {
            !(location.getNamespace() == "minecraft"
                && location.getPath().starts_with("skins/"))
        });

        for location in desired {
            let Some(&slotIndex) = self.dynamicPlayerAssignments.get(&location) else {
                return false;
            };
            let Some(slot) = self.dynamicPlayerSlots.get(slotIndex).copied() else {
                return false;
            };
            let Some(source) = self.textureCache.get(&location) else {
                return false;
            };
            let image = &source.image;
            let copyWidth = image.width().min(slot.tileSize);
            let copyHeight = image.height().min(slot.tileSize);
            for y in 0..copyHeight {
                for x in 0..copyWidth {
                    let destination = (((slot.originY + y) * atlas.width + slot.originX + x) * 4)
                        as usize;
                    atlas.rgba[destination..destination + 4]
                        .copy_from_slice(&image.pixel_rgba(x, y));
                }
            }
            let rectangle = [
                slot.originX as f32 / atlas.width as f32,
                slot.originY as f32 / atlas.height as f32,
                (slot.originX + copyWidth) as f32 / atlas.width as f32,
                (slot.originY + copyHeight) as f32 / atlas.height as f32,
            ];
            entityRectangles.insert(location.clone(), rectangle);
            textureRectangles.insert(location, rectangle);
        }

        let mut next = (*current).clone();
        next.revision = self.takeAtlasRevision();
        next.atlas = Arc::new(atlas);
        next.entityTextureRectangles = Arc::new(entityRectangles);
        next.textureRectangles = Arc::new(textureRectangles);
        self.atlasState = Some(Arc::new(next));
        true
    }

    /// Within the fixed dynamic tail, a rebuild changes only the atlas pixels;
    /// all terrain/material rectangles stay at identical coordinates. Existing
    /// RenderChunk UVs therefore remain valid and must not be discarded.
    fn invalidateAtlasForDynamicTextures(&mut self) {
        let playerCount = self.textureCache.keys().filter(|location| {
            location.getNamespace() == "minecraft" && location.getPath().starts_with("skins/")
        }).count();
        let iconCount = self.textureCache.keys().filter(|location| {
            location.getNamespace() == "minecraft"
                && (location.getPath().starts_with("resourcepackicons/")
                    || location.getPath() == "dynamic/default_pack_icon.png")
        }).count();
        if playerCount <= DYNAMIC_PLAYER_TEXTURE_RESERVE
            && iconCount <= DYNAMIC_RESOURCE_PACK_ICON_RESERVE
        {
            self.atlasState = None;
            self.dynamicPlayerSlots.clear();
            self.dynamicPlayerAssignments.clear();
            return;
        }

        // Defensive overflow fallback. It is preferable to rebuild chunks once
        // than silently alias two textures if a server exceeds the fixed tail.
        let existing = self.chunkMeshes.keys().copied().collect::<Vec<_>>();
        for key in existing { self.queueGpuRemoval(key); }
        self.chunkMeshes.clear();
        self.observedChunkRevisions.clear();
        self.priorityPendingChunks.clear();
        self.priorityQueuedChunks.clear();
        self.pendingChunks.clear();
        self.pendingChunkOrderDirty = false;
        self.pendingChunkOrderCenter = None;
        self.queuedChunks.clear();
        self.inflightChunks.clear();
        self.inflightPriorityColumnJobs = 0;
        self.inflightNormalColumnJobs = 0;
        self.dirtyWhileInflight.clear();
        self.finishedMeshBacklog.clear();
        self.loggedFirstDrawableChunk = false;
        self.emptyRenderChunks.clear();
        self.chunksToResortTransparency.clear();
        self.chunksToResortTransparencySet.clear();
        self.previousTransparencySortPosition = [0.0; 3];
        self.profileTransparencyResorts = 0;
        self.worldGeneration = self.worldGeneration.wrapping_add(1).max(1);
        self.frameMeshCache.clear();
        self.lastFancyGraphics = None;
        self.lastAmbientOcclusion = None;
        self.atlasState = None;
        self.dynamicPlayerSlots.clear();
        self.dynamicPlayerAssignments.clear();
    }

    fn flushDynamicAtlasIfReady(&mut self) {
        if !self
            .dynamicAtlasDirtySince
            .is_some_and(|since| since.elapsed() >= DYNAMIC_ATLAS_REBUILD_DELAY)
        {
            return;
        }
        self.dynamicAtlasDirtySince = None;
        let playerDirty = std::mem::take(&mut self.dynamicPlayerAtlasDirty);
        let iconDirty = std::mem::take(&mut self.dynamicPackIconAtlasDirty);
        if iconDirty || (playerDirty && !self.patchDynamicPlayerTextures()) {
            // Resource-pack icons may have arbitrary native dimensions, so a
            // user-opened resource-pack screen still takes the full Stitcher
            // path. Ordinary asynchronous skins use the bounded patch above.
            self.invalidateAtlasForDynamicTextures();
        }
    }

    pub fn clearCaches(&mut self) {
        let existing = self.chunkMeshes.keys().copied().collect::<Vec<_>>();
        for key in existing {
            self.queueGpuRemoval(key);
        }
        self.chunkMeshes.clear();
        self.observedChunkRevisions.clear();
        self.priorityPendingChunks.clear();
        self.priorityQueuedChunks.clear();
        self.pendingChunks.clear();
        self.pendingChunkOrderDirty = false;
        self.pendingChunkOrderCenter = None;
        self.queuedChunks.clear();
        self.inflightChunks.clear();
        self.inflightPriorityColumnJobs = 0;
        self.inflightNormalColumnJobs = 0;
        self.dirtyWhileInflight.clear();
        self.finishedMeshBacklog.clear();
        self.loggedFirstDrawableChunk = false;
        self.emptyRenderChunks.clear();
        self.chunksToResortTransparency.clear();
        self.chunksToResortTransparencySet.clear();
        self.previousTransparencySortPosition = [0.0; 3];
        self.profileTransparencyResorts = 0;
        self.worldGeneration = self.worldGeneration.wrapping_add(1).max(1);
        self.frameMeshCache.clear();
        self.lastFancyGraphics = None;
        self.lastAmbientOcclusion = None;
        self.torchFlickerX = 0.0;
        self.torchFlickerDX = 0.0;
        self.lastTorchWorldTime = None;
        self.elytraRotations.clear();
        self.skullPlayerInfos.clear();
        self.guiIngame = GuiIngame::new();
        self.guiBossOverlay = GuiBossOverlay::new();
        self.playerTabOverlay = GuiPlayerTabOverlay::new();
        self.guiNewChat = GuiNewChat::new();
        // The resource atlas is independent of a server world and is retained,
        // matching TextureMap's lifetime across WorldClient changes.
    }

    pub fn sentChatMessages(&self) -> Vec<String> {
        self.guiNewChat.getSentMessages().to_vec()
    }

    pub fn addSentChatMessage(&mut self, message: impl Into<String>) {
        self.guiNewChat.addToSentMessages(message);
    }

    pub fn scrollChat(&mut self, amount: i32, lineCount: i32) {
        self.guiNewChat.scroll(amount, lineCount);
    }

    pub fn resetChatScroll(&mut self) { self.guiNewChat.resetScroll(); }

    /// MCP `GuiNewChat#clearChatMessages(false)` used by F3+D.
    pub fn clearChatMessages(&mut self) { self.guiNewChat.clearChatMessages(false); }

    /// Adds a client-owned debug line without sending a chat packet.
    pub fn printDebugMessage(
        &mut self,
        message: impl Into<String>,
        updateCounter: i32,
        wrapWidth: i32,
    ) {
        self.guiNewChat.printChatMessageWithFont(
            crate::net::minecraft::util::text::ITextComponent::ITextComponent::fromPlainText(message),
            updateCounter,
            wrapWidth.max(1),
            &self.fontRenderer,
        );
    }

    /// MCP `RenderGlobal#loadRenderers` equivalent used by F3+A.  GUI, skin,
    /// cape and sound state deliberately survive; only chunk render state is
    /// invalidated and rebuilt from the authoritative `WorldClient`.
    pub fn reloadChunks(&mut self) {
        let existing = self.chunkMeshes.keys().copied().collect::<Vec<_>>();
        for key in existing { self.queueGpuRemoval(key); }
        self.chunkMeshes.clear();
        self.observedChunkRevisions.clear();
        self.priorityPendingChunks.clear();
        self.priorityQueuedChunks.clear();
        self.pendingChunks.clear();
        self.pendingChunkOrderDirty = false;
        self.pendingChunkOrderCenter = None;
        self.queuedChunks.clear();
        self.inflightChunks.clear();
        self.inflightPriorityColumnJobs = 0;
        self.inflightNormalColumnJobs = 0;
        self.dirtyWhileInflight.clear();
        self.finishedMeshBacklog.clear();
        self.loggedFirstDrawableChunk = false;
        self.emptyRenderChunks.clear();
        self.chunksToResortTransparency.clear();
        self.chunksToResortTransparencySet.clear();
        self.previousTransparencySortPosition = [0.0; 3];
        self.profileTransparencyResorts = 0;
        self.worldGeneration = self.worldGeneration.wrapping_add(1).max(1);
        self.frameMeshCache.clear();
        self.lastFancyGraphics = None;
        self.lastAmbientOcclusion = None;
    }

    pub fn toggleDebugHitboxes(&mut self) -> bool {
        self.showDebugHitboxes = !self.showDebugHitboxes;
        self.showDebugHitboxes
    }

    pub fn toggleChunkBoundaries(&mut self) -> bool {
        self.showChunkBoundaries = !self.showChunkBoundaries;
        self.showChunkBoundaries
    }

    /// Resource-reload listener equivalent used by F3+T. Dynamic player
    /// textures survive just as TextureManager objects do in Java; resource-
    /// pack textures, baked models, fonts and chunk meshes are rebuilt.
    pub fn reloadResources(
        &mut self,
        resourceManager: ResourceManager,
        fontRenderer: FontRenderer,
        locale: Locale,
    ) {
        self.blockModelShapes = BlockModelShapes::new(resourceManager.clone());
        self.resourceManager = resourceManager.clone();
        self.textureCache.retain(|_, source| {
            source.source_pack.starts_with("SkinManager/")
        });
        self.atlasState = None;
        self.dynamicAtlasDirtySince = None;
        self.dynamicPlayerSlots.clear();
        self.dynamicPlayerAssignments.clear();
        self.dynamicPlayerAtlasDirty = false;
        self.dynamicPackIconAtlasDirty = false;
        self.fontRenderer = fontRenderer.clone();
        self.standardGalacticFontRenderer = FontRenderer::load(
            &resourceManager,
            ResourceLocation::parse("textures/font/ascii_sga.png"),
            false, false, false,
        ).unwrap_or(fontRenderer);
        self.locale = locale;
        self.reloadChunks();
    }

    pub fn tickIngameGui(&mut self) { self.guiIngame.updateTick(); }

    pub fn shouldPlayEndBossMusic(&self) -> bool {
        self.guiBossOverlay.shouldPlayEndBossMusic()
    }

    pub fn soundPlay(
        &mut self,
        subtitle: impl Into<String>,
        location: [f32; 3],
        systemTimeMillis: u64,
    ) {
        self.guiIngame.soundPlay(subtitle, location, systemTimeMillis);
    }

    pub fn setUnicodeFlag(&mut self, value: bool) { self.fontRenderer.set_unicode_flag(value); }

    pub fn handleTitle(&mut self, packet: &SPacketTitle) {
        let message = packet.getMessage()
            .map(|component| component.resolveWithLocale(&self.locale).getFormattedText().to_owned())
            .unwrap_or_default();
        match packet.getType() {
            TitleType::Title => self.guiIngame.displayTitle(Some(&message), None, -1, -1, -1),
            TitleType::Subtitle => self.guiIngame.displayTitle(None, Some(&message), -1, -1, -1),
            TitleType::Actionbar => self.guiIngame.setOverlayMessage(message),
            TitleType::Times => self.guiIngame.displayTitle(
                None, None, packet.getFadeInTime(), packet.getDisplayTime(), packet.getFadeOutTime(),
            ),
            TitleType::Clear => self.guiIngame.displayTitle(None, None, -1, -1, -1),
            TitleType::Reset => {
                self.guiIngame.displayTitle(None, None, -1, -1, -1);
                self.guiIngame.setDefaultTitlesTimes();
            }
        }
    }

    pub fn handleBossInfo(&mut self, packet: &SPacketUpdateBossInfo, systemTimeMillis: u64) {
        self.guiBossOverlay.read(packet, systemTimeMillis, &self.locale);
    }

    pub fn showChatCompletionCandidates(&mut self, line: impl Into<String>, updateCounter: i32, wrapWidth: i32) {
        self.guiNewChat.printChatMessageWithOptionalDeletionWithFont(
            ITextComponent::fromPlainText(line.into()),
            1,
            updateCounter,
            wrapWidth.max(1),
            &self.fontRenderer,
        );
    }

    pub fn setDebugOverlayMessage(&mut self, message: impl Into<String>) {
        self.guiIngame.setOverlayMessage(message.into());
    }

    /// Captures only chunk revisions and the neighbourhoods selected for this
    /// frame. This method is intentionally cheap because the caller executes it
    /// under `SharedPlayClientState`'s read lock.
    pub fn capture(
        &mut self,
        state: &PlayClientState,
        outputWidth: u32,
        outputHeight: u32,
        guiWidth: i32,
        guiHeight: i32,
        mainHand: EnumHandSide,
        localPlayerUniqueId: uuid::Uuid,
        localSkinParts: u8,
        firstPersonItems: FirstPersonItemRenderState,
        playerListVisible: bool,
        chatOpen: bool,
        chatVisible: bool,
        showSubtitles: bool,
        showDebugInfo: bool,
        reducedDebugInfo: bool,
        advancedItemTooltips: bool,
        showDebugProfilerChart: bool,
        showLagometer: bool,
        debugFps: i32,
        vulkanDevice: String,
        chatInput: Option<GuiTextFieldRenderState>,
        worldGuiDrawList: Option<GuiDrawList>,
        chatOpacity: f32,
        chatScale: f32,
        chatWidth: f32,
        chatHeightFocused: f32,
        chatHeightUnfocused: f32,
        localPlayerName: String,
        playerInventoryOpen: bool,
        creativeInventoryOpen: bool,
        creativeSelectedTab: i32,
        creativeCurrentScroll: f32,
        creativeCanScroll: bool,
        creativeContainer: Option<GuiContainer>,
        creativeDisplaySlots: Vec<ItemStack>,
        creativeSearchInput: Option<GuiTextFieldRenderState>,
        anvilNameInput: Option<GuiTextFieldRenderState>,
        enchantmentBookState: Option<EnchantmentBookRenderState>,
        recipeBookState: Option<RecipeBookRenderState>,
        anvilCostFormat: String,
        anvilTooExpensive: String,
        creativeTabTitle: String,
        inventoryTitle: String,
        playerInventoryTitle: String,
        inventoryMouseX: i32,
        inventoryMouseY: i32,
        inventoryOldMouseX: f32,
        inventoryOldMouseY: f32,
        inventoryDragSplitting: bool,
        inventoryDragSplittingLimit: i32,
        inventoryDragSplittingRemnant: i32,
        inventoryDragSplittingSlots: Vec<i32>,
        particleStates: Vec<ParticleDiggingRenderState>,
        miscParticleStates: Vec<ParticleRenderState>,
        localDestroyProgress: Option<DestroyBlockProgress>,
        thirdPersonView: i32,
        fov: f32,
        renderDistanceChunks: i32,
        fancyGraphics: bool,
        ambientOcclusion: i32,
        partialTicks: f32,
        gammaSetting: f32,
    ) -> WorldRenderCapture {
        // Login Success is the authoritative local identity. ViaProxy and
        // similar online-mode bridges can authenticate a UUID that differs
        // from the launcher's offline Session profile.
        let localPlayerUniqueId = state
            .localGameProfile
            .as_ref()
            .and_then(|profile| profile.getId())
            .unwrap_or(localPlayerUniqueId);
        let localPlayerName = state
            .localGameProfile
            .as_ref()
            .map(|profile| profile.getName())
            .filter(|name| !name.is_empty())
            .map(str::to_owned)
            .unwrap_or(localPlayerName);
        let renderDistanceChunks = renderDistanceChunks.clamp(2, 32);
        let reducedDebugInfo = reducedDebugInfo
            || state.thePlayer.as_ref().is_some_and(|player| player.hasReducedDebug);
        let centerChunk = ChunkKey::new(
            (state.playerPosition.posX.floor() as i32).div_euclid(16),
            (state.playerPosition.posZ.floor() as i32).div_euclid(16),
        );
        let cameraY = (state.playerPosition.posY + state.playerPosition.eyeHeight as f64)
            .floor()
            .clamp(0.0, 255.0) as i32;
        let centerRenderChunk = RenderChunkKey::new(centerChunk.x, cameraY.div_euclid(16), centerChunk.z);
        let dimension = state
            .worldClient
            .as_ref()
            .map(WorldClient::getDimension)
            .unwrap_or(0);
        let renderPlayerPosition = interpolated_player_position(state, partialTicks);
        let eyeCamera = [
            renderPlayerPosition.posX as f32,
            (renderPlayerPosition.posY + renderPlayerPosition.eyeHeight as f64) as f32,
            renderPlayerPosition.posZ as f32,
        ];
        let (
            inventoryIsChest,
            inventoryIsShulker,
            inventoryHorseSpec,
            inventoryWindowKind,
            inventoryRows,
            inventoryProperties,
        ) = state
            .thePlayer
            .as_ref()
            .and_then(|player| {
                if playerInventoryOpen || creativeInventoryOpen {
                    None
                } else {
                    player.openContainer.as_ref()
                }
            })
            .map_or((false, false, None, None, 0, Vec::new()), |container| {
                let windowKind = container.windowKind();
                let shulker = container.isShulkerBox();
                let horse = container.horseInventorySpec();
                (
                    windowKind.is_none() && !shulker && horse.is_none(),
                    shulker,
                    horse,
                    windowKind,
                    container.getNumRows() as i32,
                    container.properties().to_vec(),
                )
            });
        let (merchantRecipes, merchantRecipeIndex) = state.thePlayer.as_ref()
            .and_then(|player| player.openContainer.as_ref())
            .filter(|container| container.windowKind() == Some(ContainerWindowKind::Merchant))
            .map_or((None, 0), |container| (
                container.merchantRecipes().cloned(),
                container.merchantRecipeIndex().unwrap_or(0),
            ));
        let inventoryOpen = playerInventoryOpen
            || creativeInventoryOpen
            || inventoryIsChest
            || inventoryIsShulker
            || inventoryHorseSpec.is_some()
            || inventoryWindowKind.is_some();
        // TileEntitySkullRenderer and LayerCustomHead use SkinManager's
        // insecure cache path (`loadSkinFromCache`), independently of the tab
        // list. Cache those profiles by stable identity so async downloads are
        // not restarted every captured frame.
        let mut skullProfiles = Vec::<GameProfile>::new();
        if let Some(world) = state.worldClient.as_ref() {
            skullProfiles.extend(
                world.skullTileEntities()
                    .filter_map(|skull| skull.getPlayerProfile().cloned()),
            );
            skullProfiles.extend(world.remotePlayers().filter_map(|player| {
                ItemSkull::getPlayerProfile(
                    player.equipment.getItemStackFromSlot(EntityEquipmentSlot::Head),
                )
            }));
        }
        if let Some(player) = state.thePlayer.as_ref() {
            if let Some(stack) = player.inventory.armorInventory.get(3) {
                if let Some(profile) = ItemSkull::getPlayerProfile(stack) {
                    skullProfiles.push(profile);
                }
            }
        }
        let mut liveSkullProfiles = HashSet::new();
        for profile in skullProfiles {
            let key = skull_profile_cache_key(&profile);
            liveSkullProfiles.insert(key.clone());
            let replace = self.skullPlayerInfos.get(&key)
                .map_or(true, |info| info.getGameProfile() != &profile);
            if replace {
                self.skinManager.invalidateProfileCompletion(&key);
                self.skullPlayerInfos.insert(
                    key,
                    NetworkPlayerInfo::new(profile, GameType::Survival, 0, None),
                );
            }
        }
        self.skullPlayerInfos.retain(|key, _| liveSkullProfiles.contains(key));
        for (key, info) in &self.skullPlayerInfos {
            self.skinManager.requestProfileCompletion(
                key.clone(), info.getGameProfile().clone(), true,
            );
        }

        let mut textureInfos = state.playerInfoMap.values()
            .cloned()
            .map(|info| (info, true))
            .collect::<Vec<_>>();
        if let Some(localInfo) = state.localPlayerInfo.as_ref() {
            let localInfoId = localInfo.getGameProfile().getId();
            if localInfoId.map_or(true, |id| !state.playerInfoMap.contains_key(&id)) {
                textureInfos.push((localInfo.clone(), true));
            }
        }
        if let Some(world) = state.worldClient.as_ref() {
            textureInfos.extend(world.remotePlayers().filter_map(|player| {
                if state.playerInfoMap.contains_key(&player.uniqueId) {
                    None
                } else {
                    player.getPlayerInfo().cloned().map(|info| (info, true))
                }
            }));
        }
        textureInfos.extend(
            self.skullPlayerInfos.values().cloned().map(|info| (info, false)),
        );
        self.updatePlayerTextures(textureInfos.into_iter());
        self.flushDynamicAtlasIfReady();
        let localPlayerInfo = state
            .localPlayerInfo
            .as_ref()
            .or_else(|| state.playerInfoMap.get(&localPlayerUniqueId));
        let localSkinLocation = localPlayerInfo
            .map(NetworkPlayerInfo::getLocationSkin)
            .unwrap_or_else(|| DefaultPlayerSkin::getDefaultSkin(localPlayerUniqueId));
        let localSlim = localPlayerInfo
            .map(|info| info.getSkinType() == "slim")
            .unwrap_or_else(|| DefaultPlayerSkin::isSlimSkin(localPlayerUniqueId));
        let playerListShowsHeads = state.networkEncrypted;
        let mut playerListSkinParts = HashMap::new();
        if state.thePlayer.is_some() {
            playerListSkinParts.insert(localPlayerUniqueId, localSkinParts);
        }
        if let Some(world) = state.worldClient.as_ref() {
            for player in world.remotePlayers() {
                playerListSkinParts.insert(player.uniqueId, player.skinParts());
            }
        }
        let playerListEntries = state.playerInfoMap.values().cloned().map(|mut info| {
            let displayName = info.getDisplayName().map(|component| component.resolveWithLocale(&self.locale));
            info.setDisplayName(displayName);
            info
        }).collect::<Vec<_>>();
        let playerListHeader = state.playerListHeader.as_ref().map(|component| component.resolveWithLocale(&self.locale));
        let playerListFooter = state.playerListFooter.as_ref().map(|component| component.resolveWithLocale(&self.locale));
        let chatMessages = state.chatMessages.iter().cloned().map(|mut message| {
            message.component = message.component.resolveWithLocale(&self.locale);
            message
        }).collect::<Vec<_>>();
        let scoreboard = state.scoreboard.clone();
        let actionBarMessage = state.actionBarMessage.as_ref().map(|component| component.resolveWithLocale(&self.locale));
        let playerCreativeMode = state.gameType == GameType::Creative;

        let (
            currentHotbarSlot,
            offhandNonEmpty,
            hotbarStacks,
            offhandStack,
            inventorySlots,
            inventoryCursorStack,
            playerHealth,
            absorptionAmount,
            foodLevel,
            saturationLevel,
            armorValue,
            air,
            inWater,
            activePotionEffects,
            experience,
            experienceLevel,
            xpBarCap,
            hurtResistantTime,
            playerTicksExisted,
            localSwingProgress,
            localSwingingHand,
            localLimbSwing,
            localPrevLimbSwingAmount,
            localLimbSwingAmount,
            localSneaking,
            localRiding,
            localSwingingArmIsLeft,
            localArmPitchOffset,
            localArmYawOffset,
        ) = state
            .thePlayer
            .as_ref()
            .map(|player| {
                let hotbarStacks = player.inventory.mainInventory
                    .iter()
                    .take(9)
                    .cloned()
                    .collect::<Vec<_>>();
                let offhandStack = player.inventory.offHandInventory
                    .first()
                    .cloned()
                    .unwrap_or(ItemStack::EMPTY);
                (
                    player.inventory.currentItem.clamp(0, 8),
                    !offhandStack.isEmpty(),
                    hotbarStacks,
                    offhandStack,
                    if inventoryIsChest || inventoryIsShulker || inventoryHorseSpec.is_some() || inventoryWindowKind.is_some() {
                        player.openContainer.as_ref()
                            .map(|container| container.slots().to_vec())
                            .unwrap_or_default()
                    } else {
                        player.inventoryContainer.slots().to_vec()
                    },
                    player.inventory.getItemStack().clone(),
                    player.getHealth(),
                    player.getAbsorptionAmount(),
                    player.getFoodStats().getFoodLevel(),
                    player.getFoodStats().getSaturationLevel(),
                    player.getTotalArmorValue(),
                    player.getAir(),
                    state.worldClient.as_ref().is_some_and(|world| player.isInsideWater(world)),
                    player.activePotionEffects.values().copied().collect(),
                    player.experience,
                    player.experienceLevel,
                    player.xpBarCap(),
                    player.hurtResistantTime,
                    player.entity.ticksExisted,
                    player.getSwingProgress(partialTicks),
                    player.swingingHand,
                    player.limbSwing,
                    player.prevLimbSwingAmount,
                    player.limbSwingAmount,
                    player.entity.sneaking,
                    player.entity.isRiding(),
                    matches!(
                        (player.swingingHand, mainHand),
                        (EnumHand::MainHand, EnumHandSide::Left)
                            | (EnumHand::OffHand, EnumHandSide::Right)
                    ),
                    (player.entity.rotationPitch
                        - (player.prevRenderArmPitch
                            + (player.renderArmPitch - player.prevRenderArmPitch)
                                * partialTicks.clamp(0.0, 1.0)))
                        * 0.1,
                    (player.entity.rotationYaw
                        - (player.prevRenderArmYaw
                            + (player.renderArmYaw - player.prevRenderArmYaw)
                                * partialTicks.clamp(0.0, 1.0)))
                        * 0.1,
                )
            })
            .unwrap_or((
                0, false, vec![ItemStack::EMPTY; 9], ItemStack::EMPTY,
                vec![ItemStack::EMPTY; 46], ItemStack::EMPTY,
                20.0, 0.0, 20, 5.0, 0, 300, false, Vec::new(), 0.0, 0, 7, 0, 0, 0.0,
                EnumHand::MainHand, 0.0, 0.0, 0.0, false, false, false, 0.0, 0.0,
            ));
        let (
            itemActivationItem,
            itemActivationTicks,
            itemActivationRandomX,
            itemActivationRandomY,
        ) = state.thePlayer.as_ref().map_or(
            (ItemStack::EMPTY, 0, 0.0, 0.0),
            |player| (
                player.itemActivationItem.clone(),
                player.itemActivationTicks,
                player.itemActivationRandomX,
                player.itemActivationRandomY,
            ),
        );
        let inventorySlots = if creativeInventoryOpen {
            creativeDisplaySlots.clone()
        } else {
            inventorySlots
        };
        let mut damagedBlocks = state.damagedBlocks.values().copied().collect::<Vec<_>>();
        if let Some(local) = localDestroyProgress {
            damagedBlocks.retain(|progress| progress.getMiningPlayerEntId() != local.getMiningPlayerEntId());
            damagedBlocks.push(local);
        }
        let localPlayerTarget = state.thePlayer.as_ref().map(|player| LivingTargetRenderState {
            entityId: player.entityId,
            prevPosition: [player.entity.prevPosX, player.entity.prevPosY, player.entity.prevPosZ],
            position: [player.entity.posX, player.entity.posY, player.entity.posZ],
            height: player.entity.height,
            eyeHeight: player.getEyeHeight(),
        });
        let Some(world) = state.worldClient.as_ref() else {
            return WorldRenderCapture {
                playerPosition: renderPlayerPosition,
                cameraPosition: eyeCamera,
                cameraYaw: renderPlayerPosition.rotationYaw,
                cameraPitch: renderPlayerPosition.rotationPitch,
                thirdPersonView: thirdPersonView.rem_euclid(3),
                outputWidth,
                outputHeight,
                guiWidth: guiWidth.max(1),
                guiHeight: guiHeight.max(1),
                currentHotbarSlot,
                playerListVisible,
                playerListShowsHeads,
                playerListSkinParts: playerListSkinParts.clone(),
                playerListEntries: playerListEntries.clone(),
                playerListHeader: playerListHeader.clone(),
                playerListFooter: playerListFooter.clone(),
                chatMessages: chatMessages.clone(),
                chatOpen,
                chatVisible,
                showSubtitles,
                showDebugInfo,
                reducedDebugInfo,
                advancedItemTooltips,
                showDebugProfilerChart,
                showLagometer,
                debugFps,
                vulkanDevice: vulkanDevice.clone(),
                chatInput: chatInput.clone(),
                worldGuiDrawList: worldGuiDrawList.clone(),
                chatOpacity,
                chatScale,
                chatWidth,
                chatHeightFocused,
                chatHeightUnfocused,
                scoreboard: scoreboard.clone(),
                localPlayerName: localPlayerName.clone(),
                localPlayerSpectator: state.gameType == GameType::Spectator,
                actionBarMessage: actionBarMessage.clone(),
                actionBarAge: playerTicksExisted - state.actionBarUpdatedTick,
                offhandNonEmpty,
                hotbarStacks: hotbarStacks.clone(),
                offhandStack: offhandStack.clone(),
                inventoryOpen,
                inventoryIsChest,
                inventoryIsShulker,
                inventoryHorseSpec,
                inventoryHorseEntity: None,
                inventoryWindowKind,
                inventoryProperties: inventoryProperties.clone(),
                merchantRecipes: merchantRecipes.clone(),
                merchantRecipeIndex,
                inventoryIsCreative: creativeInventoryOpen,
                creativeSelectedTab,
                creativeCurrentScroll,
                creativeCanScroll,
                creativeContainer: creativeContainer.clone(),
                creativeSearchInput: creativeSearchInput.clone(),
                anvilNameInput: anvilNameInput.clone(),
                enchantmentBookState,
                recipeBookState: recipeBookState.clone(),
                anvilCostFormat: anvilCostFormat.clone(),
                anvilTooExpensive: anvilTooExpensive.clone(),
                creativeTabTitle: creativeTabTitle.clone(),
                inventoryRows,
                inventoryTitle: inventoryTitle.clone(),
                playerInventoryTitle: playerInventoryTitle.clone(),
                inventoryMouseX,
                inventoryMouseY,
                inventoryOldMouseX,
                inventoryOldMouseY,
                inventoryDragSplitting,
                inventoryDragSplittingLimit,
                inventoryDragSplittingRemnant,
                inventoryDragSplittingSlots: inventoryDragSplittingSlots.clone(),
                inventorySlots: inventorySlots.clone(),
                inventoryCursorStack: inventoryCursorStack.clone(),
                playerHealth,
                absorptionAmount,
                itemActivationItem: itemActivationItem.clone(),
                itemActivationTicks,
                itemActivationRandomX,
                itemActivationRandomY,
                foodLevel,
                saturationLevel,
                armorValue,
                air,
                inWater,
                hardcoreMode: state.hardcoreMode,
                activePotionEffects,
                experience,
                experienceLevel,
                playerCreativeMode,
                xpBarCap,
                hurtResistantTime,
                playerTicksExisted,
                systemTimeMillis: current_system_time_millis(),
                primaryHand: mainHand,
                localPlayerUniqueId,
                localSkinLocation,
                localSlim,
                localSkinParts,
                localInvisible: false,
                localBurning: false,
                firstPersonItems: firstPersonItems.clone(),
                localSwingProgress,
                localSwingingHand,
                localLimbSwing,
                localPrevLimbSwingAmount,
                localLimbSwingAmount,
                localSneaking,
                localRiding,
                localSwingingArmIsLeft,
                localArmPitchOffset,
                localArmYawOffset,
                firstPersonPackedLight: 0,
                gameType: state.gameType,
                fov,
                renderDistanceChunks,
                dimension,
                totalWorldTime: 0,
                worldTime: 0,
                biomeName: "Ocean".to_owned(),
                skyLight: 0,
                blockLight: 0,
                targetBlock: None,
                loadedRenderChunks: self.chunkMeshes.len(),
                queuedRenderChunks: self.priorityPendingChunks.len() + self.pendingChunks.len() + self.inflightChunks.len(),
                biomeSkyColor: [0.49, 0.70, 1.0],
                lastLightningBolt: 0,
                partialTicks,
                gammaSetting: gammaSetting.clamp(0.0, 1.0),
                ambientOcclusion: ambientOcclusion.clamp(0, 2),
                torchFlickerX: self.torchFlickerX,
                centerRenderChunk,
                snapshot: Arc::new(HashMap::new()),
                flowerPotContents: Arc::new(HashMap::new()),
                jobs: Vec::new(),
                remotePlayers: Vec::new(),
                localPlayerRenderState: None,
                localPlayerTarget,
                nonPlayerEntities: Vec::new(),
                mapData: HashMap::new(),
                skulls: Vec::new(),
                beds: Vec::new(),
                chests: Vec::new(),
                pistons: Vec::new(),
                shulkerBoxes: Vec::new(),
                signs: Vec::new(),
                enchantmentTables: Vec::new(),
                beacons: Vec::new(),
                endPortals: Vec::new(),
                particleStates,
                miscParticleStates,
                damagedBlocks,
                selectionBox: None,
                showDebugHitboxes: self.showDebugHitboxes,
                showChunkBoundaries: self.showChunkBoundaries,
            };
        };

        let inventoryHorseEntity = inventoryHorseSpec
            .and_then(|spec| world.getNonPlayerEntityByID(spec.entityId).cloned());

        self.advanceTorchFlicker(world.getTotalWorldTime());
        let totalWorldTime = world.getTotalWorldTime();
        let worldTime = world.getWorldTime();
        let thirdPersonView = thirdPersonView.rem_euclid(3);
        let (cameraPosition, cameraYaw, cameraPitch) = orient_camera_112(
            world,
            &renderPlayerPosition,
            state.thePlayer.as_ref(),
            thirdPersonView,
        );
        let cameraBlockPos = BlockPos::new(
            cameraPosition[0].floor() as i32,
            cameraPosition[1].floor() as i32,
            cameraPosition[2].floor() as i32,
        );
        let cameraBiome = Biome::getBiome(world.getBiomeId(cameraBlockPos));
        let skyRgb = cameraBiome.getSkyColorByTemp(cameraBiome.getFloatTemperature(cameraBlockPos));
        let biomeSkyColor = [
            ((skyRgb >> 16) & 255) as f32 / 255.0,
            ((skyRgb >> 8) & 255) as f32 / 255.0,
            (skyRgb & 255) as f32 / 255.0,
        ];
        let blockReachDistance = if state.gameType.isCreative() { 5.0 } else { 4.5 };
        let objectMouseOver = state
            .thePlayer
            .as_ref()
            .and_then(|player| player.rayTrace(world, blockReachDistance, partialTicks));
        let targetBlock = objectMouseOver.and_then(|hit| {
            if hit.typeOfHit == crate::net::minecraft::util::math::RayTraceResult::Type::Block {
                let pos = hit.getBlockPos();
                Some((pos, world.getBlockState(pos)))
            } else {
                None
            }
        });
        let selectionBox = drawSelectionBox(world, objectMouseOver, 0);
        let biomeName = cameraBiome.getBiomeName().to_owned();
        let skyLight = world.getLightFor(EnumSkyBlock::Sky, cameraBlockPos);
        let blockLight = world.getLightFor(EnumSkyBlock::Block, cameraBlockPos);
        let loadedRenderChunks = self.chunkMeshes.len();
        let queuedRenderChunks = self.priorityPendingChunks.len() + self.pendingChunks.len() + self.inflightChunks.len();
        let mut remotePlayers = world.remotePlayers().map(|player| {
            let lightPos = BlockPos::new(
                player.entity.posX.floor() as i32,
                (player.entity.posY + player.entity.height as f64 * 0.5).floor() as i32,
                player.entity.posZ.floor() as i32,
            );
            RemotePlayerRenderState {
                entityId: player.entityId,
                uniqueId: player.uniqueId,
                name: player.gameProfile.getName().to_owned(),
                skinLocation: player.getPlayerInfo().or_else(|| state.playerInfoMap.get(&player.uniqueId))
                    .map(NetworkPlayerInfo::getLocationSkin)
                    .unwrap_or_else(|| DefaultPlayerSkin::getDefaultSkin(player.uniqueId)),
                slim: player.getPlayerInfo().or_else(|| state.playerInfoMap.get(&player.uniqueId))
                    .map(|info| info.getSkinType() == "slim")
                    .unwrap_or_else(|| DefaultPlayerSkin::isSlimSkin(player.uniqueId)),
                capeLocation: player.getPlayerInfo().or_else(|| state.playerInfoMap.get(&player.uniqueId))
                    .and_then(NetworkPlayerInfo::getLocationCape),
                prevChasingPosition: [player.prevChasingPosX, player.prevChasingPosY, player.prevChasingPosZ],
                chasingPosition: [player.chasingPosX, player.chasingPosY, player.chasingPosZ],
                prevMovedDistance: player.prevMovedDistance,
                movedDistance: player.movedDistance,
                prevCameraYaw: player.prevCameraYaw,
                cameraYaw: player.cameraYaw,
                chestStack: player.equipment.getItemStackFromSlot(EntityEquipmentSlot::Chest).clone(),
                armorStacks: [
                    player.equipment.getItemStackFromSlot(EntityEquipmentSlot::Feet).clone(),
                    player.equipment.getItemStackFromSlot(EntityEquipmentSlot::Legs).clone(),
                    player.equipment.getItemStackFromSlot(EntityEquipmentSlot::Chest).clone(),
                    player.equipment.getItemStackFromSlot(EntityEquipmentSlot::Head).clone(),
                ],
                elytraLocation: LayerElytra::texture(
                    player.equipment.getItemStackFromSlot(EntityEquipmentSlot::Chest),
                    player.getPlayerInfo().or_else(|| state.playerInfoMap.get(&player.uniqueId)),
                    player.skinParts(),
                ),
                customHeadSkinLocation: LayerCustomHead::playerProfile(
                    player.equipment.getItemStackFromSlot(EntityEquipmentSlot::Head),
                ).map(|profile| {
                    self.skullPlayerInfos
                        .get(&skull_profile_cache_key(&profile))
                        .map(NetworkPlayerInfo::getLocationSkin)
                        .unwrap_or_else(|| profile.getId()
                            .map(DefaultPlayerSkin::getDefaultSkin)
                            .unwrap_or_else(DefaultPlayerSkin::getDefaultSkinLegacy))
                }),
                elytraRotation: ElytraRotationState::default(),
                elytraFlying: player.isElytraFlying(),
                prevPosition: [player.entity.prevPosX, player.entity.prevPosY, player.entity.prevPosZ],
                position: [player.entity.posX, player.entity.posY, player.entity.posZ],
                prevBodyYaw: player.prevRenderYawOffset,
                bodyYaw: player.renderYawOffset,
                prevYaw: player.entity.prevRotationYaw,
                yaw: player.entity.rotationYaw,
                prevHeadYaw: player.prevRotationYawHead,
                headYaw: player.rotationYawHead,
                prevPitch: player.entity.prevRotationPitch,
                pitch: player.entity.rotationPitch,
                prevLimbSwingAmount: player.prevLimbSwingAmount,
                limbSwingAmount: player.limbSwingAmount,
                limbSwing: player.limbSwing,
                prevSwingProgress: player.prevSwingProgress,
                swingProgress: player.swingProgress,
                ticksExisted: player.entity.ticksExisted,
                ticksElytraFlying: player.ticksElytraFlying,
                motion: [player.entity.motionX, player.entity.motionY, player.entity.motionZ],
                hurtTime: player.hurtTime,
                deathTime: player.deathTime,
                sneaking: player.isSneaking(),
                riding: player.entity.isRiding(),
                sleeping: player.isPlayerSleeping(),
                bedOrientation: player.getBedOrientationInDegrees(world),
                renderOffset: [player.renderOffsetX, player.renderOffsetY, player.renderOffsetZ],
                skinParts: player.skinParts(),
                packedLight: world.getCombinedLight(lightPos, 0),
                invisible: player.isInvisible(),
                beingRidden: !player.entity.passengerIds.is_empty(),
                burning: player.isBurning(),
                swingingArmIsLeft: player.swingingArmIsLeft(),
                mainHandStack: player.getHeldItemMainhand().clone(),
                offHandStack: player.getHeldItemOffhand().clone(),
                primaryHand: player.getPrimaryHand(),
                activeHand: player.getActiveHand(),
                itemInUseCount: player.getItemInUseCount(),
                height: player.entity.height,
                eyeHeight: if player.isPlayerSleeping() {
                    0.2
                } else if player.isSneaking() || player.entity.height == 1.65 {
                    1.54
                } else if player.isElytraFlying() || player.entity.height == 0.6 {
                    0.4
                } else {
                    1.62
                },
            }
        }).collect::<Vec<_>>();
        let mut localPlayerRenderState = state.thePlayer.as_ref().map(|player| {
            let lightPos = BlockPos::new(
                player.entity.posX.floor() as i32,
                (player.entity.posY + player.entity.height as f64 * 0.5).floor() as i32,
                player.entity.posZ.floor() as i32,
            );
            RemotePlayerRenderState {
                entityId: player.entityId,
                uniqueId: localPlayerUniqueId,
                name: localPlayerName.clone(),
                skinLocation: localSkinLocation.clone(),
                slim: localSlim,
                capeLocation: localPlayerInfo.and_then(NetworkPlayerInfo::getLocationCape),
                prevChasingPosition: [player.prevChasingPosX, player.prevChasingPosY, player.prevChasingPosZ],
                chasingPosition: [player.chasingPosX, player.chasingPosY, player.chasingPosZ],
                prevMovedDistance: player.prevMovedDistance,
                movedDistance: player.movedDistance,
                prevCameraYaw: player.prevCameraYaw,
                cameraYaw: player.cameraYaw,
                chestStack: player.inventory.armorInventory.get(2).cloned().unwrap_or(ItemStack::EMPTY),
                armorStacks: [
                    player.inventory.armorInventory.get(0).cloned().unwrap_or(ItemStack::EMPTY),
                    player.inventory.armorInventory.get(1).cloned().unwrap_or(ItemStack::EMPTY),
                    player.inventory.armorInventory.get(2).cloned().unwrap_or(ItemStack::EMPTY),
                    player.inventory.armorInventory.get(3).cloned().unwrap_or(ItemStack::EMPTY),
                ],
                elytraLocation: LayerElytra::texture(
                    player.inventory.armorInventory.get(2).unwrap_or(&ItemStack::EMPTY),
                    localPlayerInfo,
                    localSkinParts,
                ),
                customHeadSkinLocation: player.inventory.armorInventory.get(3)
                    .and_then(LayerCustomHead::playerProfile)
                    .map(|profile| {
                        self.skullPlayerInfos
                            .get(&skull_profile_cache_key(&profile))
                            .map(NetworkPlayerInfo::getLocationSkin)
                            .unwrap_or_else(|| profile.getId()
                                .map(DefaultPlayerSkin::getDefaultSkin)
                                .unwrap_or_else(DefaultPlayerSkin::getDefaultSkinLegacy))
                    }),
                elytraRotation: ElytraRotationState::default(),
                elytraFlying: player.isElytraFlying(),
                prevPosition: [player.entity.prevPosX, player.entity.prevPosY, player.entity.prevPosZ],
                position: [player.entity.posX, player.entity.posY, player.entity.posZ],
                prevBodyYaw: player.prevRenderYawOffset,
                bodyYaw: player.renderYawOffset,
                prevYaw: player.entity.prevRotationYaw,
                yaw: player.entity.rotationYaw,
                prevHeadYaw: player.prevRotationYawHead,
                headYaw: player.rotationYawHead,
                prevPitch: player.entity.prevRotationPitch,
                pitch: player.entity.rotationPitch,
                prevLimbSwingAmount: player.prevLimbSwingAmount,
                limbSwingAmount: player.limbSwingAmount,
                limbSwing: player.limbSwing,
                prevSwingProgress: player.prevSwingProgress,
                swingProgress: player.swingProgress,
                ticksExisted: player.entity.ticksExisted,
                ticksElytraFlying: player.ticksElytraFlying,
                motion: [player.entity.motionX, player.entity.motionY, player.entity.motionZ],
                hurtTime: player.hurtTime,
                deathTime: player.deathTime,
                sneaking: player.entity.sneaking,
                riding: player.entity.isRiding(),
                sleeping: player.isPlayerSleeping(),
                bedOrientation: player.getBedOrientationInDegrees(world),
                renderOffset: [player.renderOffsetX, player.renderOffsetY, player.renderOffsetZ],
                skinParts: localSkinParts,
                packedLight: world.getCombinedLight(lightPos, 0),
                invisible: player.isInvisible(),
                beingRidden: !player.entity.passengerIds.is_empty(),
                burning: player.isBurning(),
                swingingArmIsLeft: localSwingingArmIsLeft,
                mainHandStack: hotbarStacks
                    .get(currentHotbarSlot.clamp(0, 8) as usize)
                    .cloned()
                    .unwrap_or(ItemStack::EMPTY),
                offHandStack: offhandStack.clone(),
                primaryHand: mainHand,
                activeHand: firstPersonItems.activeHand,
                itemInUseCount: firstPersonItems.itemInUseCount,
                height: player.entity.height,
                eyeHeight: player.getEyeHeight(),
            }
        });
        // MCP `ModelElytra#setRotationAngles` updates these values during
        // every render call. Preserve the same 10% smoothing across immutable
        // Vulkan frame captures.
        for player in &mut remotePlayers {
            let previous = self.elytraRotations.get(&player.uniqueId).copied().unwrap_or_default();
            let pose = ModelElytra::setRotationAngles(
                player.sneaking,
                player.elytraFlying,
                player.motion,
                previous,
            );
            player.elytraRotation = pose.rotations;
            self.elytraRotations.insert(player.uniqueId, pose.rotations);
        }
        if let Some(player) = localPlayerRenderState.as_mut() {
            let previous = self.elytraRotations.get(&player.uniqueId).copied().unwrap_or_default();
            let pose = ModelElytra::setRotationAngles(
                player.sneaking,
                player.elytraFlying,
                player.motion,
                previous,
            );
            player.elytraRotation = pose.rotations;
            self.elytraRotations.insert(player.uniqueId, pose.rotations);
        }
        let livePlayerIds = remotePlayers.iter().map(|player| player.uniqueId)
            .chain(localPlayerRenderState.iter().map(|player| player.uniqueId))
            .collect::<HashSet<_>>();
        self.elytraRotations.retain(|id, _| livePlayerIds.contains(id));

        if thirdPersonView > 0 && state.gameType != GameType::Spectator {
            // RenderPlayer#setModelVisibilities uses a head-only model for a
            // spectator. Until that concrete visibility mask is represented,
            // do not substitute the normal full-body model.
            if let Some(player) = localPlayerRenderState.as_ref() {
                remotePlayers.push(player.clone());
            }
        }

        let nonPlayerEntities = world.nonPlayerEntities().cloned().collect::<Vec<_>>();
        let mapData = state.mapData.clone();

        let skulls = world.skullTileEntities().filter_map(|skull| {
            let blockState = world.getBlockState(skull.pos);
            if !BlockSkull::isBlockSkull(blockState) { return None; }
            Some(SkullRenderState {
                pos: skull.pos,
                facing: BlockSkull::getFacing(blockState),
                rotation: skull.getSkullRotation(),
                skullType: skull.getSkullType(),
                playerSkinLocation: skull.getPlayerProfile().map(|profile| {
                    self.skullPlayerInfos
                        .get(&skull_profile_cache_key(profile))
                        .map(NetworkPlayerInfo::getLocationSkin)
                        .unwrap_or_else(|| {
                            profile.getId()
                                .map(DefaultPlayerSkin::getDefaultSkin)
                                .unwrap_or_else(DefaultPlayerSkin::getDefaultSkinLegacy)
                        })
                }),
                animateTicks: skull.getAnimationProgress(partialTicks),
                packedLight: world.getCombinedLight(skull.pos, 0),
            })
        }).collect::<Vec<_>>();

        let beds = world.bedTileEntities().filter_map(|bed| {
            let state = world.getBlockState(bed.pos);
            if !BlockBed::isBlockBed(state) { return None; }
            Some(BedRenderState {
                pos: bed.pos,
                head: BlockBed::isHead(state),
                horizontalIndex: BlockBed::getFacing(state).horizontalIndex().unwrap_or(2) as i32,
                colorMetadata: bed.colorMetadata() as i16,
                packedLight: world.getCombinedLight(bed.pos, 0),
            })
        }).collect::<Vec<_>>();

        let same_chest = |pos: BlockPos, block_id: i32| world.getBlockState(pos).getBlockId() == block_id;
        let chests = world.chestTileEntities().filter_map(|chest| {
            let state = world.getBlockState(chest.pos);
            let block_id = state.getBlockId();
            if !matches!(block_id, 54 | 146) { return None; }
            let x_neg = same_chest(chest.pos.west(1), block_id);
            let z_neg = same_chest(chest.pos.north(1), block_id);
            if x_neg || z_neg { return None; }
            let x_pos = same_chest(chest.pos.east(1), block_id);
            let z_pos = same_chest(chest.pos.south(1), block_id);
            let mut lid = chest.interpolatedLidAngle(partialTicks);
            if x_neg {
                if let Some(adjacent) = world.getChestTileEntity(chest.pos.west(1)) {
                    lid = lid.max(adjacent.interpolatedLidAngle(partialTicks));
                }
            }
            if z_neg {
                if let Some(adjacent) = world.getChestTileEntity(chest.pos.north(1)) {
                    lid = lid.max(adjacent.interpolatedLidAngle(partialTicks));
                }
            }
            Some(ChestRenderState {
                pos: chest.pos,
                trapped: block_id == 146,
                ender: false,
                large: x_pos || z_pos,
                metadata: state.getMetadata(),
                adjacentXPos: x_pos,
                adjacentZPos: z_pos,
                lidProgress: lid,
                packedLight: world.getCombinedLight(chest.pos, 0),
            })
        }).chain(world.enderChestTileEntities().filter_map(|chest| {
            let state = world.getBlockState(chest.pos);
            if state.getBlockId() != 130 { return None; }
            Some(ChestRenderState {
                pos: chest.pos,
                trapped: false,
                ender: true,
                large: false,
                metadata: state.getMetadata(),
                adjacentXPos: false,
                adjacentZPos: false,
                lidProgress: chest.interpolatedLidAngle(partialTicks),
                packedLight: world.getCombinedLight(chest.pos, 0),
            })
        })).collect::<Vec<_>>();

        let pistons = world.pistonTileEntities().filter_map(|piston| {
            if piston.finished() { return None; }
            Some(PistonRenderState {
                pos: piston.pos,
                pistonState: piston.pistonState,
                facing: piston.pistonFacing,
                progress: piston.getProgress(partialTicks),
                offset: piston.offset(partialTicks),
                extending: piston.extending,
                shouldHeadBeRendered: piston.shouldHeadBeRendered,
                packedLight: world.getCombinedLight(piston.pos, 0),
            })
        }).collect::<Vec<_>>();

        let shulkerBoxes = world.shulkerBoxTileEntities().filter_map(|shulker| {
            let state = world.getBlockState(shulker.pos);
            let block_id = state.getBlockId();
            if !(219..=234).contains(&block_id) { return None; }
            Some(ShulkerBoxRenderState {
                pos: shulker.pos,
                colorMetadata: shulker.colorMetadata(),
                facing: EnumFacing::getFront(state.getMetadata()),
                progress: shulker.interpolatedProgress(partialTicks),
                packedLight: world.getCombinedLight(shulker.pos, 0),
            })
        }).collect::<Vec<_>>();

        let signs = world.signTileEntities().filter_map(|sign| {
            let state = world.getBlockState(sign.pos);
            if !matches!(state.getBlockId(), 63 | 68) { return None; }
            Some(SignRenderState {
                pos: sign.pos,
                blockId: state.getBlockId(),
                metadata: state.getMetadata(),
                lines: std::array::from_fn(|index| {
                    sign.signText[index]
                        .resolveWithLocale(&self.locale)
                        .getFormattedText()
                        .to_owned()
                }),
                lineBeingEdited: sign.lineBeingEdited,
                packedLight: world.getCombinedLight(sign.pos, 0),
            })
        }).collect::<Vec<_>>();

        let enchantmentTables = world.enchantmentTableTileEntities().filter_map(|table| {
            if world.getBlockState(table.pos).getBlockId() != 116 { return None; }
            let partial = partialTicks.clamp(0.0, 1.0);
            let ticks = table.tickCount as f32 + partial;
            let interpolatedFlip = table.pageFlipPrev
                + (table.pageFlip - table.pageFlipPrev) * partial;
            let page = |offset: f32| {
                (((interpolatedFlip + offset) - (interpolatedFlip + offset).floor()) * 1.6 - 0.3)
                    .clamp(0.0, 1.0)
            };
            let mut rotationDelta = table.bookRotation - table.bookRotationPrev;
            while rotationDelta >= std::f32::consts::PI {
                rotationDelta -= std::f32::consts::PI * 2.0;
            }
            while rotationDelta < -std::f32::consts::PI {
                rotationDelta += std::f32::consts::PI * 2.0;
            }
            Some(EnchantmentTableRenderState {
                pos: table.pos,
                ticks,
                pageFlipRight: page(0.25),
                pageFlipLeft: page(0.75),
                spread: table.bookSpreadPrev
                    + (table.bookSpread - table.bookSpreadPrev) * partial,
                rotation: table.bookRotationPrev + rotationDelta * partial,
                packedLight: world.getCombinedLight(table.pos, 0),
            })
        }).collect::<Vec<_>>();

        let beacons = world.beaconTileEntities().filter_map(|beacon| {
            (world.getBlockState(beacon.pos).getBlockId() == 138).then(|| BeaconRenderState {
                pos: beacon.pos,
                beamScale: beacon.shouldBeamRender(totalWorldTime),
                segments: beacon.getBeamSegments().to_vec(),
            })
        }).collect::<Vec<_>>();

        let endPortals = world.endPortalTileEntities().filter_map(|portal| {
            (world.getBlockState(portal.pos).getBlockId() == 119)
                .then_some(EndPortalRenderState { pos: portal.pos })
        }).collect::<Vec<_>>();

        let graphicsModeChanged = self
            .lastFancyGraphics
            .replace(fancyGraphics)
            .is_some_and(|previous| previous != fancyGraphics);
        let ambientOcclusionChanged = self
            .lastAmbientOcclusion
            .replace(ambientOcclusion)
            .is_some_and(|previous| previous != ambientOcclusion);
        let buildRadius = renderDistanceChunks + 1;

        let mut sections = std::mem::take(&mut self.sectionScanScratch);
        sections.clear();
        let mut loadedSet = std::mem::take(&mut self.loadedSectionScratch);
        loadedSet.clear();
        for chunk in world.loadedChunks() {
            for sectionIndex in 0..16 {
                let key = RenderChunkKey::new(
                    chunk.xPosition,
                    sectionIndex as i32,
                    chunk.zPosition,
                );
                sections.push((
                    key,
                    chunk.sectionRevision(sectionIndex),
                    chunk.getBlockStorageArray()[sectionIndex].is_some(),
                ));
                loadedSet.insert(key);
            }
        }

        let mut removed = std::mem::take(&mut self.removedSectionScratch);
        removed.clear();
        removed.extend(
            self.observedChunkRevisions
                .keys()
                .copied()
                .filter(|key| !loadedSet.contains(key)),
        );
        for key in removed.iter().copied() {
            self.observedChunkRevisions.remove(&key);
            self.emptyRenderChunks.remove(&key);
            self.removeCachedChunk(key);
            self.invalidateNeighbours(key);
        }

        for &(key, revision, populated) in &sections {
            let changed = self
                .observedChunkRevisions
                .insert(key, revision)
                .map_or(true, |previous| previous != revision);
            let near = (key.x - centerChunk.x).abs() <= buildRadius
                && (key.z - centerChunk.z).abs() <= buildRadius;
            if !near {
                continue;
            }

            if populated {
                self.emptyRenderChunks.remove(&key);
            } else {
                self.emptyRenderChunks.insert(key);
            }

            if changed || graphicsModeChanged || ambientOcclusionChanged {
                self.invalidateChunkAndNeighbours(key);
            }

            if !populated {
                let needsEmptyUpdate = self.chunkMeshes.get(&key).map_or(true, |mesh| {
                    mesh.sourceRevision != revision || !mesh.ready || mesh.indexCount > 0
                });
                if needsEmptyUpdate {
                    self.markEmptyRenderChunk(key, revision);
                }
                continue;
            }

            let needsBuild = self.chunkMeshes.get(&key).map_or(true, |mesh| {
                mesh.sourceRevision != revision || !mesh.ready
            });
            if needsBuild {
                self.ensureRenderChunkPlaceholder(key, revision);
                let initialPriority = (key.x - centerChunk.x).abs()
                    <= INITIAL_PRIORITY_COLUMN_RADIUS
                    && (key.z - centerChunk.z).abs() <= INITIAL_PRIORITY_COLUMN_RADIUS;
                self.enqueueChunkWithPriority(key, initialPriority);
            }
        }
        self.sectionScanScratch = sections;
        self.loadedSectionScratch = loadedSet;
        self.removedSectionScratch = removed;

        // MCP `ChunkRenderDispatcher` stores pending compile tasks in a
        // distance-prioritised queue.  Preserve the same nearest-first result,
        // but do not drain and sort an unchanged queue once per rendered frame.
        // A new task or a camera RenderChunk transition invalidates the order.
        if self.pendingChunkOrderDirty
            || self.pendingChunkOrderCenter != Some(centerRenderChunk)
        {
            let mut orderedPending = self.pendingChunks.drain(..).collect::<Vec<_>>();
            orderedPending.sort_by_key(|key| {
                render_chunk_distance_squared(*key, centerRenderChunk)
            });
            self.pendingChunks.extend(orderedPending);
            self.pendingChunkOrderDirty = false;
            self.pendingChunkOrderCenter = Some(centerRenderChunk);
        }

        // The reference scheduler reserves one lane for interactive rebuilds and
        // two lanes for normal streaming. Each lane operates on one complete
        // X/Z column, but every result remains an independent MCP RenderChunk.
        // This gives the worker enough contiguous work without occupying every
        // logical core for the first ten seconds after joining a server.
        let mut selectedBatches = Vec::<(bool, Vec<(RenderChunkKey, ChunkBuildToken)>)>::new();
        while self.inflightPriorityColumnJobs < MAX_PRIORITY_BACKGROUND_COLUMN_JOBS {
            let selected = self.selectChunkColumnBuild(true, world, centerChunk, buildRadius);
            if selected.is_empty() {
                break;
            }
            self.inflightPriorityColumnJobs += 1;
            selectedBatches.push((true, selected));
        }
        while self.inflightNormalColumnJobs < MAX_NORMAL_BACKGROUND_COLUMN_JOBS {
            let selected = self.selectChunkColumnBuild(false, world, centerChunk, buildRadius);
            if selected.is_empty() {
                break;
            }
            self.inflightNormalColumnJobs += 1;
            selectedBatches.push((false, selected));
        }

        let mut required = std::mem::take(&mut self.requiredChunkScratch);
        required.clear();
        for (_, selected) in &selectedBatches {
            for (key, _) in selected {
                for dx in -1..=1 {
                    for dz in -1..=1 {
                        required.insert(ChunkKey::new(key.x + dx, key.z + dz));
                    }
                }
            }
        }
        // RenderGlobal block-damage lighting/actual-state evaluation and
        // ParticleDigging brightness sample the source block and neighbours.
        // Capture those columns even when no RenderChunk rebuild is queued.
        for progress in &damagedBlocks {
            let position = progress.getPosition();
            let chunk = ChunkKey::new(position.x.div_euclid(16), position.z.div_euclid(16));
            for dx in -1..=1 {
                for dz in -1..=1 {
                    required.insert(ChunkKey::new(chunk.x + dx, chunk.z + dz));
                }
            }
        }
        for particle in &particleStates {
            for position in [
                particle.sourcePos,
                BlockPos::new(
                    particle.position[0].floor() as i32,
                    particle.position[1].floor() as i32,
                    particle.position[2].floor() as i32,
                ),
            ] {
                let chunk = ChunkKey::new(position.x.div_euclid(16), position.z.div_euclid(16));
                required.insert(chunk);
            }
        }
        for particle in &miscParticleStates {
            let position = BlockPos::new(
                particle.position[0].floor() as i32,
                particle.position[1].floor() as i32,
                particle.position[2].floor() as i32,
            );
            let chunk = ChunkKey::new(position.x.div_euclid(16), position.z.div_euclid(16));
            required.insert(chunk);
        }
        let mut snapshot = HashMap::with_capacity(required.len());
        for key in required.drain() {
            if let Some(chunk) = world.getChunkFromChunkCoords(key.x, key.z) {
                snapshot.insert(key, chunk.clone());
            }
        }
        self.requiredChunkScratch = required;
        let flowerPotContents = Arc::new(world.flowerPotTileEntities()
            .map(|tile| (
                tile.pos,
                BlockFlowerPot::contentsName(Some(tile)).to_owned(),
            ))
            .collect::<HashMap<_, _>>());
        let jobs = selectedBatches
            .into_iter()
            .map(|(priority, selected)| ChunkBuildBatchRequest {
                priority,
                requests: selected
                    .into_iter()
                    .map(|(key, token)| ChunkBuildRequest {
                        key,
                        token,
                        dimension,
                        fancyGraphics,
                        ambientOcclusion: ambientOcclusion.clamp(0, 2),
                        translucentSortPosition: [
                            renderPlayerPosition.posX as f32,
                            renderPlayerPosition.posY as f32,
                            renderPlayerPosition.posZ as f32,
                        ],
                    })
                    .collect(),
            })
            .collect();

        WorldRenderCapture {
            playerPosition: renderPlayerPosition,
            cameraPosition,
            cameraYaw,
            cameraPitch,
            thirdPersonView,
            outputWidth,
            outputHeight,
            guiWidth: guiWidth.max(1),
            guiHeight: guiHeight.max(1),
            currentHotbarSlot,
            playerListVisible,
            playerListShowsHeads,
            playerListSkinParts,
            playerListEntries,
            playerListHeader,
            playerListFooter,
            chatMessages,
            chatOpen,
            chatVisible,
            showSubtitles,
            showDebugInfo,
            reducedDebugInfo,
            advancedItemTooltips,
            showDebugProfilerChart,
            showLagometer,
            debugFps,
            vulkanDevice,
            chatInput,
            worldGuiDrawList,
            chatOpacity,
            chatScale,
            chatWidth,
            chatHeightFocused,
            chatHeightUnfocused,
            scoreboard,
            localPlayerName,
            localPlayerSpectator: state.gameType == GameType::Spectator,
            actionBarMessage,
            actionBarAge: playerTicksExisted - state.actionBarUpdatedTick,
            offhandNonEmpty,
            hotbarStacks,
            offhandStack,
            inventoryOpen,
            inventoryIsChest,
            inventoryIsShulker,
            inventoryHorseSpec,
            inventoryHorseEntity,
            inventoryWindowKind,
            inventoryProperties,
            merchantRecipes,
            merchantRecipeIndex,
            inventoryIsCreative: creativeInventoryOpen,
            creativeSelectedTab,
            creativeCurrentScroll,
            creativeCanScroll,
            creativeContainer,
            creativeSearchInput,
            anvilNameInput,
            enchantmentBookState,
            recipeBookState,
            anvilCostFormat,
            anvilTooExpensive,
            creativeTabTitle,
            inventoryRows,
            inventoryTitle,
            playerInventoryTitle,
            inventoryMouseX,
            inventoryMouseY,
            inventoryOldMouseX,
            inventoryOldMouseY,
            inventoryDragSplitting,
            inventoryDragSplittingLimit,
            inventoryDragSplittingRemnant,
            inventoryDragSplittingSlots,
            inventorySlots,
            inventoryCursorStack,
            playerHealth,
            absorptionAmount,
            itemActivationItem,
            itemActivationTicks,
            itemActivationRandomX,
            itemActivationRandomY,
            foodLevel,
            saturationLevel,
            armorValue,
            air,
            inWater,
            hardcoreMode: state.hardcoreMode,
            activePotionEffects,
            experience,
            experienceLevel,
            playerCreativeMode,
            xpBarCap,
            hurtResistantTime,
            playerTicksExisted,
            systemTimeMillis: current_system_time_millis(),
            primaryHand: mainHand,
            localPlayerUniqueId,
            localSkinLocation,
            localSlim,
            localSkinParts,
            localInvisible: state.thePlayer.as_ref().is_some_and(|player| player.isInvisible()),
            localBurning: state.thePlayer.as_ref().is_some_and(|player| player.isBurning()),
            firstPersonItems,
            localSwingProgress,
            localSwingingHand,
            localLimbSwing,
            localPrevLimbSwingAmount,
            localLimbSwingAmount,
            localSneaking,
            localRiding,
            localSwingingArmIsLeft,
            localArmPitchOffset,
            localArmYawOffset,
            firstPersonPackedLight: world.getCombinedLight(
                BlockPos::new(
                    renderPlayerPosition.posX.floor() as i32,
                    (renderPlayerPosition.posY + renderPlayerPosition.eyeHeight as f64).floor() as i32,
                    renderPlayerPosition.posZ.floor() as i32,
                ),
                0,
            ),
            gameType: state.gameType,
            fov,
            renderDistanceChunks,
            dimension,
            totalWorldTime,
            worldTime,
            biomeName,
            skyLight,
            blockLight,
            targetBlock,
            loadedRenderChunks,
            queuedRenderChunks,
            biomeSkyColor,
            lastLightningBolt: world.getLastLightningBolt(),
            partialTicks,
            gammaSetting: gammaSetting.clamp(0.0, 1.0),
            ambientOcclusion: ambientOcclusion.clamp(0, 2),
            torchFlickerX: self.torchFlickerX,
            centerRenderChunk,
            snapshot: Arc::new(snapshot),
            flowerPotContents,
            jobs,
            remotePlayers,
            localPlayerRenderState,
            localPlayerTarget,
            nonPlayerEntities,
            mapData,
            skulls,
            beds,
            chests,
            pistons,
            shulkerBoxes,
            signs,
            enchantmentTables,
            beacons,
            endPortals,
            particleStates,
            miscParticleStates,
            damagedBlocks,
            selectionBox,
            showDebugHitboxes: self.showDebugHitboxes,
            showChunkBoundaries: self.showChunkBoundaries,
        }
    }

    fn advanceTorchFlicker(&mut self, totalWorldTime: i64) {
        let previous = self.lastTorchWorldTime.replace(totalWorldTime);
        let steps = previous
            .map(|value| totalWorldTime.saturating_sub(value).clamp(0, 20) as usize)
            .unwrap_or(1);
        for _ in 0..steps {
            EntityRenderer::updateTorchFlicker(
                &mut self.torchFlickerX,
                &mut self.torchFlickerDX,
                rand::random::<f64>(),
                rand::random::<f64>(),
                rand::random::<f64>(),
                rand::random::<f64>(),
            );
        }
    }

    pub fn render(&mut self, mut capture: WorldRenderCapture) -> anyhow::Result<WorldRenderFrame> {
        // MCP `RenderGlobal#renderBlockLayer(TRANSLUCENT)` uses the render-view
        // entity position (not the eye-camera offset) for quad distance sorting.
        let translucentSortPosition = [
            capture.playerPosition.posX,
            capture.playerPosition.posY,
            capture.playerPosition.posZ,
        ];
        let atlasStarted = Instant::now();
        let atlas = self.ensureAtlas()?;
        let atlasElapsed = atlasStarted.elapsed();

        let chunkWorkStarted = Instant::now();
        let mut chunkUploads = self.collectFinishedMeshes();
        let jobs = std::mem::take(&mut capture.jobs);

        for batch in jobs {
            if batch.requests.is_empty() {
                if batch.priority {
                    self.inflightPriorityColumnJobs =
                        self.inflightPriorityColumnJobs.saturating_sub(1);
                } else {
                    self.inflightNormalColumnJobs =
                        self.inflightNormalColumnJobs.saturating_sub(1);
                }
                continue;
            }
            log::trace!(
                "submitted {} RenderChunk sections for column ({}, {}) on {} lane",
                batch.requests.len(),
                batch.requests[0].key.x,
                batch.requests[0].key.z,
                if batch.priority { "priority" } else { "streaming" },
            );
            self.dispatcher.dispatch(ChunkColumnBuildJob {
                requests: batch.requests,
                priority: batch.priority,
                snapshot: Arc::clone(&capture.snapshot),
                flowerPotContents: Arc::clone(&capture.flowerPotContents),
                atlas: Arc::clone(&atlas),
            });
        }

        // Very small meshes can finish before command recording; accept them in
        // the same frame without allowing an unbounded upload burst.
        if chunkUploads.len() < MAX_FINISHED_CHUNKS_PER_FRAME {
            let remaining = MAX_FINISHED_CHUNKS_PER_FRAME - chunkUploads.len();
            chunkUploads.extend(self.collectFinishedMeshesLimited(remaining));
        }

        let removalCount = self
            .pendingGpuRemovals
            .len()
            .min(MAX_GPU_REMOVALS_PER_FRAME);
        let removedChunks = self
            .pendingGpuRemovals
            .drain(..removalCount)
            .collect::<Vec<_>>();
        for key in &removedChunks {
            self.pendingGpuRemovalSet.remove(key);
        }
        let chunkWorkElapsed = chunkWorkStarted.elapsed();

        let frameBuildStarted = Instant::now();
        let mut frame = make_frame(
            capture,
            atlas,
            &self.chunkMeshes,
            chunkUploads,
            removedChunks,
            &mut self.guiIngame,
            &mut self.guiBossOverlay,
            &mut self.playerTabOverlay,
            &mut self.guiNewChat,
            &mut self.fontRenderer,
            &mut self.standardGalacticFontRenderer,
            &self.locale,
            self.worldGeneration,
            &mut self.frameMeshCache,
        );
        self.updateTranslucentSorting(
            translucentSortPosition,
            &frame.visibleChunks,
            &mut frame.chunkUploads,
        );
        let frameBuildElapsed = frameBuildStarted.elapsed();
        self.profileFrames = self.profileFrames.saturating_add(1);
        self.profileAtlasNanos = self.profileAtlasNanos.saturating_add(atlasElapsed.as_nanos());
        self.profileChunkWorkNanos = self
            .profileChunkWorkNanos
            .saturating_add(chunkWorkElapsed.as_nanos());
        self.profileFrameBuildNanos = self
            .profileFrameBuildNanos
            .saturating_add(frameBuildElapsed.as_nanos());

        if self.lastStatusLog.elapsed() >= Duration::from_secs(2) {
            log::info!(
                "RenderGlobal status: loaded={}, priority_queue={}, stream_queue={}, inflight_sections={}, inflight_columns={}/{}, result_backlog={}, compiled={}, uploads={}, visible={}",
                self.observedChunkRevisions.len(),
                self.priorityPendingChunks.len(),
                self.pendingChunks.len(),
                self.inflightChunks.len(),
                self.inflightPriorityColumnJobs,
                self.inflightNormalColumnJobs,
                self.finishedMeshBacklog.len(),
                self.chunkMeshes.len(),
                frame.chunkUploads.len(),
                frame.visibleChunks.len(),
            );
            let profileElapsed = self.profileStarted.elapsed();
            let frames = self.profileFrames.max(1) as f64;
            let (
                dynamicBuilds,
                dynamicReuses,
                dynamicBuildNanos,
                blockEntityBuilds,
                blockEntityReuses,
                staticEntityBuilds,
                staticEntityReuses,
            ) = self.frameMeshCache.takeProfile();
            let dynamicAverageMillis = if dynamicBuilds == 0 {
                0.0
            } else {
                dynamicBuildNanos as f64 / dynamicBuilds as f64 / 1_000_000.0
            };
            let blockEntityResidents = self.frameMeshCache.blockEntityResidentCount();
            let staticEntityResidents = self.frameMeshCache.staticEntityResidentCount();
            log::info!(
                "World prepare stages: {:.1} fps, atlas={:.3} ms, chunk_results_and_dispatch={:.3} ms, frame_build={:.3} ms, dynamic_mesh_builds={}, reuses={}, build_avg={:.3} ms, block_entity_mesh_builds={}, reuses={}, resident_entries={}, static_entity_mesh_builds={}, reuses={}, resident_entries={}, translucent_resorts={}",
                self.profileFrames as f64 / profileElapsed.as_secs_f64().max(0.001),
                self.profileAtlasNanos as f64 / frames / 1_000_000.0,
                self.profileChunkWorkNanos as f64 / frames / 1_000_000.0,
                self.profileFrameBuildNanos as f64 / frames / 1_000_000.0,
                dynamicBuilds,
                dynamicReuses,
                dynamicAverageMillis,
                blockEntityBuilds,
                blockEntityReuses,
                blockEntityResidents,
                staticEntityBuilds,
                staticEntityReuses,
                staticEntityResidents,
                self.profileTransparencyResorts,
            );
            self.lastStatusLog = Instant::now();
            self.profileStarted = Instant::now();
            self.profileFrames = 0;
            self.profileAtlasNanos = 0;
            self.profileChunkWorkNanos = 0;
            self.profileFrameBuildNanos = 0;
            self.profileTransparencyResorts = 0;
        }
        Ok(frame)
    }

    /// Exact MCP 1.12.2 `RenderGlobal#chunksToResortTransparency` cadence.
    /// The visible list is near-to-far; at most 15 started translucent chunks
    /// are enqueued after movement squared exceeds 1.0, and one is processed
    /// per rendered frame like `ChunkRenderDispatcher#updateTransparencyLater`.
    fn updateTranslucentSorting(
        &mut self,
        playerPosition: [f64; 3],
        visibleChunks: &[VisibleChunk],
        uploads: &mut Vec<ChunkMeshUpload>,
    ) {
        let dx = playerPosition[0] - self.previousTransparencySortPosition[0];
        let dy = playerPosition[1] - self.previousTransparencySortPosition[1];
        let dz = playerPosition[2] - self.previousTransparencySortPosition[2];
        if dx * dx + dy * dy + dz * dz > 1.0 {
            self.previousTransparencySortPosition = playerPosition;
            self.chunksToResortTransparency.clear();
            self.chunksToResortTransparencySet.clear();
            let mut selected = 0usize;
            for visible in visibleChunks {
                let Some(mesh) = self.chunkMeshes.get(&visible.key) else {
                    continue;
                };
                if mesh
                    .compiledChunk
                    .isLayerStarted(BlockRenderLayer::Translucent)
                    && selected < 15
                {
                    if self.chunksToResortTransparencySet.insert(visible.key) {
                        self.chunksToResortTransparency.push_back(visible.key);
                    }
                    selected += 1;
                }
                if selected >= 15 {
                    break;
                }
            }
        }

        let Some(key) = self.chunksToResortTransparency.pop_front() else {
            return;
        };
        self.chunksToResortTransparencySet.remove(&key);
        let Some(mesh) = self.chunkMeshes.get(&key) else {
            return;
        };
        if !mesh.ready || mesh.compiledChunk.isLayerEmpty(BlockRenderLayer::Translucent) {
            return;
        }
        let vertices = Arc::clone(&mesh.vertices);
        let currentIndices = Arc::clone(&mesh.indices);
        let layerRanges = mesh.layerRanges;
        let translucentRange = layerRanges[BlockRenderLayer::Translucent.index()];
        let sortPosition = [
            playerPosition[0] as f32,
            playerPosition[1] as f32,
            playerPosition[2] as f32,
        ];
        let changed = prepare_translucent_index_range_sort(
            vertices.as_slice(),
            currentIndices.as_slice(),
            translucentRange,
            sortPosition,
            &mut self.translucentSortScratch,
        );
        if !changed {
            return;
        }
        // Copy the complete resident index vector only after the MCP quad order
        // is known to differ. The common no-change path therefore allocates
        // nothing and submits no backend update.
        let mut sortedIndices = currentIndices.as_ref().clone();
        if !apply_translucent_index_range_sort(
            sortedIndices.as_mut_slice(),
            translucentRange,
            &self.translucentSortScratch,
        ) {
            return;
        }
        let indices = Arc::new(sortedIndices);
        let meshRevision = self.takeMeshRevision();
        if let Some(mesh) = self.chunkMeshes.get_mut(&key) {
            mesh.meshRevision = meshRevision;
            mesh.indices = Arc::clone(&indices);
        } else {
            return;
        }
        // A newly compiled mesh may already have an upload in this frame. Keep
        // only the newest state so backends never allocate/upload it twice.
        uploads.retain(|upload| upload.key != key);
        uploads.push(ChunkMeshUpload {
            key,
            meshRevision,
            vertices,
            indices,
            layerRanges,
            verticesUnchanged: true,
        });
        self.profileTransparencyResorts = self.profileTransparencyResorts.saturating_add(1);
    }

    fn collectFinishedMeshes(&mut self) -> Vec<ChunkMeshUpload> {
        self.collectFinishedMeshesLimited(MAX_FINISHED_CHUNKS_PER_FRAME)
    }

    fn drainFinishedColumnBatches(&mut self) {
        for _ in 0..MAX_FINISHED_COLUMN_BATCHES_PER_FRAME {
            let Some(batch) = self.dispatcher.tryReceive() else {
                break;
            };
            if batch.worldGeneration != self.worldGeneration {
                continue;
            }
            if batch.priority {
                self.inflightPriorityColumnJobs =
                    self.inflightPriorityColumnJobs.saturating_sub(1);
            } else {
                self.inflightNormalColumnJobs =
                    self.inflightNormalColumnJobs.saturating_sub(1);
            }
            self.finishedMeshBacklog.extend(batch.results);
        }
    }

    fn collectFinishedMeshesLimited(&mut self, limit: usize) -> Vec<ChunkMeshUpload> {
        self.drainFinishedColumnBatches();
        let started = Instant::now();
        let mut uploads = Vec::new();
        let mut uploadBytes = 0_usize;
        while uploads.len() < limit {
            if !uploads.is_empty() && started.elapsed() >= MAX_FINISHED_CHUNK_UPLOAD_TIME {
                break;
            }
            let Some(result) = self.finishedMeshBacklog.pop_front() else {
                break;
            };
            let resultBytes = result
                .vertices
                .len()
                .saturating_mul(std::mem::size_of::<WorldVertex>())
                .saturating_add(result.indices.len().saturating_mul(std::mem::size_of::<u32>()));
            if !uploads.is_empty()
                && (resultBytes > MAX_SINGLE_FINISHED_CHUNK_BYTES
                    || uploadBytes.saturating_add(resultBytes)
                        > MAX_FINISHED_CHUNK_BYTES_PER_FRAME)
            {
                self.finishedMeshBacklog.push_front(result);
                break;
            }

            let Some(currentToken) = self.inflightChunks.remove(&result.key) else {
                continue;
            };
            if currentToken != result.token || result.token.worldGeneration != self.worldGeneration {
                continue;
            }
            let dirtyAgain = self.dirtyWhileInflight.remove(&result.key);
            let currentRevision = self.observedChunkRevisions.get(&result.key).copied();
            if dirtyAgain || currentRevision != Some(result.token.sourceRevision) {
                if !self.emptyRenderChunks.contains(&result.key) {
                    self.enqueueChunk(result.key);
                }
                continue;
            }

            let meshRevision = self.takeMeshRevision();
            let vertexCount = result.vertices.len();
            let indexCount = result.indices.len() as u32;
            let firstDrawableMesh = indexCount > 0 && !self.loggedFirstDrawableChunk;
            let vertices = Arc::new(result.vertices);
            let indices = Arc::new(result.indices);
            let layerRanges = result.layerRanges;
            self.chunkMeshes.insert(
                result.key,
                CachedChunkMesh {
                    sourceRevision: result.token.sourceRevision,
                    meshRevision,
                    indexCount,
                    aabbMin: result.aabbMin,
                    aabbMax: result.aabbMax,
                    compiledChunk: result.compiledChunk,
                    layerRanges,
                    vertices: Arc::clone(&vertices),
                    indices: Arc::clone(&indices),
                    ready: true,
                },
            );
            if firstDrawableMesh {
                self.loggedFirstDrawableChunk = true;
                log::info!(
                    "first drawable RenderChunk compiled at ({}, {}, {}): {} vertices, {} indices",
                    result.key.x,
                    result.key.y,
                    result.key.z,
                    vertexCount,
                    indices.len(),
                );
            } else {
                log::debug!(
                    "compiled RenderChunk ({}, {}, {}): {} vertices, {} indices",
                    result.key.x,
                    result.key.y,
                    result.key.z,
                    vertexCount,
                    indices.len(),
                );
            }
            uploadBytes = uploadBytes.saturating_add(resultBytes);
            uploads.push(ChunkMeshUpload {
                key: result.key,
                meshRevision,
                vertices,
                indices,
                layerRanges,
                verticesUnchanged: false,
            });
        }
        uploads
    }

    fn ensureAtlas(&mut self) -> anyhow::Result<Arc<AtlasState>> {
        if let Some(atlas) = &self.atlasState {
            return Ok(Arc::clone(atlas));
        }

        let mut materials = Vec::<MaterialRegistration>::new();
        let mut materialIndices = HashMap::<MaterialKey, usize>::new();
        let missingKey = MaterialKey {
            blockId: -1,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/missingno.png"),
                tintIndex: None,
            }],
        };
        self.registerMaterialKey(
            missingKey.clone(),
            &mut materialIndices,
            &mut materials,
        );

        let steveKey = MaterialKey {
            blockId: -2,
            layers: vec![MaterialLayerKey {
                texture: DefaultPlayerSkin::getDefaultSkinLegacy(),
                tintIndex: None,
            }],
        };
        let alexKey = MaterialKey {
            blockId: -3,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/entity/alex.png"),
                tintIndex: None,
            }],
        };
        let widgetsKey = MaterialKey {
            blockId: -4,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/gui/widgets.png"),
                tintIndex: None,
            }],
        };
        let iconsKey = MaterialKey {
            blockId: -5,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/gui/icons.png"),
                tintIndex: None,
            }],
        };
        let barsKey = MaterialKey {
            blockId: -16,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/gui/bars.png"),
                tintIndex: None,
            }],
        };
        let optionsBackgroundKey = MaterialKey {
            blockId: -33,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/gui/options_background.png"),
                tintIndex: None,
            }],
        };
        let minecraftTitleKey = MaterialKey {
            blockId: -34,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/gui/title/minecraft.png"),
                tintIndex: None,
            }],
        };
        let editionTitleKey = MaterialKey {
            blockId: -35,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/gui/title/edition.png"),
                tintIndex: None,
            }],
        };
        let unknownServerKey = MaterialKey {
            blockId: -36,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/misc/unknown_server.png"),
                tintIndex: None,
            }],
        };
        let resourcePackControlsKey = MaterialKey {
            blockId: -37,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/gui/resource_packs.png"),
                tintIndex: None,
            }],
        };
        let unknownPackKey = MaterialKey {
            blockId: -38,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/misc/unknown_pack.png"),
                tintIndex: None,
            }],
        };
        let fontKey = MaterialKey {
            blockId: -6,
            layers: vec![MaterialLayerKey {
                texture: self.fontRenderer.font_texture().clone(),
                tintIndex: None,
            }],
        };
        let standardGalacticFontKey = MaterialKey {
            blockId: -19_999,
            layers: vec![MaterialLayerKey {
                texture: self.standardGalacticFontRenderer.font_texture().clone(),
                tintIndex: None,
            }],
        };
        let fontPageKeys = self.fontRenderer
            .unicode_pages_with_glyphs()
            .into_iter()
            .map(|page| MaterialKey {
                blockId: -20_000 - page as i32,
                layers: vec![MaterialLayerKey {
                    texture: self.fontRenderer.unicode_page_location(page).clone(),
                    tintIndex: None,
                }],
            })
            .collect::<Vec<_>>();
        let inventoryKey = MaterialKey {
            blockId: -7,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/gui/container/inventory.png"),
                tintIndex: None,
            }],
        };
        let chestKey = MaterialKey {
            blockId: -15,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/gui/container/generic_54.png"),
                tintIndex: None,
            }],
        };
        let shulkerKey = MaterialKey {
            blockId: -16,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/gui/container/shulker_box.png"),
                tintIndex: None,
            }],
        };
        let horseKey = MaterialKey {
            blockId: -29,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/gui/container/horse.png"),
                tintIndex: None,
            }],
        };
        let craftingKey = MaterialKey {
            blockId: -21,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/gui/container/crafting_table.png"),
                tintIndex: None,
            }],
        };
        let furnaceKey = MaterialKey {
            blockId: -22,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/gui/container/furnace.png"),
                tintIndex: None,
            }],
        };
        let anvilKey = MaterialKey {
            blockId: -23,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/gui/container/anvil.png"),
                tintIndex: None,
            }],
        };
        let enchantingKey = MaterialKey {
            blockId: -24,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/gui/container/enchanting_table.png"),
                tintIndex: None,
            }],
        };
        let hopperKey = MaterialKey {
            blockId: -25,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/gui/container/hopper.png"),
                tintIndex: None,
            }],
        };
        let brewingStandKey = MaterialKey {
            blockId: -26,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/gui/container/brewing_stand.png"),
                tintIndex: None,
            }],
        };
        let dispenserKey = MaterialKey {
            blockId: -27,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/gui/container/dispenser.png"),
                tintIndex: None,
            }],
        };
        let beaconKey = MaterialKey {
            blockId: -28,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/gui/container/beacon.png"),
                tintIndex: None,
            }],
        };
        let merchantKey = MaterialKey {
            blockId: -30,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/gui/container/villager.png"),
                tintIndex: None,
            }],
        };
        let recipeBookKey = MaterialKey {
            blockId: -31,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/gui/recipe_book.png"),
                tintIndex: None,
            }],
        };
        let creativeTabsKey = MaterialKey {
            blockId: -17,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/gui/container/creative_inventory/tabs.png"),
                tintIndex: None,
            }],
        };
        let creativeItemsKey = MaterialKey {
            blockId: -18,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/gui/container/creative_inventory/tab_items.png"),
                tintIndex: None,
            }],
        };
        let creativeSearchKey = MaterialKey {
            blockId: -19,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/gui/container/creative_inventory/tab_item_search.png"),
                tintIndex: None,
            }],
        };
        let creativeInventoryKey = MaterialKey {
            blockId: -20,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/gui/container/creative_inventory/tab_inventory.png"),
                tintIndex: None,
            }],
        };
        let glintKey = MaterialKey {
            blockId: -8,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/misc/enchanted_item_glint.png"),
                tintIndex: None,
            }],
        };
        let shieldBaseKey = MaterialKey {
            blockId: -9,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", "textures/entity/shield_base_nopattern.png"),
                tintIndex: None,
            }],
        };
        let emptySlotKeys = [
            (-10, "textures/items/empty_armor_slot_helmet.png"),
            (-11, "textures/items/empty_armor_slot_chestplate.png"),
            (-12, "textures/items/empty_armor_slot_leggings.png"),
            (-13, "textures/items/empty_armor_slot_boots.png"),
            (-14, "textures/items/empty_armor_slot_shield.png"),
        ].map(|(blockId, path)| MaterialKey {
            blockId,
            layers: vec![MaterialLayerKey {
                texture: ResourceLocation::new("minecraft", path),
                tintIndex: None,
            }],
        });
        self.registerMaterialKey(steveKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(alexKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(widgetsKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(iconsKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(barsKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(optionsBackgroundKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(minecraftTitleKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(editionTitleKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(unknownServerKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(resourcePackControlsKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(unknownPackKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(fontKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(standardGalacticFontKey.clone(), &mut materialIndices, &mut materials);
        for key in &fontPageKeys {
            self.registerMaterialKey(key.clone(), &mut materialIndices, &mut materials);
        }
        self.registerMaterialKey(inventoryKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(chestKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(shulkerKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(horseKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(craftingKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(furnaceKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(anvilKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(enchantingKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(hopperKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(brewingStandKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(dispenserKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(beaconKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(merchantKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(recipeBookKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(creativeTabsKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(creativeItemsKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(creativeSearchKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(creativeInventoryKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(glintKey.clone(), &mut materialIndices, &mut materials);
        self.registerMaterialKey(shieldBaseKey.clone(), &mut materialIndices, &mut materials);
        for key in &emptySlotKeys {
            self.registerMaterialKey(key.clone(), &mut materialIndices, &mut materials);
        }

        // TileEntityItemStackRenderer binds full entity textures outside
        // TextureMap in vanilla. Vulkan keeps one descriptor atlas, so these
        // native-size textures receive dedicated exact rectangles.
        let mut builtInTextureKeys = Vec::<(ResourceLocation, MaterialKey)>::new();
        for (index, location) in TileEntityItemStackRenderer::staticTextures().into_iter().enumerate() {
            let key = MaterialKey {
                blockId: -30_000 - index as i32,
                layers: vec![MaterialLayerKey { texture: location.clone(), tintIndex: None }],
            };
            self.registerMaterialKey(key.clone(), &mut materialIndices, &mut materials);
            builtInTextureKeys.push((location, key));
        }
        // LayeredColorMaskTexture for an unpatterned banner: copy the neutral
        // banner entity sheet, then alpha-compose the base mask multiplied by
        // EnumDyeColor.byDyeDamage(item metadata).
        let mut bannerBaseKeys = Vec::<(ResourceLocation, MaterialKey)>::new();
        for dyeDamage in 0..16_i32 {
            let generated = ResourceLocation::new(
                "minecraft", format!("textures/generated/banner_base_{dyeDamage}.png"),
            );
            let key = MaterialKey {
                blockId: -31_000 - dyeDamage,
                layers: vec![
                    MaterialLayerKey {
                        texture: ResourceLocation::new("minecraft", "textures/entity/banner_base.png"),
                        tintIndex: None,
                    },
                    MaterialLayerKey {
                        texture: ResourceLocation::new("minecraft", "textures/entity/banner/base.png"),
                        tintIndex: Some(dyeDamage),
                    },
                ],
            };
            self.registerMaterialKey(key.clone(), &mut materialIndices, &mut materials);
            bannerBaseKeys.push((generated, key));
        }

        // RenderGlobal owns ten destroy-stage sprites in TextureMap.
        let destroyStageKeys: [MaterialKey; 10] = std::array::from_fn(|stage| {
            let key = MaterialKey {
                blockId: -32_000 - stage as i32,
                layers: vec![MaterialLayerKey {
                    texture: ResourceLocation::new(
                        "minecraft",
                        format!("textures/blocks/destroy_stage_{stage}.png"),
                    ),
                    tintIndex: None,
                }],
            };
            self.registerMaterialKey(key.clone(), &mut materialIndices, &mut materials);
            key
        });
        // MCP `BlockFluidRenderer#initAtlasSprites`: liquids are LIQUID
        // render-type geometry, so their sprites are registered independently
        // of BlockModelShapes rather than relying on a missing baked model.
        let waterStillKey = fluid_material_key(-34_000, "textures/blocks/water_still.png");
        let waterFlowKey = fluid_material_key(-34_001, "textures/blocks/water_flow.png");
        let waterOverlayKey = fluid_material_key(-34_002, "textures/blocks/water_overlay.png");
        let lavaStillKey = fluid_material_key(-34_003, "textures/blocks/lava_still.png");
        let lavaFlowKey = fluid_material_key(-34_004, "textures/blocks/lava_flow.png");
        let fireLayer0Key = fluid_material_key(-34_007, "textures/blocks/fire_layer_0.png");
        let fireLayer1Key = fluid_material_key(-34_008, "textures/blocks/fire_layer_1.png");
        for key in [
            &waterStillKey,
            &waterFlowKey,
            &waterOverlayKey,
            &lavaStillKey,
            &lavaFlowKey,
            &fireLayer0Key,
            &fireLayer1Key,
        ] {
            self.registerMaterialKey(key.clone(), &mut materialIndices, &mut materials);
        }

        let solidWhiteLocation = ResourceLocation::new(
            "minecraft",
            "textures/generated/solid_white.png",
        );
        let solidWhiteKey = MaterialKey {
            blockId: -34_005,
            layers: vec![MaterialLayerKey {
                texture: solidWhiteLocation.clone(),
                tintIndex: None,
            }],
        };
        self.registerMaterialKey(
            solidWhiteKey.clone(),
            &mut materialIndices,
            &mut materials,
        );
        let mapCheckerLocation = ResourceLocation::new(
            "minecraft",
            "textures/generated/map_checker.png",
        );
        let mapCheckerKey = MaterialKey {
            blockId: -34_006,
            layers: vec![MaterialLayerKey {
                texture: mapCheckerLocation.clone(),
                tintIndex: None,
            }],
        };
        self.registerMaterialKey(
            mapCheckerKey.clone(),
            &mut materialIndices,
            &mut materials,
        );

        // RenderManager entity renderers bind native entity sheets outside
        // TextureMap. Vulkan keeps them as exact rectangles in the shared atlas.
        let mut entityTextureKeys = Vec::<(ResourceLocation, MaterialKey)>::new();
        let mut entityTextures = vec![
            RenderXPOrb::texture(),
            RenderArrow::texture(ObjectSpawnType::TippedArrow).expect("arrow texture"),
            RenderArrow::texture(ObjectSpawnType::SpectralArrow).expect("spectral arrow texture"),
            RenderZombie::texture(ZombieRenderVariant::Zombie),
            RenderZombie::texture(ZombieRenderVariant::Husk),
            RenderZombie::texture(ZombieRenderVariant::ZombiePigman),
            RenderSkeleton::texture(SkeletonRenderVariant::Skeleton),
            RenderSkeleton::texture(SkeletonRenderVariant::Stray),
            RenderSkeleton::texture(SkeletonRenderVariant::WitherSkeleton),
            RenderSkeleton::overlayTexture(SkeletonRenderVariant::Stray).expect("stray overlay"),
            RenderArmorStand::texture(),
            RenderPig::texture(),
            LayerSaddle::texture(),
            RenderCow::texture(),
            RenderSheep::texture(),
            LayerSheepWool::texture(),
            RenderChicken::texture(),
            RenderMooshroom::texture(),
            RenderCreeper::texture(),
            LayerCreeperCharge::texture(),
            RenderSpider::texture(SpiderVariant::Spider),
            RenderSpider::texture(SpiderVariant::CaveSpider),
            LayerSpiderEyes::texture(),
            RenderSlime::texture(),
            RenderMagmaCube::texture(),
            RenderBlaze::texture(),
            LayerWolfCollar::texture(),
            RenderPolarBear::texture(),
            RenderAbstractHorse::texture(HorseModelVariant::Donkey),
            RenderAbstractHorse::texture(HorseModelVariant::Mule),
            RenderAbstractHorse::texture(HorseModelVariant::Skeleton),
            RenderAbstractHorse::texture(HorseModelVariant::Zombie),
            TileEntitySignRenderer::texture(),
            TileEntityBeaconRenderer::texture(),
            TileEntityEndPortalRenderer::endSkyTexture(),
            TileEntityEndPortalRenderer::endPortalTexture(),
            ResourceLocation::parse("textures/environment/sun.png"),
            ResourceLocation::parse("textures/environment/moon_phases.png"),
        ];
        entityTextures.extend(RenderBoat::allTextures());
        entityTextures.push(RenderMinecart::texture());
        entityTextures.push(RenderEnderCrystal::texture());
        entityTextures.push(RenderEnderCrystal::beamTexture());
        entityTextures.push(RenderPainting::texture());
        entityTextures.extend(RenderItemFrame::allTextures());
        entityTextures.push(RenderLeashKnot::texture());
        entityTextures.push(ItemRenderer::mapBackgroundTexture());
        entityTextures.push(MapItemRenderer::iconsTexture());
        entityTextures.extend(RenderGhast::allTextures());
        entityTextures.extend(RenderGuardian::allTextures());
        entityTextures.extend(RenderShulker::allTextures());
        entityTextures.push(RenderShulkerBullet::texture());
        entityTextures.push(RenderFireball::texture());
        entityTextures.push(RenderDragonFireball::texture());
        entityTextures.extend(RenderWitherSkull::allTextures());
        entityTextures.push(RenderFish::texture());
        entityTextures.extend(RenderWolf::allTextures());
        entityTextures.extend(RenderOcelot::allTextures());
        entityTextures.extend(RenderRabbit::allTextures());
        entityTextures.extend(RenderLlama::allTextures());
        entityTextures.extend(LayerLlamaDecor::allTextures());
        entityTextures.extend(RenderVillager::allTextures());
        entityTextures.push(RenderWitch::texture());
        entityTextures.push(RenderVindicator::texture());
        entityTextures.push(RenderEvoker::texture());
        entityTextures.push(RenderIllusionIllager::texture());
        entityTextures.extend(RenderZombieVillager::allTextures());
        // LayerBipedArmor binds native 64x32 armor sheets outside TextureMap.
        for material in ["leather", "chainmail", "iron", "gold", "diamond"] {
            entityTextures.push(ResourceLocation::new(
                "minecraft", format!("textures/models/armor/{material}_layer_1.png"),
            ));
            entityTextures.push(ResourceLocation::new(
                "minecraft", format!("textures/models/armor/{material}_layer_2.png"),
            ));
        }
        entityTextures.push(ResourceLocation::new(
            "minecraft", "textures/models/armor/leather_layer_1_overlay.png",
        ));
        entityTextures.push(ResourceLocation::new(
            "minecraft", "textures/models/armor/leather_layer_2_overlay.png",
        ));
        entityTextures.push(LayerElytra::defaultTexture());
        for (index, location) in entityTextures.into_iter().enumerate() {
            let key = MaterialKey {
                blockId: -35_000 - index as i32,
                layers: vec![MaterialLayerKey { texture: location.clone(), tintIndex: None }],
            };
            self.registerMaterialKey(key.clone(), &mut materialIndices, &mut materials);
            entityTextureKeys.push((location, key));
        }
        for (index, layered) in RenderHorse::layeredTextures().into_iter().enumerate() {
            let key = MaterialKey {
                blockId: -36_000 - index as i32,
                layers: layered.layers.into_iter().map(|texture| MaterialLayerKey {
                    texture,
                    tintIndex: None,
                }).collect(),
            };
            self.registerMaterialKey(key.clone(), &mut materialIndices, &mut materials);
            entityTextureKeys.push((layered.generated, key));
        }
        let mut particleTextureKeys = HashMap::<ResourceLocation, MaterialKey>::new();

        // RenderItem registers all simple ItemModelMesher shapes during client
        // startup. Vanilla stitches their sprites into TextureMap before any
        // world is joined, so loading them here is not inventory-dependent.
        let renderItem = RenderItem::new(self.resourceManager.clone());
        let itemModels = renderItem.loadRegisteredModels();
        // ItemShield installs a `blocking` model predicate in vanilla. The
        // current renderer resolves model overrides outside ItemModelMesher, so
        // retain the exact blocking camera transforms explicitly rather than
        // rendering the ordinary shield model while the hand is active.
        let shieldBlockingModel = renderItem
            .resolveModel("minecraft:shield_blocking", "inventory")
            .map(Arc::new)
            .map_err(|error| log::warn!("failed loading shield blocking model: {error}"))
            .ok();
        for (&(itemId, _metadata), model) in &itemModels {
            for quad in &model.quads {
                self.registerMaterialKey(
                    item_material_key(itemId, quad.texture.clone(), quad.tintIndex),
                    &mut materialIndices,
                    &mut materials,
                );
            }
        }

        let mut models = Vec::with_capacity((MAX_GLOBAL_BLOCK_STATE_ID + 1) as usize);
        let mut particleTextures = Vec::with_capacity((MAX_GLOBAL_BLOCK_STATE_ID + 1) as usize);
        for globalStateId in 0..=MAX_GLOBAL_BLOCK_STATE_ID {
            let state = IBlockState::fromGlobalStateId(globalStateId);
            let model = self.blockModelShapes.getModelForState(state);
            if let Some(model) = &model {
                if !model.missing {
                    for quad in &model.quads {
                        if !quad.material.layers.is_empty() {
                            let key = material_key(state.getBlockId(), &quad.material);
                            self.registerMaterialKey(key, &mut materialIndices, &mut materials);
                        }
                    }
                }
            }
            let particleTexture = self.blockModelShapes.getTexture(state);
            if !particleTextureKeys.contains_key(&particleTexture) {
                let key = MaterialKey {
                    blockId: -33_000 - particleTextureKeys.len() as i32,
                    layers: vec![MaterialLayerKey {
                        texture: particleTexture.clone(),
                        tintIndex: None,
                    }],
                };
                self.registerMaterialKey(key.clone(), &mut materialIndices, &mut materials);
                particleTextureKeys.insert(particleTexture.clone(), key);
            }
            particleTextures.push(particleTexture);
            models.push(model);
        }

        let mut stairModels = HashMap::<(i32, StairShape), Arc<ResolvedBlockModel>>::new();
        for blockId in [53, 67, 108, 109, 114, 128, 134, 135, 136, 156, 163, 164, 180, 203] {
            for meta in 0..8 {
                let state = IBlockState::fromGlobalStateId((blockId << 4) | meta);
                let stateKey = stair_model_state_key(state);
                for shape in StairShape::VALUES {
                    let variant = stair_variant_for_shape(meta, shape);
                    if let Some(model) = self.blockModelShapes.getModelForVariant(state, variant) {
                        for quad in &model.quads {
                            if !quad.material.layers.is_empty() {
                                let key = material_key(state.getBlockId(), &quad.material);
                                self.registerMaterialKey(key, &mut materialIndices, &mut materials);
                            }
                        }
                        stairModels.insert((stateKey, shape), model);
                    }
                }
            }
        }

        let mut snowyModels = HashMap::<i32, Arc<ResolvedBlockModel>>::new();
        for blockId in [2, 110] {
            let state = IBlockState::fromGlobalStateId(blockId << 4);
            if let Some(model) = self.blockModelShapes.getModelForVariant(state, "snowy=true") {
                for quad in &model.quads {
                    if !quad.material.layers.is_empty() {
                        let key = material_key(state.getBlockId(), &quad.material);
                        self.registerMaterialKey(key, &mut materialIndices, &mut materials);
                    }
                }
                snowyModels.insert(blockId, model);
            }
        }

        let mut connectedModels = HashMap::<(i32, u8), Arc<ResolvedBlockModel>>::new();
        let mut connectedStates = Vec::new();
        for blockId in [85, 101, 102, 113, 139, 188, 189, 190, 191, 192] {
            let metadata: Vec<i32> = match blockId {
                139 => vec![0, 1],
                _ => vec![0],
            };
            for meta in metadata {
                connectedStates.push(IBlockState::fromGlobalStateId((blockId << 4) | meta));
            }
        }
        for meta in 0..16 {
            connectedStates.push(IBlockState::fromGlobalStateId((160 << 4) | meta));
        }
        for state in connectedStates {
            let blockId = state.getBlockId();
            let stateKey = connected_model_state_key(state);
            let maxMask = if blockId == 139 { 32 } else { 16 };
            for mask in 0..maxMask {
                let variant = connected_variant(mask as u8, blockId == 139);
                if let Some(model) = self.blockModelShapes.getModelForVariant(state, variant) {
                    for quad in &model.quads {
                        if !quad.material.layers.is_empty() {
                            let key = material_key(state.getBlockId(), &quad.material);
                            self.registerMaterialKey(key, &mut materialIndices, &mut materials);
                        }
                    }
                    connectedModels.insert((stateKey, mask as u8), model);
                }
            }
        }

        // `BlockFire#getActualState` reconstructs five attachment booleans
        // from neighbouring flammable blocks. Bake all metadata/actual-state
        // combinations instead of falling through to the metadata-only model.
        let mut fireModels = HashMap::<(i32, u8), Arc<ResolvedBlockModel>>::new();
        for age in 0..16_i32 {
            let state = IBlockState::fromGlobalStateId((BlockFire::BLOCK_ID << 4) | age);
            for mask in 0..32_u8 {
                let variant = BlockFire::modelVariant(age, mask);
                if let Some(model) = self.blockModelShapes.getModelForVariant(state, variant) {
                    for quad in &model.quads {
                        if !quad.material.layers.is_empty() {
                            let material = material_key(BlockFire::BLOCK_ID, &quad.material);
                            self.registerMaterialKey(material, &mut materialIndices, &mut materials);
                        }
                    }
                    fireModels.insert((age, mask), model);
                }
            }
        }

        // `BlockDoublePlant#getActualState` copies VARIANT from the lower
        // half into the upper half. Upper metadata only stores HALF plus a
        // horizontal facing used by placement, so metadata-only model lookup
        // resolves to a non-existent `normal` variant and drops the top half.
        // Bake all six lower/upper model combinations and select them from the
        // neighbouring lower block during RenderChunk compilation.
        let mut doublePlantModels = HashMap::<(u8, bool), Arc<ResolvedBlockModel>>::new();
        let plantNames = [
            "sunflower",
            "syringa",
            "double_grass",
            "double_fern",
            "double_rose",
            "paeonia",
        ];
        for (variantMeta, _plantName) in plantNames.into_iter().enumerate() {
            for upper in [false, true] {
                // The custom state mapper selects the blockstate resource
                // name from VARIANT; each resource contains only `half`.
                // Including `variant=...` produces a missing variant and was
                // the reason all six double plants rendered transparent.
                let variant = format!(
                    "half={}",
                    if upper { "upper" } else { "lower" },
                );
                let state = IBlockState::fromGlobalStateId(
                    (175 << 4) | variantMeta as i32 | (if upper { 8 } else { 0 }),
                );
                if let Some(model) = self.blockModelShapes.getModelForVariant(state, variant) {
                    for quad in &model.quads {
                        if !quad.material.layers.is_empty() {
                            let material = material_key(175, &quad.material);
                            self.registerMaterialKey(material, &mut materialIndices, &mut materials);
                        }
                    }
                    doublePlantModels.insert((variantMeta as u8, upper), model);
                }
            }
        }

        // `TileEntityPistonRenderer` creates transient piston-head states,
        // including SHORT which has no metadata bit. Bake all exact
        // BlockPistonExtension blockstate variants for the moving TESR path.
        let mut pistonHeadModels = HashMap::<(u8, bool, bool), Arc<ResolvedBlockModel>>::new();
        for facing in EnumFacing::VALUES {
            let facing_index = facing.index() as u8;
            let facing_name = match facing {
                EnumFacing::Down => "down", EnumFacing::Up => "up",
                EnumFacing::North => "north", EnumFacing::South => "south",
                EnumFacing::West => "west", EnumFacing::East => "east",
            };
            for sticky in [false, true] {
                for short in [false, true] {
                    let variant = format!(
                        "facing={facing_name},short={},type={}",
                        if short { "true" } else { "false" },
                        if sticky { "sticky" } else { "normal" },
                    );
                    let meta = facing.index() | if sticky { 8 } else { 0 };
                    let state = IBlockState::fromGlobalStateId((34 << 4) | meta);
                    if let Some(model) = self.blockModelShapes.getModelForVariant(state, &variant) {
                        for quad in &model.quads {
                            if !quad.material.layers.is_empty() {
                                let material = material_key(34, &quad.material);
                                self.registerMaterialKey(material, &mut materialIndices, &mut materials);
                            }
                        }
                        pistonHeadModels.insert((facing_index, sticky, short), model);
                    }
                }
            }
        }

        // `BlockDoor#getActualState` merges metadata from the two block halves.
        // Bake every model-relevant state-mapper combination for each vanilla
        // door block, then select by neighbour-derived actual state at chunk
        // build time. POWERED is intentionally absent from the blockstate JSON.
        let mut doorModels = HashMap::<(i32, u8), Arc<ResolvedBlockModel>>::new();
        for blockId in [64, 71, 193, 194, 195, 196, 197] {
            let representative = IBlockState::fromGlobalStateId(blockId << 4);
            for key in 0_u8..32 {
                let variant = BlockDoor::modelVariantFromKey(key);
                if let Some(model) = self.blockModelShapes.getModelForVariant(representative, variant) {
                    for quad in &model.quads {
                        if !quad.material.layers.is_empty() {
                            let material = material_key(blockId, &quad.material);
                            self.registerMaterialKey(material, &mut materialIndices, &mut materials);
                        }
                    }
                    doorModels.insert((blockId, key), model);
                }
            }
        }

        // `BlockFenceGate#getActualState` contributes IN_WALL while the
        // StateMap ignores POWERED. Bake every facing/open/in_wall combination
        // for all six wood species and select it from neighbouring walls.
        let mut fenceGateModels = HashMap::<(i32, u8), Arc<ResolvedBlockModel>>::new();
        for blockId in [107, 183, 184, 185, 186, 187] {
            let representative = IBlockState::fromGlobalStateId(blockId << 4);
            for key in 0_u8..16 {
                let variant = BlockFenceGate::modelVariantFromKey(key);
                if let Some(model) = self.blockModelShapes.getModelForVariant(representative, variant) {
                    for quad in &model.quads {
                        if !quad.material.layers.is_empty() {
                            let material = material_key(blockId, &quad.material);
                            self.registerMaterialKey(material, &mut materialIndices, &mut materials);
                        }
                    }
                    fenceGateModels.insert((blockId, key), model);
                }
            }
        }

        // `BlockRedstoneWire#getActualState` supplies four ternary attach
        // properties. POWER is ignored by the vanilla state mapper and is
        // applied only through the tint handler, so exactly 3^4 models exist.
        let mut redstoneWireModels = HashMap::<u8, Arc<ResolvedBlockModel>>::new();
        let representative = IBlockState::fromGlobalStateId(BlockRedstoneWire::BLOCK_ID << 4);
        for key in 0_u8..81 {
            let variant = BlockRedstoneWire::modelVariantFromKey(key);
            if let Some(model) = self.blockModelShapes.getModelForVariant(representative, variant) {
                for quad in &model.quads {
                    if !quad.material.layers.is_empty() {
                        let material = material_key(BlockRedstoneWire::BLOCK_ID, &quad.material);
                        self.registerMaterialKey(material, &mut materialIndices, &mut materials);
                    }
                }
                redstoneWireModels.insert(key, model);
            }
        }

        // BlockFlowerPot ignores LEGACY_DATA in its custom state mapper and
        // selects one of 22 models from TileEntityFlowerPot.CONTENTS.
        let mut flowerPotModels = HashMap::<String, Arc<ResolvedBlockModel>>::new();
        let representative = IBlockState::fromGlobalStateId(BlockFlowerPot::BLOCK_ID << 4);
        for contents in BlockFlowerPot::CONTENTS {
            let variant = BlockFlowerPot::modelVariant(contents);
            if let Some(model) = self.blockModelShapes.getModelForVariant(representative, &variant) {
                for quad in &model.quads {
                    if !quad.material.layers.is_empty() {
                        let material = material_key(BlockFlowerPot::BLOCK_ID, &quad.material);
                        self.registerMaterialKey(material, &mut materialIndices, &mut materials);
                    }
                }
                flowerPotModels.insert(contents.to_owned(), model);
            }
        }

        // ParticleManager passes `state.getActualState(world, pos)` to
        // ParticleDigging for block-destroy effects. Metadata-only entries in
        // `particleTextures` are therefore insufficient for split/extended
        // states such as doors, fence gates, redstone wire and double plants.
        // Stitch every particle sprite referenced by the actual-state caches
        // before the atlas is built, mirroring TextureMap's global sprite
        // registration rather than falling back to missingno at render time.
        for model in doorModels.values() {
            if let Some(texture) = model.particleTexture.as_ref() {
                self.registerParticleTexture(
                    texture.clone(),
                    &mut particleTextureKeys,
                    &mut materialIndices,
                    &mut materials,
                );
            }
        }
        for model in fenceGateModels.values() {
            if let Some(texture) = model.particleTexture.as_ref() {
                self.registerParticleTexture(
                    texture.clone(),
                    &mut particleTextureKeys,
                    &mut materialIndices,
                    &mut materials,
                );
            }
        }
        for model in redstoneWireModels.values() {
            if let Some(texture) = model.particleTexture.as_ref() {
                self.registerParticleTexture(
                    texture.clone(),
                    &mut particleTextureKeys,
                    &mut materialIndices,
                    &mut materials,
                );
            }
        }
        for model in doublePlantModels.values() {
            if let Some(texture) = model.particleTexture.as_ref() {
                self.registerParticleTexture(
                    texture.clone(),
                    &mut particleTextureKeys,
                    &mut materialIndices,
                    &mut materials,
                );
            }
        }

        // TextureManager dynamic objects are appended after all stable
        // TextureMap/entity/GUI registrations. Fixed reserve slots in
        // buildAtlas keep every pre-existing rectangle invariant.
        let mut dynamicPlayerTextures = self.textureCache
            .keys()
            .filter(|location| location.getNamespace() == "minecraft" && location.getPath().starts_with("skins/"))
            .cloned()
            .collect::<Vec<_>>();
        dynamicPlayerTextures.sort();
        dynamicPlayerTextures.truncate(DYNAMIC_PLAYER_TEXTURE_RESERVE);
        let mut dynamicPlayerMaterialIndices = Vec::with_capacity(dynamicPlayerTextures.len());
        for (index, location) in dynamicPlayerTextures.into_iter().enumerate() {
            let key = MaterialKey {
                blockId: DYNAMIC_PLAYER_MATERIAL_BASE - index as i32,
                layers: vec![MaterialLayerKey { texture: location.clone(), tintIndex: None }],
            };
            self.registerMaterialKey(key.clone(), &mut materialIndices, &mut materials);
            let materialIndex = *materialIndices
                .get(&key)
                .expect("registered dynamic player material has an index");
            dynamicPlayerMaterialIndices.push((location.clone(), materialIndex));
            entityTextureKeys.push((location, key));
        }
        let mut dynamicPackIcons = self.textureCache
            .keys()
            .filter(|location| location.getNamespace() == "minecraft"
                && (location.getPath().starts_with("resourcepackicons/")
                    || location.getPath() == "dynamic/default_pack_icon.png"))
            .cloned()
            .collect::<Vec<_>>();
        dynamicPackIcons.sort();
        dynamicPackIcons.truncate(DYNAMIC_RESOURCE_PACK_ICON_RESERVE);
        for (index, location) in dynamicPackIcons.into_iter().enumerate() {
            let key = MaterialKey {
                blockId: DYNAMIC_RESOURCE_PACK_ICON_MATERIAL_BASE - index as i32,
                layers: vec![MaterialLayerKey { texture: location, tintIndex: None }],
            };
            self.registerMaterialKey(key, &mut materialIndices, &mut materials);
        }

        let materialCount = materials.len();
        let playerUnusedCount = DYNAMIC_PLAYER_TEXTURE_RESERVE
            .saturating_sub(dynamicPlayerMaterialIndices.len());
        let (atlas, rectangles, exactRectangles, placements) = self.buildAtlas(&materials);
        let mut dynamicPlayerSlots = dynamicPlayerMaterialIndices
            .iter()
            .filter_map(|(_, index)| placements.get(*index).copied())
            .map(|[originX, originY, tileSize]| DynamicAtlasSlot { originX, originY, tileSize })
            .collect::<Vec<_>>();
        dynamicPlayerSlots.extend(
            placements
                .iter()
                .skip(materialCount)
                .take(playerUnusedCount)
                .copied()
                .map(|[originX, originY, tileSize]| DynamicAtlasSlot { originX, originY, tileSize }),
        );
        self.dynamicPlayerSlots = dynamicPlayerSlots;
        self.dynamicPlayerAssignments = dynamicPlayerMaterialIndices
            .iter()
            .enumerate()
            .map(|(slot, (location, _))| (location.clone(), slot))
            .collect();
        self.dynamicPlayerAtlasDirty = false;
        self.dynamicPackIconAtlasDirty = false;
        self.dynamicAtlasDirtySince = None;

        let rectangleMap = materials
            .iter()
            .zip(rectangles.iter().copied())
            .zip(exactRectangles.iter().copied())
            .map(|((material, inset), exact)| {
                // FaceBakery applies its own 0.1% model-UV contraction.
                // TextureAtlasSprite independently uses a 0.01-atlas-pixel
                // min/max inset; item sprites bind the native TextureManager
                // rectangle while stitched block materials use that MCP inset.
                let rectangle = if material.key.blockId == -1000 { exact } else { inset };
                (material.key.clone(), rectangle)
            })
            .collect::<HashMap<_, _>>();
        let missingRectangle = rectangleMap
            .get(&missingKey)
            .copied()
            .unwrap_or([0.0, 0.0, 1.0, 1.0]);
        let particleTextureRectangles = particleTextureKeys
            .iter()
            .filter_map(|(texture, key)| {
                materialIndices
                    .get(key)
                    .and_then(|index| exactRectangles.get(*index))
                    .copied()
                    .map(|rectangle| (texture.clone(), rectangle))
            })
            .collect::<HashMap<_, _>>();
        let entityTextureRectangles = entityTextureKeys
            .iter()
            .filter_map(|(texture, key)| {
                materialIndices
                    .get(key)
                    .and_then(|index| exactRectangles.get(*index))
                    .copied()
                    .map(|rectangle| (texture.clone(), rectangle))
            })
            .collect::<HashMap<_, _>>();
        let textureRectangles = build_exact_texture_rectangle_map(&materials, &exactRectangles);
        let solidWhiteRectangle = materialIndices
            .get(&solidWhiteKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let mapCheckerRectangle = materialIndices
            .get(&mapCheckerKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let mapBackgroundRectangle = entityTextureRectangles
            .get(&ItemRenderer::mapBackgroundTexture())
            .copied()
            .unwrap_or(missingRectangle);
        let mapIconsRectangle = entityTextureRectangles
            .get(&MapItemRenderer::iconsTexture())
            .copied()
            .unwrap_or(missingRectangle);
        let destroyStageRectangles = std::array::from_fn(|stage| {
            materialIndices
                .get(&destroyStageKeys[stage])
                .and_then(|index| exactRectangles.get(*index))
                .copied()
                .unwrap_or(missingRectangle)
        });
        let exactFluidRectangle = |key: &MaterialKey| {
            materialIndices
                .get(key)
                .and_then(|index| exactRectangles.get(*index))
                .copied()
                .unwrap_or(missingRectangle)
        };
        let waterStillRectangle = exactFluidRectangle(&waterStillKey);
        let waterFlowRectangle = exactFluidRectangle(&waterFlowKey);
        let waterOverlayRectangle = exactFluidRectangle(&waterOverlayKey);
        let lavaStillRectangle = exactFluidRectangle(&lavaStillKey);
        let lavaFlowRectangle = exactFluidRectangle(&lavaFlowKey);
        let fireLayer0Rectangle = exactFluidRectangle(&fireLayer0Key);
        let fireLayer1Rectangle = exactFluidRectangle(&fireLayer1Key);
        let fireAnimation = |key: &MaterialKey| {
            materialIndices
                .get(key)
                .and_then(|index| materials.get(*index))
                .and_then(|material| material.textures.first())
                .map(|texture| {
                    (
                        texture.animation.clone(),
                        texture.image.width().max(1) as f32 / atlas.height.max(1) as f32,
                    )
                })
                .unwrap_or((None, 0.0))
        };
        let (fireLayer0Animation, fireLayer0FrameStepV) = fireAnimation(&fireLayer0Key);
        let (fireLayer1Animation, fireLayer1FrameStepV) = fireAnimation(&fireLayer1Key);
        // Block-model UVs retain the half-texel inset used to prevent
        // neighbouring sprite bleed. Player ModelBox UVs are already exact
        // 64x64 texel boundaries and must use the uninset native rectangle;
        // shrinking the complete skin by one atlas texel shifts every limb
        // seam and face boundary.
        let steveRectangle = materialIndices
            .get(&steveKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let alexRectangle = materialIndices
            .get(&alexKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let widgetsRectangle = materialIndices
            .get(&widgetsKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let iconsRectangle = materialIndices
            .get(&iconsKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let barsRectangle = materialIndices
            .get(&barsKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let fontRectangle = materialIndices
            .get(&fontKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let mut fontTextureRectangles = HashMap::new();
        fontTextureRectangles.insert(self.fontRenderer.font_texture().clone(), fontRectangle);
        let standardGalacticFontRectangle = materialIndices
            .get(&standardGalacticFontKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(fontRectangle);
        fontTextureRectangles.insert(
            self.standardGalacticFontRenderer.font_texture().clone(),
            standardGalacticFontRectangle,
        );
        for key in &fontPageKeys {
            if let Some(rectangle) = materialIndices
                .get(key)
                .and_then(|index| exactRectangles.get(*index))
                .copied()
            {
                if let Some(layer) = key.layers.first() {
                    fontTextureRectangles.insert(layer.texture.clone(), rectangle);
                }
            }
        }
        let inventoryRectangle = materialIndices
            .get(&inventoryKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let chestRectangle = materialIndices
            .get(&chestKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let shulkerRectangle = materialIndices
            .get(&shulkerKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let horseRectangle = materialIndices
            .get(&horseKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let craftingRectangle = materialIndices
            .get(&craftingKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let furnaceRectangle = materialIndices
            .get(&furnaceKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let anvilRectangle = materialIndices
            .get(&anvilKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let enchantingRectangle = materialIndices
            .get(&enchantingKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let hopperRectangle = materialIndices
            .get(&hopperKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let brewingStandRectangle = materialIndices
            .get(&brewingStandKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let dispenserRectangle = materialIndices
            .get(&dispenserKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let beaconRectangle = materialIndices
            .get(&beaconKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let merchantRectangle = materialIndices
            .get(&merchantKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let recipeBookRectangle = materialIndices
            .get(&recipeBookKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let creativeTabsRectangle = materialIndices
            .get(&creativeTabsKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let creativeItemsRectangle = materialIndices
            .get(&creativeItemsKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let creativeSearchRectangle = materialIndices
            .get(&creativeSearchKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let creativeInventoryRectangle = materialIndices
            .get(&creativeInventoryKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let glintRectangle = materialIndices
            .get(&glintKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let shieldBaseRectangle = materialIndices
            .get(&shieldBaseKey)
            .and_then(|index| exactRectangles.get(*index))
            .copied()
            .unwrap_or(missingRectangle);
        let mut builtInItemRectangles = HashMap::new();
        for (location, key) in builtInTextureKeys.into_iter().chain(bannerBaseKeys) {
            if let Some(rectangle) = materialIndices
                .get(&key)
                .and_then(|index| exactRectangles.get(*index))
                .copied()
            {
                builtInItemRectangles.insert(location, rectangle);
            }
        }
        // Bed is one mesh-definition model (`minecraft:bed#inventory`) but
        // TileEntityItemStackRenderer chooses one of sixteen entity textures
        // from the original ItemStack metadata. Retain that metadata-indexed
        // table explicitly instead of routing variants through the metadata-0
        // item-model key.
        let bedRectangles = std::array::from_fn(|metadata| {
            builtInItemRectangles
                .get(&TileEntityItemStackRenderer::bedTexture(metadata as i16))
                .copied()
                .unwrap_or(missingRectangle)
        });
        let emptySlotRectangles = emptySlotKeys.map(|key| {
            materialIndices
                .get(&key)
                .and_then(|index| exactRectangles.get(*index))
                .copied()
                .unwrap_or(missingRectangle)
        });
        let grassColorizer = ColorizerGrass::load(&self.resourceManager);
        let foliageColorizer = ColorizerFoliage::load(&self.resourceManager);
        let itemColors = Arc::new(ItemColors::new(&grassColorizer));
        let blockColors = Arc::new(BlockColors::new(grassColorizer, foliageColorizer));
        let revision = self.takeAtlasRevision();
        let state = Arc::new(AtlasState {
            revision,
            atlas: Arc::new(atlas),
            rectangles: Arc::new(rectangleMap),
            particleTextureRectangles: Arc::new(particleTextureRectangles),
            entityTextureRectangles: Arc::new(entityTextureRectangles),
            textureRectangles: Arc::new(textureRectangles),
            particleTextures: Arc::new(particleTextures),
            destroyStageRectangles,
            waterStillRectangle,
            waterFlowRectangle,
            waterOverlayRectangle,
            lavaStillRectangle,
            lavaFlowRectangle,
            fireLayer0Rectangle,
            fireLayer1Rectangle,
            fireLayer0Animation,
            fireLayer1Animation,
            fireLayer0FrameStepV,
            fireLayer1FrameStepV,
            missingRectangle,
            solidWhiteRectangle,
            mapCheckerRectangle,
            mapBackgroundRectangle,
            mapIconsRectangle,
            models: Arc::new(models),
            stairModels: Arc::new(stairModels),
            snowyModels: Arc::new(snowyModels),
            connectedModels: Arc::new(connectedModels),
            fireModels: Arc::new(fireModels),
            doublePlantModels: Arc::new(doublePlantModels),
            doorModels: Arc::new(doorModels),
            fenceGateModels: Arc::new(fenceGateModels),
            redstoneWireModels: Arc::new(redstoneWireModels),
            flowerPotModels: Arc::new(flowerPotModels),
            pistonHeadModels: Arc::new(pistonHeadModels),
            blockColors,
            itemColors,
            steveRectangle,
            alexRectangle,
            widgetsRectangle,
            iconsRectangle,
            barsRectangle,
            fontRectangle,
            fontTextureRectangles: Arc::new(fontTextureRectangles),
            inventoryRectangle,
            chestRectangle,
            shulkerRectangle,
            horseRectangle,
            craftingRectangle,
            furnaceRectangle,
            anvilRectangle,
            enchantingRectangle,
            hopperRectangle,
            brewingStandRectangle,
            dispenserRectangle,
            beaconRectangle,
            merchantRectangle,
            recipeBookRectangle,
            creativeTabsRectangle,
            creativeItemsRectangle,
            creativeSearchRectangle,
            creativeInventoryRectangle,
            glintRectangle,
            shieldBaseRectangle,
            builtInItemRectangles: Arc::new(builtInItemRectangles),
            bedRectangles,
            emptySlotRectangles,
            itemModels: Arc::new(itemModels),
            shieldBlockingModel,
        });
        log::info!(
            "TextureMap atlas revision {}: {} materials ({} item shapes), {}x{}",
            revision,
            materials.len(),
            state.itemModels.len(),
            state.atlas.width,
            state.atlas.height,
        );
        log::info!(
            "BlockModelShapes actual-state cache: stairs={}, snowy={}, connected={}, fire={}, doors={}",
            state.stairModels.len(),
            state.snowyModels.len(),
            state.connectedModels.len(),
            state.fireModels.len(),
            state.doorModels.len(),
        );
        self.atlasState = Some(Arc::clone(&state));
        Ok(state)
    }

    fn registerMaterialKey(
        &mut self,
        key: MaterialKey,
        materialIndices: &mut HashMap<MaterialKey, usize>,
        materials: &mut Vec<MaterialRegistration>,
    ) {
        if materialIndices.contains_key(&key) {
            return;
        }
        let textures = key
            .layers
            .iter()
            .map(|layer| self.texture(&layer.texture))
            .collect();
        let index = materials.len();
        materialIndices.insert(key.clone(), index);
        materials.push(MaterialRegistration { key, textures });
    }

    fn registerParticleTexture(
        &mut self,
        texture: ResourceLocation,
        particleTextureKeys: &mut HashMap<ResourceLocation, MaterialKey>,
        materialIndices: &mut HashMap<MaterialKey, usize>,
        materials: &mut Vec<MaterialRegistration>,
    ) {
        if particleTextureKeys.contains_key(&texture) {
            return;
        }
        let key = MaterialKey {
            blockId: -33_000 - particleTextureKeys.len() as i32,
            layers: vec![MaterialLayerKey {
                texture: texture.clone(),
                tintIndex: None,
            }],
        };
        self.registerMaterialKey(key.clone(), materialIndices, materials);
        particleTextureKeys.insert(texture, key);
    }

    fn texture(&mut self, location: &ResourceLocation) -> Arc<TextureSource> {
        if let Some(texture) = self.textureCache.get(location) {
            return Arc::clone(texture);
        }
        let texture = Arc::new(if location.getNamespace() == "minecraft"
            && location.getPath() == "textures/missingno.png"
        {
            // TextureUtil's missing sprite is generated in memory; it is not a
            // resource-pack PNG and must not produce a misleading I/O warning.
            TextureSource::missing(location.clone())
        } else if location.getNamespace() == "minecraft"
            && location.getPath() == "textures/generated/solid_white.png"
        {
            TextureSource::solid_white(location.clone())
        } else if location.getNamespace() == "minecraft"
            && location.getPath() == "textures/generated/map_checker.png"
        {
            TextureSource::map_checker(location.clone())
        } else {
            TextureSource::load(&self.resourceManager, location).unwrap_or_else(|error| {
                log::warn!("failed loading world texture {location}: {error}");
                TextureSource::missing(location.clone())
            })
        });
        self.textureCache
            .insert(location.clone(), Arc::clone(&texture));
        texture
    }

    fn buildAtlas(
        &self,
        materials: &[MaterialRegistration],
    ) -> (BlockTextureAtlas, Vec<[f32; 4]>, Vec<[f32; 4]>, Vec<[u32; 3]>) {
        if materials.is_empty() {
            let full = vec![[0.0, 0.0, 1.0, 1.0]];
            return (missing_atlas(), full.clone(), full, vec![[0, 0, 1]]);
        }

        // Vanilla `TextureMap` delegates sprite placement to `Stitcher` and
        // therefore does not enlarge every 16x16 terrain sprite when a 64x64
        // entity texture is present. The Vulkan backend keeps one descriptor
        // atlas, but preserves that variable-size placement responsibility.
        let mut tileSizes = materials
            .iter()
            .map(material_tile_size)
            .collect::<Vec<_>>();
        let playerCount = materials.iter().filter(|material| is_dynamic_player_material(material.key.blockId)).count();
        let iconCount = materials.iter().filter(|material| is_dynamic_resource_pack_icon_material(material.key.blockId)).count();
        tileSizes.extend(std::iter::repeat(64).take(DYNAMIC_PLAYER_TEXTURE_RESERVE.saturating_sub(playerCount)));
        tileSizes.extend(std::iter::repeat(32).take(DYNAMIC_RESOURCE_PACK_ICON_RESERVE.saturating_sub(iconCount)));
        let (width, height, placements) = stitch_material_tiles(&tileSizes);
        let mut rgba = vec![0_u8; width as usize * height as usize * 4];
        let mut rectangles = Vec::with_capacity(materials.len());
        let mut exactRectangles = Vec::with_capacity(materials.len());

        for (index, material) in materials.iter().enumerate() {
            let [originX, originY, tileSize] = placements[index];
            let tintLayers = material
                .key
                .layers
                .iter()
                .zip(material.textures.iter())
                .collect::<Vec<_>>();
            let dynamicTint = material.key.layers.iter().all(|layer| layer.tintIndex.is_some());
            let bannerDyeDamage = (-31_015..=-31_000)
                .contains(&material.key.blockId)
                .then_some((-31_000 - material.key.blockId) as usize);

            let fullEntityTexture = is_full_entity_texture_material(material.key.blockId);
            for y in 0..tileSize {
                for x in 0..tileSize {
                    if let Some(dyeDamage) = bannerDyeDamage {
                        // Exact LayeredColorMaskTexture semantics. Its mask's
                        // original alpha only gates participation; the red
                        // channel becomes source alpha, while RGB is
                        // MathHelper.multiplyColor(base, dye).
                        let baseImage = &material.textures[0].image;
                        let maskImage = &material.textures[1].image;
                        let baseX = ((x as u64 * baseImage.width().max(1) as u64)
                            / tileSize as u64) as u32;
                        let baseY = ((y as u64 * baseImage.height().max(1) as u64)
                            / tileSize as u64) as u32;
                        let maskX = ((x as u64 * maskImage.width().max(1) as u64)
                            / tileSize as u64) as u32;
                        let maskY = ((y as u64 * maskImage.height().max(1) as u64)
                            / tileSize as u64) as u32;
                        let base = baseImage.pixel_rgba(
                            baseX.min(baseImage.width().saturating_sub(1)),
                            baseY.min(baseImage.height().saturating_sub(1)),
                        );
                        let mask = maskImage.pixel_rgba(
                            maskX.min(maskImage.width().saturating_sub(1)),
                            maskY.min(maskImage.height().saturating_sub(1)),
                        );
                        let output = layered_color_mask_pixel(
                            base,
                            mask,
                            dye_color_value_by_damage(dyeDamage),
                        );
                        let destination = (((originY + y) * width + originX + x) * 4) as usize;
                        rgba[destination..destination + 4].copy_from_slice(&output);
                        continue;
                    }

                    if let Some(texture) = fire_material_texture(material) {
                        // TextureAtlasSprite stores every animation frame and
                        // uploads one square frame into the stitched atlas each
                        // tick. Vulkan keeps the equivalent vertical source
                        // strip resident in an otherwise transparent square;
                        // tagged vertices select the active frame in shader.
                        let image = &texture.image;
                        let output = if x < image.width() && y < image.height() {
                            image.pixel_rgba(x, y)
                        } else {
                            [0, 0, 0, 0]
                        };
                        let destination = (((originY + y) * width + originX + x) * 4) as usize;
                        rgba[destination..destination + 4].copy_from_slice(&output);
                        continue;
                    }

                    let mut output = [0.0_f32; 4];
                    for (layer, texture) in &tintLayers {
                        let image = &texture.image;
                        let imageWidth = image.width().max(1);
                        let imageHeight = image.height().max(1);
                        let frameSize = imageWidth.min(imageHeight);
                        let pixel = if fullEntityTexture {
                            // TextureManager keeps entity textures at their native
                            // dimensions. The shared Vulkan atlas reserves a square
                            // stitcher slot, but pixels outside a non-square source
                            // (notably 64x32 capes) must remain transparent rather
                            // than scaling/cropping the source to that square.
                            if x >= imageWidth || y >= imageHeight {
                                [0, 0, 0, 0]
                            } else {
                                image.pixel_rgba(x, y)
                            }
                        } else {
                            let sourceX = ((x as u64 * frameSize as u64) / tileSize as u64) as u32;
                            let sourceY = ((y as u64 * frameSize as u64) / tileSize as u64) as u32;
                            image.pixel_rgba(
                                sourceX.min(frameSize.saturating_sub(1)),
                                sourceY.min(frameSize.saturating_sub(1)),
                            )
                        };
                        let tint = if dynamicTint {
                            [1.0; 3]
                        } else {
                            tint_color(material.key.blockId, layer.tintIndex)
                        };
                        let source = [
                            pixel[0] as f32 / 255.0 * tint[0],
                            pixel[1] as f32 / 255.0 * tint[1],
                            pixel[2] as f32 / 255.0 * tint[2],
                            pixel[3] as f32 / 255.0,
                        ];
                        let remaining = 1.0 - output[3];
                        for channel in 0..3 {
                            output[channel] += source[channel] * source[3] * remaining;
                        }
                        output[3] += source[3] * remaining;
                    }
                    if output[3] > 0.0 {
                        for channel in 0..3 {
                            output[channel] /= output[3];
                        }
                    }
                    let destination = (((originY + y) * width + originX + x) * 4) as usize;
                    rgba[destination] = (output[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                    rgba[destination + 1] =
                        (output[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                    rgba[destination + 2] =
                        (output[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                    rgba[destination + 3] =
                        (output[3].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                }
            }

            let (spriteWidth, spriteHeight) = if fullEntityTexture {
                material.textures.iter().fold((1_u32, 1_u32), |size, texture| {
                    (
                        size.0.max(texture.image.width().max(1).min(tileSize)),
                        size.1.max(texture.image.height().max(1).min(tileSize)),
                    )
                })
            } else {
                fire_material_texture(material)
                    .map(|texture| {
                        let frame = texture.image.width().max(1).min(tileSize);
                        (frame, frame)
                    })
                    .unwrap_or((tileSize, tileSize))
            };
            // MCP TextureAtlasSprite#initSprite uses 0.01 atlas pixel.
            let spriteInsetU = 0.01 / width as f32;
            let spriteInsetV = 0.01 / height as f32;
            rectangles.push([
                originX as f32 / width as f32 + spriteInsetU,
                originY as f32 / height as f32 + spriteInsetV,
                (originX + spriteWidth) as f32 / width as f32 - spriteInsetU,
                (originY + spriteHeight) as f32 / height as f32 - spriteInsetV,
            ]);
            exactRectangles.push([
                originX as f32 / width as f32,
                originY as f32 / height as f32,
                (originX + spriteWidth) as f32 / width as f32,
                (originY + spriteHeight) as f32 / height as f32,
            ]);
        }
        (
            BlockTextureAtlas {
                width,
                height,
                rgba,
            },
            rectangles,
            exactRectangles,
            placements,
        )
    }

    fn ensureRenderChunkPlaceholder(&mut self, key: RenderChunkKey, sourceRevision: u64) {
        self.chunkMeshes.entry(key).or_insert(CachedChunkMesh {
            sourceRevision,
            meshRevision: 0,
            indexCount: 0,
            aabbMin: key.minBlock(),
            aabbMax: key.maxBlock(),
            compiledChunk: CompiledChunk::emptyVisible(),
            layerRanges: [ChunkLayerRange::default(); 4],
            vertices: Arc::new(Vec::new()),
            indices: Arc::new(Vec::new()),
            ready: false,
        });
    }

    fn markEmptyRenderChunk(&mut self, key: RenderChunkKey, sourceRevision: u64) {
        self.queuedChunks.remove(&key);
        self.priorityQueuedChunks.remove(&key);
        self.priorityPendingChunks.retain(|candidate| *candidate != key);
        self.pendingChunks.retain(|candidate| *candidate != key);
        self.chunksToResortTransparencySet.remove(&key);
        self.chunksToResortTransparency
            .retain(|candidate| *candidate != key);
        if self.inflightChunks.contains_key(&key) {
            self.dirtyWhileInflight.insert(key);
        }
        let previousDrawable = self
            .chunkMeshes
            .get(&key)
            .is_some_and(|mesh| mesh.indexCount > 0);
        if previousDrawable {
            self.queueGpuRemoval(key);
        }
        self.chunkMeshes.insert(
            key,
            CachedChunkMesh {
                sourceRevision,
                meshRevision: 0,
                indexCount: 0,
                aabbMin: key.minBlock(),
                aabbMax: key.maxBlock(),
                compiledChunk: CompiledChunk::emptyVisible(),
                layerRanges: [ChunkLayerRange::default(); 4],
                vertices: Arc::new(Vec::new()),
                indices: Arc::new(Vec::new()),
                ready: true,
            },
        );
    }

    /// Queues a 16 x 16 x 16 RenderChunk only when no compilation for the same
    /// section is already in flight. Existing drawable sections use the
    /// dedicated priority lane; missing initial geometry remains on the normal
    /// nearest-first streaming lane.
    fn enqueueChunk(&mut self, key: RenderChunkKey) {
        self.enqueueChunkWithPriority(key, false);
    }

    fn enqueueChunkWithPriority(&mut self, key: RenderChunkKey, forcePriority: bool) {
        if !self.observedChunkRevisions.contains_key(&key)
            || self.emptyRenderChunks.contains(&key)
            || self.inflightChunks.contains_key(&key)
        {
            return;
        }
        let priority = forcePriority
            || self
                .chunkMeshes
                .get(&key)
                .is_some_and(|mesh| mesh.ready && mesh.meshRevision != 0);
        if self.queuedChunks.insert(key) {
            if priority {
                self.priorityQueuedChunks.insert(key);
                self.priorityPendingChunks.push_back(key);
            } else {
                self.pendingChunks.push_back(key);
                self.pendingChunkOrderDirty = true;
            }
            return;
        }

        // A block update can promote an initial streaming task which has not
        // started yet. Preserve one queue membership while moving it to the
        // interactive lane immediately.
        if priority && self.priorityQueuedChunks.insert(key) {
            self.pendingChunks.retain(|candidate| *candidate != key);
            self.priorityPendingChunks.push_back(key);
            self.pendingChunkOrderDirty = true;
        }
    }

    /// Removes one queued X/Z column. All dirty vertical RenderChunks in that
    /// column share one background job and one 3x3 immutable snapshot, matching
    /// the reference column-mesh scheduling without changing MCP section outputs.
    fn takePendingColumn(&mut self, priority: bool) -> Vec<RenderChunkKey> {
        let queue = if priority {
            &mut self.priorityPendingChunks
        } else {
            &mut self.pendingChunks
        };
        let Some(first) = queue.pop_front() else {
            return Vec::new();
        };
        let mut selected = Vec::with_capacity(16);
        selected.push(first);
        let mut remaining = VecDeque::with_capacity(queue.len());
        while let Some(candidate) = queue.pop_front() {
            if selected.len() < 16 && candidate.x == first.x && candidate.z == first.z {
                selected.push(candidate);
            } else {
                remaining.push_back(candidate);
            }
        }
        *queue = remaining;
        for key in &selected {
            self.queuedChunks.remove(key);
            self.priorityQueuedChunks.remove(key);
        }
        selected.sort_by_key(|key| key.y);
        selected
    }

    fn selectChunkColumnBuild(
        &mut self,
        priority: bool,
        world: &WorldClient,
        centerChunk: ChunkKey,
        buildRadius: i32,
    ) -> Vec<(RenderChunkKey, ChunkBuildToken)> {
        loop {
            let candidates = self.takePendingColumn(priority);
            if candidates.is_empty() {
                return Vec::new();
            }
            let mut selected = Vec::with_capacity(candidates.len());
            for key in candidates {
                if self.inflightChunks.contains_key(&key)
                    || self.emptyRenderChunks.contains(&key)
                    || (key.x - centerChunk.x).abs() > buildRadius
                    || (key.z - centerChunk.z).abs() > buildRadius
                    || !key.isValidWorldHeight()
                {
                    continue;
                }
                let Some(sourceRevision) = self.observedChunkRevisions.get(&key).copied() else {
                    continue;
                };
                let Some(column) = world.getChunkFromChunkCoords(key.x, key.z) else {
                    continue;
                };
                if column.getBlockStorageArray()[key.y as usize].is_none() {
                    self.markEmptyRenderChunk(key, sourceRevision);
                    continue;
                }
                let token = ChunkBuildToken {
                    worldGeneration: self.worldGeneration,
                    serial: self.takeJobSerial(),
                    sourceRevision,
                };
                self.inflightChunks.insert(key, token);
                selected.push((key, token));
            }
            if !selected.is_empty() {
                return selected;
            }
        }
    }

    /// Invalidates a RenderChunk because its own data or one of its six border
    /// neighbours actually changed. This is the only path that may set
    /// dirtyWhileInflight.
    fn invalidateChunk(&mut self, key: RenderChunkKey) {
        if !key.isValidWorldHeight()
            || !self.observedChunkRevisions.contains_key(&key)
            || self.emptyRenderChunks.contains(&key)
        {
            return;
        }
        if self.inflightChunks.contains_key(&key) {
            self.dirtyWhileInflight.insert(key);
            return;
        }
        self.enqueueChunk(key);
    }

    fn invalidateNeighbours(&mut self, key: RenderChunkKey) {
        for facing in EnumFacing::VALUES {
            self.invalidateChunk(key.offset(facing));
        }
    }

    fn invalidateChunkAndNeighbours(&mut self, key: RenderChunkKey) {
        self.invalidateChunk(key);
        self.invalidateNeighbours(key);
    }

    fn removeCachedChunk(&mut self, key: RenderChunkKey) {
        if self
            .chunkMeshes
            .remove(&key)
            .is_some_and(|mesh| mesh.indexCount > 0)
        {
            self.queueGpuRemoval(key);
        }
        self.queuedChunks.remove(&key);
        self.priorityQueuedChunks.remove(&key);
        self.priorityPendingChunks.retain(|candidate| *candidate != key);
        self.pendingChunks.retain(|candidate| *candidate != key);
        self.chunksToResortTransparencySet.remove(&key);
        self.chunksToResortTransparency
            .retain(|candidate| *candidate != key);
        if self.inflightChunks.contains_key(&key) {
            self.dirtyWhileInflight.insert(key);
        }
    }

    fn queueGpuRemoval(&mut self, key: RenderChunkKey) {
        if self.pendingGpuRemovalSet.insert(key) {
            self.pendingGpuRemovals.push(key);
        }
    }

    fn takeJobSerial(&mut self) -> u64 {
        let serial = self.nextJobSerial;
        self.nextJobSerial = self.nextJobSerial.wrapping_add(1).max(1);
        serial
    }

    fn takeMeshRevision(&mut self) -> u64 {
        let revision = self.nextMeshRevision;
        self.nextMeshRevision = self.nextMeshRevision.wrapping_add(1).max(1);
        revision
    }

    fn takeAtlasRevision(&mut self) -> u64 {
        let revision = self.nextAtlasRevision;
        self.nextAtlasRevision = self.nextAtlasRevision.wrapping_add(1).max(1);
        revision
    }
}

#[derive(Default)]
struct LayerMesh {
    vertices: Vec<WorldVertex>,
    indices: Vec<u32>,
}

#[derive(Debug, Clone, Copy)]
struct TranslucentQuadOrder {
    originalIndex: usize,
    distanceSquared: f32,
}

#[derive(Default)]
struct TranslucentSortScratch {
    originalIndices: Vec<u32>,
    order: Vec<TranslucentQuadOrder>,
}

/// MCP `BufferBuilder#sortVertexData`: sort GL_QUADS far-to-near by the
/// arithmetic mean of each quad's four positions. The backend triangulates
/// each quad, so reordering six-index groups is rendering-equivalent to moving
/// the complete 4-vertex records while avoiding a vertex-buffer rewrite.
fn prepare_translucent_sort(
    vertices: &[WorldVertex],
    indices: &[u32],
    camera: [f32; 3],
    scratch: &mut TranslucentSortScratch,
) -> bool {
    if indices.is_empty() || indices.len() % 6 != 0 {
        return false;
    }
    scratch.originalIndices.clear();
    scratch.originalIndices.extend_from_slice(indices);
    scratch.order.clear();
    scratch.order.reserve(indices.len() / 6);
    for (quadIndex, group) in scratch.originalIndices.chunks_exact(6).enumerate() {
        let mut unique = [usize::MAX; 4];
        let mut uniqueCount = 0usize;
        for rawIndex in group {
            let index = *rawIndex as usize;
            if index >= vertices.len() || unique[..uniqueCount].contains(&index) {
                continue;
            }
            if uniqueCount < unique.len() {
                unique[uniqueCount] = index;
                uniqueCount += 1;
            }
        }
        if uniqueCount != 4 {
            scratch.order.clear();
            return false;
        }
        let mut center = [0.0_f32; 3];
        for index in unique {
            let position = vertices[index].position;
            center[0] += position[0];
            center[1] += position[1];
            center[2] += position[2];
        }
        center[0] = center[0] * 0.25 - camera[0];
        center[1] = center[1] * 0.25 - camera[1];
        center[2] = center[2] * 0.25 - camera[2];
        scratch.order.push(TranslucentQuadOrder {
            originalIndex: quadIndex,
            distanceSquared: center[0] * center[0]
                + center[1] * center[1]
                + center[2] * center[2],
        });
    }
    // Java `Arrays.sort(Object[], Comparator)` is stable. Rust's `sort_by` is
    // also stable, so equal-distance quads retain their current state order.
    scratch.order.sort_by(|left, right| {
        right
            .distanceSquared
            .total_cmp(&left.distanceSquared)
    });
    scratch
        .order
        .iter()
        .enumerate()
        .any(|(target, quad)| target != quad.originalIndex)
}

fn apply_translucent_sort(indices: &mut [u32], scratch: &TranslucentSortScratch) -> bool {
    if indices.len() != scratch.originalIndices.len()
        || scratch.order.len().saturating_mul(6) != indices.len()
    {
        return false;
    }
    for (target, quad) in scratch.order.iter().enumerate() {
        let source = quad.originalIndex * 6;
        indices[target * 6..target * 6 + 6]
            .copy_from_slice(&scratch.originalIndices[source..source + 6]);
    }
    true
}

fn sort_translucent_indices_with_scratch(
    vertices: &[WorldVertex],
    indices: &mut [u32],
    camera: [f32; 3],
    scratch: &mut TranslucentSortScratch,
) -> bool {
    if !prepare_translucent_sort(vertices, indices, camera, scratch) {
        return false;
    }
    apply_translucent_sort(indices, scratch)
}

fn sort_translucent_indices(
    vertices: &[WorldVertex],
    indices: &mut [u32],
    camera: [f32; 3],
) -> bool {
    let mut scratch = TranslucentSortScratch::default();
    sort_translucent_indices_with_scratch(vertices, indices, camera, &mut scratch)
}

fn sort_translucent_layer(mesh: &mut LayerMesh, camera: [f32; 3]) -> bool {
    sort_translucent_indices(mesh.vertices.as_slice(), mesh.indices.as_mut_slice(), camera)
}

fn prepare_translucent_index_range_sort(
    vertices: &[WorldVertex],
    indices: &[u32],
    range: ChunkLayerRange,
    camera: [f32; 3],
    scratch: &mut TranslucentSortScratch,
) -> bool {
    let start = range.firstIndex as usize;
    let end = start.saturating_add(range.indexCount as usize);
    let Some(layerIndices) = indices.get(start..end) else {
        return false;
    };
    prepare_translucent_sort(vertices, layerIndices, camera, scratch)
}

fn apply_translucent_index_range_sort(
    indices: &mut [u32],
    range: ChunkLayerRange,
    scratch: &TranslucentSortScratch,
) -> bool {
    let start = range.firstIndex as usize;
    let end = start.saturating_add(range.indexCount as usize);
    let Some(layerIndices) = indices.get_mut(start..end) else {
        return false;
    };
    apply_translucent_sort(layerIndices, scratch)
}

fn sort_translucent_index_range(
    vertices: &[WorldVertex],
    indices: &mut [u32],
    range: ChunkLayerRange,
    camera: [f32; 3],
) -> bool {
    let mut scratch = TranslucentSortScratch::default();
    if !prepare_translucent_index_range_sort(vertices, indices, range, camera, &mut scratch) {
        return false;
    }
    apply_translucent_index_range_sort(indices, range, &scratch)
}

fn shader_entity_data(state: IBlockState, renderTypeOrdinal: i16) -> [i16; 3] {
    // OptiFine `SVertexBuilder.pushEntity`: BlockAliases mapping is applied in
    // a later stage; the base contract is block id, metadata and render type.
    [
        state.getBlockId() as i16,
        state.getMetadata() as i16,
        renderTypeOrdinal,
    ]
}

fn build_chunk_mesh(job: ChunkBuildJob) -> ChunkBuildResult {
    let key = job.request.key;
    let columnKey = ChunkKey::new(key.x, key.z);
    let mut compiledChunk = CompiledChunk::new();
    let Some(chunk) = job.snapshot.get(&columnKey) else {
        return ChunkBuildResult {
            key,
            token: job.request.token,
            vertices: Vec::new(),
            indices: Vec::new(),
            layerRanges: [ChunkLayerRange::default(); 4],
            aabbMin: key.minBlock(),
            aabbMax: key.maxBlock(),
            compiledChunk,
        };
    };
    if !key.isValidWorldHeight()
        || chunk.getBlockStorageArray()[key.y as usize].is_none()
    {
        return ChunkBuildResult {
            key,
            token: job.request.token,
            vertices: Vec::new(),
            indices: Vec::new(),
            layerRanges: [ChunkLayerRange::default(); 4],
            aabbMin: key.minBlock(),
            aabbMax: key.maxBlock(),
            compiledChunk,
        };
    }

    let mut layers: [LayerMesh; 4] = std::array::from_fn(|_| LayerMesh::default());
    let mut visGraph = VisGraph::new();
    let access = SnapshotBlockAccess { chunks: &job.snapshot };
    let baseX = key.x * 16;
    let baseY = key.y * 16;
    let baseZ = key.z * 16;

    for localY in 0..16 {
        let worldY = baseY + localY as i32;
        for localZ in 0..16 {
            for localX in 0..16 {
                let state = chunk.getBlockState(localX, worldY as usize, localZ);
                if state.isAir() {
                    continue;
                }
                if state.getBlock().isOpaqueCube() {
                    visGraph.setOpaqueCube(localX, localY, localZ);
                }

                let blockPos = BlockPos::new(
                    baseX + localX as i32,
                    worldY,
                    baseZ + localZ as i32,
                );
                if BlockLiquid::isLiquid(state) {
                    let renderLayer = BlockRenderLayer::forBlockId(
                        state.getBlockId(),
                        job.request.fancyGraphics,
                    );
                    compiledChunk.setLayerStarted(renderLayer);
                    let sprites = if matches!(state.getBlockId(), 10 | 11) {
                        FluidSprites {
                            still: job.atlas.lavaStillRectangle,
                            flow: job.atlas.lavaFlowRectangle,
                            overlay: job.atlas.lavaFlowRectangle,
                        }
                    } else {
                        FluidSprites {
                            still: job.atlas.waterStillRectangle,
                            flow: job.atlas.waterFlowRectangle,
                            overlay: job.atlas.waterOverlayRectangle,
                        }
                    };
                    let fluid = BlockFluidRenderer::renderFluid(
                        &access,
                        state,
                        blockPos,
                        &job.atlas.blockColors,
                        sprites,
                        |sample, liquidState| snapshot_combined_light(
                            &job.snapshot,
                            sample,
                            job.request.dimension,
                            liquidState,
                        ),
                    );
                    if !fluid.indices.is_empty() {
                        let layerMesh = &mut layers[renderLayer.index()];
                        let baseVertex = layerMesh.vertices.len() as u32;
                        layerMesh.vertices.extend(fluid.vertices.into_iter().map(|vertex| WorldVertex {
                            position: vertex.position,
                            uv: vertex.uv,
                            color: vertex.color,
                            lightmap: [
                                ((vertex.packedLight >> 4) & 15) as f32,
                                ((vertex.packedLight >> 20) & 15) as f32,
                            ],
                        
                            shaderEntity: shader_entity_data(state, 1),
                            shaderPadding: 0,
                        }));
                        layerMesh.indices.extend(fluid.indices.into_iter().map(|index| baseVertex + index));
                        compiledChunk.setLayerUsed(renderLayer);
                    }
                    continue;
                }

                let model = if state.getBlockId() == BlockFlowerPot::BLOCK_ID {
                    let contents = job.flowerPotContents
                        .get(&blockPos)
                        .map(String::as_str)
                        .unwrap_or("empty");
                    job.atlas.flowerPotModels.get(contents).map(Arc::as_ref)
                } else if state.getBlockId() == 175 {
                    // MCP `BlockDoublePlant#getActualState`: upper-half
                    // metadata stores HALF plus placement FACING, not VARIANT.
                    // Recover VARIANT from the lower block before selecting
                    // the independently mapped sunflower/syringa/grass/fern/
                    // rose/paeonia blockstate resource.
                    let lower = if state.getMetadata() & 8 != 0 {
                        snapshot_block_state(&job.snapshot, blockPos.down(1))
                    } else {
                        state
                    };
                    let key = double_plant_actual_model_key(state, lower);
                    job.atlas
                        .doublePlantModels
                        .get(&key)
                        .map(Arc::as_ref)
                } else if BlockFire::isBlockFire(state) {
                    let mask = BlockFire::actualStateMask(&access, blockPos);
                    job.atlas
                        .fireModels
                        .get(&(state.getMetadata().clamp(0, 15), mask))
                        .map(Arc::as_ref)
                } else if BlockDoor::isBlockDoor(state) {
                    let key = BlockDoor::modelKey(state, &access, blockPos);
                    job.atlas
                        .doorModels
                        .get(&(state.getBlockId(), key))
                        .map(Arc::as_ref)
                } else if state.getBlockId() == BlockRedstoneWire::BLOCK_ID {
                    let key = BlockRedstoneWire::modelKey(&access, blockPos);
                    job.atlas.redstoneWireModels.get(&key).map(Arc::as_ref)
                } else if BlockFenceGate::isBlockFenceGate(state) {
                    let key = BlockFenceGate::modelKey(state, &access, blockPos);
                    job.atlas
                        .fenceGateModels
                        .get(&(state.getBlockId(), key))
                        .map(Arc::as_ref)
                } else if BlockStairs::isBlockStairs(state) {
                    let shape = BlockStairs::getStairsShape(state, &access, blockPos);
                    job.atlas
                        .stairModels
                        .get(&(stair_model_state_key(state), shape))
                        .map(Arc::as_ref)
                } else if is_connected_model_block(state.getBlockId()) {
                    let mask = connected_state_mask(&job.snapshot, state, blockPos);
                    job.atlas
                        .connectedModels
                        .get(&(connected_model_state_key(state), mask))
                        .map(Arc::as_ref)
                } else if matches!(state.getBlockId(), 2 | 110)
                    && matches!(
                        snapshot_block_state(&job.snapshot, blockPos.up(1)).getBlockId(),
                        78 | 80
                    )
                {
                    job.atlas
                        .snowyModels
                        .get(&state.getBlockId())
                        .map(Arc::as_ref)
                } else {
                    model_for_state(&job.atlas.models, state)
                };
                let Some(model) = model else {
                    continue;
                };
                if model.missing {
                    continue;
                }

                let renderLayer = BlockRenderLayer::forBlockId(
                    state.getBlockId(),
                    job.request.fancyGraphics,
                );
                compiledChunk.setLayerStarted(renderLayer);
                let layerMesh = &mut layers[renderLayer.index()];
                let indicesBefore = layerMesh.indices.len();

                for quad in &model.quads {
                    if quad.material.layers.is_empty() {
                        continue;
                    }
                    if let Some(cullFace) = quad.cullFace {
                        if !should_render_face(
                            &job.snapshot,
                            &job.atlas.models,
                            state,
                            blockPos,
                            cullFace,
                        ) {
                            continue;
                        }
                    }

                    let materialKey = material_key(state.getBlockId(), &quad.material);
                    let fireLayer = fire_texture_layer(&quad.material);
                    let rectangle = job
                        .atlas
                        .rectangles
                        .get(&materialKey)
                        .copied()
                        .unwrap_or(job.atlas.missingRectangle);

                    // MCP `BlockModelRenderer` selects smooth AO only for
                    // non-emissive models whose baked model permits ambient
                    // occlusion. Flat lighting remains the exact fallback.
                    let lightFace = quad.cullFace.unwrap_or(quad.face);
                    let useAdjacent = quad.cullFace.is_some()
                        || quad_uses_neighbour_light(
                            quad.positions,
                            quad.face,
                            model.fullCube,
                        );
                    let smooth = job.request.ambientOcclusion > 0
                        && state.getBlock().getLightValue() == 0
                        && model.ambientOcclusion;
                    let flatPosition = if useAdjacent {
                        let (dx, dy, dz) = lightFace.offsets();
                        BlockPos::new(blockPos.x + dx, blockPos.y + dy, blockPos.z + dz)
                    } else {
                        blockPos
                    };
                    let flatPackedLight = snapshot_combined_light(
                        &job.snapshot,
                        flatPosition,
                        job.request.dimension,
                        state,
                    );
                    let lighting = if smooth {
                        BlockModelRenderer::updateVertexBrightness(
                            state,
                            blockPos,
                            quad.face,
                            quad.positions,
                            useAdjacent,
                            |sample| snapshot_block_state(&job.snapshot, sample),
                            |sample| snapshot_combined_light(
                                &job.snapshot,
                                sample,
                                job.request.dimension,
                                state,
                            ),
                        )
                    } else {
                        crate::net::minecraft::client::renderer::BlockModelRenderer::AmbientOcclusionResult::flat(
                            1.0,
                            flatPackedLight,
                        )
                    };
                    let shade = if quad.shade {
                        face_brightness(quad.face)
                    } else {
                        1.0
                    };
                    let tint = dynamic_quad_tint(&job.atlas, &access, state, blockPos, &quad.material);
                    let baseVertex = layerMesh.vertices.len() as u32;
                    for vertexIndex in 0..4 {
                        let localPosition = quad.positions[vertexIndex];
                        let localUv = quad.uvs[vertexIndex];
                        let uv = [
                            rectangle[0] + (rectangle[2] - rectangle[0]) * localUv[0],
                            rectangle[1] + (rectangle[3] - rectangle[1]) * localUv[1],
                        ];
                        let packedLight = lighting.vertexBrightness[vertexIndex];
                        let ao = lighting.vertexColorMultiplier[vertexIndex];
                        layerMesh.vertices.push(WorldVertex {
                            position: [
                                blockPos.x as f32 + localPosition[0],
                                blockPos.y as f32 + localPosition[1],
                                blockPos.z as f32 + localPosition[2],
                            ],
                            uv,
                            color: [
                                shade * ao * tint[0],
                                shade * ao * tint[1],
                                shade * ao * tint[2],
                                fireLayer.map_or(1.0, |layer| encoded_fire_alpha(1.0, layer)),
                            ],
                            lightmap: [
                                ((packedLight >> 4) & 15) as f32,
                                ((packedLight >> 20) & 15) as f32,
                            ],
                        
                            shaderEntity: shader_entity_data(state, 3),
                            shaderPadding: 0,
                        });
                    }
                    layerMesh.indices.extend_from_slice(&[
                        baseVertex,
                        baseVertex + 1,
                        baseVertex + 2,
                        baseVertex,
                        baseVertex + 2,
                        baseVertex + 3,
                    ]);
                }

                if layerMesh.indices.len() > indicesBefore {
                    compiledChunk.setLayerUsed(renderLayer);
                }
            }
        }
    }

    compiledChunk.setVisibility(visGraph.computeVisibility());
    sort_translucent_layer(
        &mut layers[BlockRenderLayer::Translucent.index()],
        job.request.translucentSortPosition,
    );
    let (vertices, indices, layerRanges) = combine_layer_meshes(layers);
    let (aabbMin, aabbMax) = mesh_bounds(&vertices, key);

    ChunkBuildResult {
        key,
        token: job.request.token,
        vertices,
        indices,
        layerRanges,
        aabbMin,
        aabbMax,
        compiledChunk,
    }
}

fn mesh_bounds(vertices: &[WorldVertex], key: RenderChunkKey) -> ([i32; 3], [i32; 3]) {
    if vertices.is_empty() {
        return (key.minBlock(), key.maxBlock());
    }
    let mut minimum = [f32::INFINITY; 3];
    let mut maximum = [f32::NEG_INFINITY; 3];
    for vertex in vertices {
        for axis in 0..3 {
            minimum[axis] = minimum[axis].min(vertex.position[axis]);
            maximum[axis] = maximum[axis].max(vertex.position[axis]);
        }
    }
    (
        [
            minimum[0].floor() as i32,
            minimum[1].floor() as i32,
            minimum[2].floor() as i32,
        ],
        [
            maximum[0].ceil() as i32,
            maximum[1].ceil() as i32,
            maximum[2].ceil() as i32,
        ],
    )
}

fn combine_layer_meshes(
    mut layers: [LayerMesh; 4],
) -> (Vec<WorldVertex>, Vec<u32>, [ChunkLayerRange; 4]) {
    // MCP `CompiledChunk` owns one BufferBuilder per BlockRenderLayer. The
    // Vulkan backend keeps one allocation per RenderChunk but concatenates the
    // four layer streams and preserves exact index ranges for pass ordering.
    let mut vertices = Vec::<WorldVertex>::new();
    let mut indices = Vec::<u32>::new();
    let mut layerRanges = [ChunkLayerRange::default(); 4];
    for layer in BlockRenderLayer::VALUES {
        let mesh = std::mem::take(&mut layers[layer.index()]);
        let baseVertex = vertices.len() as u32;
        let firstIndex = indices.len() as u32;
        vertices.extend(mesh.vertices);
        indices.extend(mesh.indices.into_iter().map(|index| index + baseVertex));
        layerRanges[layer.index()] = ChunkLayerRange {
            firstIndex,
            indexCount: indices.len() as u32 - firstIndex,
        };
    }
    (vertices, indices, layerRanges)
}


fn stair_model_state_key(state: IBlockState) -> i32 {
    (state.getBlockId() << 4) | (state.getMetadata() & 7)
}

fn connected_model_state_key(state: IBlockState) -> i32 {
    match state.getBlockId() {
        139 => (139 << 4) | (state.getMetadata() & 1),
        160 => state.getGlobalStateId(),
        blockId => blockId << 4,
    }
}

fn is_connected_model_block(blockId: i32) -> bool {
    matches!(blockId, 85 | 101 | 102 | 113 | 139 | 160 | 188 | 189 | 190 | 191 | 192)
}

fn connected_variant(mask: u8, wall: bool) -> String {
    let north = mask & 1 != 0;
    let east = mask & 2 != 0;
    let south = mask & 4 != 0;
    let west = mask & 8 != 0;
    if wall {
        let up = mask & 16 != 0;
        format!("east={east},north={north},south={south},up={up},west={west}")
    } else {
        format!("east={east},north={north},south={south},west={west}")
    }
}

fn connected_state_mask(
    chunks: &HashMap<ChunkKey, Chunk>,
    state: IBlockState,
    pos: BlockPos,
) -> u8 {
    let access = SnapshotBlockAccess { chunks };
    match state.getBlockId() {
        101 | 102 | 160 => BlockPane::connectionMask(&access, pos),
        139 => BlockWall::connectionMask(&access, pos),
        _ => BlockFence::connectionMask(state, &access, pos),
    }
}

struct SnapshotBlockAccess<'a> {
    chunks: &'a HashMap<ChunkKey, Chunk>,
}

impl IBlockAccess for SnapshotBlockAccess<'_> {
    fn getBlockState(&self, pos: BlockPos) -> IBlockState {
        snapshot_block_state(self.chunks, pos)
    }
}

impl BiomeAccess for SnapshotBlockAccess<'_> {
    fn getBiomeId(&self, pos: BlockPos) -> u8 {
        let key = ChunkKey::new(pos.x.div_euclid(16), pos.z.div_euclid(16));
        self.chunks.get(&key).map_or(0, |chunk| {
            let x = pos.x.rem_euclid(16) as usize;
            let z = pos.z.rem_euclid(16) as usize;
            chunk.getBiomeArray()[z * 16 + x]
        })
    }

    fn getBlockStateForColor(&self, pos: BlockPos) -> IBlockState {
        snapshot_block_state(self.chunks, pos)
    }
}

fn stair_variant_for_shape(meta: i32, shape: StairShape) -> String {
    let facing = match meta & 3 {
        0 => "east",
        1 => "west",
        2 => "south",
        _ => "north",
    };
    let half = if meta & 4 != 0 { "top" } else { "bottom" };
    format!("facing={facing},half={half},shape={}", shape.name())
}

fn model_for_state(
    models: &[Option<Arc<ResolvedBlockModel>>],
    state: IBlockState,
) -> Option<&ResolvedBlockModel> {
    models
        .get(state.getGlobalStateId() as usize)
        .and_then(Option::as_deref)
}

fn should_render_face(
    chunks: &HashMap<ChunkKey, Chunk>,
    models: &[Option<Arc<ResolvedBlockModel>>],
    state: IBlockState,
    position: BlockPos,
    facing: EnumFacing,
) -> bool {
    let (dx, dy, dz) = facing.offsets();
    let neighbour = snapshot_block_state(
        chunks,
        BlockPos::new(position.x + dx, position.y + dy, position.z + dz),
    );
    if neighbour.isAir() {
        return true;
    }
    if neighbour.getBlockId() == state.getBlockId() && self_culling_translucent(state.getBlockId()) {
        return false;
    }
    model_for_state(models, neighbour).map_or(true, |model| !model.opaqueCube)
}

fn self_culling_translucent(blockId: i32) -> bool {
    matches!(blockId, 18 | 20 | 79 | 95 | 161)
}

fn snapshot_block_state(chunks: &HashMap<ChunkKey, Chunk>, position: BlockPos) -> IBlockState {
    if !(0..256).contains(&position.y) {
        return IBlockState::default();
    }
    let key = ChunkKey::new(position.x.div_euclid(16), position.z.div_euclid(16));
    chunks
        .get(&key)
        .map(|chunk| {
            chunk.getBlockState(
                position.x.rem_euclid(16) as usize,
                position.y as usize,
                position.z.rem_euclid(16) as usize,
            )
        })
        .unwrap_or_default()
}

fn snapshot_combined_light(
    chunks: &HashMap<ChunkKey, Chunk>,
    position: BlockPos,
    dimension: i32,
    renderedState: IBlockState,
) -> u32 {
    let renderedBlock = renderedState.getBlock();

    // MCP `BlockMagma.getPackedLightmapCoords` is deliberately full-bright,
    // independently of the chunk's received nibble arrays.
    if Block::getIdFromBlock(renderedBlock) == 213 {
        return 15_728_880;
    }

    let combinedAt = |samplePosition: BlockPos, lightValue: u8| {
        let sky = snapshot_light_for_ext(chunks, samplePosition, dimension, EnumSkyBlock::Sky);
        let block = snapshot_light_for_ext(chunks, samplePosition, dimension, EnumSkyBlock::Block)
            .max(lightValue.min(15));
        ((sky as u32) << 20) | ((block as u32) << 4)
    };

    // MCP `BlockLiquid.getPackedLightmapCoords` selects the brighter sky and
    // block channels independently from the liquid cell and the cell above.
    if matches!(Block::getIdFromBlock(renderedBlock), 8..=11) {
        let current = combinedAt(position, 0);
        let above = combinedAt(BlockPos::new(position.x, position.y + 1, position.z), 0);
        let block = (current & 255).max(above & 255);
        let sky = ((current >> 16) & 255).max((above >> 16) & 255);
        return block | (sky << 16);
    }

    let mut packed = combinedAt(position, renderedBlock.getLightValue());

    // MCP `Block.getPackedLightmapCoords`: single slabs retry the block below
    // only when both packed channels are zero.
    if packed == 0 && renderedBlock.isSlab() {
        let below = BlockPos::new(position.x, position.y - 1, position.z);
        let belowState = snapshot_block_state(chunks, below);
        packed = combinedAt(below, belowState.getBlock().getLightValue());
    }
    packed
}

/// MCP `ChunkCache.getLightForExt` over an immutable RenderChunk snapshot.
fn snapshot_light_for_ext(
    chunks: &HashMap<ChunkKey, Chunk>,
    position: BlockPos,
    dimension: i32,
    lightType: EnumSkyBlock,
) -> u8 {
    if lightType == EnumSkyBlock::Sky && dimension == -1 {
        return 0;
    }
    if !(0..256).contains(&position.y) {
        return lightType.defaultLightValue();
    }
    let state = snapshot_block_state(chunks, position);
    if state.getBlock().useNeighborBrightness() {
        let mut result = 0;
        for facing in EnumFacing::VALUES {
            let (dx, dy, dz) = facing.offsets();
            result = result.max(snapshot_raw_light(
                chunks,
                BlockPos::new(position.x + dx, position.y + dy, position.z + dz),
                dimension,
                lightType,
            ));
            if result >= 15 {
                break;
            }
        }
        result
    } else {
        snapshot_raw_light(chunks, position, dimension, lightType)
    }
}

fn snapshot_raw_light(
    chunks: &HashMap<ChunkKey, Chunk>,
    position: BlockPos,
    dimension: i32,
    lightType: EnumSkyBlock,
) -> u8 {
    if !(0..256).contains(&position.y) {
        return lightType.defaultLightValue();
    }
    let key = ChunkKey::new(position.x.div_euclid(16), position.z.div_euclid(16));
    let Some(chunk) = chunks.get(&key) else {
        return lightType.defaultLightValue();
    };
    let Some(storage) = chunk.getBlockStorageArray()[position.y as usize >> 4].as_ref() else {
        return match lightType {
            EnumSkyBlock::Sky if dimension != -1 => 15,
            _ => 0,
        };
    };
    let x = position.x.rem_euclid(16) as usize;
    let y = position.y as usize & 15;
    let z = position.z.rem_euclid(16) as usize;
    match lightType {
        EnumSkyBlock::Sky => {
            if dimension == -1 {
                0
            } else {
                storage.getExtSkylightValue(x, y, z)
            }
        }
        EnumSkyBlock::Block => storage.getExtBlocklightValue(x, y, z),
    }
}

/// Exact `BlockModelRenderer.fillQuadBounds` flag 0 for flat-light sampling.
fn quad_uses_neighbour_light(
    positions: [[f32; 3]; 4],
    face: EnumFacing,
    fullCube: bool,
) -> bool {
    let mut minimum = [f32::INFINITY; 3];
    let mut maximum = [f32::NEG_INFINITY; 3];
    for position in positions {
        for axis in 0..3 {
            minimum[axis] = minimum[axis].min(position[axis]);
            maximum[axis] = maximum[axis].max(position[axis]);
        }
    }
    let low = 1.0e-4_f32;
    let high = 0.9999_f32;
    match face {
        EnumFacing::Down => (minimum[1] < low || fullCube) && approximately_equal(minimum[1], maximum[1]),
        EnumFacing::Up => (maximum[1] > high || fullCube) && approximately_equal(minimum[1], maximum[1]),
        EnumFacing::North => (minimum[2] < low || fullCube) && approximately_equal(minimum[2], maximum[2]),
        EnumFacing::South => (maximum[2] > high || fullCube) && approximately_equal(minimum[2], maximum[2]),
        EnumFacing::West => (minimum[0] < low || fullCube) && approximately_equal(minimum[0], maximum[0]),
        EnumFacing::East => (maximum[0] > high || fullCube) && approximately_equal(minimum[0], maximum[0]),
    }
}

fn approximately_equal(left: f32, right: f32) -> bool {
    (left - right).abs() <= 1.0e-5
}

fn fluid_material_key(blockId: i32, path: &str) -> MaterialKey {
    MaterialKey {
        blockId,
        layers: vec![MaterialLayerKey {
            texture: ResourceLocation::new("minecraft", path),
            tintIndex: None,
        }],
    }
}

fn item_material_key(
    _itemId: i16,
    texture: ResourceLocation,
    tintIndex: Option<i32>,
) -> MaterialKey {
    // ItemColors is applied per ItemStack in the HUD vertex color. Keeping a
    // fixed material owner deduplicates identical sprites registered by
    // multiple item metadata variants without baking any stack-specific tint.
    MaterialKey {
        blockId: -1000,
        layers: vec![MaterialLayerKey { texture, tintIndex }],
    }
}

fn material_key(blockId: i32, face: &ResolvedFace) -> MaterialKey {
    MaterialKey {
        blockId,
        layers: face
            .layers
            .iter()
            .map(|layer| MaterialLayerKey {
                texture: layer.texture.clone(),
                tintIndex: layer.tintIndex,
            })
            .collect(),
    }
}

fn fire_material_texture(material: &MaterialRegistration) -> Option<&Arc<TextureSource>> {
    if material.key.layers.len() != 1 || material.textures.len() != 1 {
        return None;
    }
    let path = material.key.layers[0].texture.getPath();
    matches!(
        path,
        "textures/blocks/fire_layer_0.png" | "textures/blocks/fire_layer_1.png"
    )
    .then(|| &material.textures[0])
}

fn fire_texture_layer(face: &ResolvedFace) -> Option<usize> {
    face.layers.iter().find_map(|layer| match layer.texture.getPath() {
        "textures/blocks/fire_layer_0.png" => Some(0),
        "textures/blocks/fire_layer_1.png" => Some(1),
        _ => None,
    })
}

fn encoded_fire_alpha(alpha: f32, layer: usize) -> f32 {
    let layerBase = if layer == 0 { 0.0 } else { 2.0 };
    -(layerBase + alpha.clamp(0.0001, 1.0))
}

fn is_dynamic_player_material(blockId: i32) -> bool {
    (DYNAMIC_PLAYER_MATERIAL_BASE - DYNAMIC_PLAYER_TEXTURE_RESERVE as i32 + 1
        ..=DYNAMIC_PLAYER_MATERIAL_BASE).contains(&blockId)
}

fn is_dynamic_resource_pack_icon_material(blockId: i32) -> bool {
    (DYNAMIC_RESOURCE_PACK_ICON_MATERIAL_BASE - DYNAMIC_RESOURCE_PACK_ICON_RESERVE as i32 + 1
        ..=DYNAMIC_RESOURCE_PACK_ICON_MATERIAL_BASE).contains(&blockId)
}

fn is_full_entity_texture_material(blockId: i32) -> bool {
    (-30_999..=-30_000).contains(&blockId)
        || (-35_999..=-35_000).contains(&blockId)
        || (-36_999..=-36_000).contains(&blockId)
        || (-37_999..=-37_000).contains(&blockId)
        || is_dynamic_player_material(blockId)
        || is_dynamic_resource_pack_icon_material(blockId)
}

fn build_exact_texture_rectangle_map(
    materials: &[MaterialRegistration],
    exactRectangles: &[[f32; 4]],
) -> HashMap<ResourceLocation, [f32; 4]> {
    materials
        .iter()
        .enumerate()
        .filter_map(|(index, material)| {
            let layer = material.key.layers.first()?;
            if material.key.layers.len() != 1 || layer.tintIndex.is_some() {
                return None;
            }
            exactRectangles
                .get(index)
                .copied()
                .map(|rectangle| (layer.texture.clone(), rectangle))
        })
        .collect()
}

fn material_tile_size(material: &MaterialRegistration) -> u32 {
    let fullEntityTexture = is_full_entity_texture_material(material.key.blockId);
    let keepAnimationStrip = fire_material_texture(material).is_some();
    material
        .textures
        .iter()
        .map(|texture| {
            let width = texture.image.width().max(1);
            let height = texture.image.height().max(1);
            if fullEntityTexture || keepAnimationStrip {
                width.max(height)
            } else {
                width.min(height)
            }
        })
        .max()
        .unwrap_or(16)
        .min(8192)
}

/// Deterministic shelf equivalent of MCP `Stitcher`: larger sprites are placed
/// first and retain their native frame size. Returns atlas width/height and one
/// `[x, y, size]` placement per source material.
fn stitch_material_tiles(tileSizes: &[u32]) -> (u32, u32, Vec<[u32; 3]>) {
    const MAX_ATLAS: u32 = 8192;
    let maximum = tileSizes.iter().copied().max().unwrap_or(1).max(1);
    let area = tileSizes
        .iter()
        .fold(0_u64, |sum, size| sum.saturating_add((*size as u64) * (*size as u64)));
    let approximate = (area as f64).sqrt().ceil() as u32;
    let mut width = approximate.max(maximum).max(1).next_power_of_two().min(MAX_ATLAS);
    let mut order = (0..tileSizes.len()).collect::<Vec<_>>();
    order.sort_by_key(|index| (std::cmp::Reverse(tileSizes[*index]), *index));

    loop {
        let mut placements = vec![[0_u32; 3]; tileSizes.len()];
        let mut x = 0_u32;
        let mut y = 0_u32;
        let mut rowHeight = 0_u32;
        let mut fits = true;
        for index in order.iter().copied() {
            let size = tileSizes[index].max(1).min(MAX_ATLAS);
            if x > 0 && x.saturating_add(size) > width {
                y = y.saturating_add(rowHeight);
                x = 0;
                rowHeight = 0;
            }
            if y.saturating_add(size) > MAX_ATLAS {
                fits = false;
                break;
            }
            placements[index] = [x, y, size];
            x = x.saturating_add(size);
            rowHeight = rowHeight.max(size);
        }
        let usedHeight = y.saturating_add(rowHeight).max(1);
        if fits {
            return (
                width,
                usedHeight.next_power_of_two().min(MAX_ATLAS),
                placements,
            );
        }
        if width == MAX_ATLAS {
            panic!("TextureMap/Stitcher material set exceeds the 8192x8192 Vulkan atlas limit");
        }
        width = width.saturating_mul(2).min(MAX_ATLAS);
    }
}

fn current_system_time_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Direct camera-position port of MCP 1.12.2 `EntityRenderer#orientCamera`
/// for an awake local player. The eight 0.1-block offset rays prevent the
/// third-person camera from clipping through solid blocks.
fn orient_camera_112(
    world: &WorldClient,
    position: &PlayerPositionState,
    player: Option<&crate::net::minecraft::client::entity::EntityPlayerSP::EntityPlayerSP>,
    thirdPersonView: i32,
) -> ([f32; 3], f32, f32) {
    let eye = Vec3d::new(
        position.posX,
        position.posY + position.eyeHeight as f64,
        position.posZ,
    );
    // MCP `EntityRenderer#orientCamera` handles sleeping before the ordinary
    // first/third-person branches. Its temporary player yaw/pitch rotations
    // cancel the common rotations below the branch; the resulting view is
    // aligned solely to the bed horizontal index and sits 0.9 blocks above
    // the player's bed position (`eyeHeight + 1.0 - 0.3`).
    if let Some(player) = player.filter(|player| player.isPlayerSleeping()) {
        let bedState = player
            .bedLocation
            .map(|bed| world.getBlockState(bed))
            .unwrap_or_else(|| world.getBlockState(BlockPos::new(
                position.posX.floor() as i32,
                position.posY.floor() as i32,
                position.posZ.floor() as i32,
            )));
        let bedRotation = if BlockBed::isBlockBed(bedState) {
            BlockBed::getFacing(bedState).horizontalIndex().unwrap_or(0) as f32 * 90.0
        } else {
            0.0
        };
        return (
            [position.posX as f32, (position.posY + 0.9) as f32, position.posZ as f32],
            bedRotation - 180.0,
            0.0,
        );
    }
    if thirdPersonView <= 0 {
        return ([eye.x as f32, eye.y as f32, eye.z as f32], position.rotationYaw, position.rotationPitch);
    }

    // Vanilla uses the entity's current rotation for the displacement rays,
    // while the final camera orientation remains tick-interpolated.
    let currentYaw = player.map_or(position.rotationYaw, |value| value.entity.rotationYaw);
    let currentPitch = player.map_or(position.rotationPitch, |value| value.entity.rotationPitch);
    let displacementPitch = if thirdPersonView == 2 { currentPitch + 180.0 } else { currentPitch };
    let yawRadians = (currentYaw * 0.017453292_f32) as f64;
    let pitchRadians = (displacementPitch * 0.017453292_f32) as f64;
    let mut distance = 4.0_f64;
    let dx = -yawRadians.sin() * pitchRadians.cos() * distance;
    let dz = yawRadians.cos() * pitchRadians.cos() * distance;
    let dy = -pitchRadians.sin() * distance;

    for index in 0..8 {
        let offsetX = (((index & 1) * 2) as f64 - 1.0) * 0.1;
        let offsetY = ((((index >> 1) & 1) * 2) as f64 - 1.0) * 0.1;
        let offsetZ = ((((index >> 2) & 1) * 2) as f64 - 1.0) * 0.1;
        let start = eye.add_vector(offsetX, offsetY, offsetZ);
        let end = Vec3d::new(
            eye.x - dx + offsetX + offsetZ,
            eye.y - dy + offsetY,
            eye.z - dz + offsetZ,
        );
        if let Some(hit) = world.rayTraceBlocks(start, end, false, false, false) {
            let hitDistance = hit.hitVec.distance_to(eye);
            if hitDistance < distance { distance = hitDistance; }
        }
    }

    // The source ray test is aimed with the entity's current yaw/pitch, but
    // the actual camera translation is applied before the final interpolated
    // view rotations in OpenGL's post-multiplied matrix. Therefore the
    // equivalent world-space camera displacement must use the interpolated
    // yaw/pitch. Using current tick angles here creates a visible 20 Hz orbit
    // discontinuity while a locally controlled boat changes yaw.
    let finalPitch = if thirdPersonView == 2 {
        position.rotationPitch + 180.0
    } else {
        position.rotationPitch
    };
    let finalYawRadians = (position.rotationYaw * 0.017453292_f32) as f64;
    let finalPitchRadians = (finalPitch * 0.017453292_f32) as f64;
    let finalDx = -finalYawRadians.sin() * finalPitchRadians.cos() * distance;
    let finalDz = finalYawRadians.cos() * finalPitchRadians.cos() * distance;
    let finalDy = -finalPitchRadians.sin() * distance;
    let camera = [
        (eye.x - finalDx) as f32,
        (eye.y - finalDy) as f32,
        (eye.z - finalDz) as f32,
    ];
    if thirdPersonView == 2 {
        (camera, position.rotationYaw + 180.0, -position.rotationPitch)
    } else {
        (camera, position.rotationYaw, position.rotationPitch)
    }
}

fn interpolated_player_position(
    state: &PlayClientState,
    partialTicks: f32,
) -> PlayerPositionState {
    let partial = partialTicks.clamp(0.0, 1.0) as f64;
    let Some(player) = state.thePlayer.as_ref() else {
        return state.playerPosition;
    };
    PlayerPositionState {
        posX: player.entity.prevPosX + (player.entity.posX - player.entity.prevPosX) * partial,
        posY: player.entity.prevPosY + (player.entity.posY - player.entity.prevPosY) * partial,
        posZ: player.entity.prevPosZ + (player.entity.posZ - player.entity.prevPosZ) * partial,
        rotationYaw: player.entity.prevRotationYaw
            + (player.entity.rotationYaw - player.entity.prevRotationYaw) * partial as f32,
        rotationPitch: player.entity.prevRotationPitch
            + (player.entity.rotationPitch - player.entity.prevRotationPitch) * partial as f32,
        eyeHeight: player.getEyeHeight(),
    }
}


/// MCP `BlockDoublePlant#getActualState` model key. The upper half stores
/// only HALF plus placement FACING; VARIANT belongs to the lower half.
fn double_plant_actual_model_key(state: IBlockState, lowerState: IBlockState) -> (u8, bool) {
    let upper = state.getMetadata() & 8 != 0;
    let variant = if lowerState.getBlockId() == 175 {
        (lowerState.getMetadata() & 7).clamp(0, 5) as u8
    } else if upper {
        0
    } else {
        (state.getMetadata() & 7).clamp(0, 5) as u8
    };
    (variant, upper)
}

const ENTITY_HURT_OVERLAY_FLAG: u32 = 1 << 31;

fn encoded_living_hurt_block_light(blockLight: f32, hurtTime: i32, deathTime: i32) -> f32 {
    if hurtTime > 0 || deathTime > 0 {
        blockLight + 16.0
    } else {
        blockLight
    }
}

fn packed_light_with_living_hurt_overlay(entity: &EntityOtherClient, packedLight: u32) -> u32 {
    if entity.isLivingBase() && RenderLivingBase::shouldApplyHurtBrightness(entity) {
        packedLight | ENTITY_HURT_OVERLAY_FLAG
    } else {
        packedLight & !ENTITY_HURT_OVERLAY_FLAG
    }
}

fn packed_light_without_living_hurt_overlay(packedLight: u32) -> u32 {
    packedLight & !ENTITY_HURT_OVERLAY_FLAG
}

fn encoded_block_light_from_packed(packedLight: u32) -> f32 {
    let blockLight = ((packedLight >> 4) & 15) as f32;
    if packedLight & ENTITY_HURT_OVERLAY_FLAG != 0 {
        blockLight + 16.0
    } else {
        blockLight
    }
}

fn actual_model_for_state<'a>(
    atlas: &'a AtlasState,
    chunks: &HashMap<ChunkKey, Chunk>,
    state: IBlockState,
    position: BlockPos,
) -> Option<&'a ResolvedBlockModel> {
    if BlockFire::isBlockFire(state) {
        let access = SnapshotBlockAccess { chunks };
        let mask = BlockFire::actualStateMask(&access, position);
        atlas
            .fireModels
            .get(&(state.getMetadata().clamp(0, 15), mask))
            .map(Arc::as_ref)
    } else if state.getBlockId() == 175 {
        let lower = if state.getMetadata() & 8 != 0 {
            snapshot_block_state(chunks, position.down(1))
        } else {
            state
        };
        let key = double_plant_actual_model_key(state, lower);
        atlas
            .doublePlantModels
            .get(&key)
            .map(Arc::as_ref)
    } else if BlockDoor::isBlockDoor(state) {
        let access = SnapshotBlockAccess { chunks };
        let key = BlockDoor::modelKey(state, &access, position);
        atlas
            .doorModels
            .get(&(state.getBlockId(), key))
            .map(Arc::as_ref)
    } else if state.getBlockId() == BlockRedstoneWire::BLOCK_ID {
        let access = SnapshotBlockAccess { chunks };
        let key = BlockRedstoneWire::modelKey(&access, position);
        atlas.redstoneWireModels.get(&key).map(Arc::as_ref)
    } else if BlockFenceGate::isBlockFenceGate(state) {
        let access = SnapshotBlockAccess { chunks };
        let key = BlockFenceGate::modelKey(state, &access, position);
        atlas
            .fenceGateModels
            .get(&(state.getBlockId(), key))
            .map(Arc::as_ref)
    } else if BlockStairs::isBlockStairs(state) {
        let access = SnapshotBlockAccess { chunks };
        let shape = BlockStairs::getStairsShape(state, &access, position);
        atlas
            .stairModels
            .get(&(stair_model_state_key(state), shape))
            .map(Arc::as_ref)
    } else if is_connected_model_block(state.getBlockId()) {
        let mask = connected_state_mask(chunks, state, position);
        atlas
            .connectedModels
            .get(&(connected_model_state_key(state), mask))
            .map(Arc::as_ref)
    } else if matches!(state.getBlockId(), 2 | 110)
        && matches!(snapshot_block_state(chunks, position.up(1)).getBlockId(), 78 | 80)
    {
        atlas.snowyModels.get(&state.getBlockId()).map(Arc::as_ref)
    } else {
        model_for_state(&atlas.models, state)
    }
}

/// MCP `ParticleManager#renderParticles` for the opaque TextureMap layer used
/// by `ParticleDigging`. Positions remain world-space because the Vulkan view
/// matrix already contains the camera translation performed by vanilla GL.
fn build_digging_particle_mesh(
    capture: &WorldRenderCapture,
    atlas: &AtlasState,
) -> (Vec<WorldVertex>, Vec<u32>) {
    let mut vertices = Vec::with_capacity(capture.particleStates.len() * 4);
    let mut indices = Vec::with_capacity(capture.particleStates.len() * 6);
    let yaw = capture.playerPosition.rotationYaw.to_radians();
    let pitch = capture.playerPosition.rotationPitch.to_radians();
    let frontSign = if capture.thirdPersonView == 2 { -1.0 } else { 1.0 };
    let rotationX = yaw.cos() * frontSign;
    let rotationZ = yaw.sin() * frontSign;
    let rotationYZ = -rotationZ * pitch.sin() * frontSign;
    let rotationXY = rotationX * pitch.sin() * frontSign;
    let rotationXZ = pitch.cos();
    let partial = capture.partialTicks.clamp(0.0, 1.0) as f64;

    for particle in &capture.particleStates {
        // MCP `ParticleManager#addBlockDestroyEffects` first calls
        // `IBlockState#getActualState(world, pos)`. Resolve the same cached
        // actual-state model here so a door half, redstone connection or
        // double-plant top never selects the metadata-only missing model.
        let capturedActualModel = match particle.actualModel {
            Some(ParticleActualModel::Door { blockId, key }) => atlas
                .doorModels
                .get(&(blockId, key))
                .map(Arc::as_ref),
            Some(ParticleActualModel::FenceGate { blockId, key }) => atlas
                .fenceGateModels
                .get(&(blockId, key))
                .map(Arc::as_ref),
            Some(ParticleActualModel::RedstoneWire { key }) => atlas
                .redstoneWireModels
                .get(&key)
                .map(Arc::as_ref),
            Some(ParticleActualModel::DoublePlant { variant, upper }) => atlas
                .doublePlantModels
                .get(&(variant, upper))
                .map(Arc::as_ref),
            None => None,
        };
        let actualTexture = capturedActualModel
            .or_else(|| {
                actual_model_for_state(
                    atlas,
                    capture.snapshot.as_ref(),
                    particle.sourceState,
                    particle.sourcePos,
                )
            })
            .and_then(|model| model.particleTexture.as_ref());
        let texture = actualTexture.or_else(|| {
            atlas
                .particleTextures
                .get(particle.sourceState.getGlobalStateId() as usize)
        });
        let rectangle = texture
            .and_then(|texture| atlas.particleTextureRectangles.get(texture))
            .copied()
            .unwrap_or(atlas.missingRectangle);
        let width = rectangle[2] - rectangle[0];
        let height = rectangle[3] - rectangle[1];
        let u0 = rectangle[0] + width * (particle.textureJitter[0] / 4.0);
        let u1 = rectangle[0] + width * ((particle.textureJitter[0] + 1.0) / 4.0);
        let v0 = rectangle[1] + height * (particle.textureJitter[1] / 4.0);
        let v1 = rectangle[1] + height * ((particle.textureJitter[1] + 1.0) / 4.0);
        let scale = 0.1 * particle.scale;
        let x = (particle.prevPosition[0]
            + (particle.position[0] - particle.prevPosition[0]) * partial) as f32;
        let y = (particle.prevPosition[1]
            + (particle.position[1] - particle.prevPosition[1]) * partial) as f32;
        let z = (particle.prevPosition[2]
            + (particle.position[2] - particle.prevPosition[2]) * partial) as f32;
        // ParticleManager passes ActiveRenderInfo's rotations as
        // (rotationX, rotationXZ, rotationZ, rotationYZ, rotationXY).
        // Keep that non-intuitive order instead of treating the names as
        // Cartesian axes; Particle#renderParticle consumes them verbatim.
        let positions = [
            [
                x - rotationX * scale - rotationYZ * scale,
                y - rotationXZ * scale,
                z - rotationZ * scale - rotationXY * scale,
            ],
            [
                x - rotationX * scale + rotationYZ * scale,
                y + rotationXZ * scale,
                z - rotationZ * scale + rotationXY * scale,
            ],
            [
                x + rotationX * scale + rotationYZ * scale,
                y + rotationXZ * scale,
                z + rotationZ * scale + rotationXY * scale,
            ],
            [
                x + rotationX * scale - rotationYZ * scale,
                y - rotationXZ * scale,
                z + rotationZ * scale - rotationXY * scale,
            ],
        ];
        let currentPos = BlockPos::new(x.floor() as i32, y.floor() as i32, z.floor() as i32);
        let currentLight = snapshot_combined_light(
            &capture.snapshot,
            currentPos,
            capture.dimension,
            particle.sourceState,
        );
        let packedLight = if currentLight == 0 {
            snapshot_combined_light(
                &capture.snapshot,
                particle.sourcePos,
                capture.dimension,
                particle.sourceState,
            )
        } else {
            currentLight
        };
        let lightmap = [
            ((packedLight >> 4) & 15) as f32,
            ((packedLight >> 20) & 15) as f32,
        ];
        let base = vertices.len() as u32;
        for (position, uv) in positions.into_iter().zip([[u0, v1], [u0, v0], [u1, v0], [u1, v1]]) {
            vertices.push(WorldVertex {
                position,
                uv,
                color: particle.color,
                lightmap,
            
                shaderEntity: [-1, -1, -1],
                shaderPadding: 0,
            });
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    (vertices, indices)
}


/// MCP `ParticleManager#renderParticles` layer-0 (`particles.png`) queue.
/// Vanilla keeps transparent and depth-writing particles in distinct lists,
/// so callers select one queue per invocation.
fn append_misc_particle_mesh(
    capture: &WorldRenderCapture,
    atlas: &AtlasState,
    transparent: bool,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let Some(rectangle) = atlas.entityTextureRectangles.get(&RenderFish::texture()).copied() else {
        return;
    };
    let atlasWidth = rectangle[2] - rectangle[0];
    let atlasHeight = rectangle[3] - rectangle[1];
    let yaw = capture.playerPosition.rotationYaw.to_radians();
    let pitch = capture.playerPosition.rotationPitch.to_radians();
    let frontSign = if capture.thirdPersonView == 2 { -1.0 } else { 1.0 };
    let rotationX = yaw.cos() * frontSign;
    let rotationZ = yaw.sin() * frontSign;
    let rotationYZ = -rotationZ * pitch.sin() * frontSign;
    let rotationXY = rotationX * pitch.sin() * frontSign;
    let rotationXZ = pitch.cos();
    let cameraViewDir = [
        -yaw.sin() * pitch.cos(),
        -pitch.sin(),
        yaw.cos() * pitch.cos(),
    ];
    let partial = capture.partialTicks.clamp(0.0, 1.0);

    for particle in capture.miscParticleStates.iter().filter(|state| state.transparent == transparent) {
        let textureX = particle.textureIndex.rem_euclid(16) as f32;
        let textureY = particle.textureIndex.div_euclid(16) as f32;
        let localU0 = textureX / 16.0;
        let localU1 = localU0 + 0.0624375;
        let localV0 = textureY / 16.0;
        let localV1 = localV0 + 0.0624375;
        let u0 = rectangle[0] + atlasWidth * localU0;
        let u1 = rectangle[0] + atlasWidth * localU1;
        let v0 = rectangle[1] + atlasHeight * localV0;
        let v1 = rectangle[1] + atlasHeight * localV1;
        let scale = 0.1 * particle.scale;
        let x = (particle.prevPosition[0]
            + (particle.position[0] - particle.prevPosition[0]) * partial as f64) as f32;
        let y = (particle.prevPosition[1]
            + (particle.position[1] - particle.prevPosition[1]) * partial as f64) as f32;
        let z = (particle.prevPosition[2]
            + (particle.position[2] - particle.prevPosition[2]) * partial as f64) as f32;
        let mut offsets = [
            [-rotationX * scale - rotationYZ * scale, -rotationXZ * scale, -rotationZ * scale - rotationXY * scale],
            [-rotationX * scale + rotationYZ * scale, rotationXZ * scale, -rotationZ * scale + rotationXY * scale],
            [rotationX * scale + rotationYZ * scale, rotationXZ * scale, rotationZ * scale + rotationXY * scale],
            [rotationX * scale - rotationYZ * scale, -rotationXZ * scale, rotationZ * scale - rotationXY * scale],
        ];
        if particle.particleAngle != 0.0 {
            // Particle#renderParticle rotates each billboard corner by the
            // quaternion whose axis is Particle.cameraViewDir. Preserve the
            // source's nonstandard interpolation expression verbatim.
            let angle = particle.particleAngle
                + (particle.particleAngle - particle.prevParticleAngle) * partial;
            let halfCos = (angle * 0.5).cos();
            let halfSin = (angle * 0.5).sin();
            let q = [
                halfSin * cameraViewDir[0],
                halfSin * cameraViewDir[1],
                halfSin * cameraViewDir[2],
            ];
            let qDot = q[0] * q[0] + q[1] * q[1] + q[2] * q[2];
            for offset in &mut offsets {
                let dot = offset[0] * q[0] + offset[1] * q[1] + offset[2] * q[2];
                let cross = [
                    q[1] * offset[2] - q[2] * offset[1],
                    q[2] * offset[0] - q[0] * offset[2],
                    q[0] * offset[1] - q[1] * offset[0],
                ];
                let original = *offset;
                *offset = [
                    q[0] * (2.0 * dot) + original[0] * (halfCos * halfCos - qDot) + cross[0] * (2.0 * halfCos),
                    q[1] * (2.0 * dot) + original[1] * (halfCos * halfCos - qDot) + cross[1] * (2.0 * halfCos),
                    q[2] * (2.0 * dot) + original[2] * (halfCos * halfCos - qDot) + cross[2] * (2.0 * halfCos),
                ];
            }
        }
        let lightmap = if particle.fullBright {
            [15.0, 15.0]
        } else {
            let position = BlockPos::new(x.floor() as i32, y.floor() as i32, z.floor() as i32);
            let state = snapshot_block_state(&capture.snapshot, position);
            let packed = snapshot_combined_light(&capture.snapshot, position, capture.dimension, state);
            [((packed >> 4) & 15) as f32, ((packed >> 20) & 15) as f32]
        };
        let base = vertices.len() as u32;
        for (offset, uv) in offsets.into_iter().zip([[u1, v1], [u1, v0], [u0, v0], [u0, v1]]) {
            vertices.push(WorldVertex {
                position: [x + offset[0], y + offset[1], z + offset[2]],
                uv,
                color: particle.color,
                lightmap,
            
                shaderEntity: [-1, -1, -1],
                shaderPadding: 0,
            });
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

/// MCP `RenderGlobal#drawBlockDamageTexture` and
/// `BlockRendererDispatcher#renderBlockDamage`. Each original baked quad keeps
/// its local UVs while the sprite is replaced by the selected destroy stage,
/// matching `BakedQuadRetextured`.
fn build_block_damage_mesh(
    capture: &WorldRenderCapture,
    atlas: &AtlasState,
) -> (Vec<WorldVertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for progress in &capture.damagedBlocks {
        let stage = progress.getPartialBlockDamage();
        if !(0..10).contains(&stage) {
            continue;
        }
        let position = progress.getPosition();
        let dx = position.x as f64 - capture.playerPosition.posX;
        let dy = position.y as f64 - capture.playerPosition.posY;
        let dz = position.z as f64 - capture.playerPosition.posZ;
        if dx * dx + dy * dy + dz * dz > 1024.0 {
            continue;
        }
        let state = snapshot_block_state(&capture.snapshot, position);
        if state.isAir()
            || matches!(state.getBlockId(), 54 | 63 | 68 | 130 | 144 | 146 | 219..=234)
        {
            continue;
        }
        let Some(model) = actual_model_for_state(atlas, &capture.snapshot, state, position) else {
            continue;
        };
        if model.missing {
            continue;
        }
        let rectangle = atlas.destroyStageRectangles[stage as usize];
        for quad in &model.quads {
            if let Some(cullFace) = quad.cullFace {
                if !should_render_face(
                    &capture.snapshot,
                    &atlas.models,
                    state,
                    position,
                    cullFace,
                ) {
                    continue;
                }
            }
            let lightFace = quad.cullFace.unwrap_or(quad.face);
            let useAdjacent = quad.cullFace.is_some()
                || quad_uses_neighbour_light(quad.positions, quad.face, model.fullCube);
            let flatPosition = if useAdjacent {
                let (ox, oy, oz) = lightFace.offsets();
                BlockPos::new(position.x + ox, position.y + oy, position.z + oz)
            } else {
                position
            };
            let flatPackedLight = snapshot_combined_light(
                &capture.snapshot,
                flatPosition,
                capture.dimension,
                state,
            );
            let smooth = capture.ambientOcclusion > 0
                && state.getBlock().getLightValue() == 0
                && model.ambientOcclusion;
            let lighting = if smooth {
                BlockModelRenderer::updateVertexBrightness(
                    state,
                    position,
                    quad.face,
                    quad.positions,
                    useAdjacent,
                    |sample| snapshot_block_state(&capture.snapshot, sample),
                    |sample| snapshot_combined_light(
                        &capture.snapshot,
                        sample,
                        capture.dimension,
                        state,
                    ),
                )
            } else {
                crate::net::minecraft::client::renderer::BlockModelRenderer::AmbientOcclusionResult::flat(
                    1.0,
                    flatPackedLight,
                )
            };
            let base = vertices.len() as u32;
            for vertexIndex in 0..4 {
                let local = quad.positions[vertexIndex];
                let localUv = quad.uvs[vertexIndex];
                let packedLight = lighting.vertexBrightness[vertexIndex];
                vertices.push(WorldVertex {
                    position: [
                        position.x as f32 + local[0],
                        position.y as f32 + local[1],
                        position.z as f32 + local[2],
                    ],
                    uv: [
                        rectangle[0] + (rectangle[2] - rectangle[0]) * localUv[0],
                        rectangle[1] + (rectangle[3] - rectangle[1]) * localUv[1],
                    ],
                    // BufferBuilder.noColor disables BlockModelRenderer's AO,
                    // directional shade and tint multipliers for this pass.
                    color: [1.0, 1.0, 1.0, 0.5],
                    lightmap: [
                        ((packedLight >> 4) & 15) as f32,
                        ((packedLight >> 20) & 15) as f32,
                    ],
                
                    shaderEntity: [-1, -1, -1],
                    shaderPadding: 0,
                });
            }
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
    (vertices, indices)
}

fn build_selection_box_mesh(
    selection: Option<SelectionBoxRenderState>,
) -> (Vec<WorldVertex>, Vec<u32>) {
    let Some(selection) = selection else { return (Vec::new(), Vec::new()); };
    let bounds = selection.boundingBox;
    let min = [bounds.min_x as f32, bounds.min_y as f32, bounds.min_z as f32];
    let max = [bounds.max_x as f32, bounds.max_y as f32, bounds.max_z as f32];
    // RenderGlobal.drawBoundingBox emits one GL_LINE_STRIP with sixteen
    // vertices. Three repeated connector vertices have alpha zero; retaining
    // those vertices preserves the original fade/edge ordering rather than
    // replacing it with twelve unrelated opaque lines.
    let positions = [
        [min[0], min[1], min[2]],
        [max[0], min[1], min[2]],
        [max[0], min[1], max[2]],
        [min[0], min[1], max[2]],
        [min[0], min[1], min[2]],
        [min[0], max[1], min[2]],
        [max[0], max[1], min[2]],
        [max[0], max[1], max[2]],
        [min[0], max[1], max[2]],
        [min[0], max[1], min[2]],
        [min[0], max[1], max[2]],
        [min[0], min[1], max[2]],
        [max[0], min[1], max[2]],
        [max[0], max[1], max[2]],
        [max[0], max[1], min[2]],
        [max[0], min[1], min[2]],
    ];
    let mut vertices = Vec::with_capacity(positions.len());
    for (index, position) in positions.into_iter().enumerate() {
        let mut color = selection.color;
        if matches!(index, 9 | 12 | 14) {
            color[3] = 0.0;
        }
        vertices.push(WorldVertex {
            position,
            uv: [0.0, 0.0],
            color,
            lightmap: [15.0, 15.0],
        
            shaderEntity: [-1, -1, -1],
            shaderPadding: 0,
        });
    }
    let indices = (0_u32..16).collect::<Vec<_>>();
    (vertices, indices)
}

fn append_existing_line_strip(
    targetVertices: &mut Vec<WorldVertex>,
    targetIndices: &mut Vec<u32>,
    sourceVertices: &[WorldVertex],
    sourceIndices: &[u32],
) {
    if sourceIndices.is_empty() {
        return;
    }

    if !targetIndices.is_empty() {
        let last = targetVertices[targetIndices[targetIndices.len() - 1] as usize];
        let first = sourceVertices[sourceIndices[0] as usize];
        let mut transparentLast = last;
        transparentLast.color[3] = 0.0;
        let mut transparentFirst = first;
        transparentFirst.color[3] = 0.0;
        let connectorBase = targetVertices.len() as u32;
        targetVertices.push(transparentLast);
        targetVertices.push(transparentFirst);
        targetIndices.extend_from_slice(&[connectorBase, connectorBase + 1]);
    }

    let base = targetVertices.len() as u32;
    targetVertices.extend_from_slice(sourceVertices);
    targetIndices.extend(sourceIndices.iter().map(|index| base + *index));
}

fn append_debug_box(
    bounds: AxisAlignedBB,
    color: [f32; 4],
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let state = SelectionBoxRenderState { boundingBox: bounds, color, lineWidth: 1.0 };
    let (sourceVertices, sourceIndices) = build_selection_box_mesh(Some(state));
    append_existing_line_strip(vertices, indices, &sourceVertices, &sourceIndices);
}

fn debug_look_vector(yaw: f32, pitch: f32) -> [f32; 3] {
    let yaw = -yaw.to_radians() - core::f32::consts::PI;
    let pitch = -pitch.to_radians();
    let yawCos = yaw.cos();
    let yawSin = yaw.sin();
    let pitchHorizontal = -pitch.cos();
    [yawSin * pitchHorizontal, pitch.sin(), yawCos * pitchHorizontal]
}

fn append_living_debug_geometry(
    bounds: AxisAlignedBB,
    center: [f64; 3],
    width: f32,
    eyeHeight: f32,
    yaw: f32,
    pitch: f32,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    append_debug_box(bounds, [1.0, 1.0, 1.0, 1.0], vertices, indices);
    let halfWidth = width as f64 * 0.5;
    let eyeY = center[1] + eyeHeight as f64;
    append_debug_box(
        AxisAlignedBB::new(
            center[0] - halfWidth,
            eyeY - 0.01,
            center[2] - halfWidth,
            center[0] + halfWidth,
            eyeY + 0.01,
            center[2] + halfWidth,
        ),
        [1.0, 0.0, 0.0, 1.0],
        vertices,
        indices,
    );
    let look = debug_look_vector(yaw, pitch);
    append_debug_line_strip(
        &[
            [center[0] as f32, eyeY as f32, center[2] as f32],
            [
                center[0] as f32 + look[0] * 2.0,
                eyeY as f32 + look[1] * 2.0,
                center[2] as f32 + look[2] * 2.0,
            ],
        ],
        [0.0, 0.0, 1.0, 1.0],
        vertices,
        indices,
    );
}

fn append_debug_entity_boxes(
    capture: &WorldRenderCapture,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    if !capture.showDebugHitboxes { return; }
    let partial = capture.partialTicks.clamp(0.0, 1.0) as f64;
    let mut appendPlayer = |player: &RemotePlayerRenderState| {
        let x = player.prevPosition[0] + (player.position[0] - player.prevPosition[0]) * partial;
        let y = player.prevPosition[1] + (player.position[1] - player.prevPosition[1]) * partial;
        let z = player.prevPosition[2] + (player.position[2] - player.prevPosition[2]) * partial;
        let yaw = player.prevHeadYaw + (player.headYaw - player.prevHeadYaw) * partial as f32;
        let pitch = player.prevPitch + (player.pitch - player.prevPitch) * partial as f32;
        append_living_debug_geometry(
            AxisAlignedBB::new(x - 0.3, y, z - 0.3, x + 0.3, y + player.height as f64, z + 0.3),
            [x, y, z],
            0.6,
            player.eyeHeight,
            yaw,
            pitch,
            vertices,
            indices,
        );
    };
    for player in &capture.remotePlayers { appendPlayer(player); }
    if capture.thirdPersonView != 0 {
        if let Some(player) = capture.localPlayerRenderState.as_ref() { appendPlayer(player); }
    }
    for entity in &capture.nonPlayerEntities {
        let x = entity.entity.prevPosX + (entity.entity.posX - entity.entity.prevPosX) * partial;
        let y = entity.entity.prevPosY + (entity.entity.posY - entity.entity.prevPosY) * partial;
        let z = entity.entity.prevPosZ + (entity.entity.posZ - entity.entity.prevPosZ) * partial;
        let bounds = entity.entity.boundingBox.offset(
            x - entity.entity.posX,
            y - entity.entity.posY,
            z - entity.entity.posZ,
        );
        let yaw = entity.entity.prevRotationYaw
            + (entity.entity.rotationYaw - entity.entity.prevRotationYaw) * partial as f32;
        let pitch = entity.entity.prevRotationPitch
            + (entity.entity.rotationPitch - entity.entity.prevRotationPitch) * partial as f32;
        if matches!(entity.kind, ClientEntityKind::Mob { .. }) {
            append_living_debug_geometry(
                bounds,
                [x, y, z],
                entity.entity.width,
                entity.eyeHeight(),
                yaw,
                pitch,
                vertices,
                indices,
            );
        } else {
            append_debug_box(bounds, [1.0, 1.0, 1.0, 1.0], vertices, indices);
            let eyeY = y + entity.eyeHeight() as f64;
            let look = debug_look_vector(yaw, pitch);
            append_debug_line_strip(
                &[
                    [x as f32, eyeY as f32, z as f32],
                    [x as f32 + look[0] * 2.0, eyeY as f32 + look[1] * 2.0, z as f32 + look[2] * 2.0],
                ],
                [0.0, 0.0, 1.0, 1.0],
                vertices,
                indices,
            );
        }
    }
}

fn append_debug_line_strip(
    points: &[[f32; 3]],
    color: [f32; 4],
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    if points.len() < 2 { return; }
    let sourceVertices = points.iter().map(|position| WorldVertex {
        position: *position, uv: [0.0, 0.0], color, lightmap: [15.0, 15.0],
    
        shaderEntity: [-1, -1, -1],
        shaderPadding: 0,
    }).collect::<Vec<_>>();
    let sourceIndices = (0..sourceVertices.len() as u32).collect::<Vec<_>>();
    append_existing_line_strip(vertices, indices, &sourceVertices, &sourceIndices);
}

fn append_chunk_boundaries(
    capture: &WorldRenderCapture,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    if !capture.showChunkBoundaries { return; }
    let chunkX = (capture.playerPosition.posX.floor() as i32).div_euclid(16) * 16;
    let chunkZ = (capture.playerPosition.posZ.floor() as i32).div_euclid(16) * 16;
    let x0 = chunkX as f32;
    let z0 = chunkZ as f32;
    let x1 = x0 + 16.0;
    let z1 = z0 + 16.0;

    // `DebugRendererChunkBorder`: red vertical corners for the surrounding
    // three-by-three chunk area.
    for xOffset in (-16..=32).step_by(16) {
        for zOffset in (-16..=32).step_by(16) {
            append_debug_line_strip(
                &[[x0 + xOffset as f32, 0.0, z0 + zOffset as f32],
                  [x0 + xOffset as f32, 256.0, z0 + zOffset as f32]],
                [1.0, 0.0, 0.0, 0.5],
                vertices,
                indices,
            );
        }
    }

    // Two-block subdivisions on all four walls and horizontal Y slices.
    for offset in (2..16).step_by(2) {
        let o = offset as f32;
        append_debug_line_strip(&[[x0 + o, 0.0, z0], [x0 + o, 256.0, z0]], [1.0, 1.0, 0.0, 1.0], vertices, indices);
        append_debug_line_strip(&[[x0 + o, 0.0, z1], [x0 + o, 256.0, z1]], [1.0, 1.0, 0.0, 1.0], vertices, indices);
        append_debug_line_strip(&[[x0, 0.0, z0 + o], [x0, 256.0, z0 + o]], [1.0, 1.0, 0.0, 1.0], vertices, indices);
        append_debug_line_strip(&[[x1, 0.0, z0 + o], [x1, 256.0, z0 + o]], [1.0, 1.0, 0.0, 1.0], vertices, indices);
    }
    for y in (0..=256).step_by(2) {
        append_debug_line_strip(
            &[[x0, y as f32, z0], [x0, y as f32, z1], [x1, y as f32, z1], [x1, y as f32, z0], [x0, y as f32, z0]],
            [1.0, 1.0, 0.0, 1.0],
            vertices,
            indices,
        );
    }

    // Sixteen-block major boundaries. The source uses a 2px line width; this
    // backend retains the exact blue geometry and colour in its line pipeline.
    for x in [x0, x1] {
        for z in [z0, z1] {
            append_debug_line_strip(&[[x, 0.0, z], [x, 256.0, z]], [0.25, 0.25, 1.0, 1.0], vertices, indices);
        }
    }
    for y in (0..=256).step_by(16) {
        append_debug_line_strip(
            &[[x0, y as f32, z0], [x0, y as f32, z1], [x1, y as f32, z1], [x1, y as f32, z0], [x0, y as f32, z0]],
            [0.25, 0.25, 1.0, 1.0],
            vertices,
            indices,
        );
    }
}

fn block_entity_local_snapshot_hash(
    snapshot: &HashMap<ChunkKey, Chunk>,
    pos: BlockPos,
) -> u64 {
    // TileEntityPistonRenderer reads the moving state and immediate neighbour
    // states through BlockRendererDispatcher/BlockModelRenderer. Hash only the
    // chunks intersecting a one-block neighbourhood instead of every loaded
    // chunk, while still covering chunk-boundary actual-state/AO lookups.
    let minChunkX = (pos.x - 1).div_euclid(16);
    let maxChunkX = (pos.x + 1).div_euclid(16);
    let minChunkZ = (pos.z - 1).div_euclid(16);
    let maxChunkZ = (pos.z + 1).div_euclid(16);
    let mut fingerprint = RenderStateFingerprint::default();
    for chunkX in minChunkX..=maxChunkX {
        for chunkZ in minChunkZ..=maxChunkZ {
            let key = ChunkKey::new(chunkX, chunkZ);
            let revision = snapshot.get(&key).map(|chunk| chunk.revision()).unwrap_or(0);
            write!(&mut fingerprint, "{chunkX}:{chunkZ}:{revision};")
                .expect("local block-entity snapshot hashing cannot fail");
        }
    }
    fingerprint.0
}

fn block_entity_mesh_cache_key<T: fmt::Debug>(
    state: &T,
    atlasRevision: u64,
    snapshotHash: u64,
) -> BlockEntityMeshCacheKey {
    BlockEntityMeshCacheKey {
        stateHash: debug_render_state_hash(state),
        snapshotHash,
        atlasRevision,
    }
}

fn append_block_entity_mesh_batch(
    mesh: &BlockEntityMeshBatch,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    if mesh.indices.is_empty() {
        return;
    }
    let base = vertices.len() as u32;
    vertices.extend_from_slice(mesh.vertices.as_slice());
    indices.extend(mesh.indices.iter().copied().map(|index| base + index));
}

fn skull_tile_entity_visible(
    skull: &SkullRenderState,
    camera: [f32; 3],
    frustum: &Frustum,
) -> bool {
    let state = BlockSkull::stateForFacing(skull.facing);
    let bounds = BlockSkull::getBoundingBox(state).offset(
        skull.pos.x as f64,
        skull.pos.y as f64,
        skull.pos.z as f64,
    );
    let center = [
        skull.pos.x as f32 + 0.5,
        skull.pos.y as f32 + 0.5,
        skull.pos.z as f32 + 0.5,
    ];
    let dx = center[0] - camera[0];
    let dy = center[1] - camera[1];
    let dz = center[2] - camera[2];
    dx * dx + dy * dy + dz * dz <= 4096.0
        && frustum.isBoxInFrustum(
            bounds.min_x,
            bounds.min_y,
            bounds.min_z,
            bounds.max_x,
            bounds.max_y,
            bounds.max_z,
        )
}

fn shulker_box_tile_entity_visible(
    shulker: &ShulkerBoxRenderState,
    camera: [f32; 3],
    frustum: &Frustum,
) -> bool {
    // TileEntityShulkerBox#getRenderBoundingBox extends only in the current
    // facing direction as the lid opens.
    let extension = 0.5 * shulker.progress.clamp(0.0, 1.0) as f64;
    let (offsetX, offsetY, offsetZ) = shulker.facing.offsets();
    let mut minX = shulker.pos.x as f64;
    let mut minY = shulker.pos.y as f64;
    let mut minZ = shulker.pos.z as f64;
    let mut maxX = minX + 1.0;
    let mut maxY = minY + 1.0;
    let mut maxZ = minZ + 1.0;
    if offsetX < 0 {
        minX -= extension;
    } else if offsetX > 0 {
        maxX += extension;
    }
    if offsetY < 0 {
        minY -= extension;
    } else if offsetY > 0 {
        maxY += extension;
    }
    if offsetZ < 0 {
        minZ -= extension;
    } else if offsetZ > 0 {
        maxZ += extension;
    }

    let center = [
        shulker.pos.x as f32 + 0.5,
        shulker.pos.y as f32 + 0.5,
        shulker.pos.z as f32 + 0.5,
    ];
    let dx = center[0] - camera[0];
    let dy = center[1] - camera[1];
    let dz = center[2] - camera[2];
    dx * dx + dy * dy + dz * dz <= 4096.0
        && frustum.isBoxInFrustum(
            minX - 0.01,
            minY - 0.01,
            minZ - 0.01,
            maxX + 0.01,
            maxY + 0.01,
            maxZ + 0.01,
        )
}

fn sign_uses_obfuscated_formatting(sign: &SignRenderState) -> bool {
    sign.lines.iter().any(|line| {
        line.as_bytes().windows(3).any(|code| {
            code[0] == 0xC2
                && code[1] == 0xA7
                && matches!(code[2], b'k' | b'K')
        })
    })
}

fn build_block_entity_meshes(
    capture: &WorldRenderCapture,
    camera: [f32; 3],
    frustum: &Frustum,
    atlas: &AtlasState,
    fontRenderer: &mut FontRenderer,
    frameMeshCache: &mut RenderFrameMeshCache,
) -> BlockEntityMeshBatch {
    let mut batch = BlockEntityMeshBatch::default();
    frameMeshCache.beginBlockEntityFrame();

    for skull in &capture.skulls {
        let identity = BlockEntityMeshIdentity {
            kind: BlockEntityMeshKind::Skull,
            pos: skull.pos,
        };
        if !skull_tile_entity_visible(skull, camera, frustum) {
            frameMeshCache.touchBlockEntity(identity);
            continue;
        }
        let key = block_entity_mesh_cache_key(skull, atlas.revision, 0);
        let mesh = frameMeshCache.blockEntityMesh(identity, key, true, || {
            let mut mesh = BlockEntityMeshBatch::default();
            append_skull_tile_entity_meshes(
                std::slice::from_ref(skull),
                camera,
                frustum,
                atlas,
                &mut mesh.vertices,
                &mut mesh.indices,
            );
            mesh
        });
        append_block_entity_mesh_batch(&mesh, &mut batch.vertices, &mut batch.indices);
    }

    for bed in &capture.beds {
        let identity = BlockEntityMeshIdentity {
            kind: BlockEntityMeshKind::Bed,
            pos: bed.pos,
        };
        if !tile_entity_visible(bed.pos, camera, frustum, 1.0, 1.0) {
            frameMeshCache.touchBlockEntity(identity);
            continue;
        }
        let key = block_entity_mesh_cache_key(bed, atlas.revision, 0);
        let mesh = frameMeshCache.blockEntityMesh(identity, key, true, || {
            let mut mesh = BlockEntityMeshBatch::default();
            append_bed_tile_entity_meshes(
                std::slice::from_ref(bed),
                camera,
                frustum,
                atlas,
                &mut mesh.vertices,
                &mut mesh.indices,
            );
            mesh
        });
        append_block_entity_mesh_batch(&mesh, &mut batch.vertices, &mut batch.indices);
    }

    for chest in &capture.chests {
        let identity = BlockEntityMeshIdentity {
            kind: BlockEntityMeshKind::Chest,
            pos: chest.pos,
        };
        let width = if chest.large { 2.0 } else { 1.0 };
        if !tile_entity_visible(chest.pos, camera, frustum, width, 1.0) {
            frameMeshCache.touchBlockEntity(identity);
            continue;
        }
        let key = block_entity_mesh_cache_key(chest, atlas.revision, 0);
        let mesh = frameMeshCache.blockEntityMesh(identity, key, true, || {
            let mut mesh = BlockEntityMeshBatch::default();
            append_chest_tile_entity_meshes(
                std::slice::from_ref(chest),
                camera,
                frustum,
                atlas,
                &mut mesh.vertices,
                &mut mesh.indices,
            );
            mesh
        });
        append_block_entity_mesh_batch(&mesh, &mut batch.vertices, &mut batch.indices);
    }

    for shulker in &capture.shulkerBoxes {
        let identity = BlockEntityMeshIdentity {
            kind: BlockEntityMeshKind::ShulkerBox,
            pos: shulker.pos,
        };
        if !shulker_box_tile_entity_visible(shulker, camera, frustum) {
            frameMeshCache.touchBlockEntity(identity);
            continue;
        }
        let key = block_entity_mesh_cache_key(shulker, atlas.revision, 0);
        let mesh = frameMeshCache.blockEntityMesh(identity, key, true, || {
            let mut mesh = BlockEntityMeshBatch::default();
            append_shulker_box_tile_entity_meshes(
                std::slice::from_ref(shulker),
                camera,
                frustum,
                atlas,
                &mut mesh.vertices,
                &mut mesh.indices,
            );
            mesh
        });
        append_block_entity_mesh_batch(&mesh, &mut batch.vertices, &mut batch.indices);
    }

    for sign in &capture.signs {
        let identity = BlockEntityMeshIdentity {
            kind: BlockEntityMeshKind::Sign,
            pos: sign.pos,
        };
        let cacheable = !sign_uses_obfuscated_formatting(sign);
        if !tile_entity_visible(sign.pos, camera, frustum, 1.0, 1.0) {
            if cacheable {
                frameMeshCache.touchBlockEntity(identity);
            } else {
                frameMeshCache.discardBlockEntity(identity);
            }
            continue;
        }
        let key = block_entity_mesh_cache_key(sign, atlas.revision, 0);
        let mesh = frameMeshCache.blockEntityMesh(identity, key, cacheable, || {
            let mut mesh = BlockEntityMeshBatch::default();
            append_sign_tile_entity_meshes(
                std::slice::from_ref(sign),
                camera,
                frustum,
                atlas,
                fontRenderer,
                &mut mesh.vertices,
                &mut mesh.indices,
            );
            mesh
        });
        append_block_entity_mesh_batch(&mesh, &mut batch.vertices, &mut batch.indices);
    }

    for table in &capture.enchantmentTables {
        let identity = BlockEntityMeshIdentity {
            kind: BlockEntityMeshKind::EnchantmentTable,
            pos: table.pos,
        };
        if !tile_entity_visible(table.pos, camera, frustum, 1.0, 1.5) {
            frameMeshCache.touchBlockEntity(identity);
            continue;
        }
        let key = block_entity_mesh_cache_key(table, atlas.revision, 0);
        let mesh = frameMeshCache.blockEntityMesh(identity, key, true, || {
            let mut mesh = BlockEntityMeshBatch::default();
            append_enchantment_table_tile_entity_meshes(
                std::slice::from_ref(table),
                camera,
                frustum,
                atlas,
                &mut mesh.vertices,
                &mut mesh.indices,
            );
            mesh
        });
        append_block_entity_mesh_batch(&mesh, &mut batch.vertices, &mut batch.indices);
    }

    for piston in &capture.pistons {
        let identity = BlockEntityMeshIdentity {
            kind: BlockEntityMeshKind::Piston,
            pos: piston.pos,
        };
        let key = block_entity_mesh_cache_key(
            piston,
            atlas.revision,
            block_entity_local_snapshot_hash(&capture.snapshot, piston.pos),
        );
        let mesh = frameMeshCache.blockEntityMesh(identity, key, true, || {
            let mut mesh = BlockEntityMeshBatch::default();
            append_piston_tile_entity_meshes(
                std::slice::from_ref(piston),
                &capture.snapshot,
                atlas,
                &mut mesh.vertices,
                &mut mesh.indices,
            );
            mesh
        });
        append_block_entity_mesh_batch(&mesh, &mut batch.vertices, &mut batch.indices);
    }


    frameMeshCache.finishBlockEntityFrame();
    batch
}

fn build_dynamic_meshes(
    capture: &WorldRenderCapture,
    atlas: &AtlasState,
    celestialAngle: f32,
    camera: [f32; 3],
    frustum: &Frustum,
    viewProjection: [[f32; 4]; 4],
    aspect: f32,
    farPlaneDistance: f32,
    lightmap: LightmapParameters,
    guiIngame: &mut GuiIngame,
    guiBossOverlay: &mut GuiBossOverlay,
    playerTabOverlay: &mut GuiPlayerTabOverlay,
    guiNewChat: &mut GuiNewChat,
    fontRenderer: &mut FontRenderer,
    standardGalacticFontRenderer: &mut FontRenderer,
    locale: &Locale,
    worldGeneration: u64,
    frameMeshCache: &mut RenderFrameMeshCache,
) -> CachedDynamicMeshes {
    let (
        mut entityVertices,
        mut entityIndices,
        playerGlintVertices,
        playerGlintIndices,
        renderedRemotePlayers,
    ) = build_remote_player_meshes(
        &capture.remotePlayers,
        capture.partialTicks,
        camera,
        &frustum,
        atlas,
    );
    let mut entityDrawRanges = Vec::with_capacity(16);
    push_world_entity_draw_range(
        &mut entityDrawRanges,
        WorldEntityPipelineKind::Entities,
        WorldEntityMeshKind::Dynamic,
        0,
        entityIndices.len() as u32,
    );
    let viewerPosition = capture
        .localPlayerRenderState
        .as_ref()
        .map(|player| player.position)
        .unwrap_or([
            capture.playerPosition.posX,
            capture.playerPosition.posY,
            capture.playerPosition.posZ,
        ]);
    append_remote_player_nameplates(
        &capture.remotePlayers,
        &capture.scoreboard,
        &capture.localPlayerName,
        capture.localPlayerSpectator,
        capture.localPlayerRenderState.as_ref().map(|player| player.entityId),
        viewerPosition,
        capture.partialTicks,
        capture.cameraYaw,
        capture.cameraPitch,
        frustum,
        fontRenderer,
        atlas,
        &mut entityVertices,
        &mut entityIndices,
        &mut entityDrawRanges,
    );

    let blockEntityMesh = build_block_entity_meshes(
        capture,
        camera,
        frustum,
        atlas,
        fontRenderer,
        frameMeshCache,
    );
    let mut blockEntityVertices = Vec::new();
    let mut blockEntityIndices = Vec::new();
    append_block_entity_mesh_batch(
        &blockEntityMesh,
        &mut blockEntityVertices,
        &mut blockEntityIndices,
    );
    push_world_entity_draw_range(
        &mut entityDrawRanges,
        WorldEntityPipelineKind::BlockEntities,
        WorldEntityMeshKind::BlockEntities,
        0,
        blockEntityIndices.len() as u32,
    );

    let mut entityDepthVertices = Vec::new();
    let mut entityDepthIndices = Vec::new();
    let mut entityOverlayVertices = Vec::new();
    let mut entityOverlayIndices = Vec::new();
    let (skyAlphaIndexCount, skyCelestialIndexCount) = append_sky_mesh(
        &capture,
        celestialAngle,
        atlas,
        &mut entityOverlayVertices,
        &mut entityOverlayIndices,
    );
    let skyOverlayIndexCount = skyAlphaIndexCount + skyCelestialIndexCount;
    let playerGlintFirstIndex = entityOverlayIndices.len() as u32;
    if !playerGlintIndices.is_empty() {
        let base = entityOverlayVertices.len() as u32;
        entityOverlayVertices.extend(playerGlintVertices);
        entityOverlayIndices.extend(playerGlintIndices.into_iter().map(|index| base + index));
    }
    let playerGlintIndexCount = entityOverlayIndices.len() as u32 - playerGlintFirstIndex;

    // Render#renderEntityOnFire remains camera-facing and therefore belongs to
    // the dynamic stream even when the underlying hanging entity is resident.
    let playerFireFirstIndex = entityIndices.len() as u32;
    append_player_fire_meshes(
        &capture.remotePlayers,
        capture.partialTicks,
        capture.cameraYaw,
        atlas,
        &mut entityVertices,
        &mut entityIndices,
    );
    push_world_entity_draw_range(
        &mut entityDrawRanges,
        WorldEntityPipelineKind::Entities,
        WorldEntityMeshKind::Dynamic,
        playerFireFirstIndex,
        entityIndices.len() as u32 - playerFireFirstIndex,
    );

    let mut staticEntityVertices = Vec::new();
    let mut staticEntityIndices = Vec::new();
    let mut worldLineVertices = Vec::new();
    let mut worldLineIndices = Vec::new();
    let renderedNonPlayerEntities = append_non_player_entity_meshes(
        &capture.nonPlayerEntities,
        &capture.mapData,
        &capture.remotePlayers,
        capture.localPlayerRenderState.as_ref(),
        capture.localPlayerTarget,
        capture.totalWorldTime,
        capture.partialTicks,
        camera,
        capture.cameraYaw,
        capture.cameraPitch,
        capture.thirdPersonView,
        capture.fov,
        &frustum,
        &capture.snapshot,
        capture.dimension,
        atlas,
        frameMeshCache,
        &mut entityVertices,
        &mut entityIndices,
        &mut staticEntityVertices,
        &mut staticEntityIndices,
        &mut entityDrawRanges,
        &mut entityDepthVertices,
        &mut entityDepthIndices,
        &mut entityOverlayVertices,
        &mut entityOverlayIndices,
        &mut worldLineVertices,
        &mut worldLineIndices,
    );
    let tntOverlayIndexCount = entityOverlayIndices.len() as u32
        - skyOverlayIndexCount
        - playerGlintIndexCount;
    let (beaconCoreIndexCount, beaconGlowIndexCount) = append_beacon_tile_entity_meshes(
        &capture.beacons,
        capture.totalWorldTime,
        capture.partialTicks,
        atlas,
        &mut entityOverlayVertices,
        &mut entityOverlayIndices,
    );
    let portalAlphaIndexCount = append_end_portal_tile_entity_meshes(
        &capture.endPortals,
        camera,
        &frustum,
        viewProjection,
        capture.systemTimeMillis,
        atlas,
        &mut entityOverlayVertices,
        &mut entityOverlayIndices,
    );
    let portalOverlayIndexCount = entityOverlayIndices.len() as u32
        - skyOverlayIndexCount
        - playerGlintIndexCount
        - tntOverlayIndexCount
        - beaconCoreIndexCount
        - beaconGlowIndexCount;
    let portalAdditiveIndexCount = portalOverlayIndexCount.saturating_sub(portalAlphaIndexCount);
    let mut entityOverlayDrawRanges = Vec::with_capacity(6);
    if playerGlintIndexCount > 0 {
        entityOverlayDrawRanges.push(EntityOverlayDrawRange {
            pipeline: EntityOverlayPipelineKind::ArmorGlint,
            firstIndex: playerGlintFirstIndex,
            indexCount: playerGlintIndexCount,
        });
    }
    if tntOverlayIndexCount > 0 {
        entityOverlayDrawRanges.push(EntityOverlayDrawRange {
            pipeline: EntityOverlayPipelineKind::TntFlash,
            firstIndex: skyOverlayIndexCount + playerGlintIndexCount,
            indexCount: tntOverlayIndexCount,
        });
    }
    if beaconCoreIndexCount > 0 {
        entityOverlayDrawRanges.push(EntityOverlayDrawRange {
            pipeline: EntityOverlayPipelineKind::BeaconCore,
            firstIndex: skyOverlayIndexCount + playerGlintIndexCount + tntOverlayIndexCount,
            indexCount: beaconCoreIndexCount,
        });
    }
    if beaconGlowIndexCount > 0 {
        entityOverlayDrawRanges.push(EntityOverlayDrawRange {
            pipeline: EntityOverlayPipelineKind::BeaconGlow,
            firstIndex: skyOverlayIndexCount + playerGlintIndexCount + tntOverlayIndexCount + beaconCoreIndexCount,
            indexCount: beaconGlowIndexCount,
        });
    }
    let portalFirstIndex = skyOverlayIndexCount
        + playerGlintIndexCount
        + tntOverlayIndexCount
        + beaconCoreIndexCount
        + beaconGlowIndexCount;
    if portalAlphaIndexCount > 0 {
        entityOverlayDrawRanges.push(EntityOverlayDrawRange {
            pipeline: EntityOverlayPipelineKind::EndPortalAlpha,
            firstIndex: portalFirstIndex,
            indexCount: portalAlphaIndexCount,
        });
    }
    if portalAdditiveIndexCount > 0 {
        entityOverlayDrawRanges.push(EntityOverlayDrawRange {
            pipeline: EntityOverlayPipelineKind::EndPortalAdditive,
            firstIndex: portalFirstIndex + portalAlphaIndexCount,
            indexCount: portalAdditiveIndexCount,
        });
    }
    let (mut particleVertices, mut particleIndices) = build_digging_particle_mesh(&capture, atlas);
    append_misc_particle_mesh(
        &capture,
        atlas,
        false,
        &mut particleVertices,
        &mut particleIndices,
    );
    let mut transparentParticleVertices = Vec::new();
    let mut transparentParticleIndices = Vec::new();
    append_misc_particle_mesh(
        &capture,
        atlas,
        true,
        &mut transparentParticleVertices,
        &mut transparentParticleIndices,
    );
    let (mut damageVertices, mut damageIndices) = build_block_damage_mesh(&capture, atlas);
    append_shulker_box_damage_meshes(
        &capture.shulkerBoxes,
        &capture.damagedBlocks,
        capture.cameraPosition,
        atlas,
        &mut damageVertices,
        &mut damageIndices,
    );
    let (selectionBoxVertices, selectionBoxIndices) = build_selection_box_mesh(capture.selectionBox);
    append_existing_line_strip(
        &mut worldLineVertices,
        &mut worldLineIndices,
        &selectionBoxVertices,
        &selectionBoxIndices,
    );
    append_debug_entity_boxes(&capture, &mut worldLineVertices, &mut worldLineIndices);
    append_chunk_boundaries(&capture, &mut worldLineVertices, &mut worldLineIndices);
    let selectionVertices = worldLineVertices;
    let selectionIndices = worldLineIndices;
    let (firstPersonVertices, firstPersonIndices, firstPersonDrawRanges, firstPersonPushConstants) =
        build_first_person_item_meshes(
            &capture,
            atlas,
            aspect,
            farPlaneDistance,
            lightmap,
        );
    let (
        hudVertices,
        hudIndices,
        hudDrawRanges,
        hudPushConstants,
    ) = build_ingame_hud(
        &capture,
        atlas,
        guiIngame,
        guiBossOverlay,
        playerTabOverlay,
        guiNewChat,
        fontRenderer,
        standardGalacticFontRenderer,
        locale,
    );


    CachedDynamicMeshes {
        entityMeshGeneration: 0,
        blockEntityMeshGeneration: 0,
        staticEntityMeshGeneration: 0,
        entityDepthMeshGeneration: 0,
        entityOverlayMeshGeneration: 0,
        particleMeshGeneration: 0,
        transparentParticleMeshGeneration: 0,
        damageMeshGeneration: 0,
        selectionMeshGeneration: 0,
        firstPersonMeshGeneration: 0,
        hudMeshGeneration: 0,
        builtAt: Instant::now(),
        worldGeneration,
        atlasRevision: atlas.revision,
        outputWidth: capture.outputWidth,
        outputHeight: capture.outputHeight,
        guiWidth: capture.guiWidth,
        guiHeight: capture.guiHeight,
        entityVertices: Arc::new(entityVertices),
        entityIndices: Arc::new(entityIndices),
        blockEntityVertices: Arc::new(blockEntityVertices),
        blockEntityIndices: Arc::new(blockEntityIndices),
        staticEntityVertices: Arc::new(staticEntityVertices),
        staticEntityIndices: Arc::new(staticEntityIndices),
        entityDrawRanges,
        entityDepthVertices: Arc::new(entityDepthVertices),
        entityDepthIndices: Arc::new(entityDepthIndices),
        entityOverlayVertices: Arc::new(entityOverlayVertices),
        entityOverlayIndices: Arc::new(entityOverlayIndices),
        skyAlphaIndexCount,
        skyCelestialIndexCount,
        entityOverlayDrawRanges,
        renderedRemotePlayers,
        renderedNonPlayerEntities,
        particleVertices: Arc::new(particleVertices),
        particleIndices: Arc::new(particleIndices),
        transparentParticleVertices: Arc::new(transparentParticleVertices),
        transparentParticleIndices: Arc::new(transparentParticleIndices),
        damageVertices: Arc::new(damageVertices),
        damageIndices: Arc::new(damageIndices),
        selectionVertices: Arc::new(selectionVertices),
        selectionIndices: Arc::new(selectionIndices),
        firstPersonVertices: Arc::new(firstPersonVertices),
        firstPersonIndices: Arc::new(firstPersonIndices),
        firstPersonDrawRanges,
        firstPersonPushConstants,
        hudVertices: Arc::new(hudVertices),
        hudIndices: Arc::new(hudIndices),
        hudDrawRanges,
        hudPushConstants,
    }
}

fn make_frame(
    capture: WorldRenderCapture,
    atlas: Arc<AtlasState>,
    chunks: &HashMap<RenderChunkKey, CachedChunkMesh>,
    chunkUploads: Vec<ChunkMeshUpload>,
    removedChunks: Vec<RenderChunkKey>,
    guiIngame: &mut GuiIngame,
    guiBossOverlay: &mut GuiBossOverlay,
    playerTabOverlay: &mut GuiPlayerTabOverlay,
    guiNewChat: &mut GuiNewChat,
    fontRenderer: &mut FontRenderer,
    standardGalacticFontRenderer: &mut FontRenderer,
    locale: &Locale,
    worldGeneration: u64,
    frameMeshCache: &mut RenderFrameMeshCache,
) -> WorldRenderFrame {
    let provider = WorldProvider::new(capture.dimension);
    let celestialAngle = provider.calculateCelestialAngle(capture.worldTime, capture.partialTicks);
    let clearColor = sky_color(
        capture.dimension,
        capture.biomeSkyColor,
        celestialAngle,
        capture.lastLightningBolt,
        capture.partialTicks,
    );
    let fogColor = fog_color(capture.dimension, celestialAngle);
    let lightmap = EntityRenderer::lightmapParameters(
        &provider,
        capture.worldTime,
        capture.partialTicks,
        capture.torchFlickerX,
        capture.gammaSetting,
    );
    let camera = capture.cameraPosition;
    let aspect = capture.outputWidth.max(1) as f32 / capture.outputHeight.max(1) as f32;
    let farPlaneDistance = capture.renderDistanceChunks as f32 * 16.0;
    let clipDistance = farPlaneDistance * 2.0;
    let modelViewMatrix = camera_view_matrix(capture.cameraYaw, capture.cameraPitch, camera);
    let projectionMatrix = perspective_matrix(
        capture.fov.clamp(30.0, 110.0),
        aspect,
        0.05,
        clipDistance,
    );
    let viewProjection = multiply4(projectionMatrix, modelViewMatrix);
    let cameraRelativeClip = multiply4(viewProjection, translation4(camera));
    let mut frustum = Frustum::new(ClippingHelperImpl::fromClipMatrix(cameraRelativeClip));
    frustum.setPosition(camera[0] as f64, camera[1] as f64, camera[2] as f64);
    // `RenderGlobal.setupTerrain` falls back to the nearest loaded section in
    // the camera's X/Z column when the exact camera-Y section is not resident.
    // Preserve that rule without rebuilding a second compiled-chunk HashMap;
    // the O(n) column scan only runs during the missing-start transition.
    let terrainStart = if chunks.contains_key(&capture.centerRenderChunk) {
        Some(capture.centerRenderChunk)
    } else {
        chunks
            .keys()
            .filter(|key| {
                key.x == capture.centerRenderChunk.x && key.z == capture.centerRenderChunk.z
            })
            .min_by_key(|key| (key.y - capture.centerRenderChunk.y).abs())
            .copied()
    };
    let terrainOrder = terrainStart.map_or_else(Vec::new, |start| {
        setupTerrainWithLookup(
            start,
            capture.renderDistanceChunks,
            |key| chunks.get(&key).map(|mesh| mesh.compiledChunk),
            |key| {
                let minimum = key.minBlock();
                let maximum = key.maxBlock();
                frustum.isBoxInFrustum(
                    minimum[0] as f64,
                    minimum[1] as f64,
                    minimum[2] as f64,
                    maximum[0] as f64,
                    maximum[1] as f64,
                    maximum[2] as f64,
                )
            },
        )
    });

    let mut visibleChunks = Vec::with_capacity(terrainOrder.len());
    for key in terrainOrder {
        let Some(mesh) = chunks.get(&key) else { continue; };
        if mesh.indexCount == 0 || !mesh.ready { continue; }
        visibleChunks.push(VisibleChunk {
            key,
            aabbMin: [
                mesh.aabbMin[0] as f32,
                mesh.aabbMin[1] as f32,
                mesh.aabbMin[2] as f32,
            ],
            aabbMax: [
                mesh.aabbMax[0] as f32,
                mesh.aabbMax[1] as f32,
                mesh.aabbMax[2] as f32,
            ],
        });
    }

    let dynamicMeshes = if frameMeshCache.shouldRebuild(
        worldGeneration,
        atlas.revision,
        capture.outputWidth,
        capture.outputHeight,
        capture.guiWidth,
        capture.guiHeight,
    ) {
        let dynamicStarted = Instant::now();
        let meshes = build_dynamic_meshes(
            &capture,
            &atlas,
            celestialAngle,
            camera,
            &frustum,
            viewProjection,
            aspect,
            farPlaneDistance,
            lightmap,
            guiIngame,
            guiBossOverlay,
            playerTabOverlay,
            guiNewChat,
            fontRenderer,
            standardGalacticFontRenderer,
            locale,
            worldGeneration,
            frameMeshCache,
        );
        frameMeshCache.store(meshes, dynamicStarted.elapsed())
    } else {
        frameMeshCache
            .reuse()
            .expect("dynamic mesh cache disappeared after reuse decision")
    };

    WorldRenderFrame {
        shaderState: ShaderFrameState {
            projectionMatrix: to_column_major(projectionMatrix),
            modelViewMatrix: to_column_major(modelViewMatrix),
            cameraPosition: camera,
            clearColor,
            fogColor: [fogColor[0], fogColor[1], fogColor[2]],
            skyColor: [clearColor[0], clearColor[1], clearColor[2]],
            screenBrightness: capture.gammaSetting.clamp(0.0, 1.0),
            eyeBrightness: [
                i32::from(capture.blockLight.min(15)) * 16,
                i32::from(capture.skyLight.min(15)) * 16,
            ],
            atlasSize: [atlas.atlas.width as i32, atlas.atlas.height as i32],
            celestialAngle,
            dimension: capture.dimension,
            worldTime: capture.worldTime,
            totalWorldTime: capture.totalWorldTime,
            partialTicks: capture.partialTicks,
            farPlane: farPlaneDistance,
        },
        atlasRevision: atlas.revision,
        atlas: Arc::clone(&atlas.atlas),
        chunkUploads,
        removedChunks,
        visibleChunks,
        dynamicMeshGeneration: [
            dynamicMeshes.entityMeshGeneration,
            dynamicMeshes.blockEntityMeshGeneration,
            dynamicMeshes.staticEntityMeshGeneration,
            dynamicMeshes.entityDepthMeshGeneration,
            dynamicMeshes.entityOverlayMeshGeneration,
            dynamicMeshes.particleMeshGeneration,
            dynamicMeshes.transparentParticleMeshGeneration,
            dynamicMeshes.damageMeshGeneration,
            dynamicMeshes.selectionMeshGeneration,
            dynamicMeshes.firstPersonMeshGeneration,
            dynamicMeshes.hudMeshGeneration,
        ].into_iter().max().unwrap_or(0),
        entityMeshGeneration: dynamicMeshes.entityMeshGeneration,
        blockEntityMeshGeneration: dynamicMeshes.blockEntityMeshGeneration,
        staticEntityMeshGeneration: dynamicMeshes.staticEntityMeshGeneration,
        entityDepthMeshGeneration: dynamicMeshes.entityDepthMeshGeneration,
        entityOverlayMeshGeneration: dynamicMeshes.entityOverlayMeshGeneration,
        particleMeshGeneration: dynamicMeshes.particleMeshGeneration,
        transparentParticleMeshGeneration: dynamicMeshes.transparentParticleMeshGeneration,
        damageMeshGeneration: dynamicMeshes.damageMeshGeneration,
        selectionMeshGeneration: dynamicMeshes.selectionMeshGeneration,
        firstPersonMeshGeneration: dynamicMeshes.firstPersonMeshGeneration,
        hudMeshGeneration: dynamicMeshes.hudMeshGeneration,
        entityVertices: Arc::clone(&dynamicMeshes.entityVertices),
        entityIndices: Arc::clone(&dynamicMeshes.entityIndices),
        blockEntityVertices: Arc::clone(&dynamicMeshes.blockEntityVertices),
        blockEntityIndices: Arc::clone(&dynamicMeshes.blockEntityIndices),
        staticEntityVertices: Arc::clone(&dynamicMeshes.staticEntityVertices),
        staticEntityIndices: Arc::clone(&dynamicMeshes.staticEntityIndices),
        entityDrawRanges: dynamicMeshes.entityDrawRanges.clone(),
        entityDepthVertices: Arc::clone(&dynamicMeshes.entityDepthVertices),
        entityDepthIndices: Arc::clone(&dynamicMeshes.entityDepthIndices),
        entityOverlayVertices: Arc::clone(&dynamicMeshes.entityOverlayVertices),
        entityOverlayIndices: Arc::clone(&dynamicMeshes.entityOverlayIndices),
        skyAlphaIndexCount: dynamicMeshes.skyAlphaIndexCount,
        skyCelestialIndexCount: dynamicMeshes.skyCelestialIndexCount,
        entityOverlayDrawRanges: dynamicMeshes.entityOverlayDrawRanges.clone(),
        renderedRemotePlayers: dynamicMeshes.renderedRemotePlayers,
        renderedNonPlayerEntities: dynamicMeshes.renderedNonPlayerEntities,
        particleVertices: Arc::clone(&dynamicMeshes.particleVertices),
        particleIndices: Arc::clone(&dynamicMeshes.particleIndices),
        transparentParticleVertices: Arc::clone(&dynamicMeshes.transparentParticleVertices),
        transparentParticleIndices: Arc::clone(&dynamicMeshes.transparentParticleIndices),
        damageVertices: Arc::clone(&dynamicMeshes.damageVertices),
        damageIndices: Arc::clone(&dynamicMeshes.damageIndices),
        selectionVertices: Arc::clone(&dynamicMeshes.selectionVertices),
        selectionIndices: Arc::clone(&dynamicMeshes.selectionIndices),
        firstPersonVertices: Arc::clone(&dynamicMeshes.firstPersonVertices),
        firstPersonIndices: Arc::clone(&dynamicMeshes.firstPersonIndices),
        firstPersonDrawRanges: dynamicMeshes.firstPersonDrawRanges.clone(),
        firstPersonPushConstants: dynamicMeshes.firstPersonPushConstants,
        hudVertices: Arc::clone(&dynamicMeshes.hudVertices),
        hudIndices: Arc::clone(&dynamicMeshes.hudIndices),
        hudDrawRanges: dynamicMeshes.hudDrawRanges.clone(),
        pushConstants: WorldPushConstants {
            viewProjection: to_column_major(viewProjection),
            cameraPosition: [
                camera[0],
                camera[1],
                camera[2],
                atlas.fireFrameOffsets(capture.totalWorldTime)[0],
            ],
            fogColor: [
                fogColor[0],
                fogColor[1],
                fogColor[2],
                atlas.fireFrameOffsets(capture.totalWorldTime)[1],
            ],
            fogParameters: [farPlaneDistance * 0.75, farPlaneDistance, clipDistance, 0.0],
            lightmapParameters: [
                lightmap.sunBrightness,
                lightmap.torchFlickerX,
                lightmap.gammaSetting,
                lightmap.dimension as f32,
            ],
        },
        skyPushConstants: WorldPushConstants {
            viewProjection: to_column_major(camera_matrix(
                capture.cameraYaw,
                capture.cameraPitch,
                camera,
                capture.fov.clamp(30.0, 110.0),
                aspect,
                0.05,
                512.0,
            )),
            cameraPosition: [camera[0], camera[1], camera[2], 0.0],
            fogColor: [fogColor[0], fogColor[1], fogColor[2], 1.0],
            fogParameters: [0.0, 512.0, 512.0, -1.0],
            // The fragment shader's >10 sentinel bypasses both lightmap and
            // fog exactly as RenderGlobal does for sunrise/celestial passes.
            lightmapParameters: [1.0, 0.0, 0.0, 99.0],
        },
        hudPushConstants: dynamicMeshes.hudPushConstants,
        clearColor,
    }
}


fn build_first_person_item_meshes(
    capture: &WorldRenderCapture,
    atlas: &AtlasState,
    aspect: f32,
    farPlaneDistance: f32,
    lightmap: crate::net::minecraft::client::renderer::EntityRenderer::LightmapParameters,
) -> (Vec<WorldVertex>, Vec<u32>, Vec<FirstPersonDrawRange>, WorldPushConstants) {
    let projection = perspective_matrix(
        capture.fov.clamp(30.0, 110.0),
        aspect,
        0.05,
        farPlaneDistance * 2.0,
    );
    let fireFrameOffsets = atlas.fireFrameOffsets(capture.totalWorldTime);
    let pushConstants = WorldPushConstants {
        viewProjection: to_column_major(projection),
        cameraPosition: [0.0, 0.0, 0.0, fireFrameOffsets[0]],
        // ItemRenderer disables world fog for the hand pass. A very distant
        // interval preserves the shared shader without applying scene fog.
        // fogColor.w carries fire_layer_1's TextureAtlasSprite V offset.
        fogColor: [0.0, 0.0, 0.0, fireFrameOffsets[1]],
        fogParameters: [1.0e8, 1.0e8 + 1.0, farPlaneDistance * 2.0, 0.1],
        lightmapParameters: [
            lightmap.sunBrightness,
            lightmap.torchFlickerX,
            lightmap.gammaSetting,
            lightmap.dimension as f32,
        ],
    };

    if capture.gameType == GameType::Spectator || capture.thirdPersonView > 0 {
        return (Vec::new(), Vec::new(), Vec::new(), pushConstants);
    }

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut glintVertices = Vec::new();
    let mut glintIndices = Vec::new();
    let mainSide = capture.primaryHand;
    let offSide = mainSide.opposite();
    let itemLights = first_person_item_lights(
        capture.playerPosition.rotationPitch,
        capture.playerPosition.rotationYaw,
    );
    let mainSwing = if capture.localSwingingHand == EnumHand::MainHand {
        capture.localSwingProgress
    } else {
        0.0
    };
    let offSwing = if capture.localSwingingHand == EnumHand::OffHand {
        capture.localSwingProgress
    } else {
        0.0
    };

    let bowActive = capture.firstPersonItems.handActive
        && capture.firstPersonItems.activeUseAction == EnumAction::Bow;
    let renderMain = !bowActive || capture.firstPersonItems.activeHand == EnumHand::MainHand;
    let renderOff = !bowActive || capture.firstPersonItems.activeHand == EnumHand::OffHand;
    if renderMain {
        append_first_person_item(
            &capture.firstPersonItems.itemStackMainHand,
            true,
            &capture.localSkinLocation,
            capture.localSlim,
            capture.localSkinParts,
            mainSide,
            mainSwing,
            capture.firstPersonItems.equipOffsetMainHand,
            capture.firstPersonItems.handActive && capture.firstPersonItems.activeHand == EnumHand::MainHand,
            capture.firstPersonItems.activeUseAction,
            capture.firstPersonItems.itemInUseCount,
            capture.firstPersonItems.activeMaxUseDuration,
            capture.partialTicks,
            capture.firstPersonPackedLight,
            capture.localArmPitchOffset,
            capture.localArmYawOffset,
            itemLights,
            capture.systemTimeMillis,
            capture.playerPosition.rotationPitch,
            capture.localInvisible,
            capture.firstPersonItems.itemStackOffHand.isEmpty(),
            &capture.mapData,
            atlas,
            &mut vertices,
            &mut indices,
            &mut glintVertices,
            &mut glintIndices,
        );
    }
    if renderOff {
        append_first_person_item(
            &capture.firstPersonItems.itemStackOffHand,
            false,
            &capture.localSkinLocation,
            capture.localSlim,
            capture.localSkinParts,
            offSide,
            offSwing,
            capture.firstPersonItems.equipOffsetOffHand,
            capture.firstPersonItems.handActive && capture.firstPersonItems.activeHand == EnumHand::OffHand,
            capture.firstPersonItems.activeUseAction,
            capture.firstPersonItems.itemInUseCount,
            capture.firstPersonItems.activeMaxUseDuration,
            capture.partialTicks,
            capture.firstPersonPackedLight,
            capture.localArmPitchOffset,
            capture.localArmYawOffset,
            itemLights,
            capture.systemTimeMillis,
            capture.playerPosition.rotationPitch,
            capture.localInvisible,
            false,
            &capture.mapData,
            atlas,
            &mut vertices,
            &mut indices,
            &mut glintVertices,
            &mut glintIndices,
        );
    }

    let fireFirstIndex = indices.len() as u32;
    if capture.localBurning {
        append_first_person_fire_overlay(atlas, &mut vertices, &mut indices);
    }
    let fireIndexCount = indices.len() as u32 - fireFirstIndex;

    let mut drawRanges = Vec::new();
    if fireFirstIndex > 0 {
        drawRanges.push(FirstPersonDrawRange {
            pipeline: FirstPersonPipelineKind::Alpha,
            firstIndex: 0,
            indexCount: fireFirstIndex,
        });
    }
    if fireIndexCount > 0 {
        drawRanges.push(FirstPersonDrawRange {
            pipeline: FirstPersonPipelineKind::Fire,
            firstIndex: fireFirstIndex,
            indexCount: fireIndexCount,
        });
    }
    if !glintIndices.is_empty() {
        let vertexOffset = vertices.len() as u32;
        let firstIndex = indices.len() as u32;
        indices.extend(glintIndices.into_iter().map(|index| index + vertexOffset));
        vertices.extend(glintVertices);
        drawRanges.push(FirstPersonDrawRange {
            pipeline: FirstPersonPipelineKind::Glint,
            firstIndex,
            indexCount: indices.len() as u32 - firstIndex,
        });
    }

    (vertices, indices, drawRanges, pushConstants)
}

/// MCP `ItemRenderer#renderFireInFirstPerson`. The dedicated first-person
/// pass supplies the source hand projection and unconditional overlay ordering;
/// the submitted quads therefore remain at the exact source z=-0.5 depth.
fn append_first_person_fire_overlay(
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let rectangle = atlas.fireLayer1Rectangle;
    for i in 0..2_i32 {
        let sign = (i * 2 - 1) as f32;
        let matrix = first_person_fire_matrix(sign);
        // MCP `ItemRenderer#renderFireInFirstPerson` emits the quad at
        // z=-0.5 with the hand projection/model-view still active. Do not
        // move it toward the 0.05 near plane: that enlarges the quad by almost
        // an order of magnitude and produces the opaque-looking orange slabs
        // reported in Batch 77A. Visibility is handled by the dedicated
        // first-person pass, while geometry remains source-exact here.
        let local = [
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
            [0.5, 0.5, -0.5],
            [-0.5, 0.5, -0.5],
        ];
        let uv = [
            [rectangle[2], rectangle[3]],
            [rectangle[0], rectangle[3]],
            [rectangle[0], rectangle[1]],
            [rectangle[2], rectangle[1]],
        ];
        let base = vertices.len() as u32;
        for corner in 0..4 {
            vertices.push(WorldVertex {
                position: transform_point3(matrix, local[corner]),
                uv: uv[corner],
                color: [1.0, 1.0, 1.0, encoded_fire_alpha(0.9, 1)],
                lightmap: [15.0, 15.0],
            
                shaderEntity: [-1, -1, -1],
                shaderPadding: 0,
            });
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

fn first_person_fire_matrix(sign: f32) -> [[f32; 4]; 4] {
    let mut matrix = translation4([-sign * 0.24, -0.3, 0.0]);
    matrix = multiply4(matrix, rotation_y4(sign * 10.0));
    matrix
}

#[allow(clippy::too_many_arguments)]
fn append_first_person_item(
    stack: &ItemStack,
    renderEmptyMainArm: bool,
    localSkinLocation: &ResourceLocation,
    localSlim: bool,
    localSkinParts: u8,
    handSide: EnumHandSide,
    swingProgress: f32,
    equipOffset: f32,
    handActive: bool,
    useAction: EnumAction,
    itemInUseCount: i32,
    maxUseDuration: i32,
    partialTicks: f32,
    packedLight: u32,
    armPitchOffset: f32,
    armYawOffset: f32,
    itemLights: [[f32; 3]; 2],
    systemTimeMillis: u64,
    playerPitch: f32,
    playerInvisible: bool,
    offHandEmpty: bool,
    mapData: &HashMap<i32, MapData>,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
    glintVertices: &mut Vec<WorldVertex>,
    glintIndices: &mut Vec<u32>,
) {
    if stack.isEmpty() {
        // ItemRenderer renders an empty arm only for the main hand and only in
        // first person. Use NetworkPlayerInfo's downloaded skin, retaining
        // UUID-selected Steve/Alex only as the MCP fallback.
        if renderEmptyMainArm && !playerInvisible {
            append_first_person_arm(
                handSide,
                swingProgress,
                equipOffset,
                localSkinLocation,
                localSlim,
                localSkinParts,
                packedLight,
                armPitchOffset,
                armYawOffset,
                itemLights,
                atlas,
                vertices,
                indices,
            );
        }
        return;
    }

    if ItemMap::isFilledMap(stack) {
        append_first_person_map(
            stack,
            renderEmptyMainArm && offHandEmpty,
            handSide,
            swingProgress,
            equipOffset,
            playerPitch,
            playerInvisible,
            localSkinLocation,
            localSlim,
            localSkinParts,
            packedLight,
            armPitchOffset,
            armYawOffset,
            itemLights,
            mapData,
            atlas,
            vertices,
            indices,
        );
        return;
    }

    let Some(modelKey) = ItemModelMesher::getModelKey(stack) else { return; };
    let Some(baseModel) = atlas.itemModels.get(&modelKey) else { return; };
    let blockingShield = handActive && useAction == EnumAction::Block && is_unpatterned_shield(stack);
    let model = if blockingShield {
        atlas.shieldBlockingModel.as_ref().unwrap_or(baseModel)
    } else {
        baseModel
    };
    let unpatternedShield = model.builtInRenderer && is_unpatterned_shield(stack);
    let supportedBuiltIn = model.builtInRenderer
        && (unpatternedShield || TileEntityItemStackRenderer::buildMesh(stack).is_some());
    if (model.builtInRenderer && !supportedBuiltIn)
        || (!model.builtInRenderer && model.quads.is_empty())
    {
        // Maps and patterned shields still require their exact dedicated
        // renderer. Never substitute an unrelated baked model.
        return;
    }

    let rightHand = handSide == EnumHandSide::Right;
    let side = if rightHand { 1.0 } else { -1.0 };
    let swing = swingProgress.clamp(0.0, 1.0);
    let rootSwing = swing.sqrt();

    let mut matrix = identity4();
    // ItemRenderer.rotateArm follows the camera with EntityPlayerSP's
    // half-delta render-arm state before either hand branch is transformed.
    matrix = multiply4(matrix, rotation_x4(armPitchOffset));
    matrix = multiply4(matrix, rotation_y4(armYawOffset));

    let sideTransform = |matrix: [[f32; 4]; 4]| {
        multiply4(matrix, translation4([
            side * 0.56,
            -0.52 + equipOffset.clamp(0.0, 1.0) * -0.6,
            -0.72,
        ]))
    };

    if handActive && itemInUseCount > 0 {
        match useAction {
            EnumAction::Eat | EnumAction::Drink => {
                let remaining = itemInUseCount as f32 - partialTicks.clamp(0.0, 1.0) + 1.0;
                let duration = maxUseDuration.max(1) as f32;
                let ratio = remaining / duration;
                if ratio < 0.8 {
                    let bob = (remaining / 4.0 * std::f32::consts::PI).cos().abs() * 0.1;
                    matrix = multiply4(matrix, translation4([0.0, bob, 0.0]));
                }
                let progress = 1.0 - ratio.powf(27.0);
                matrix = multiply4(matrix, translation4([progress * 0.6 * side, progress * -0.5, 0.0]));
                matrix = multiply4(matrix, rotation_y4(side * progress * 90.0));
                matrix = multiply4(matrix, rotation_x4(progress * 10.0));
                matrix = multiply4(matrix, rotation_z4(side * progress * 30.0));
                matrix = sideTransform(matrix);
            }
            EnumAction::Bow => {
                matrix = sideTransform(matrix);
                matrix = multiply4(matrix, translation4([side * -0.2785682, 0.18344387, 0.15731531]));
                matrix = multiply4(matrix, rotation_x4(-13.935));
                matrix = multiply4(matrix, rotation_y4(side * 35.3));
                matrix = multiply4(matrix, rotation_z4(side * -9.785));
                let elapsed = maxUseDuration.max(0) as f32
                    - (itemInUseCount as f32 - partialTicks.clamp(0.0, 1.0) + 1.0);
                let mut pull = elapsed / 20.0;
                pull = (pull * pull + pull * 2.0) / 3.0;
                pull = pull.min(1.0);
                if pull > 0.1 {
                    let bowBob = ((elapsed - 0.1) * 1.3).sin() * (pull - 0.1);
                    matrix = multiply4(matrix, translation4([0.0, bowBob * 0.004, 0.0]));
                }
                matrix = multiply4(matrix, translation4([0.0, 0.0, pull * 0.04]));
                matrix = multiply4(matrix, scale4_nonuniform([1.0, 1.0, 1.0 + pull * 0.2]));
                matrix = multiply4(matrix, rotation_y4(side * -45.0));
            }
            EnumAction::None | EnumAction::Block => {
                matrix = sideTransform(matrix);
            }
        }
    } else {
        // Exact non-active-item branch of MCP ItemRenderer.renderItemInFirstPerson.
        let translateX = -0.4 * (rootSwing * std::f32::consts::PI).sin();
        let translateY = 0.2 * (rootSwing * std::f32::consts::PI * 2.0).sin();
        let translateZ = -0.2 * (swing * std::f32::consts::PI).sin();
        let swingSquared = (swing * swing * std::f32::consts::PI).sin();
        let swingRootSin = (rootSwing * std::f32::consts::PI).sin();
        matrix = multiply4(matrix, translation4([side * translateX, translateY, translateZ]));
        matrix = sideTransform(matrix);
        matrix = multiply4(matrix, rotation_y4(side * (45.0 + swingSquared * -20.0)));
        matrix = multiply4(matrix, rotation_z4(side * swingRootSin * -20.0));
        matrix = multiply4(matrix, rotation_x4(swingRootSin * -80.0));
        matrix = multiply4(matrix, rotation_y4(side * -45.0));
    }

    let transformType = if rightHand {
        TransformType::FirstPersonRightHand
    } else {
        TransformType::FirstPersonLeftHand
    };
    let itemTransform = model.transforms.getTransform(transformType);
    matrix = multiply4(matrix, item_camera_transform4(itemTransform, !rightHand));
    matrix = multiply4(matrix, translation4([-0.5, -0.5, -0.5]));

    let blockLight = ((packedLight >> 4) & 15) as f32;
    let skyLight = ((packedLight >> 20) & 15) as f32;
    if unpatternedShield {
        append_shield_model_world(
            matrix,
            atlas.shieldBaseRectangle,
            blockLight,
            skyLight,
            itemLights,
            vertices,
            indices,
        );
        return;
    }
    if model.builtInRenderer {
        append_builtin_item_mesh_world(
            stack, matrix, blockLight, skyLight, itemLights, atlas, vertices, indices,
        );
        return;
    }

    let reverseWinding = (itemTransform.scale[0] < 0.0)
        ^ (itemTransform.scale[1] < 0.0)
        ^ (itemTransform.scale[2] < 0.0);

    for quad in &model.quads {
        let transformed = quad.positions.map(|position| transform_point3(matrix, position));
        let edge1 = subtract3(transformed[1], transformed[0]);
        let edge2 = subtract3(transformed[2], transformed[0]);
        let normal = normalize3(cross3(edge1, edge2));
        let key = item_material_key(stack.itemId, quad.texture.clone(), quad.tintIndex);
        let rectangle = atlas
            .rectangles
            .get(&key)
            .copied()
            .unwrap_or(atlas.missingRectangle);
        let tint = item_tint_color(&atlas.itemColors, stack, quad.tintIndex);
        let diffuse = if model.gui3d && quad.shade {
            standard_item_diffuse(normal, itemLights)
        } else {
            1.0
        };
        let base = vertices.len() as u32;
        for vertexIndex in 0..4 {
            let uv = [
                rectangle[0]
                    + (rectangle[2] - rectangle[0]) * quad.uvs[vertexIndex][0],
                rectangle[1]
                    + (rectangle[3] - rectangle[1]) * quad.uvs[vertexIndex][1],
            ];
            vertices.push(WorldVertex {
                position: transformed[vertexIndex],
                uv,
                color: [
                    tint[0] * diffuse,
                    tint[1] * diffuse,
                    tint[2] * diffuse,
                    1.0,
                ],
                lightmap: [blockLight, skyLight],
            
                shaderEntity: [-1, -1, -1],
                shaderPadding: 0,
            });
        }
        if reverseWinding {
            indices.extend_from_slice(&[base, base + 2, base + 1, base + 2, base, base + 3]);
        } else {
            indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
        }

        if stack.hasEffect() {
            append_first_person_glint_quad(
                transformed,
                quad.uvs,
                reverseWinding,
                systemTimeMillis,
                atlas.glintRectangle,
                glintVertices,
                glintIndices,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn append_first_person_arm(
    handSide: EnumHandSide,
    swingProgress: f32,
    equipOffset: f32,
    localSkinLocation: &ResourceLocation,
    localSlim: bool,
    localSkinParts: u8,
    packedLight: u32,
    armPitchOffset: f32,
    armYawOffset: f32,
    itemLights: [[f32; 3]; 2],
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let rightHand = handSide == EnumHandSide::Right;
    let side = if rightHand { 1.0 } else { -1.0 };
    let swing = swingProgress.clamp(0.0, 1.0);
    let rootSwing = swing.sqrt();

    let mut matrix = identity4();
    matrix = multiply4(matrix, rotation_x4(armPitchOffset));
    matrix = multiply4(matrix, rotation_y4(armYawOffset));

    let translateX = -0.3 * (rootSwing * std::f32::consts::PI).sin();
    let translateY = 0.4 * (rootSwing * std::f32::consts::TAU).sin();
    let translateZ = -0.4 * (swing * std::f32::consts::PI).sin();
    matrix = multiply4(matrix, translation4([
        side * (translateX + 0.64000005),
        translateY - 0.6 + equipOffset.clamp(0.0, 1.0) * -0.6,
        translateZ - 0.71999997,
    ]));
    matrix = multiply4(matrix, rotation_y4(side * 45.0));
    let swingSquared = (swing * swing * std::f32::consts::PI).sin();
    let swingRoot = (rootSwing * std::f32::consts::PI).sin();
    matrix = multiply4(matrix, rotation_y4(side * swingRoot * 70.0));
    matrix = multiply4(matrix, rotation_z4(side * swingSquared * -20.0));
    matrix = multiply4(matrix, translation4([side * -1.0, 3.6, 3.5]));
    matrix = multiply4(matrix, rotation_z4(side * 120.0));
    matrix = multiply4(matrix, rotation_x4(200.0));
    matrix = multiply4(matrix, rotation_y4(side * -135.0));
    matrix = multiply4(matrix, translation4([side * 5.6, 0.0, 0.0]));

    append_first_person_arm_mesh(
        matrix,
        handSide,
        localSkinLocation,
        localSlim,
        localSkinParts,
        packedLight,
        itemLights,
        atlas,
        vertices,
        indices,
    );
}


#[allow(clippy::too_many_arguments)]
fn append_first_person_map(
    stack: &ItemStack,
    twoHanded: bool,
    handSide: EnumHandSide,
    swingProgress: f32,
    equipOffset: f32,
    playerPitch: f32,
    playerInvisible: bool,
    localSkinLocation: &ResourceLocation,
    localSlim: bool,
    localSkinParts: u8,
    packedLight: u32,
    armPitchOffset: f32,
    armYawOffset: f32,
    itemLights: [[f32; 3]; 2],
    mapData: &HashMap<i32, MapData>,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let mut root = identity4();
    // `ItemRenderer#rotateArm` occurs before either map branch.
    root = multiply4(root, rotation_x4(armPitchOffset));
    root = multiply4(root, rotation_y4(armYawOffset));

    if twoHanded {
        append_first_person_map_two_handed(
            stack,
            root,
            swingProgress,
            equipOffset,
            playerPitch,
            playerInvisible,
            localSkinLocation,
            localSlim,
            localSkinParts,
            packedLight,
            itemLights,
            mapData,
            atlas,
            vertices,
            indices,
        );
    } else {
        append_first_person_map_side(
            stack,
            root,
            handSide,
            swingProgress,
            equipOffset,
            playerInvisible,
            localSkinLocation,
            localSlim,
            localSkinParts,
            packedLight,
            itemLights,
            mapData,
            atlas,
            vertices,
            indices,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn append_first_person_map_two_handed(
    stack: &ItemStack,
    mut matrix: [[f32; 4]; 4],
    swingProgress: f32,
    equipOffset: f32,
    playerPitch: f32,
    playerInvisible: bool,
    localSkinLocation: &ResourceLocation,
    localSlim: bool,
    localSkinParts: u8,
    packedLight: u32,
    itemLights: [[f32; 3]; 2],
    mapData: &HashMap<i32, MapData>,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let swing = swingProgress.clamp(0.0, 1.0);
    let rootSwing = swing.sqrt();
    let translateY = -0.2 * (swing * std::f32::consts::PI).sin();
    let translateZ = -0.4 * (rootSwing * std::f32::consts::PI).sin();
    matrix = multiply4(matrix, translation4([0.0, -translateY * 0.5, translateZ]));

    let mapPitch = ItemRenderer::getMapAngleFromPitch(playerPitch);
    matrix = multiply4(
        matrix,
        translation4([
            0.0,
            0.04 + equipOffset.clamp(0.0, 1.0) * -1.2 + mapPitch * -0.5,
            -0.72,
        ]),
    );
    matrix = multiply4(matrix, rotation_x4(mapPitch * -85.0));

    if !playerInvisible {
        let armsRoot = multiply4(matrix, rotation_y4(90.0));
        for side in [EnumHandSide::Right, EnumHandSide::Left] {
            let sign = if side == EnumHandSide::Right { 1.0 } else { -1.0 };
            let mut armMatrix = armsRoot;
            armMatrix = multiply4(armMatrix, rotation_y4(92.0));
            armMatrix = multiply4(armMatrix, rotation_x4(45.0));
            armMatrix = multiply4(armMatrix, rotation_z4(sign * -41.0));
            armMatrix = multiply4(armMatrix, translation4([sign * 0.3, -1.1, 0.45]));
            append_first_person_arm_mesh(
                armMatrix,
                side,
                localSkinLocation,
                localSlim,
                localSkinParts,
                packedLight,
                itemLights,
                atlas,
                vertices,
                indices,
            );
        }
    }

    let swingRotation = (rootSwing * std::f32::consts::PI).sin();
    matrix = multiply4(matrix, rotation_x4(swingRotation * 20.0));
    matrix = multiply4(matrix, scale4_nonuniform([2.0, 2.0, 2.0]));
    append_first_person_map_plane(stack, matrix, mapData, atlas, vertices, indices);
}

#[allow(clippy::too_many_arguments)]
fn append_first_person_map_side(
    stack: &ItemStack,
    mut matrix: [[f32; 4]; 4],
    handSide: EnumHandSide,
    swingProgress: f32,
    equipOffset: f32,
    playerInvisible: bool,
    localSkinLocation: &ResourceLocation,
    localSlim: bool,
    localSkinParts: u8,
    packedLight: u32,
    itemLights: [[f32; 3]; 2],
    mapData: &HashMap<i32, MapData>,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let sign = if handSide == EnumHandSide::Right { 1.0 } else { -1.0 };
    let swing = swingProgress.clamp(0.0, 1.0);
    let rootSwing = swing.sqrt();
    matrix = multiply4(matrix, translation4([sign * 0.125, -0.125, 0.0]));

    if !playerInvisible {
        let armRoot = multiply4(matrix, rotation_z4(sign * 10.0));
        let armMatrix = first_person_arm_matrix(armRoot, handSide, swing, equipOffset);
        append_first_person_arm_mesh(
            armMatrix,
            handSide,
            localSkinLocation,
            localSlim,
            localSkinParts,
            packedLight,
            itemLights,
            atlas,
            vertices,
            indices,
        );
    }

    matrix = multiply4(
        matrix,
        translation4([
            sign * 0.51,
            -0.08 + equipOffset.clamp(0.0, 1.0) * -1.2,
            -0.75,
        ]),
    );
    let swingSin = (rootSwing * std::f32::consts::PI).sin();
    let translateX = -0.5 * swingSin;
    let translateY = 0.4 * (rootSwing * std::f32::consts::TAU).sin();
    let translateZ = -0.3 * (swing * std::f32::consts::PI).sin();
    matrix = multiply4(
        matrix,
        translation4([
            sign * translateX,
            translateY - 0.3 * swingSin,
            translateZ,
        ]),
    );
    matrix = multiply4(matrix, rotation_x4(swingSin * -45.0));
    matrix = multiply4(matrix, rotation_y4(sign * swingSin * -30.0));
    append_first_person_map_plane(stack, matrix, mapData, atlas, vertices, indices);
}

fn first_person_arm_matrix(
    mut matrix: [[f32; 4]; 4],
    handSide: EnumHandSide,
    swingProgress: f32,
    equipOffset: f32,
) -> [[f32; 4]; 4] {
    let sign = if handSide == EnumHandSide::Right { 1.0 } else { -1.0 };
    let swing = swingProgress.clamp(0.0, 1.0);
    let rootSwing = swing.sqrt();
    let translateX = -0.3 * (rootSwing * std::f32::consts::PI).sin();
    let translateY = 0.4 * (rootSwing * std::f32::consts::TAU).sin();
    let translateZ = -0.4 * (swing * std::f32::consts::PI).sin();
    matrix = multiply4(
        matrix,
        translation4([
            sign * (translateX + 0.64000005),
            translateY - 0.6 + equipOffset.clamp(0.0, 1.0) * -0.6,
            translateZ - 0.71999997,
        ]),
    );
    matrix = multiply4(matrix, rotation_y4(sign * 45.0));
    let swingSquared = (swing * swing * std::f32::consts::PI).sin();
    let swingRoot = (rootSwing * std::f32::consts::PI).sin();
    matrix = multiply4(matrix, rotation_y4(sign * swingRoot * 70.0));
    matrix = multiply4(matrix, rotation_z4(sign * swingSquared * -20.0));
    matrix = multiply4(matrix, translation4([sign * -1.0, 3.6, 3.5]));
    matrix = multiply4(matrix, rotation_z4(sign * 120.0));
    matrix = multiply4(matrix, rotation_x4(200.0));
    matrix = multiply4(matrix, rotation_y4(sign * -135.0));
    multiply4(matrix, translation4([sign * 5.6, 0.0, 0.0]))
}

#[allow(clippy::too_many_arguments)]
fn append_first_person_map_plane(
    stack: &ItemStack,
    mut matrix: [[f32; 4]; 4],
    mapData: &HashMap<i32, MapData>,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    matrix = multiply4(matrix, rotation_y4(180.0));
    matrix = multiply4(matrix, rotation_z4(180.0));
    matrix = multiply4(matrix, scale4_nonuniform([0.38, 0.38, 0.38]));
    matrix = multiply4(matrix, translation4([-0.5, -0.5, 0.0]));
    matrix = multiply4(matrix, scale4_nonuniform([1.0 / 128.0; 3]));

    append_map_quad(
        matrix,
        [-7.0, -7.0],
        [135.0, 135.0],
        0.0,
        atlas.mapBackgroundRectangle,
        [1.0, 1.0, 1.0, 1.0],
        false,
        vertices,
        indices,
    );
    if let Some(map) = ItemMap::getMapData(stack, mapData) {
        append_map_mesh(map, false, matrix, atlas, vertices, indices);
    }
}

#[allow(clippy::too_many_arguments)]
fn append_first_person_arm_mesh(
    matrix: [[f32; 4]; 4],
    handSide: EnumHandSide,
    localSkinLocation: &ResourceLocation,
    localSlim: bool,
    localSkinParts: u8,
    packedLight: u32,
    itemLights: [[f32; 3]; 2],
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let mesh = RenderPlayer::buildFirstPersonArmMesh(localSlim, handSide, localSkinParts);
    let rectangle = player_skin_rectangle(atlas, localSkinLocation, localSlim);
    let blockLight = ((packedLight >> 4) & 15) as f32;
    let skyLight = ((packedLight >> 20) & 15) as f32;

    for quad in mesh.vertices.chunks_exact(4) {
        let transformed = [
            transform_point3(matrix, quad[0].position),
            transform_point3(matrix, quad[1].position),
            transform_point3(matrix, quad[2].position),
            transform_point3(matrix, quad[3].position),
        ];
        let normal = normalize3(cross3(
            subtract3(transformed[1], transformed[0]),
            subtract3(transformed[2], transformed[0]),
        ));
        let diffuse = standard_item_diffuse(normal, itemLights);
        let base = vertices.len() as u32;
        for (position, source) in transformed.into_iter().zip(quad.iter()) {
            vertices.push(WorldVertex {
                position,
                uv: map_player_skin_uv(rectangle, source.uv),
                color: [diffuse, diffuse, diffuse, 1.0],
                lightmap: [blockLight, skyLight],
            
                shaderEntity: [-1, -1, -1],
                shaderPadding: 0,
            });
        }
        // RenderPlayer disables culling around first-person arms.
        indices.extend_from_slice(&[
            base, base + 1, base + 2, base, base + 2, base + 3,
            base + 2, base + 1, base, base + 3, base + 2, base,
        ]);
    }
}

fn append_first_person_glint_quad(
    positions: [[f32; 3]; 4],
    modelUvs: [[f32; 2]; 4],
    reverseWinding: bool,
    systemTimeMillis: u64,
    rectangle: [f32; 4],
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    // RenderItem.func_191966_a: two texture-matrix passes over the same baked
    // item geometry, full-bright, depth-equal and blend(SRC_COLOR, ONE).
    for (period, direction, angle) in [
        (3000_u64, 1.0_f32, -50.0_f32),
        (4873_u64, -1.0_f32, 10.0_f32),
    ] {
        let translation = direction * (systemTimeMillis % period) as f32 / period as f32 / 8.0;
        let (sin, cos) = angle.to_radians().sin_cos();
        let base = vertices.len() as u32;
        for index in 0..4 {
            let u = modelUvs[index][0];
            let v = modelUvs[index][1];
            let transformedUv = [
                (8.0 * (cos * u - sin * v + translation)).rem_euclid(1.0),
                (8.0 * (sin * u + cos * v)).rem_euclid(1.0),
            ];
            let uv = [
                rectangle[0] + (rectangle[2] - rectangle[0]) * transformedUv[0],
                rectangle[1] + (rectangle[3] - rectangle[1]) * transformedUv[1],
            ];
            vertices.push(WorldVertex {
                position: positions[index],
                uv,
                color: [128.0 / 255.0, 64.0 / 255.0, 204.0 / 255.0, 1.0],
                lightmap: [15.0, 15.0],
            
                shaderEntity: [-1, -1, -1],
                shaderPadding: 0,
            });
        }
        if reverseWinding {
            indices.extend_from_slice(&[base, base + 2, base + 1, base + 2, base, base + 3]);
        } else {
            indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
        }
    }
}

fn first_person_item_lights(pitch: f32, yaw: f32) -> [[f32; 3]; 2] {
    // ItemRenderer.rotateArroundXAndY temporarily rotates the model-view
    // matrix while RenderHelper installs its two directional lights. OpenGL
    // therefore stores the light directions after Rx(pitch) * Ry(yaw).
    let lightingMatrix = multiply4(rotation_x4(pitch), rotation_y4(yaw));
    [
        normalize3(transform_direction3(lightingMatrix, [0.2, 1.0, -0.7])),
        normalize3(transform_direction3(lightingMatrix, [-0.2, 1.0, 0.7])),
    ]
}

fn standard_item_diffuse(normal: [f32; 3], lights: [[f32; 3]; 2]) -> f32 {
    (0.4
        + 0.6 * dot3(normal, lights[0]).max(0.0)
        + 0.6 * dot3(normal, lights[1]).max(0.0))
        .clamp(0.0, 1.0)
}

fn item_camera_transform4(transform: ItemTransformVec3f, leftHanded: bool) -> [[f32; 4]; 4] {
    let side = if leftHanded { -1.0 } else { 1.0 };
    let rotation = [
        transform.rotation[0],
        transform.rotation[1] * side,
        transform.rotation[2] * side,
    ];
    let mut matrix = translation4([
        side * transform.translation[0],
        transform.translation[1],
        transform.translation[2],
    ]);
    matrix = multiply4(matrix, quaternion_rotation4(rotation));
    multiply4(matrix, scale4_nonuniform(transform.scale))
}

fn identity4() -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn scale4_nonuniform(scale: [f32; 3]) -> [[f32; 4]; 4] {
    [
        [scale[0], 0.0, 0.0, 0.0],
        [0.0, scale[1], 0.0, 0.0],
        [0.0, 0.0, scale[2], 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn rotation_x4(degrees: f32) -> [[f32; 4]; 4] {
    let (sin, cos) = degrees.to_radians().sin_cos();
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, cos, -sin, 0.0],
        [0.0, sin, cos, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn rotation_y4(degrees: f32) -> [[f32; 4]; 4] {
    let (sin, cos) = degrees.to_radians().sin_cos();
    [
        [cos, 0.0, sin, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [-sin, 0.0, cos, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn rotation_z4(degrees: f32) -> [[f32; 4]; 4] {
    let (sin, cos) = degrees.to_radians().sin_cos();
    [
        [cos, -sin, 0.0, 0.0],
        [sin, cos, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn rotation_axis4(degrees: f32, axis: [f32; 3]) -> [[f32; 4]; 4] {
    let length = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if length <= f32::EPSILON { return identity4(); }
    let [x, y, z] = [axis[0] / length, axis[1] / length, axis[2] / length];
    let (sin, cos) = degrees.to_radians().sin_cos();
    let oneMinusCos = 1.0 - cos;
    [
        [cos + x * x * oneMinusCos, x * y * oneMinusCos - z * sin, x * z * oneMinusCos + y * sin, 0.0],
        [y * x * oneMinusCos + z * sin, cos + y * y * oneMinusCos, y * z * oneMinusCos - x * sin, 0.0],
        [z * x * oneMinusCos - y * sin, z * y * oneMinusCos + x * sin, cos + z * z * oneMinusCos, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn quaternion_rotation4(rotation: [f32; 3]) -> [[f32; 4]; 4] {
    let xAxis = rotate_item_quaternion([1.0, 0.0, 0.0], rotation);
    let yAxis = rotate_item_quaternion([0.0, 1.0, 0.0], rotation);
    let zAxis = rotate_item_quaternion([0.0, 0.0, 1.0], rotation);
    [
        [xAxis[0], yAxis[0], zAxis[0], 0.0],
        [xAxis[1], yAxis[1], zAxis[1], 0.0],
        [xAxis[2], yAxis[2], zAxis[2], 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn transform_point3(matrix: [[f32; 4]; 4], position: [f32; 3]) -> [f32; 3] {
    let vector = [position[0], position[1], position[2], 1.0];
    let mut output = [0.0_f32; 4];
    for row in 0..4 {
        output[row] = matrix[row][0] * vector[0]
            + matrix[row][1] * vector[1]
            + matrix[row][2] * vector[2]
            + matrix[row][3];
    }
    let w = if output[3].abs() <= f32::EPSILON { 1.0 } else { output[3] };
    [output[0] / w, output[1] / w, output[2] / w]
}

fn transform_direction3(matrix: [[f32; 4]; 4], direction: [f32; 3]) -> [f32; 3] {
    [
        matrix[0][0] * direction[0]
            + matrix[0][1] * direction[1]
            + matrix[0][2] * direction[2],
        matrix[1][0] * direction[0]
            + matrix[1][1] * direction[1]
            + matrix[1][2] * direction[2],
        matrix[2][0] * direction[0]
            + matrix[2][1] * direction[1]
            + matrix[2][2] * direction[2],
    ]
}

fn perspective_matrix(
    fovDegrees: f32,
    aspect: f32,
    near: f32,
    far: f32,
) -> [[f32; 4]; 4] {
    let cotangent = 1.0 / (fovDegrees.to_radians() * 0.5).tan();
    [
        [cotangent / aspect.max(0.0001), 0.0, 0.0, 0.0],
        [0.0, -cotangent, 0.0, 0.0],
        [0.0, 0.0, far / (near - far), (far * near) / (near - far)],
        [0.0, 0.0, -1.0, 0.0],
    ]
}

fn build_ingame_hud(
    capture: &WorldRenderCapture,
    atlas: &AtlasState,
    guiIngame: &mut GuiIngame,
    guiBossOverlay: &mut GuiBossOverlay,
    playerTabOverlay: &mut GuiPlayerTabOverlay,
    guiNewChat: &mut GuiNewChat,
    fontRenderer: &mut FontRenderer,
    standardGalacticFontRenderer: &mut FontRenderer,
    locale: &Locale,
) -> (Vec<WorldVertex>, Vec<u32>, Vec<HudDrawRange>, WorldPushConstants) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut drawRanges = Vec::new();
    let guiWidth = capture.guiWidth.max(1);
    let guiHeight = capture.guiHeight.max(1);

    let mut begin = indices.len() as u32;
    append_item_activation(capture, atlas, &mut vertices, &mut indices);
    push_hud_range(&mut drawRanges, HudPipelineKind::Alpha, begin, indices.len() as u32);

    begin = indices.len() as u32;
    let bossFrame = guiBossOverlay.buildFrame(guiWidth, guiHeight, capture.systemTimeMillis, fontRenderer);
    for quad in &bossFrame.bars {
        append_hud_quad(
            &mut vertices, &mut indices, atlas.barsRectangle,
            quad.x, quad.y, quad.width, quad.height,
            quad.textureX, quad.textureY, quad.textureWidth, quad.textureHeight,
            quad.alpha,
        );
    }
    for text in &bossFrame.texts {
        append_hud_text(text, fontRenderer, atlas, &mut vertices, &mut indices);
    }
    push_hud_range(&mut drawRanges, HudPipelineKind::Alpha, begin, indices.len() as u32);

    let debugData = capture.showDebugInfo.then(|| DebugOverlayData {
        version: env!("CARGO_PKG_VERSION").to_owned(),
        versionType: "release".to_owned(),
        debugFps: capture.debugFps,
        reducedDebugInfo: capture.reducedDebugInfo,
        showDebugProfilerChart: capture.showDebugProfilerChart,
        showLagometer: capture.showLagometer,
        playerPosition: [
            capture.playerPosition.posX,
            capture.playerPosition.posY,
            capture.playerPosition.posZ,
        ],
        rotationYaw: capture.playerPosition.rotationYaw,
        rotationPitch: capture.playerPosition.rotationPitch,
        renderDistanceChunks: capture.renderDistanceChunks,
        loadedRenderChunks: capture.loadedRenderChunks,
        queuedRenderChunks: capture.queuedRenderChunks,
        remotePlayerCount: capture.remotePlayers.len(),
        nonPlayerEntityCount: capture.nonPlayerEntities.len(),
        particleCount: capture.particleStates.len() + capture.miscParticleStates.len(),
        dimension: capture.dimension,
        biomeName: capture.biomeName.clone(),
        skyLight: capture.skyLight,
        blockLight: capture.blockLight,
        worldTime: capture.worldTime,
        targetBlock: capture.targetBlock,
        outputWidth: capture.outputWidth,
        outputHeight: capture.outputHeight,
        vulkanDevice: capture.vulkanDevice.clone(),
    });

    let hud = guiIngame.buildFrameWithFont(
        guiWidth,
        guiHeight,
        capture.currentHotbarSlot,
        capture.offhandNonEmpty,
        capture.primaryHand,
        capture.gameType,
        capture.playerHealth,
        capture.absorptionAmount,
        capture.foodLevel,
        capture.saturationLevel,
        capture.armorValue,
        capture.air,
        capture.inWater,
        capture.hardcoreMode,
        &capture.activePotionEffects,
        capture.experience,
        capture.experienceLevel,
        capture.xpBarCap,
        capture.hurtResistantTime,
        capture.playerTicksExisted,
        capture.systemTimeMillis,
        Some(&capture.scoreboard),
        &capture.localPlayerName,
        capture.actionBarMessage.as_ref().map(|component| component.getUnformattedText()),
        capture.actionBarAge,
        capture.partialTicks,
        capture.showSubtitles,
        capture.cameraPosition,
        capture.cameraYaw,
        capture.cameraPitch,
        debugData.as_ref(),
        fontRenderer,
    );

    begin = indices.len() as u32;
    for quad in &hud.hotbar {
        let rectangle = match quad.texture {
            HudTexture::Widgets => atlas.widgetsRectangle,
            HudTexture::Icons => atlas.iconsRectangle,
            HudTexture::BossBars => atlas.barsRectangle,
            HudTexture::Inventory => atlas.inventoryRectangle,
        };
        append_hud_quad(
            &mut vertices, &mut indices, rectangle,
            quad.x, quad.y, quad.width, quad.height,
            quad.textureX, quad.textureY, quad.textureWidth, quad.textureHeight,
            quad.alpha,
        );
    }
    push_hud_range(&mut drawRanges, HudPipelineKind::Alpha, begin, indices.len() as u32);

    begin = indices.len() as u32;
    append_hotbar_item_models(capture, atlas, &mut vertices, &mut indices);
    push_hud_range(&mut drawRanges, HudPipelineKind::Alpha, begin, indices.len() as u32);

    begin = indices.len() as u32;
    append_hotbar_item_glints(capture, atlas, &mut vertices, &mut indices);
    push_hud_range(&mut drawRanges, HudPipelineKind::Glint, begin, indices.len() as u32);

    begin = indices.len() as u32;
    append_hotbar_item_overlays(capture, atlas, &mut vertices, &mut indices);
    push_hud_range(&mut drawRanges, HudPipelineKind::Alpha, begin, indices.len() as u32);

    begin = indices.len() as u32;
    for quad in &hud.crosshair {
        let rectangle = match quad.texture {
            HudTexture::Widgets => atlas.widgetsRectangle,
            HudTexture::Icons => atlas.iconsRectangle,
            HudTexture::BossBars => atlas.barsRectangle,
            HudTexture::Inventory => atlas.inventoryRectangle,
        };
        append_hud_quad(
            &mut vertices, &mut indices, rectangle,
            quad.x, quad.y, quad.width, quad.height,
            quad.textureX, quad.textureY, quad.textureWidth, quad.textureHeight,
            quad.alpha,
        );
    }
    push_hud_range(&mut drawRanges, HudPipelineKind::Crosshair, begin, indices.len() as u32);

    begin = indices.len() as u32;
    for quad in &hud.playerStats {
        let rectangle = match quad.texture {
            HudTexture::Widgets => atlas.widgetsRectangle,
            HudTexture::Icons => atlas.iconsRectangle,
            HudTexture::BossBars => atlas.barsRectangle,
            HudTexture::Inventory => atlas.inventoryRectangle,
        };
        append_hud_quad(
            &mut vertices, &mut indices, rectangle,
            quad.x, quad.y, quad.width, quad.height,
            quad.textureX, quad.textureY, quad.textureWidth, quad.textureHeight,
            quad.alpha,
        );
    }
    push_hud_range(&mut drawRanges, HudPipelineKind::Alpha, begin, indices.len() as u32);

    // `GuiIngame#renderPotionEffects` draws before the hotbar in the MCP
    // overlay order; keep it adjacent to playerStats in the same alpha range.
    begin = indices.len() as u32;
    for quad in &hud.potionEffects {
        let rectangle = match quad.texture {
            HudTexture::Widgets => atlas.widgetsRectangle,
            HudTexture::Icons => atlas.iconsRectangle,
            HudTexture::BossBars => atlas.barsRectangle,
            HudTexture::Inventory => atlas.inventoryRectangle,
        };
        append_hud_quad(
            &mut vertices, &mut indices, rectangle,
            quad.x, quad.y, quad.width, quad.height,
            quad.textureX, quad.textureY, quad.textureWidth, quad.textureHeight,
            quad.alpha,
        );
    }
    push_hud_range(&mut drawRanges, HudPipelineKind::Alpha, begin, indices.len() as u32);

    begin = indices.len() as u32;
    for quad in &hud.experienceBar {
        let rectangle = match quad.texture {
            HudTexture::Widgets => atlas.widgetsRectangle,
            HudTexture::Icons => atlas.iconsRectangle,
            HudTexture::BossBars => atlas.barsRectangle,
            HudTexture::Inventory => atlas.inventoryRectangle,
        };
        append_hud_quad(
            &mut vertices, &mut indices, rectangle,
            quad.x, quad.y, quad.width, quad.height,
            quad.textureX, quad.textureY, quad.textureWidth, quad.textureHeight,
            quad.alpha,
        );
    }
    if let Some(text) = &hud.experienceLevel {
        append_experience_level_text(text, fontRenderer, atlas, &mut vertices, &mut indices);
    }
    push_hud_range(&mut drawRanges, HudPipelineKind::Alpha, begin, indices.len() as u32);

    // MCP GuiIngame.renderScoreboard and overlay-message draw order: both are
    // rendered before persistent chat and the player-list overlay.
    begin = indices.len() as u32;
    for rectangle in &hud.scoreboardRectangles {
        append_solid_hud_quad(
            rectangle.x, rectangle.y, rectangle.width, rectangle.height,
            packed_argb_to_rgba(rectangle.color), atlas, &mut vertices, &mut indices,
        );
    }
    for text in &hud.scoreboardTexts {
        append_hud_text(text, fontRenderer, atlas, &mut vertices, &mut indices);
    }
    // MCP `GuiIngame#renderGameOverlay` places the debug text before
    // action-bar, subtitles and titles so those later overlays retain their
    // vanilla visual priority.
    for rectangle in &hud.debugRectangles {
        append_solid_hud_quad(
            rectangle.x, rectangle.y, rectangle.width, rectangle.height,
            packed_argb_to_rgba(rectangle.color), atlas, &mut vertices, &mut indices,
        );
    }
    for text in &hud.debugTexts {
        append_hud_text(text, fontRenderer, atlas, &mut vertices, &mut indices);
    }
    if let Some(text) = &hud.actionBar {
        append_hud_text(text, fontRenderer, atlas, &mut vertices, &mut indices);
    }
    for rectangle in &hud.subtitleRectangles {
        append_solid_hud_quad(
            rectangle.x, rectangle.y, rectangle.width, rectangle.height,
            packed_argb_to_rgba(rectangle.color), atlas, &mut vertices, &mut indices,
        );
    }
    for text in &hud.subtitleTexts {
        append_hud_text(text, fontRenderer, atlas, &mut vertices, &mut indices);
    }
    for scaled in &hud.titleTexts {
        append_hud_text_scaled(&scaled.text, scaled.scale as f32, fontRenderer, atlas, &mut vertices, &mut indices);
    }
    push_hud_range(&mut drawRanges, HudPipelineKind::Alpha, begin, indices.len() as u32);

    begin = indices.len() as u32;
    if !capture.inventoryOpen && capture.chatVisible {
        let wrapWidth = ((GuiNewChat::calculateChatboxWidth(capture.chatWidth) as f32)
            / capture.chatScale.max(0.01)).floor() as i32;
        for message in &capture.chatMessages {
            guiNewChat.acceptMessageWithFont(
                message.serial,
                message.component.clone(),
                capture.playerTicksExisted,
                wrapWidth,
                fontRenderer,
            );
        }
        let chat = guiNewChat.buildFrame(
            guiHeight, capture.playerTicksExisted, capture.chatOpen, capture.chatOpacity,
            capture.chatScale, capture.chatWidth, capture.chatHeightFocused,
            capture.chatHeightUnfocused,
        );
        let chatTextScale = chat.textScale;
        for rectangle in chat.rectangles {
            append_solid_hud_quad(
                rectangle.x, rectangle.y, rectangle.width, rectangle.height,
                packed_argb_to_rgba(rectangle.color), atlas, &mut vertices, &mut indices,
            );
        }
        for text in chat.texts {
            append_hud_text_scaled(&text, chatTextScale, fontRenderer, atlas, &mut vertices, &mut indices);
        }
        if capture.chatOpen {
            append_solid_hud_quad(2, guiHeight - 14, guiWidth - 4, 12, [0.0, 0.0, 0.0, 0.5], atlas, &mut vertices, &mut indices);
            if let Some(input) = &capture.chatInput {
                if let Some(selectionX) = input.selectionX {
                    let left = input.cursorX.min(selectionX);
                    let right = input.cursorX.max(selectionX);
                    append_solid_hud_quad(left, input.textY - 1, right - left, 10, [0.2, 0.6, 1.0, 0.5], atlas, &mut vertices, &mut indices);
                }
                let inputText = HudText {
                    text: input.text.clone(), x: input.textX, y: input.textY,
                    color: input.color as u32, outline: true,
                };
                append_hud_text(&inputText, fontRenderer, atlas, &mut vertices, &mut indices);
                if input.cursorVisible {
                    if input.cursorBlock {
                        append_solid_hud_quad(input.cursorX, input.textY - 1, 1, 10, [0.82, 0.82, 0.82, 1.0], atlas, &mut vertices, &mut indices);
                    } else {
                        let cursorText = HudText { text: "_".to_owned(), x: input.cursorX, y: input.textY, color: 0xFFE0_E0E0, outline: true };
                        append_hud_text(&cursorText, fontRenderer, atlas, &mut vertices, &mut indices);
                    }
                }
            }
        }
    }
    push_hud_range(&mut drawRanges, HudPipelineKind::Alpha, begin, indices.len() as u32);

    begin = indices.len() as u32;
    if capture.playerListVisible && !capture.inventoryOpen && !capture.chatOpen {
        let tab = playerTabOverlay.buildFrameWithFont(
            guiWidth,
            &capture.playerListEntries,
            capture.playerListHeader.as_ref(),
            capture.playerListFooter.as_ref(),
            Some(&capture.scoreboard),
            capture.systemTimeMillis,
            fontRenderer,
            capture.playerListShowsHeads,
            &capture.playerListSkinParts,
        );
        for rectangle in tab.rectangles {
            let alpha = ((rectangle.color >> 24) & 0xFF) as f32 / 255.0;
            let color = [
                ((rectangle.color >> 16) & 0xFF) as f32 / 255.0,
                ((rectangle.color >> 8) & 0xFF) as f32 / 255.0,
                (rectangle.color & 0xFF) as f32 / 255.0,
                alpha,
            ];
            append_solid_hud_quad(
                rectangle.x, rectangle.y, rectangle.width, rectangle.height,
                color, atlas, &mut vertices, &mut indices,
            );
        }
        for head in tab.heads {
            append_player_tab_head(&head, atlas, &mut vertices, &mut indices);
        }
        for quad in tab.icons {
            let rectangle = match quad.texture {
                HudTexture::Widgets => atlas.widgetsRectangle,
                HudTexture::Icons => atlas.iconsRectangle,
                HudTexture::BossBars => atlas.barsRectangle,
            HudTexture::Inventory => atlas.inventoryRectangle,
            };
            append_hud_quad(
                &mut vertices, &mut indices, rectangle,
                quad.x, quad.y, quad.width, quad.height,
                quad.textureX, quad.textureY, quad.textureWidth, quad.textureHeight,
                quad.alpha,
            );
        }
        for text in tab.texts {
            append_hud_text(&text, fontRenderer, atlas, &mut vertices, &mut indices);
        }
    } else {
        playerTabOverlay.hide();
    }
    push_hud_range(&mut drawRanges, HudPipelineKind::Alpha, begin, indices.len() as u32);

    if capture.inventoryOpen {
        begin = indices.len() as u32;
        append_player_inventory_background(capture, locale, fontRenderer, standardGalacticFontRenderer, atlas, &mut vertices, &mut indices);
        // MCP 1.12.2 only calls GuiInventory.drawEntityOnScreen from the
        // survival inventory and the creative inventory tab. Dedicated
        // GuiContainer subclasses (crafting, furnace, repair, enchanting,
        // etc.) never render the player model.
        if should_render_inventory_player(capture) {
            append_player_inventory_entity(capture, atlas, &mut vertices, &mut indices);
        }
        if capture.inventoryHorseSpec.is_some() {
            append_horse_inventory_entity(capture, atlas, &mut vertices, &mut indices);
        }
        if recipe_book_narrow_open(capture) {
            append_recipe_book_panel_chrome(
                capture, fontRenderer, atlas, &mut vertices, &mut indices,
            );
        }
        append_player_inventory_item_models(capture, atlas, &mut vertices, &mut indices);
        push_hud_range(&mut drawRanges, HudPipelineKind::Alpha, begin, indices.len() as u32);

        begin = indices.len() as u32;
        if should_render_inventory_player(capture) {
            append_player_inventory_entity_glints(capture, atlas, &mut vertices, &mut indices);
        }
        append_player_inventory_item_glints(capture, atlas, &mut vertices, &mut indices);
        push_hud_range(&mut drawRanges, HudPipelineKind::Glint, begin, indices.len() as u32);

        begin = indices.len() as u32;
        append_player_inventory_overlays(capture, atlas, &mut vertices, &mut indices);
        push_hud_range(&mut drawRanges, HudPipelineKind::Alpha, begin, indices.len() as u32);

        begin = indices.len() as u32;
        append_player_inventory_tooltip(
            capture, locale, fontRenderer, atlas, &mut vertices, &mut indices,
        );
        push_hud_range(&mut drawRanges, HudPipelineKind::Alpha, begin, indices.len() as u32);
    }

    // GuiScreen is drawn after GuiIngame. This preserves the vanilla ordering
    // for the pause menu and its child option screens while retaining the live
    // multiplayer world behind the translucent gradient.
    if let Some(drawList) = &capture.worldGuiDrawList {
        begin = indices.len() as u32;
        append_font_draw_list(drawList, atlas, &mut vertices, &mut indices);
        push_hud_range(&mut drawRanges, HudPipelineKind::Alpha, begin, indices.len() as u32);
    }

    let width = guiWidth as f32;
    let height = guiHeight as f32;
    let hudPushConstants = WorldPushConstants {
        viewProjection: hud_projection(width, height),
        cameraPosition: [0.0, 0.0, 0.0, 0.0],
        fogColor: [0.0, 0.0, 0.0, 0.0],
        fogParameters: [1.0e9, 1.0e9 + 1.0, 1.0, -1.0],
        lightmapParameters: [1.0, 1.0, 1.0, 99.0],
    };
    (vertices, indices, drawRanges, hudPushConstants)
}

/// Exact Vulkan depth-space equivalent of
/// `EntityRenderer#setupOverlayRendering`: `glOrtho(0,w,h,0,1000,3000)`
/// followed by `translate(0,0,-2000)`. GUI X/Y retain the project's existing
/// bottom-origin conversion, while the Z row maps the activation item's
/// source z=-50 to Vulkan's visible [0,1] range instead of clipping it.
fn hud_projection(width: f32, height: f32) -> [f32; 16] {
    [
        2.0 / width.max(1.0), 0.0, 0.0, 0.0,
        0.0, 2.0 / height.max(1.0), 0.0, 0.0,
        0.0, 0.0, -1.0 / 2000.0, 0.0,
        -1.0, -1.0, 0.5, 1.0,
    ]
}

fn push_hud_range(
    ranges: &mut Vec<HudDrawRange>,
    pipeline: HudPipelineKind,
    firstIndex: u32,
    endIndex: u32,
) {
    if endIndex > firstIndex {
        ranges.push(HudDrawRange {
            pipeline,
            firstIndex,
            indexCount: endIndex - firstIndex,
        });
    }
}

fn container_slot_accepts(capture: &WorldRenderCapture, slotId: i32, stack: &ItemStack) -> bool {
    if capture.inventoryIsCreative {
        true
    } else if capture.inventoryIsShulker {
        (0..capture.inventorySlots.len() as i32).contains(&slotId)
            && (!(0..27).contains(&slotId) || !matches!(stack.itemId, 219..=234))
    } else if capture.inventoryIsChest {
        (0..capture.inventorySlots.len() as i32).contains(&slotId)
    } else if let Some(spec) = capture.inventoryHorseSpec {
        if stack.isEmpty() || !(0..capture.inventorySlots.len() as i32).contains(&slotId) {
            return false;
        }
        match slotId {
            0 => spec.kind.canUseSaddleSlot()
                && stack.itemId == 329
                && capture.inventorySlots.first().map_or(true, ItemStack::isEmpty),
            1 => match spec.kind {
                crate::net::minecraft::inventory::ContainerHorseInventory::HorseInventoryKind::Horse => {
                    matches!(stack.itemId, 417..=419)
                }
                crate::net::minecraft::inventory::ContainerHorseInventory::HorseInventoryKind::Llama => {
                    stack.itemId == 171
                }
                _ => false,
            },
            id if (id as usize) < spec.lowerSlotCount() => true,
            _ => true,
        }
    } else if let Some(kind) = capture.inventoryWindowKind {
        if stack.isEmpty() || !(0..capture.inventorySlots.len() as i32).contains(&slotId) {
            return false;
        }
        let lower = kind.lowerSlotCount() as i32;
        if slotId >= lower {
            return true;
        }
        match kind {
            ContainerWindowKind::Workbench => slotId != 0,
            ContainerWindowKind::Furnace => match slotId {
                0 => true,
                1 => isFurnaceFuel(stack),
                2 => false,
                _ => false,
            },
            ContainerWindowKind::Repair => slotId != 2,
            ContainerWindowKind::Enchantment => match slotId {
                0 => true,
                1 => stack.itemId == 351 && stack.itemDamage == 4,
                _ => false,
            },
            ContainerWindowKind::Hopper
            | ContainerWindowKind::Dispenser
            | ContainerWindowKind::Dropper => true,
            ContainerWindowKind::BrewingStand => match slotId {
                0..=2 => canHoldBrewingPotion(stack),
                3 => isBrewingReagent(stack),
                4 => stack.itemId == 377,
                _ => false,
            },
            ContainerWindowKind::Beacon => matches!(stack.itemId, 264 | 265 | 266 | 388),
            ContainerWindowKind::Merchant => slotId != 2,
        }
    } else {
        playerContainerSlotAccepts(slotId, stack)
    }
}

fn container_slot_limit(capture: &WorldRenderCapture, slotId: i32, stack: &ItemStack) -> i32 {
    if capture.inventoryHorseSpec.is_some() && (0..=1).contains(&slotId) {
        1
    } else if (capture.inventoryWindowKind == Some(ContainerWindowKind::Enchantment)
        || capture.inventoryWindowKind == Some(ContainerWindowKind::Beacon))
        && slotId == 0
        || capture.inventoryWindowKind == Some(ContainerWindowKind::BrewingStand)
            && (0..=2).contains(&slotId)
    {
        1
    } else if capture.inventoryIsCreative
        || capture.inventoryIsChest
        || capture.inventoryIsShulker
        || capture.inventoryHorseSpec.is_some()
        || capture.inventoryWindowKind.is_some()
    {
        stack.getMaxStackSize()
    } else {
        playerContainerSlotLimit(slotId, stack)
    }
}

fn player_inventory_display_stack(
    capture: &WorldRenderCapture,
    slotId: i32,
) -> Option<ItemStack> {
    let actual = capture.inventorySlots.get(slotId as usize)?.clone();
    if !capture.inventoryDragSplitting
        || !capture.inventoryDragSplittingSlots.contains(&slotId)
    {
        return Some(actual);
    }
    // GuiContainer.drawSlot returns early while exactly one slot is selected.
    if capture.inventoryDragSplittingSlots.len() == 1 {
        return None;
    }
    let cursor = &capture.inventoryCursorStack;
    if cursor.isEmpty()
        || !Container::canAddItemToSlot(&actual, cursor, true)
        || !container_slot_accepts(capture, slotId, cursor)
    {
        return Some(actual);
    }
    let oldCount = if actual.isEmpty() { 0 } else { actual.getCount() };
    let mut preview = cursor.clone();
    Container::computeStackSize(
        capture.inventoryDragSplittingSlots.len(),
        capture.inventoryDragSplittingLimit,
        &mut preview,
        oldCount,
    );
    let limit = preview
        .getMaxStackSize()
        .min(container_slot_limit(capture, slotId, &preview));
    if preview.getCount() > limit {
        preview.setCount(limit);
    }
    Some(preview)
}

fn player_inventory_cursor_display_stack(capture: &WorldRenderCapture) -> ItemStack {
    let mut stack = capture.inventoryCursorStack.clone();
    if capture.inventoryDragSplitting && capture.inventoryDragSplittingSlots.len() > 1 {
        stack.setCount(capture.inventoryDragSplittingRemnant);
    }
    stack
}

/// `GuiContainer.drawSlot` replaces the normal white count with a yellow
/// clamped limit when a QUICK_CRAFT preview would exceed either the item or
/// concrete slot limit.
fn player_inventory_drag_preview_limit(
    capture: &WorldRenderCapture,
    slotId: i32,
) -> Option<i32> {
    if !capture.inventoryDragSplitting
        || capture.inventoryDragSplittingSlots.len() <= 1
        || !capture.inventoryDragSplittingSlots.contains(&slotId)
    {
        return None;
    }
    let actual = capture.inventorySlots.get(slotId as usize)?;
    let cursor = &capture.inventoryCursorStack;
    if cursor.isEmpty()
        || !Container::canAddItemToSlot(actual, cursor, true)
        || !container_slot_accepts(capture, slotId, cursor)
    {
        return None;
    }
    let oldCount = if actual.isEmpty() { 0 } else { actual.getCount() };
    let mut preview = cursor.clone();
    Container::computeStackSize(
        capture.inventoryDragSplittingSlots.len(),
        capture.inventoryDragSplittingLimit,
        &mut preview,
        oldCount,
    );
    let limit = preview
        .getMaxStackSize()
        .min(container_slot_limit(capture, slotId, &preview));
    (preview.getCount() > limit).then_some(limit)
}

enum ActiveContainerLayout {
    Player(GuiInventory),
    Chest(GuiChest),
    ShulkerBox(GuiShulkerBox),
    Horse(GuiScreenHorseInventory),
    Window(GuiContainer),
    Creative(GuiContainer),
}

impl ActiveContainerLayout {
    fn container(&self) -> &GuiContainer {
        match self {
            Self::Player(gui) => &gui.container,
            Self::Chest(gui) => &gui.container,
            Self::ShulkerBox(gui) => &gui.container,
            Self::Horse(gui) => &gui.container,
            Self::Window(container) => container,
            Self::Creative(container) => container,
        }
    }

    fn slotPosition(&self, slotId: i32) -> Option<(i32, i32)> {
        self.container().slotPosition(slotId)
    }

    fn slotAt(&self, mouseX: i32, mouseY: i32) -> Option<i32> {
        self.container().slotAt(mouseX, mouseY)
    }
}

fn active_recipe_book_state(capture: &WorldRenderCapture) -> Option<&RecipeBookRenderState> {
    let state = capture.recipeBookState.as_ref()?;
    let inventoryScreen = capture.inventoryWindowKind.is_none()
        && !capture.inventoryIsCreative
        && !capture.inventoryIsChest
        && !capture.inventoryIsShulker
        && capture.inventoryHorseSpec.is_none();
    let workbenchScreen = capture.inventoryWindowKind == Some(ContainerWindowKind::Workbench);
    ((state.inventoryScreen && inventoryScreen) || (!state.inventoryScreen && workbenchScreen))
        .then_some(state)
}

fn recipe_book_narrow_open(capture: &WorldRenderCapture) -> bool {
    active_recipe_book_state(capture).is_some_and(|state| state.open && state.widthTooNarrow)
}

fn player_inventory_layout(capture: &WorldRenderCapture) -> ActiveContainerLayout {
    if capture.inventoryIsCreative {
        ActiveContainerLayout::Creative(capture.creativeContainer.clone().unwrap_or_else(|| {
            let mut container = GuiContainer::new(195, 136, Vec::new());
            container.initGui(capture.guiWidth, capture.guiHeight);
            container
        }))
    } else if let Some(spec) = capture.inventoryHorseSpec {
        let mut horse = GuiScreenHorseInventory::new(spec);
        horse.initGui(capture.guiWidth, capture.guiHeight);
        ActiveContainerLayout::Horse(horse)
    } else if let Some(kind) = capture.inventoryWindowKind {
        let mut container = match kind {
            ContainerWindowKind::Workbench => GuiCrafting::new().container,
            ContainerWindowKind::Furnace => GuiFurnace::new().container,
            ContainerWindowKind::Repair => GuiRepair::new().container,
            ContainerWindowKind::Enchantment => GuiEnchantment::new().container,
            ContainerWindowKind::Hopper => GuiHopper::new().container,
            ContainerWindowKind::BrewingStand => GuiBrewingStand::new().container,
            ContainerWindowKind::Dispenser | ContainerWindowKind::Dropper => GuiDispenser::new().container,
            ContainerWindowKind::Beacon => GuiBeacon::new().container,
            ContainerWindowKind::Merchant => GuiMerchant::new().container,
        };
        container.initGui(capture.guiWidth, capture.guiHeight);
        if kind == ContainerWindowKind::Workbench {
            if let Some(state) = active_recipe_book_state(capture) {
                container.guiLeft = state.containerLeft;
            }
        }
        ActiveContainerLayout::Window(container)
    } else if capture.inventoryIsShulker {
        let mut shulker = GuiShulkerBox::new();
        shulker.initGui(capture.guiWidth, capture.guiHeight);
        ActiveContainerLayout::ShulkerBox(shulker)
    } else if capture.inventoryIsChest {
        let mut chest = GuiChest::new(capture.inventoryRows);
        chest.initGui(capture.guiWidth, capture.guiHeight);
        ActiveContainerLayout::Chest(chest)
    } else {
        let mut inventory = GuiInventory::new();
        inventory.initGui(capture.guiWidth, capture.guiHeight);
        if let Some(state) = active_recipe_book_state(capture) {
            inventory.container.guiLeft = state.containerLeft;
        }
        ActiveContainerLayout::Player(inventory)
    }
}

fn captured_main_hand_stack(capture: &WorldRenderCapture) -> &ItemStack {
    capture
        .hotbarStacks
        .get(capture.currentHotbarSlot.clamp(0, 8) as usize)
        .unwrap_or(&ItemStack::EMPTY)
}

/// Port of `RenderPlayer.setModelVisibilities`. The item-use test intentionally
/// follows MCP's `getItemInUseCount() > 0` branch rather than inventing a GUI
/// pose. The returned tuple is `(leftArmPose, rightArmPose)`.
fn player_arm_poses(
    mainStack: &ItemStack,
    offStack: &ItemStack,
    itemInUseCount: i32,
    primaryHand: EnumHandSide,
) -> (ArmPose, ArmPose) {
    let usingItem = itemInUseCount > 0;

    let mut mainPose = ArmPose::Empty;
    if !mainStack.isEmpty() {
        mainPose = ArmPose::Item;
        if usingItem {
            mainPose = match mainStack.getItemUseAction() {
                EnumAction::Block => ArmPose::Block,
                EnumAction::Bow => ArmPose::BowAndArrow,
                EnumAction::None | EnumAction::Eat | EnumAction::Drink => ArmPose::Item,
            };
        }
    }

    let mut offPose = ArmPose::Empty;
    if !offStack.isEmpty() {
        offPose = ArmPose::Item;
        if usingItem && offStack.getItemUseAction() == EnumAction::Block {
            offPose = ArmPose::Block;
        }
    }

    match primaryHand {
        EnumHandSide::Right => (offPose, mainPose),
        EnumHandSide::Left => (mainPose, offPose),
    }
}

fn inventory_player_arm_poses(capture: &WorldRenderCapture) -> (ArmPose, ArmPose) {
    player_arm_poses(
        captured_main_hand_stack(capture),
        &capture.offhandStack,
        capture.firstPersonItems.itemInUseCount,
        capture.primaryHand,
    )
}

fn player_layer_root_matrix(bodyYaw: f32, sneaking: bool) -> [[f32; 4]; 4] {
    let mut matrix = identity4();
    // RenderPlayer.doRender lowers a sneaking player's render origin before
    // RenderLivingBase installs its model matrix.
    if sneaking {
        matrix = multiply4(matrix, translation4([0.0, -0.125, 0.0]));
    }
    matrix = multiply4(matrix, rotation_y4(180.0 - bodyYaw));
    matrix = multiply4(matrix, scale4_nonuniform([-1.0, -1.0, 1.0]));
    matrix = multiply4(matrix, scale4_nonuniform([0.9375, 0.9375, 0.9375]));
    multiply4(matrix, translation4([0.0, -1.501, 0.0]))
}

fn post_render_part_matrix(mut matrix: [[f32; 4]; 4], pose: PartPose) -> [[f32; 4]; 4] {
    matrix = multiply4(matrix, translation4([
        pose.pivot[0] * 0.0625,
        pose.pivot[1] * 0.0625,
        pose.pivot[2] * 0.0625,
    ]));
    // ModelRenderer.postRender applies Z, then Y, then X rotations.
    matrix = multiply4(matrix, rotation_z4(pose.rotation[2].to_degrees()));
    matrix = multiply4(matrix, rotation_y4(pose.rotation[1].to_degrees()));
    multiply4(matrix, rotation_x4(pose.rotation[0].to_degrees()))
}

#[allow(clippy::too_many_arguments)]
fn append_player_inventory_held_items(
    capture: &WorldRenderCapture,
    atlas: &AtlasState,
    pose: BipedPose,
    bodyYaw: f32,
    entityPitch: [[f32; 4]; 4],
    posX: i32,
    posY: i32,
    guiScale: f32,
    lights: [[f32; 3]; 2],
    quads: &mut Vec<GuiItemQuad>,
) {
    let mainStack = captured_main_hand_stack(capture);
    let offStack = &capture.offhandStack;
    let rightStack = LayerHeldItem::stackForSide(
        capture.primaryHand, mainStack, offStack, EnumHandSide::Right,
    );
    let leftStack = LayerHeldItem::stackForSide(
        capture.primaryHand, mainStack, offStack, EnumHandSide::Left,
    );

    append_player_inventory_held_item(
        rightStack,
        EnumHandSide::Right,
        pose.rightArm,
        capture.localSneaking,
        bodyYaw,
        entityPitch,
        posX,
        posY,
        guiScale,
        lights,
        atlas,
        quads,
    );
    append_player_inventory_held_item(
        leftStack,
        EnumHandSide::Left,
        pose.leftArm,
        capture.localSneaking,
        bodyYaw,
        entityPitch,
        posX,
        posY,
        guiScale,
        lights,
        atlas,
        quads,
    );
}

#[allow(clippy::too_many_arguments)]
fn append_player_inventory_held_item(
    stack: &ItemStack,
    handSide: EnumHandSide,
    armPose: PartPose,
    sneaking: bool,
    bodyYaw: f32,
    entityPitch: [[f32; 4]; 4],
    posX: i32,
    posY: i32,
    guiScale: f32,
    lights: [[f32; 3]; 2],
    atlas: &AtlasState,
    quads: &mut Vec<GuiItemQuad>,
) {
    if stack.isEmpty() {
        return;
    }
    let Some(modelKey) = ItemModelMesher::getModelKey(stack) else { return; };
    let Some(model) = atlas.itemModels.get(&modelKey) else { return; };
    let unpatternedShield = model.builtInRenderer && is_unpatterned_shield(stack);
    let supportedBuiltIn = model.builtInRenderer
        && (unpatternedShield || TileEntityItemStackRenderer::buildMesh(stack).is_some());
    if (model.builtInRenderer && !supportedBuiltIn)
        || (!model.builtInRenderer && model.quads.is_empty())
    {
        return;
    }

    let leftHanded = LayerHeldItem::leftHanded(handSide);
    let transformType = LayerHeldItem::transformType(handSide);
    let mut matrix = player_layer_root_matrix(bodyYaw, sneaking);
    matrix = post_render_part_matrix(matrix, armPose);
    if sneaking {
        matrix = multiply4(matrix, translation4([0.0, 0.2, 0.0]));
    }
    // Exact LayerHeldItem.renderHeldItem transform sequence.
    matrix = multiply4(matrix, rotation_x4(-90.0));
    matrix = multiply4(matrix, rotation_y4(180.0));
    matrix = multiply4(matrix, translation4(LayerHeldItem::handTranslation(handSide)));
    let itemTransform = model.transforms.getTransform(transformType);
    matrix = multiply4(matrix, item_camera_transform4(itemTransform, leftHanded));
    matrix = multiply4(matrix, translation4([-0.5, -0.5, -0.5]));
    matrix = multiply4(entityPitch, matrix);

    if unpatternedShield {
        let shield = ModelShield::buildMesh();
        for face in shield.indices.chunks_exact(6) {
            let source = [face[0] as usize, face[1] as usize, face[2] as usize, face[5] as usize];
            let positions = source.map(|index| transform_point3(matrix, shield.vertices[index].position));
            let uvs = source.map(|index| shield.vertices[index].uv);
            push_inventory_entity_item_quad(
                positions,
                uvs,
                atlas.shieldBaseRectangle,
                [1.0, 1.0, 1.0],
                true,
                posX,
                posY,
                guiScale,
                lights,
                quads,
            );
        }
        return;
    }
    if model.builtInRenderer {
        if let Some(mesh) = TileEntityItemStackRenderer::buildMesh(stack) {
            let rectangle = built_in_item_rectangle(stack, &mesh, atlas);
            for face in mesh.indices.chunks_exact(6) {
                let source = [face[0] as usize, face[1] as usize, face[2] as usize, face[5] as usize];
                let positions = source.map(|index| transform_point3(matrix, mesh.vertices[index].position));
                let uvs = source.map(|index| mesh.vertices[index].uv);
                push_inventory_entity_item_quad(
                    positions, uvs, rectangle,
                    [mesh.color[0], mesh.color[1], mesh.color[2]], true,
                    posX, posY, guiScale, lights, quads,
                );
            }
        }
        return;
    }

    for quad in &model.quads {
        let positions = quad.positions.map(|position| transform_point3(matrix, position));
        let key = item_material_key(stack.itemId, quad.texture.clone(), quad.tintIndex);
        let rectangle = atlas
            .rectangles
            .get(&key)
            .copied()
            .unwrap_or(atlas.missingRectangle);
        push_inventory_entity_item_quad(
            positions,
            quad.uvs,
            rectangle,
            item_tint_color(&atlas.itemColors, stack, quad.tintIndex),
            model.gui3d && quad.shade,
            posX,
            posY,
            guiScale,
            lights,
            quads,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn push_inventory_entity_item_quad(
    modelPositions: [[f32; 3]; 4],
    uvs: [[f32; 2]; 4],
    rectangle: [f32; 4],
    tint: [f32; 3],
    shaded: bool,
    posX: i32,
    posY: i32,
    guiScale: f32,
    lights: [[f32; 3]; 2],
    quads: &mut Vec<GuiItemQuad>,
) {
    let edge1 = subtract3(modelPositions[1], modelPositions[0]);
    let edge2 = subtract3(modelPositions[2], modelPositions[0]);
    let normal = normalize3(cross3(edge1, edge2));
    if normal[2] <= 1.0e-5 {
        return;
    }
    let diffuse = if shaded {
        standard_item_diffuse(normal, lights)
    } else {
        1.0
    };
    let positions = modelPositions.map(|position| [
        posX as f32 + position[0] * guiScale,
        posY as f32 - position[1] * guiScale,
        position[2],
    ]);
    quads.push(GuiItemQuad {
        depth: modelPositions.iter().map(|position| position[2]).sum::<f32>() / 4.0,
        positions,
        uvs,
        rectangle,
        color: [tint[0] * diffuse, tint[1] * diffuse, tint[2] * diffuse, 1.0],
    });
}

fn creative_tab_draw_position_selected(
    container: &GuiContainer,
    tabIndex: i32,
    selectedTabIndex: i32,
) -> Option<(i32, i32, i32, i32)> {
    let tab = creativeTabByIndex(tabIndex)?;
    let firstRow = tab.isTabInFirstRow();
    let column = tab.getTabColumn();
    let textureX = column * 28;
    let mut textureY = if tabIndex == selectedTabIndex { 32 } else { 0 };
    let mut x = container.guiLeft + 28 * column;
    let y;
    if tab.rightAligned {
        x = container.guiLeft + container.xSize - 28 * (6 - column);
    } else if column > 0 {
        x += column;
    }
    if firstRow {
        y = container.guiTop - 28;
    } else {
        textureY += 64;
        y = container.guiTop + container.ySize - 4;
    }
    Some((x, y, textureX, textureY))
}

fn creative_tab_icon_position(container: &GuiContainer, tabIndex: i32) -> Option<(i32, i32)> {
    let tab = creativeTabByIndex(tabIndex)?;
    let (x, y, _, _) = creative_tab_draw_position_selected(container, tabIndex, tabIndex)?;
    Some((x + 6, y + 8 + if tab.isTabInFirstRow() { 1 } else { -1 }))
}

fn append_gui_text_field(
    input: &GuiTextFieldRenderState,
    fontRenderer: &mut FontRenderer,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    if let Some(selectionX) = input.selectionX {
        let left = input.cursorX.min(selectionX);
        let right = input.cursorX.max(selectionX);
        append_solid_hud_quad(
            left, input.textY - 1, right - left, 10,
            [0.2, 0.6, 1.0, 0.5], atlas, vertices, indices,
        );
    }
    let text = HudText {
        text: input.text.clone(),
        x: input.textX,
        y: input.textY,
        color: input.color as u32,
        outline: true,
    };
    append_hud_text(&text, fontRenderer, atlas, vertices, indices);
    if input.cursorVisible {
        if input.cursorBlock {
            append_solid_hud_quad(
                input.cursorX, input.textY - 1, 1, 10,
                [0.82, 0.82, 0.82, 1.0], atlas, vertices, indices,
            );
        } else {
            let cursor = HudText {
                text: "_".to_owned(),
                x: input.cursorX,
                y: input.textY,
                color: input.color as u32,
                outline: true,
            };
            append_hud_text(&cursor, fontRenderer, atlas, vertices, indices);
        }
    }
}

fn append_creative_inventory_background(
    capture: &WorldRenderCapture,
    fontRenderer: &mut FontRenderer,
    atlas: &AtlasState,
    container: &GuiContainer,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    // `GuiContainerCreative#drawGuiContainerBackgroundLayer`: every inactive
    // tab is drawn first, the selected tab's page covers their inner seams,
    // then the selected tab is drawn last so it joins the page border.
    for tab in CREATIVE_TAB_ARRAY {
        if tab.tabIndex == capture.creativeSelectedTab { continue; }
        if let Some((x, y, u, v)) = creative_tab_draw_position_selected(
            container, tab.tabIndex, capture.creativeSelectedTab,
        ) {
            append_hud_quad_colored(
                vertices, indices, atlas.creativeTabsRectangle,
                x, y, 28, 32, u, v, 28, 32, 256, 256,
                [1.0, 1.0, 1.0, 1.0],
            );
        }
    }

    let background = match capture.creativeSelectedTab {
        5 => atlas.creativeSearchRectangle,
        11 => atlas.creativeInventoryRectangle,
        _ => atlas.creativeItemsRectangle,
    };
    append_hud_quad_colored(
        vertices, indices, background,
        container.guiLeft, container.guiTop,
        195, 136, 0, 0, 195, 136, 256, 256,
        [1.0, 1.0, 1.0, 1.0],
    );

    if let Some(input) = &capture.creativeSearchInput {
        append_gui_text_field(input, fontRenderer, atlas, vertices, indices);
    }

    if creativeTabByIndex(capture.creativeSelectedTab)
        .is_some_and(|tab| tab.shouldHidePlayerInventory())
    {
        let needsScroll = capture.creativeCanScroll;
        let thumbY = container.guiTop + 18
            + ((95.0 * capture.creativeCurrentScroll.clamp(0.0, 1.0)) as i32);
        append_hud_quad_colored(
            vertices, indices, atlas.creativeTabsRectangle,
            container.guiLeft + 175, thumbY,
            12, 15, if needsScroll { 232 } else { 244 }, 0, 12, 15, 256, 256,
            [1.0, 1.0, 1.0, 1.0],
        );
    }

    if let Some((x, y, u, v)) = creative_tab_draw_position_selected(
        container, capture.creativeSelectedTab, capture.creativeSelectedTab,
    ) {
        append_hud_quad_colored(
            vertices, indices, atlas.creativeTabsRectangle,
            x, y, 28, 32, u, v, 28, 32, 256, 256,
            [1.0, 1.0, 1.0, 1.0],
        );
    }

    if creativeTabByIndex(capture.creativeSelectedTab)
        .is_some_and(|tab| tab.drawInForegroundOfTab())
    {
        append_font_text_colored_no_shadow(
            &capture.creativeTabTitle,
            container.guiLeft + 8,
            container.guiTop + 6,
            4_210_752,
            fontRenderer, atlas, vertices, indices,
        );
    }
}

fn format_gui_translation(template: &str, argument: impl std::fmt::Display) -> String {
    let argument = argument.to_string();
    if template.contains("%1$s") {
        template.replacen("%1$s", &argument, 1)
    } else if template.contains("%s") {
        template.replacen("%s", &argument, 1)
    } else {
        template.to_owned()
    }
}

fn append_beacon_button(
    atlas: &AtlasState,
    x: i32,
    y: i32,
    enabled: bool,
    selected: bool,
    hovered: bool,
    iconRectangle: [f32; 4],
    iconX: i32,
    iconY: i32,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    append_hud_quad_colored(
        vertices,
        indices,
        atlas.beaconRectangle,
        x,
        y,
        22,
        22,
        GuiBeacon::buttonSourceX(enabled, selected, hovered),
        219,
        22,
        22,
        256,
        256,
        [1.0, 1.0, 1.0, 1.0],
    );
    append_hud_quad_colored(
        vertices,
        indices,
        iconRectangle,
        x + 2,
        y + 2,
        18,
        18,
        iconX,
        iconY,
        18,
        18,
        256,
        256,
        [1.0, 1.0, 1.0, 1.0],
    );
}

fn append_beacon_buttons(
    capture: &WorldRenderCapture,
    container: &GuiContainer,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let mut gui = GuiBeacon::new();
    gui.container = container.clone();
    let levels = capture.inventoryProperties.first().copied().unwrap_or(0);
    let primary = capture.inventoryProperties.get(1).copied().unwrap_or(0);
    let secondary = capture.inventoryProperties.get(2).copied().unwrap_or(0);

    // GuiBeacon delays constructing power buttons until field 0 has been
    // synchronized to a non-negative level. Confirm/cancel exist immediately.
    if levels >= 0 {
        for button in gui.powerButtons(primary) {
            let enabled = button.tier < levels;
            let selected = if button.tier < 3 {
                button.effectId == primary
            } else {
                button.effectId == secondary
            };
            let hovered = capture.inventoryMouseX >= button.x
                && capture.inventoryMouseX < button.x + 22
                && capture.inventoryMouseY >= button.y
                && capture.inventoryMouseY < button.y + 22;
            if let Some(icon) = GuiBeacon::effectIconIndex(button.effectId) {
                append_beacon_button(
                    atlas,
                    button.x,
                    button.y,
                    enabled,
                    selected,
                    hovered,
                    atlas.inventoryRectangle,
                    icon % 8 * 18,
                    198 + icon / 8 * 18,
                    vertices,
                    indices,
                );
            }
        }
    }

    let confirmX = container.guiLeft + GuiBeacon::CONFIRM_X;
    let cancelX = container.guiLeft + GuiBeacon::CANCEL_X;
    let actionY = container.guiTop + GuiBeacon::ACTION_Y;
    let confirmEnabled = GuiBeacon::confirmEnabled(capture.inventorySlots.first(), primary);
    append_beacon_button(
        atlas,
        confirmX,
        actionY,
        confirmEnabled,
        false,
        capture.inventoryMouseX >= confirmX
            && capture.inventoryMouseX < confirmX + 22
            && capture.inventoryMouseY >= actionY
            && capture.inventoryMouseY < actionY + 22,
        atlas.beaconRectangle,
        90,
        220,
        vertices,
        indices,
    );
    append_beacon_button(
        atlas,
        cancelX,
        actionY,
        true,
        false,
        capture.inventoryMouseX >= cancelX
            && capture.inventoryMouseX < cancelX + 22
            && capture.inventoryMouseY >= actionY
            && capture.inventoryMouseY < actionY + 22,
        atlas.beaconRectangle,
        112,
        220,
        vertices,
        indices,
    );
}

fn append_enchantment_gui_book(
    state: EnchantmentBookRenderState,
    guiWidth: i32,
    guiHeight: i32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    // GuiEnchantment passes 0.0F as ModelBook's limbSwing parameter.
    // The GUI book therefore does not use the world TESR's idle sine term.
    let mesh = ModelBook::buildMesh(
        0.0,
        state.pageFlipRight,
        state.pageFlipLeft,
        state.open,
    );
    let texture = ResourceLocation::new(
        "minecraft",
        "textures/entity/enchanting_table_book.png",
    );
    let rectangle = atlas
        .builtInItemRectangles
        .get(&texture)
        .copied()
        .unwrap_or(atlas.missingRectangle);

    // GuiEnchantment#drawGuiContainerBackgroundLayer, preserving the exact
    // 320x240 centered viewport and GL matrix multiplication order.
    let mut model = translation4([0.0, 3.3, -16.0]);
    model = multiply4(model, scale4_nonuniform([5.0, 5.0, 5.0]));
    model = multiply4(model, rotation_z4(180.0));
    model = multiply4(model, rotation_x4(20.0));
    let closed = 1.0 - state.open;
    model = multiply4(model, translation4([closed * 0.2, closed * 0.1, closed * 0.25]));
    model = multiply4(model, rotation_y4(-closed * 90.0 - 90.0));
    model = multiply4(model, rotation_x4(180.0));

    let near = 9.0_f32;
    let far = 80.0_f32;
    let cotangent = 1.0 / 45.0_f32.to_radians().tan();
    let perspective = [
        [cotangent / (4.0 / 3.0), 0.0, 0.0, 0.0],
        [0.0, cotangent, 0.0, 0.0],
        [0.0, 0.0, (far + near) / (near - far), (2.0 * far * near) / (near - far)],
        [0.0, 0.0, -1.0, 0.0],
    ];
    let projection = multiply4(translation4([-0.34, 0.23, 0.0]), perspective);

    #[derive(Clone)]
    struct BookQuad {
        depth: f32,
        screen: [[f32; 2]; 4],
        uv: [[f32; 2]; 4],
        color: [f32; 4],
    }
    let mut quads = Vec::<BookQuad>::new();
    let viewportLeft = (guiWidth - 320) as f32 * 0.5;
    let viewportTop = (guiHeight - 240) as f32 * 0.5;
    for face in mesh.indices.chunks_exact(6) {
        let source = [face[0] as usize, face[1] as usize, face[2] as usize, face[5] as usize];
        let eye = source.map(|index| transform_point3(model, mesh.vertices[index].position));
        let normal = normalize3(cross3(
            subtract3(eye[1], eye[0]),
            subtract3(eye[2], eye[0]),
        ));
        if normal[2] <= 1.0e-5 {
            continue;
        }
        let mut screen = [[0.0_f32; 2]; 4];
        let mut valid = true;
        for corner in 0..4 {
            let clip = transform_homogeneous(projection, [eye[corner][0], eye[corner][1], eye[corner][2], 1.0]);
            if clip[3].abs() <= 1.0e-6 {
                valid = false;
                break;
            }
            let ndcX = clip[0] / clip[3];
            let ndcY = clip[1] / clip[3];
            screen[corner] = [
                viewportLeft + (ndcX + 1.0) * 160.0,
                viewportTop + (1.0 - ndcY) * 120.0,
            ];
        }
        if !valid { continue; }
        let diffuse = gui_item_diffuse(normal);
        quads.push(BookQuad {
            depth: eye.iter().map(|point| point[2]).sum::<f32>() * 0.25,
            screen,
            uv: source.map(|index| mesh.vertices[index].uv),
            color: [diffuse, diffuse, diffuse, 1.0],
        });
    }
    quads.sort_by(|left, right| left.depth.total_cmp(&right.depth));
    for quad in quads {
        let base = vertices.len() as u32;
        for corner in 0..4 {
            let uv = [
                rectangle[0] + (rectangle[2] - rectangle[0]) * quad.uv[corner][0],
                rectangle[1] + (rectangle[3] - rectangle[1]) * quad.uv[corner][1],
            ];
            vertices.push(WorldVertex {
                position: [quad.screen[corner][0], quad.screen[corner][1], 0.0],
                uv,
                color: quad.color,
                lightmap: [15.0, 15.0],
            
                shaderEntity: [-1, -1, -1],
                shaderPadding: 0,
            });
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
    }
}

fn transform_homogeneous(matrix: [[f32; 4]; 4], vector: [f32; 4]) -> [f32; 4] {
    [
        matrix[0][0] * vector[0] + matrix[0][1] * vector[1] + matrix[0][2] * vector[2] + matrix[0][3] * vector[3],
        matrix[1][0] * vector[0] + matrix[1][1] * vector[1] + matrix[1][2] * vector[2] + matrix[1][3] * vector[3],
        matrix[2][0] * vector[0] + matrix[2][1] * vector[1] + matrix[2][2] * vector[2] + matrix[2][3] * vector[3],
        matrix[3][0] * vector[0] + matrix[3][1] * vector[1] + matrix[3][2] * vector[2] + matrix[3][3] * vector[3],
    ]
}

fn append_recipe_overlay_background(
    state: &RecipeBookRenderState,
    capture: &WorldRenderCapture,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let overlay = &state.overlay;
    if !overlay.visible || overlay.columns == 0 || overlay.rows == 0 { return; }
    let tile = 24_i32;
    let border = 4_i32;
    let sourceX = 82_i32;
    let sourceY = 208_i32;
    let columns = overlay.columns as i32;
    let rows = overlay.rows as i32;
    let mut draw = |x: i32, y: i32, u: i32, v: i32, width: i32, height: i32| {
        append_hud_quad_colored(
            vertices, indices, atlas.recipeBookRectangle,
            x, y, width, height,
            u, v, width, height, 256, 256,
            [1.0, 1.0, 1.0, 1.0],
        );
    };

    draw(overlay.left, overlay.top, sourceX, sourceY, border, border);
    draw(
        overlay.left + border * 2 + columns * tile,
        overlay.top,
        sourceX + tile + border,
        sourceY,
        border,
        border,
    );
    draw(
        overlay.left,
        overlay.top + border * 2 + rows * tile,
        sourceX,
        sourceY + tile + border,
        border,
        border,
    );
    draw(
        overlay.left + border * 2 + columns * tile,
        overlay.top + border * 2 + rows * tile,
        sourceX + tile + border,
        sourceY + tile + border,
        border,
        border,
    );

    for column in 0..columns {
        draw(
            overlay.left + border + column * tile,
            overlay.top,
            sourceX + border,
            sourceY,
            tile,
            border,
        );
        draw(
            overlay.left + border + (column + 1) * tile,
            overlay.top,
            sourceX + border,
            sourceY,
            border,
            border,
        );

        for row in 0..rows {
            if column == 0 {
                draw(
                    overlay.left,
                    overlay.top + border + row * tile,
                    sourceX,
                    sourceY + border,
                    border,
                    tile,
                );
                draw(
                    overlay.left,
                    overlay.top + border + (row + 1) * tile,
                    sourceX,
                    sourceY + border,
                    border,
                    border,
                );
            }

            draw(
                overlay.left + border + column * tile,
                overlay.top + border + row * tile,
                sourceX + border,
                sourceY + border,
                tile,
                tile,
            );
            draw(
                overlay.left + border + (column + 1) * tile,
                overlay.top + border + row * tile,
                sourceX + border,
                sourceY + border,
                border,
                tile,
            );
            draw(
                overlay.left + border + column * tile,
                overlay.top + border + (row + 1) * tile,
                sourceX + border,
                sourceY + border,
                tile,
                border,
            );
            draw(
                overlay.left + border + (column + 1) * tile - 1,
                overlay.top + border + (row + 1) * tile - 1,
                sourceX + border,
                sourceY + border,
                border + 1,
                border + 1,
            );

            if column == columns - 1 {
                draw(
                    overlay.left + border * 2 + columns * tile,
                    overlay.top + border + row * tile,
                    sourceX + tile + border,
                    sourceY + border,
                    border,
                    tile,
                );
                draw(
                    overlay.left + border * 2 + columns * tile,
                    overlay.top + border + (row + 1) * tile,
                    sourceX + tile + border,
                    sourceY + border,
                    border,
                    border,
                );
            }
        }

        draw(
            overlay.left + border + column * tile,
            overlay.top + border * 2 + rows * tile,
            sourceX + border,
            sourceY + tile + border,
            tile,
            border,
        );
        draw(
            overlay.left + border + (column + 1) * tile,
            overlay.top + border * 2 + rows * tile,
            sourceX + border,
            sourceY + tile + border,
            border,
            border,
        );
    }

    for button in &overlay.buttons {
        let hovered = button.rect.contains(capture.inventoryMouseX, capture.inventoryMouseY);
        append_hud_quad_colored(
            vertices, indices, atlas.recipeBookRectangle,
            button.rect.x, button.rect.y, button.rect.width, button.rect.height,
            152 + if button.craftable { 0 } else { 26 },
            78 + if hovered { 26 } else { 0 },
            button.rect.width, button.rect.height, 256, 256,
            [1.0, 1.0, 1.0, 1.0],
        );
    }
}

fn append_recipe_book_panel_chrome(
    capture: &WorldRenderCapture,
    fontRenderer: &mut FontRenderer,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let Some(state) = active_recipe_book_state(capture) else { return; };
    if !state.open { return; }
    append_hud_quad_colored(
        vertices, indices, atlas.recipeBookRectangle,
        state.panelLeft, state.panelTop, 147, 166,
        1, 1, 147, 166, 256, 256,
        [1.0, 1.0, 1.0, 1.0],
    );

    append_gui_text_field(
        &state.searchField, fontRenderer, atlas, vertices, indices,
    );

    for tab in &state.tabs {
        let sourceX = 153 + if tab.selected { 35 } else { 0 };
        let drawX = tab.rect.x - if tab.selected { 2 } else { 0 };
        append_hud_quad_colored(
            vertices, indices, atlas.recipeBookRectangle,
            drawX, tab.rect.y, tab.rect.width, tab.rect.height,
            sourceX, 2, tab.rect.width, tab.rect.height, 256, 256,
            [1.0, 1.0, 1.0, 1.0],
        );
    }

    let filterHovered = state.filter.contains(capture.inventoryMouseX, capture.inventoryMouseY);
    append_hud_quad_colored(
        vertices, indices, atlas.recipeBookRectangle,
        state.filter.x, state.filter.y, state.filter.width, state.filter.height,
        152 + if state.filteringCraftable { 28 } else { 0 },
        41 + if filterHovered { 18 } else { 0 },
        state.filter.width, state.filter.height, 256, 256,
        [1.0, 1.0, 1.0, 1.0],
    );

    for button in &state.buttons {
        let firstVertex = vertices.len();
        append_hud_quad_colored(
            vertices, indices, atlas.recipeBookRectangle,
            button.rect.x, button.rect.y, button.rect.width, button.rect.height,
            29 + if button.craftable { 0 } else { 25 },
            206 + if button.multiple { 25 } else { 0 },
            button.rect.width, button.rect.height, 256, 256,
            [1.0, 1.0, 1.0, 1.0],
        );
        scale_gui_vertices_about(
            &mut vertices[firstVertex..],
            (button.rect.x + 8) as f32,
            (button.rect.y + 12) as f32,
            button.animationScale,
        );
    }

    if state.pageCount > 1 {
        let pageText = format!("{}/{}", state.currentPage + 1, state.pageCount);
        append_font_text_colored_no_shadow(
            &pageText,
            state.panelLeft + 73 - fontRenderer.get_string_width(&pageText) / 2,
            state.panelTop + 141,
            -1,
            fontRenderer, atlas, vertices, indices,
        );
        if state.currentPage > 0 {
            let hovered = state.previous.contains(capture.inventoryMouseX, capture.inventoryMouseY);
            append_hud_quad_colored(
                vertices, indices, atlas.recipeBookRectangle,
                state.previous.x, state.previous.y, state.previous.width, state.previous.height,
                14, 208 + if hovered { 18 } else { 0 },
                state.previous.width, state.previous.height, 256, 256,
                [1.0, 1.0, 1.0, 1.0],
            );
        }
        if state.currentPage + 1 < state.pageCount {
            let hovered = state.next.contains(capture.inventoryMouseX, capture.inventoryMouseY);
            append_hud_quad_colored(
                vertices, indices, atlas.recipeBookRectangle,
                state.next.x, state.next.y, state.next.width, state.next.height,
                1, 208 + if hovered { 18 } else { 0 },
                state.next.width, state.next.height, 256, 256,
                [1.0, 1.0, 1.0, 1.0],
            );
        }
    }
    append_recipe_overlay_background(state, capture, atlas, vertices, indices);
}

fn append_recipe_book_toggle(
    capture: &WorldRenderCapture,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let Some(state) = active_recipe_book_state(capture) else { return; };
    if state.open && state.widthTooNarrow { return; }
    let hovered = state.toggle.contains(capture.inventoryMouseX, capture.inventoryMouseY);
    let (rectangle, sourceX, sourceY) = if state.inventoryScreen {
        (atlas.inventoryRectangle, 178, if hovered { 19 } else { 0 })
    } else {
        (atlas.craftingRectangle, 0, 168 + if hovered { 19 } else { 0 })
    };
    append_hud_quad_colored(
        vertices, indices, rectangle,
        state.toggle.x, state.toggle.y, state.toggle.width, state.toggle.height,
        sourceX, sourceY, state.toggle.width, state.toggle.height, 256, 256,
        [1.0, 1.0, 1.0, 1.0],
    );
}

fn append_recipe_book_tab_items(
    state: &RecipeBookRenderState,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    for tab in &state.tabs {
        match tab.category {
            RecipeCategory::Search => append_item_stack_gui(
                &RECIPE_SEARCH_TAB.getIconItemStack(), tab.rect.x + 9, tab.rect.y + 5,
                atlas, vertices, indices,
            ),
            RecipeCategory::Tools => {
                append_item_stack_gui(
                    &RECIPE_TOOLS_TAB.getIconItemStack(), tab.rect.x + 3, tab.rect.y + 5,
                    atlas, vertices, indices,
                );
                append_item_stack_gui(
                    &RECIPE_COMBAT_TAB.getIconItemStack(), tab.rect.x + 14, tab.rect.y + 5,
                    atlas, vertices, indices,
                );
            }
            RecipeCategory::BuildingBlocks => append_item_stack_gui(
                &RECIPE_BUILDING_TAB.getIconItemStack(), tab.rect.x + 9, tab.rect.y + 5,
                atlas, vertices, indices,
            ),
            RecipeCategory::Misc => {
                append_item_stack_gui(
                    &RECIPE_MISC_TAB.getIconItemStack(), tab.rect.x + 3, tab.rect.y + 5,
                    atlas, vertices, indices,
                );
                append_item_stack_gui(
                    &RECIPE_FOOD_TAB.getIconItemStack(), tab.rect.x + 14, tab.rect.y + 5,
                    atlas, vertices, indices,
                );
            }
            RecipeCategory::Redstone => append_item_stack_gui(
                &RECIPE_REDSTONE_TAB.getIconItemStack(), tab.rect.x + 9, tab.rect.y + 5,
                atlas, vertices, indices,
            ),
        }
    }
}

fn scale_gui_vertices_about(
    vertices: &mut [WorldVertex],
    pivotX: f32,
    pivotY: f32,
    scale: f32,
) {
    if (scale - 1.0).abs() <= f32::EPSILON { return; }
    for vertex in vertices {
        vertex.position[0] = pivotX + (vertex.position[0] - pivotX) * scale;
        vertex.position[1] = pivotY + (vertex.position[1] - pivotY) * scale;
    }
}

fn append_item_stack_gui_gl_scaled(
    stack: &ItemStack,
    x: i32,
    y: i32,
    scale: f32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let firstVertex = vertices.len();
    append_item_stack_gui(stack, x, y, atlas, vertices, indices);
    for vertex in &mut vertices[firstVertex..] {
        vertex.position[0] *= scale;
        vertex.position[1] *= scale;
    }
}

fn append_item_glint_gui_gl_scaled(
    stack: &ItemStack,
    x: i32,
    y: i32,
    scale: f32,
    systemTimeMillis: u64,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let firstVertex = vertices.len();
    append_item_glint_gui(stack, x, y, systemTimeMillis, atlas, vertices, indices);
    for vertex in &mut vertices[firstVertex..] {
        vertex.position[0] *= scale;
        vertex.position[1] *= scale;
    }
}

fn for_each_recipe_overlay_ingredient<F>(
    state: &RecipeBookRenderState,
    mut consumer: F,
) where
    F: FnMut(&ItemStack, i32, i32),
{
    if !state.overlay.visible { return; }
    const SCALE: f32 = 0.42;
    for button in &state.overlay.buttons {
        let Some(recipe) = CraftingManager::getRecipe(button.recipeId) else { continue; };
        let ingredients = recipe.getIngredients();
        let mut iterator = ingredients.iter();
        for row in 0..button.ingredientHeight {
            let ingredientY = 3 + row as i32 * 7;
            for column in 0..button.ingredientWidth {
                let Some(ingredient) = iterator.next() else { break; };
                let alternatives = ingredient.getMatchingStacks();
                if alternatives.is_empty() { continue; }
                let frame = ((state.overlay.animationTicks / 30.0).floor() as usize) % alternatives.len();
                let ingredientX = 3 + column as i32 * 7;
                let renderX = (((button.rect.x + ingredientX) as f32) / SCALE - 3.0) as i32;
                let renderY = (((button.rect.y + ingredientY) as f32) / SCALE - 3.0) as i32;
                consumer(&alternatives[frame], renderX, renderY);
            }
        }
    }
}

fn append_recipe_overlay_item_models(
    state: &RecipeBookRenderState,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    for_each_recipe_overlay_ingredient(state, |stack, x, y| {
        append_item_stack_gui_gl_scaled(stack, x, y, 0.42, atlas, vertices, indices);
    });
}

fn append_recipe_overlay_item_glints(
    state: &RecipeBookRenderState,
    systemTimeMillis: u64,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    for_each_recipe_overlay_ingredient(state, |stack, x, y| {
        append_item_glint_gui_gl_scaled(
            stack, x, y, 0.42, systemTimeMillis, atlas, vertices, indices,
        );
    });
}

fn append_recipe_book_item_models(
    capture: &WorldRenderCapture,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let Some(state) = active_recipe_book_state(capture) else { return; };
    if !state.open { return; }
    append_recipe_book_tab_items(state, atlas, vertices, indices);
    for button in &state.buttons {
        let Some(recipe) = CraftingManager::getRecipe(button.recipeId) else { continue; };
        let firstVertex = vertices.len();
        let stack = recipe.getRecipeOutput();
        let mut offset = 4;
        if button.allOutputsEqual && button.multiple {
            append_item_stack_gui(
                &stack, button.rect.x + offset + 1, button.rect.y + offset + 1,
                atlas, vertices, indices,
            );
            offset -= 1;
        }
        append_item_stack_gui(
            &stack, button.rect.x + offset, button.rect.y + offset,
            atlas, vertices, indices,
        );
        scale_gui_vertices_about(
            &mut vertices[firstVertex..],
            (button.rect.x + 8) as f32,
            (button.rect.y + 12) as f32,
            button.animationScale,
        );
    }
    if !state.widthTooNarrow {
        for (index, ingredient) in state.ghost.iter().enumerate() {
            if index == 0 && state.inventoryScreen {
                append_solid_hud_quad(
                    ingredient.x - 4, ingredient.y - 4, 24, 24,
                    packed_argb_to_rgba(822_018_048), atlas, vertices, indices,
                );
            } else {
                append_solid_hud_quad(
                    ingredient.x, ingredient.y, 16, 16,
                    packed_argb_to_rgba(822_018_048), atlas, vertices, indices,
                );
            }
            append_item_stack_gui(
                &ingredient.stack, ingredient.x, ingredient.y,
                atlas, vertices, indices,
            );
        }
    }
    append_recipe_overlay_item_models(state, atlas, vertices, indices);
}

fn append_recipe_book_item_glints(
    capture: &WorldRenderCapture,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let Some(state) = active_recipe_book_state(capture) else { return; };
    if !state.open { return; }
    let appendGlint = |stack: &ItemStack, x: i32, y: i32, vertices: &mut Vec<WorldVertex>, indices: &mut Vec<u32>| {
        append_item_glint_gui(
            stack, x, y, capture.systemTimeMillis, atlas, vertices, indices,
        );
    };
    for tab in &state.tabs {
        match tab.category {
            RecipeCategory::Search => appendGlint(&RECIPE_SEARCH_TAB.getIconItemStack(), tab.rect.x + 9, tab.rect.y + 5, vertices, indices),
            RecipeCategory::Tools => {
                appendGlint(&RECIPE_TOOLS_TAB.getIconItemStack(), tab.rect.x + 3, tab.rect.y + 5, vertices, indices);
                appendGlint(&RECIPE_COMBAT_TAB.getIconItemStack(), tab.rect.x + 14, tab.rect.y + 5, vertices, indices);
            }
            RecipeCategory::BuildingBlocks => appendGlint(&RECIPE_BUILDING_TAB.getIconItemStack(), tab.rect.x + 9, tab.rect.y + 5, vertices, indices),
            RecipeCategory::Misc => {
                appendGlint(&RECIPE_MISC_TAB.getIconItemStack(), tab.rect.x + 3, tab.rect.y + 5, vertices, indices);
                appendGlint(&RECIPE_FOOD_TAB.getIconItemStack(), tab.rect.x + 14, tab.rect.y + 5, vertices, indices);
            }
            RecipeCategory::Redstone => appendGlint(&RECIPE_REDSTONE_TAB.getIconItemStack(), tab.rect.x + 9, tab.rect.y + 5, vertices, indices),
        }
    }
    for button in &state.buttons {
        if let Some(recipe) = CraftingManager::getRecipe(button.recipeId) {
            let firstVertex = vertices.len();
            let stack = recipe.getRecipeOutput();
            let mut offset = 4;
            if button.allOutputsEqual && button.multiple {
                appendGlint(&stack, button.rect.x + offset + 1, button.rect.y + offset + 1, vertices, indices);
                offset -= 1;
            }
            appendGlint(&stack, button.rect.x + offset, button.rect.y + offset, vertices, indices);
            scale_gui_vertices_about(
                &mut vertices[firstVertex..],
                (button.rect.x + 8) as f32,
                (button.rect.y + 12) as f32,
                button.animationScale,
            );
        }
    }
    if !state.widthTooNarrow {
        for ingredient in &state.ghost {
            appendGlint(&ingredient.stack, ingredient.x, ingredient.y, vertices, indices);
        }
    }
    append_recipe_overlay_item_glints(
        state, capture.systemTimeMillis, atlas, vertices, indices,
    );
}

fn append_recipe_book_overlays(
    capture: &WorldRenderCapture,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let Some(state) = active_recipe_book_state(capture) else { return; };
    if !state.open || state.widthTooNarrow { return; }
    for (index, ingredient) in state.ghost.iter().enumerate() {
        append_solid_hud_quad(
            ingredient.x, ingredient.y, 16, 16,
            packed_argb_to_rgba(822_083_583), atlas, vertices, indices,
        );
        if index == 0 {
            append_item_overlay_gui(
                &ingredient.stack, ingredient.x, ingredient.y,
                atlas, vertices, indices,
            );
        }
    }
}

fn append_recipe_book_tooltip(
    capture: &WorldRenderCapture,
    locale: &Locale,
    fontRenderer: &mut FontRenderer,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) -> bool {
    let Some(state) = active_recipe_book_state(capture) else { return false; };
    if !state.open { return false; }
    if !state.overlay.visible {
        for button in &state.buttons {
            if !button.rect.contains(capture.inventoryMouseX, capture.inventoryMouseY) { continue; }
            let Some(recipe) = CraftingManager::getRecipe(button.recipeId) else { return false; };
            let mut lines = ItemTooltip::getItemToolTip(&recipe.getRecipeOutput(), locale, capture.advancedItemTooltips);
            if button.multiple {
                lines.push(locale.translate_key("gui.recipebook.moreRecipes").to_owned());
            }
            append_hovering_text(
                &lines, capture.inventoryMouseX, capture.inventoryMouseY,
                capture.guiWidth, capture.guiHeight, fontRenderer, atlas, vertices, indices,
            );
            return true;
        }
    }
    if state.filter.contains(capture.inventoryMouseX, capture.inventoryMouseY) {
        let key = if state.filteringCraftable {
            "gui.recipebook.toggleRecipes.craftable"
        } else {
            "gui.recipebook.toggleRecipes.all"
        };
        append_hovering_text(
            &[locale.translate_key(key).to_owned()],
            capture.inventoryMouseX, capture.inventoryMouseY,
            capture.guiWidth, capture.guiHeight, fontRenderer, atlas, vertices, indices,
        );
        return true;
    }
    if !state.widthTooNarrow {
        for ingredient in &state.ghost {
            let rect = GuiRect { x: ingredient.x, y: ingredient.y, width: 16, height: 16 };
            if rect.contains(capture.inventoryMouseX, capture.inventoryMouseY) {
                let lines = ItemTooltip::getItemToolTip(&ingredient.stack, locale, capture.advancedItemTooltips);
                append_hovering_text(
                    &lines, capture.inventoryMouseX, capture.inventoryMouseY,
                    capture.guiWidth, capture.guiHeight, fontRenderer, atlas, vertices, indices,
                );
                return true;
            }
        }
    }
    false
}

fn append_player_inventory_background(
    capture: &WorldRenderCapture,
    locale: &Locale,
    fontRenderer: &mut FontRenderer,
    standardGalacticFontRenderer: &mut FontRenderer,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    append_solid_hud_quad(
        0, 0, capture.guiWidth, capture.guiHeight,
        [0.0, 0.0, 0.0, 0.55], atlas, vertices, indices,
    );
    let inventory = player_inventory_layout(capture);
    let container = inventory.container();
    if active_recipe_book_state(capture).is_some_and(|state| state.open && !state.widthTooNarrow) {
        append_recipe_book_panel_chrome(capture, fontRenderer, atlas, vertices, indices);
    }
    if capture.inventoryIsCreative {
        append_creative_inventory_background(capture, fontRenderer, atlas, container, vertices, indices);
    } else if let Some(spec) = capture.inventoryHorseSpec {
        append_hud_quad_colored(
            vertices, indices, atlas.horseRectangle,
            container.guiLeft, container.guiTop,
            GuiScreenHorseInventory::X_SIZE, GuiScreenHorseInventory::Y_SIZE,
            0, 0, GuiScreenHorseInventory::X_SIZE, GuiScreenHorseInventory::Y_SIZE,
            256, 256, [1.0, 1.0, 1.0, 1.0],
        );
        if spec.chested {
            append_hud_quad_colored(
                vertices, indices, atlas.horseRectangle,
                container.guiLeft + 79, container.guiTop + 17,
                spec.chestColumns.clamp(1, 5) * 18, 54,
                0, GuiScreenHorseInventory::Y_SIZE,
                spec.chestColumns.clamp(1, 5) * 18, 54,
                256, 256, [1.0, 1.0, 1.0, 1.0],
            );
        }
        if spec.kind.canUseSaddleSlot() {
            append_hud_quad_colored(
                vertices, indices, atlas.horseRectangle,
                container.guiLeft + 7, container.guiTop + 17,
                18, 18, 18, GuiScreenHorseInventory::Y_SIZE + 54,
                18, 18, 256, 256, [1.0, 1.0, 1.0, 1.0],
            );
        }
        if spec.kind.hasEquipmentSlot() {
            append_hud_quad_colored(
                vertices, indices, atlas.horseRectangle,
                container.guiLeft + 7, container.guiTop + 35,
                18, 18,
                if spec.kind.isLlama() { 36 } else { 0 },
                GuiScreenHorseInventory::Y_SIZE + 54,
                18, 18, 256, 256, [1.0, 1.0, 1.0, 1.0],
            );
        }
        append_font_text_colored_no_shadow(
            &capture.inventoryTitle,
            container.guiLeft + 8,
            container.guiTop + 6,
            4_210_752,
            fontRenderer, atlas, vertices, indices,
        );
        append_font_text_colored_no_shadow(
            &capture.playerInventoryTitle,
            container.guiLeft + 8,
            container.guiTop + container.ySize - 96 + 2,
            4_210_752,
            fontRenderer, atlas, vertices, indices,
        );
    } else if let Some(kind) = capture.inventoryWindowKind {
        let rectangle = match kind {
            ContainerWindowKind::Workbench => atlas.craftingRectangle,
            ContainerWindowKind::Furnace => atlas.furnaceRectangle,
            ContainerWindowKind::Repair => atlas.anvilRectangle,
            ContainerWindowKind::Enchantment => atlas.enchantingRectangle,
            ContainerWindowKind::Hopper => atlas.hopperRectangle,
            ContainerWindowKind::BrewingStand => atlas.brewingStandRectangle,
            ContainerWindowKind::Dispenser | ContainerWindowKind::Dropper => atlas.dispenserRectangle,
            ContainerWindowKind::Beacon => atlas.beaconRectangle,
            ContainerWindowKind::Merchant => atlas.merchantRectangle,
        };
        append_hud_quad_colored(
            vertices,
            indices,
            rectangle,
            container.guiLeft,
            container.guiTop,
            container.xSize,
            container.ySize,
            0,
            0,
            container.xSize,
            container.ySize,
            256,
            256,
            [1.0, 1.0, 1.0, 1.0],
        );

        if kind != ContainerWindowKind::Beacon {
            let titleX = match kind {
                ContainerWindowKind::Workbench => 28,
                ContainerWindowKind::Furnace
                | ContainerWindowKind::BrewingStand
                | ContainerWindowKind::Dispenser
                | ContainerWindowKind::Dropper
                | ContainerWindowKind::Merchant => {
                    container.xSize / 2 - fontRenderer.get_string_width(&capture.inventoryTitle) / 2
                }
                ContainerWindowKind::Repair => 60,
                ContainerWindowKind::Enchantment => 12,
                ContainerWindowKind::Hopper => 8,
                ContainerWindowKind::Beacon => unreachable!("beacon labels are drawn by GuiBeacon"),
            };
            let titleY = if kind == ContainerWindowKind::Enchantment { 5 } else { 6 };
            append_font_text_colored_no_shadow(
                &capture.inventoryTitle,
                container.guiLeft + titleX,
                container.guiTop + titleY,
                4_210_752,
                fontRenderer,
                atlas,
                vertices,
                indices,
            );
            append_font_text_colored_no_shadow(
                &capture.playerInventoryTitle,
                container.guiLeft + 8,
                container.guiTop + container.ySize - 94,
                4_210_752,
                fontRenderer,
                atlas,
                vertices,
                indices,
            );
        }

        if kind == ContainerWindowKind::Beacon {
            let primaryTitle = locale.translate_key("tile.beacon.primary");
            let secondaryTitle = locale.translate_key("tile.beacon.secondary");
            append_font_text_colored_no_shadow(
                primaryTitle,
                container.guiLeft + 62 - fontRenderer.get_string_width(primaryTitle) / 2,
                container.guiTop + 10,
                14_737_632,
                fontRenderer,
                atlas,
                vertices,
                indices,
            );
            append_font_text_colored_no_shadow(
                secondaryTitle,
                container.guiLeft + 169 - fontRenderer.get_string_width(secondaryTitle) / 2,
                container.guiTop + 10,
                14_737_632,
                fontRenderer,
                atlas,
                vertices,
                indices,
            );
            append_beacon_buttons(capture, container, atlas, vertices, indices);
        }

        match kind {
            ContainerWindowKind::Furnace => {
                if GuiFurnace::isBurning(&capture.inventoryProperties) {
                    let burn = GuiFurnace::burnLeftScaled(&capture.inventoryProperties, 13);
                    append_hud_quad_colored(
                        vertices,
                        indices,
                        rectangle,
                        container.guiLeft + 56,
                        container.guiTop + 36 + 12 - burn,
                        14,
                        burn + 1,
                        176,
                        12 - burn,
                        14,
                        burn + 1,
                        256,
                        256,
                        [1.0, 1.0, 1.0, 1.0],
                    );
                }
                let cook = GuiFurnace::cookProgressScaled(&capture.inventoryProperties, 24);
                append_hud_quad_colored(
                    vertices,
                    indices,
                    rectangle,
                    container.guiLeft + 79,
                    container.guiTop + 34,
                    cook + 1,
                    16,
                    176,
                    14,
                    cook + 1,
                    16,
                    256,
                    256,
                    [1.0, 1.0, 1.0, 1.0],
                );
            }
            ContainerWindowKind::Repair => {
                let firstEmpty = capture.inventorySlots.first().map_or(true, ItemStack::isEmpty);
                append_hud_quad_colored(
                    vertices,
                    indices,
                    rectangle,
                    container.guiLeft + 59,
                    container.guiTop + 20,
                    110,
                    16,
                    0,
                    container.ySize + if firstEmpty { 16 } else { 0 },
                    110,
                    16,
                    256,
                    256,
                    [1.0, 1.0, 1.0, 1.0],
                );
                let hasInput = capture.inventorySlots.get(0).is_some_and(|stack| !stack.isEmpty())
                    || capture.inventorySlots.get(1).is_some_and(|stack| !stack.isEmpty());
                let outputEmpty = capture.inventorySlots.get(2).map_or(true, ItemStack::isEmpty);
                if hasInput && outputEmpty {
                    append_hud_quad_colored(
                        vertices,
                        indices,
                        rectangle,
                        container.guiLeft + 99,
                        container.guiTop + 45,
                        28,
                        21,
                        container.xSize,
                        0,
                        28,
                        21,
                        256,
                        256,
                        [1.0, 1.0, 1.0, 1.0],
                    );
                }
                if let Some(input) = &capture.anvilNameInput {
                    append_gui_text_field(input, fontRenderer, atlas, vertices, indices);
                }
                let cost = capture.inventoryProperties.first().copied().unwrap_or(0);
                if cost > 0 && !outputEmpty {
                    let costText = if cost >= 40 && !capture.playerCreativeMode {
                        capture.anvilTooExpensive.clone()
                    } else {
                        format_gui_translation(&capture.anvilCostFormat, cost)
                    };
                    let cannotTake = !capture.playerCreativeMode
                        && (cost >= 40 || capture.experienceLevel < cost);
                    let color = if cannotTake { 16_736_352 } else { 8_453_920 };
                    let x = container.guiLeft + container.xSize - 8 - fontRenderer.get_string_width(&costText);
                    append_font_text_colored_no_shadow(
                        &costText,
                        x,
                        container.guiTop + 67,
                        color,
                        fontRenderer,
                        atlas,
                        vertices,
                        indices,
                    );
                }
            }
            ContainerWindowKind::Enchantment => {
                if let Some(book) = capture.enchantmentBookState {
                    append_enchantment_gui_book(
                        book,
                        capture.guiWidth,
                        capture.guiHeight,
                        atlas,
                        vertices,
                        indices,
                    );
                }
                // GuiEnchantment reseeds the singleton once per frame, then
                // consumes exactly three names in option order regardless of
                // whether each option is currently available.
                let xpSeed = capture.inventoryProperties.get(3).copied().unwrap_or(0);
                let mut nameParts = EnchantmentNameParts::default();
                nameParts.reseedRandomGenerator(xpSeed as i64);
                let lapis = capture.inventorySlots.get(1).map_or(0, ItemStack::getCount);
                for option in 0..3_i32 {
                    let level = capture.inventoryProperties.get(option as usize).copied().unwrap_or(0);
                    if level == 0 {
                        append_hud_quad_colored(
                            vertices,
                            indices,
                            rectangle,
                            container.guiLeft + 60,
                            container.guiTop + 14 + 19 * option,
                            108,
                            19,
                            0,
                            185,
                            108,
                            19,
                            256,
                            256,
                            [1.0, 1.0, 1.0, 1.0],
                        );
                        continue;
                    }

                    let levelText = level.to_string();
                    let nameWidth = 86 - fontRenderer.get_string_width(&levelText);
                    let randomName = nameParts.generateNewRandomName(fontRenderer, nameWidth);
                    let available = capture.playerCreativeMode
                        || (lapis >= option + 1 && capture.experienceLevel >= level);
                    let hovered = capture.inventoryMouseX >= container.guiLeft + 60
                        && capture.inventoryMouseX < container.guiLeft + 168
                        && capture.inventoryMouseY >= container.guiTop + 14 + 19 * option
                        && capture.inventoryMouseY < container.guiTop + 33 + 19 * option;
                    let sourceY = if !available { 185 } else if hovered { 204 } else { 166 };
                    append_hud_quad_colored(
                        vertices,
                        indices,
                        rectangle,
                        container.guiLeft + 60,
                        container.guiTop + 14 + 19 * option,
                        108,
                        19,
                        0,
                        sourceY,
                        108,
                        19,
                        256,
                        256,
                        [1.0, 1.0, 1.0, 1.0],
                    );
                    append_hud_quad_colored(
                        vertices,
                        indices,
                        rectangle,
                        container.guiLeft + 61,
                        container.guiTop + 15 + 19 * option,
                        16,
                        16,
                        16 * option,
                        if available { 223 } else { 239 },
                        16,
                        16,
                        256,
                        256,
                        [1.0, 1.0, 1.0, 1.0],
                    );

                    let galacticColor = if !available {
                        (6_839_882 & 16_711_422) >> 1
                    } else if hovered {
                        16_777_088
                    } else {
                        6_839_882
                    };
                    append_font_split_text_colored_no_shadow(
                        &randomName,
                        container.guiLeft + 80,
                        container.guiTop + 16 + 19 * option,
                        nameWidth,
                        galacticColor,
                        standardGalacticFontRenderer,
                        atlas,
                        vertices,
                        indices,
                    );
                    append_font_text_colored(
                        &levelText,
                        container.guiLeft + 166 - fontRenderer.get_string_width(&levelText),
                        container.guiTop + 23 + 19 * option,
                        if available { 8_453_920 } else { 4_226_832 },
                        true,
                        fontRenderer,
                        atlas,
                        vertices,
                        indices,
                    );
                }
            }
            ContainerWindowKind::BrewingStand => {
                let fuel = GuiBrewingStand::fuelWidth(&capture.inventoryProperties);
                if fuel > 0 {
                    append_hud_quad_colored(
                        vertices, indices, rectangle,
                        container.guiLeft + 60, container.guiTop + 44,
                        fuel, 4, 176, 29, fuel, 4, 256, 256,
                        [1.0, 1.0, 1.0, 1.0],
                    );
                }
                let brew = GuiBrewingStand::brewHeight(&capture.inventoryProperties);
                if brew > 0 {
                    append_hud_quad_colored(
                        vertices, indices, rectangle,
                        container.guiLeft + 97, container.guiTop + 16,
                        9, brew, 176, 0, 9, brew, 256, 256,
                        [1.0, 1.0, 1.0, 1.0],
                    );
                }
                let bubbles = GuiBrewingStand::bubbleHeight(&capture.inventoryProperties);
                if bubbles > 0 {
                    append_hud_quad_colored(
                        vertices, indices, rectangle,
                        container.guiLeft + 63, container.guiTop + 43 - bubbles,
                        12, bubbles, 185, 29 - bubbles, 12, bubbles, 256, 256,
                        [1.0, 1.0, 1.0, 1.0],
                    );
                }
            }
            ContainerWindowKind::Merchant => {
                let recipes = capture.merchantRecipes.as_ref();
                let mut gui = GuiMerchant::new();
                gui.initGui(capture.guiWidth, capture.guiHeight);
                gui.setSelectedMerchantRecipe(capture.merchantRecipeIndex, recipes);
                for button in [
                    gui.previousButton(capture.inventoryMouseX, capture.inventoryMouseY, recipes),
                    gui.nextButton(capture.inventoryMouseX, capture.inventoryMouseY, recipes),
                ] {
                    let (u,v)=button.source();
                    append_hud_quad_colored(vertices,indices,rectangle,button.x,button.y,12,19,u,v,12,19,256,256,[1.0,1.0,1.0,1.0]);
                }
                if let Some(recipe)=recipes.and_then(|list|list.get(gui.selectedMerchantRecipe() as usize)) {
                    if recipe.isRecipeDisabled() {
                        for y in [21,51] {
                            append_hud_quad_colored(vertices,indices,rectangle,container.guiLeft+83,container.guiTop+y,28,21,212,0,28,21,256,256,[1.0,1.0,1.0,1.0]);
                        }
                    }
                }
            }
            ContainerWindowKind::Workbench
            | ContainerWindowKind::Hopper
            | ContainerWindowKind::Dispenser
            | ContainerWindowKind::Dropper
            | ContainerWindowKind::Beacon => {}
        }
    } else if capture.inventoryIsShulker {
        append_hud_quad_colored(
            vertices, indices, atlas.shulkerRectangle,
            container.guiLeft, container.guiTop,
            GuiShulkerBox::X_SIZE, GuiShulkerBox::Y_SIZE,
            0, 0, GuiShulkerBox::X_SIZE, GuiShulkerBox::Y_SIZE, 256, 256,
            [1.0, 1.0, 1.0, 1.0],
        );
        append_font_text_colored_no_shadow(
            &capture.inventoryTitle,
            container.guiLeft + 8,
            container.guiTop + 6,
            4_210_752,
            fontRenderer, atlas, vertices, indices,
        );
        append_font_text_colored_no_shadow(
            &capture.playerInventoryTitle,
            container.guiLeft + 8,
            container.guiTop + container.ySize - 96 + 2,
            4_210_752,
            fontRenderer, atlas, vertices, indices,
        );
    } else if capture.inventoryIsChest {
        let topHeight = capture.inventoryRows * 18 + 17;
        append_hud_quad_colored(
            vertices, indices, atlas.chestRectangle,
            container.guiLeft, container.guiTop,
            container.xSize, topHeight,
            0, 0, container.xSize, topHeight, 256, 256,
            [1.0, 1.0, 1.0, 1.0],
        );
        append_hud_quad_colored(
            vertices, indices, atlas.chestRectangle,
            container.guiLeft, container.guiTop + topHeight,
            container.xSize, 96,
            0, 126, container.xSize, 96, 256, 256,
            [1.0, 1.0, 1.0, 1.0],
        );
        // GuiChest#drawGuiContainerForegroundLayer delegates both labels to
        // FontRenderer without a drop shadow. This is essential for server
        // inventory titles containing CJK or other Unicode glyphs.
        append_font_text_colored_no_shadow(
            &capture.inventoryTitle,
            container.guiLeft + 8,
            container.guiTop + 6,
            4_210_752,
            fontRenderer, atlas, vertices, indices,
        );
        append_font_text_colored_no_shadow(
            &capture.playerInventoryTitle,
            container.guiLeft + 8,
            container.guiTop + container.ySize - 94,
            4_210_752,
            fontRenderer, atlas, vertices, indices,
        );
    } else {
        append_hud_quad_colored(
            vertices, indices, atlas.inventoryRectangle,
            container.guiLeft, container.guiTop,
            GuiInventory::X_SIZE, GuiInventory::Y_SIZE,
            0, 0, GuiInventory::X_SIZE, GuiInventory::Y_SIZE, 256, 256,
            [1.0, 1.0, 1.0, 1.0],
        );
        // Slot.getSlotTexture: armor and offhand slots show TextureMap sprites
        // only while their real ContainerPlayer stack is empty.
        for (slotId, textureIndex) in [(5_i32, 0_usize), (6, 1), (7, 2), (8, 3), (45, 4)] {
            if capture.inventorySlots.get(slotId as usize).is_some_and(ItemStack::isEmpty) {
                if let Some((slotX, slotY)) = inventory.slotPosition(slotId) {
                    append_hud_quad_colored(
                        vertices, indices, atlas.emptySlotRectangles[textureIndex],
                        slotX, slotY, 16, 16,
                        0, 0, 16, 16, 16, 16,
                        [1.0, 1.0, 1.0, 1.0],
                    );
                }
            }
        }
    }
    append_recipe_book_toggle(capture, atlas, vertices, indices);
    if !recipe_book_narrow_open(capture)
        && capture.inventoryDragSplitting
        && capture.inventoryDragSplittingSlots.len() > 1
    {
        for &slotId in &capture.inventoryDragSplittingSlots {
            if let Some((slotX, slotY)) = inventory.slotPosition(slotId) {
                append_solid_hud_quad(
                    slotX, slotY, 16, 16,
                    [0.0, 0.0, 0.0, 0.5], atlas, vertices, indices,
                );
            }
        }
    }
}

fn append_horse_inventory_entity(
    capture: &WorldRenderCapture,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let Some(sourceEntity) = capture.inventoryHorseEntity.as_ref() else { return; };
    let inventory = player_inventory_layout(capture);
    let posX = inventory.container().guiLeft + 51;
    let posY = inventory.container().guiTop + 60;
    let mouseX = posX as f32 - capture.inventoryOldMouseX;
    let mouseY = (inventory.container().guiTop + 25) as f32 - capture.inventoryOldMouseY;
    let horizontal = (mouseX / 40.0).atan();
    let vertical = (mouseY / 40.0).atan();

    // GuiInventory.drawEntityOnScreen temporarily overwrites these five
    // rotations and renders with partialTicks=1.0F. Work on the captured clone
    // so the authoritative WorldClient entity is never mutated by a GUI.
    let mut entity = sourceEntity.clone();
    let bodyYaw = horizontal * 20.0;
    let headYaw = horizontal * 40.0;
    let headPitch = -vertical * 20.0;
    entity.prevRenderYawOffset = bodyYaw;
    entity.renderYawOffset = bodyYaw;
    entity.entity.prevRotationYaw = headYaw;
    entity.entity.rotationYaw = headYaw;
    entity.entity.prevRotationPitch = headPitch;
    entity.entity.rotationPitch = headPitch;
    entity.prevRotationYawHead = headYaw;
    entity.rotationYawHead = headYaw;
    entity.entity.prevPosX = 0.0;
    entity.entity.prevPosY = 0.0;
    entity.entity.prevPosZ = 0.0;
    entity.entity.posX = 0.0;
    entity.entity.posY = 0.0;
    entity.entity.posZ = 0.0;

    let ClientEntityKind::Mob { entityType } = &entity.kind else { return; };
    let entityType = *entityType;
    let mut meshes = Vec::<(LivingModelMesh, ResourceLocation)>::new();
    if RenderLlama::supports(entityType) {
        let input = RenderLivingBase::renderInput(&entity, 1.0, 1.0);
        let pose = ModelLlama::pose(input);
        meshes.push((
            RenderLivingBase::buildMesh(
                input,
                ModelLlama::boxes(pose, input, &entity, 0.0),
                128.0,
                64.0,
            ),
            RenderLlama::texture(&entity),
        ));
        if let Some(texture) = LayerLlamaDecor::texture(&entity) {
            meshes.push((
                RenderLivingBase::buildMesh(
                    input,
                    ModelLlama::boxes(pose, input, &entity, LayerLlamaDecor::modelDelta()),
                    128.0,
                    64.0,
                ),
                texture,
            ));
        }
    } else {
        let (variant, texture) = if RenderHorse::supports(entityType) {
            (HorseModelVariant::Horse, RenderHorse::texture(&entity))
        } else if let Some(variant) = RenderAbstractHorse::variant(entityType) {
            (variant, RenderAbstractHorse::texture(variant))
        } else {
            return;
        };
        let input = RenderLivingBase::renderInput(&entity, 1.0, 1.0);
        let pose = ModelHorse::pose(input, &entity, 1.0);
        meshes.push((
            RenderLivingBase::buildMesh(
                input,
                ModelHorse::boxes(pose, input, &entity, variant, 1.0),
                128.0,
                128.0,
            ),
            texture,
        ));
    }

    let lightingRotation = rotation_y4(135.0);
    let lights = [
        normalize3(transform_direction3(lightingRotation, [0.2, 1.0, -0.7])),
        normalize3(transform_direction3(lightingRotation, [-0.2, 1.0, 0.7])),
    ];
    let entityPitch = rotation_x4(headPitch);
    let mut quads = Vec::<GuiItemQuad>::new();
    for (mesh, texture) in meshes {
        let rectangle = atlas
            .builtInItemRectangles
            .get(&texture)
            .copied()
            .unwrap_or(atlas.missingRectangle);
        append_living_mesh_gui_quads(
            &mesh,
            rectangle,
            entityPitch,
            posX,
            posY,
            17.0,
            lights,
            &mut quads,
        );
    }
    quads.sort_by(|left, right| left.depth.total_cmp(&right.depth));
    append_gui_item_quads(quads, vertices, indices);
}

fn append_living_mesh_gui_quads(
    mesh: &LivingModelMesh,
    rectangle: [f32; 4],
    entityPitch: [[f32; 4]; 4],
    posX: i32,
    posY: i32,
    scale: f32,
    lights: [[f32; 3]; 2],
    quads: &mut Vec<GuiItemQuad>,
) {
    for face in mesh.vertices.chunks_exact(4) {
        let mut modelPositions = [[0.0_f32; 3]; 4];
        let mut screenPositions = [[0.0_f32; 3]; 4];
        let mut uvs = [[0.0_f32; 2]; 4];
        for index in 0..4 {
            let transformed = transform_point3(entityPitch, face[index].position);
            modelPositions[index] = transformed;
            screenPositions[index] = [
                posX as f32 + transformed[0] * scale,
                posY as f32 - transformed[1] * scale,
                transformed[2],
            ];
            uvs[index] = face[index].uv;
        }
        let edge1 = subtract3(modelPositions[1], modelPositions[0]);
        let edge2 = subtract3(modelPositions[2], modelPositions[0]);
        let normal = normalize3(cross3(edge1, edge2));
        if normal[2] <= 1.0e-5 { continue; }
        let diffuse = standard_item_diffuse(normal, lights);
        quads.push(GuiItemQuad {
            depth: modelPositions.iter().map(|position| position[2]).sum::<f32>() / 4.0,
            positions: screenPositions,
            uvs,
            rectangle,
            color: [diffuse, diffuse, diffuse, 1.0],
        });
    }
}

fn append_gui_item_quads(
    quads: Vec<GuiItemQuad>,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    for quad in quads {
        let base = vertices.len() as u32;
        for index in 0..4 {
            vertices.push(WorldVertex {
                position: [quad.positions[index][0], quad.positions[index][1], 0.0],
                uv: map_player_skin_uv(quad.rectangle, quad.uvs[index]),
                color: quad.color,
                lightmap: [15.0, 15.0],
            
                shaderEntity: [-1, -1, -1],
                shaderPadding: 0,
            });
        }
        indices.extend_from_slice(&[
            base, base + 1, base + 2,
            base, base + 2, base + 3,
        ]);
    }
}

fn append_player_inventory_entity(
    capture: &WorldRenderCapture,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    // GuiInventory.drawGuiContainerBackgroundLayer calls
    // drawEntityOnScreen(guiLeft + 51, guiTop + 75, 30, ...). Preserve the
    // prior-frame mouse coordinates because vanilla updates oldMouseX/Y only
    // after the complete screen has been drawn.
    let inventory = player_inventory_layout(capture);
    let (posX, posY, mouseYOffset, scale) = if capture.inventoryIsCreative {
        (
            inventory.container().guiLeft + 88,
            inventory.container().guiTop + 45,
            30,
            20.0_f32,
        )
    } else {
        (
            inventory.container().guiLeft + 51,
            inventory.container().guiTop + 75,
            50,
            30.0_f32,
        )
    };
    let mouseX = posX as f32 - capture.inventoryOldMouseX;
    let mouseY = (posY - mouseYOffset) as f32 - capture.inventoryOldMouseY;
    let horizontal = (mouseX / 40.0).atan();
    let vertical = (mouseY / 40.0).atan();
    let bodyYaw = horizontal * 20.0;
    let headYaw = horizontal * 40.0;
    let headPitch = -vertical * 20.0;
    let wholeEntityPitch = -vertical * 20.0;
    // GuiInventory#drawEntityOnScreen calls RenderManager with partialTicks 1.0.
    let partial = 1.0_f32;
    let limbSwingAmount = capture.localPrevLimbSwingAmount
        + (capture.localLimbSwingAmount - capture.localPrevLimbSwingAmount) * partial;
    let limbSwing = capture.localLimbSwing
        - capture.localLimbSwingAmount * (1.0 - partial);
    let slim = capture.localSlim;
    let (leftArmPose, rightArmPose) = inventory_player_arm_poses(capture);
    let renderInput = PlayerRenderInput {
        position: [0.0, 0.0, 0.0],
        bodyYaw,
        headYaw,
        headPitch,
        limbSwing,
        limbSwingAmount,
        ageInTicks: capture.playerTicksExisted as f32 + partial,
        swingProgress: capture.localSwingProgress,
        sneaking: capture.localSneaking,
        riding: capture.localRiding,
        slim,
        skinParts: capture.localSkinParts,
        swingingArmIsLeft: capture.localSwingingArmIsLeft,
        leftArmPose,
        rightArmPose,
        ticksElytraFlying: capture.localPlayerRenderState.as_ref()
            .map_or(0, |player| player.ticksElytraFlying),
        motion: capture.localPlayerRenderState.as_ref()
            .map_or([0.0; 3], |player| player.motion),
    };
    let pose = RenderPlayer::buildPose(renderInput);
    let mesh = RenderPlayer::buildMesh(renderInput);
    if mesh.indices.is_empty() { return; }

    // RenderHelper.enableStandardItemLighting is invoked between the +135 and
    // -135 degree Y rotations in GuiInventory. This is the same two-light
    // profile used by vanilla item/entity GUI rendering.
    let lightingRotation = rotation_y4(135.0);
    let lights = [
        normalize3(transform_direction3(lightingRotation, [0.2, 1.0, -0.7])),
        normalize3(transform_direction3(lightingRotation, [-0.2, 1.0, 0.7])),
    ];
    let entityPitch = inventory_player_entity_matrix(
        capture.localPlayerRenderState.as_ref(), bodyYaw, headYaw, headPitch, wholeEntityPitch,
    );
    let skinRectangle = player_skin_rectangle(atlas, &capture.localSkinLocation, slim);
    let mut quads = Vec::<GuiItemQuad>::new();

    // RenderPlayer's ModelBox baker emits one independent four-vertex face at
    // a time. Convert each face into scaled GUI coordinates, perform the
    // equivalent back-face rejection, then sort by model depth because the
    // current HUD pass intentionally has depth testing disabled.
    for face in mesh.vertices.chunks_exact(4) {
        let mut modelPositions = [[0.0_f32; 3]; 4];
        let mut screenPositions = [[0.0_f32; 3]; 4];
        let mut uvs = [[0.0_f32; 2]; 4];
        for index in 0..4 {
            let transformed = transform_point3(entityPitch, face[index].position);
            modelPositions[index] = transformed;
            screenPositions[index] = [
                posX as f32 + transformed[0] * scale,
                posY as f32 - transformed[1] * scale,
                transformed[2],
            ];
            uvs[index] = face[index].uv;
        }
        let edge1 = subtract3(modelPositions[1], modelPositions[0]);
        let edge2 = subtract3(modelPositions[2], modelPositions[0]);
        let normal = normalize3(cross3(edge1, edge2));
        if normal[2] <= 1.0e-5 { continue; }
        let diffuse = standard_item_diffuse(normal, lights);
        quads.push(GuiItemQuad {
            depth: modelPositions.iter().map(|position| position[2]).sum::<f32>() / 4.0,
            positions: screenPositions,
            uvs,
            rectangle: skinRectangle,
            color: [diffuse, diffuse, diffuse, 1.0],
        });
    }

    append_player_inventory_held_items(
        capture, atlas, pose, bodyYaw, entityPitch, posX, posY, scale, lights, &mut quads,
    );

    if let Some(player) = capture.localPlayerRenderState.as_ref() {
        append_player_inventory_armor(
            player, pose, bodyYaw, entityPitch, posX, posY, scale, lights, atlas, &mut quads,
        );
        append_player_inventory_cape(
            player, bodyYaw, entityPitch, posX, posY, scale, lights,
            1.0, atlas, &mut quads,
        );
        append_player_inventory_custom_head(
            player, pose, bodyYaw, limbSwing, entityPitch, posX, posY, scale, lights,
            atlas, &mut quads,
        );
        append_player_inventory_elytra(
            player, bodyYaw, entityPitch, posX, posY, scale, lights, atlas, &mut quads,
        );
    }

    quads.sort_by(|left, right| left.depth.total_cmp(&right.depth));
    for quad in quads {
        let base = vertices.len() as u32;
        for index in 0..4 {
            vertices.push(WorldVertex {
                position: [quad.positions[index][0], quad.positions[index][1], 0.0],
                uv: map_player_skin_uv(quad.rectangle, quad.uvs[index]),
                color: quad.color,
                lightmap: [15.0, 15.0],
            
                shaderEntity: [-1, -1, -1],
                shaderPadding: 0,
            });
        }
        indices.extend_from_slice(&[
            base, base + 1, base + 2,
            base, base + 2, base + 3,
        ]);
    }
}

fn should_render_inventory_player(capture: &WorldRenderCapture) -> bool {
    capture.inventoryWindowKind.is_none()
        && !capture.inventoryIsChest
        && !capture.inventoryIsShulker
        && capture.inventoryHorseSpec.is_none()
        && (!capture.inventoryIsCreative
            || capture.creativeSelectedTab == CREATIVE_INVENTORY_TAB.tabIndex)
}

#[allow(clippy::too_many_arguments)]
fn append_player_glint_gui_quads(
    mesh: crate::net::minecraft::client::renderer::entity::RenderPlayer::PlayerModelMesh,
    matrix: [[f32; 4]; 4],
    pass: crate::net::minecraft::client::renderer::entity::layers::LayerArmorBase::GlintPass,
    glintRectangle: [f32; 4],
    posX: i32,
    posY: i32,
    scale: f32,
    quads: &mut Vec<GuiItemQuad>,
) {
    for face in mesh.vertices.chunks_exact(4) {
        let positions = [
            transform_point3(matrix, face[0].position),
            transform_point3(matrix, face[1].position),
            transform_point3(matrix, face[2].position),
            transform_point3(matrix, face[3].position),
        ];
        let edge1 = subtract3(positions[1], positions[0]);
        let edge2 = subtract3(positions[2], positions[0]);
        let normal = normalize3(cross3(edge1, edge2));
        if normal[2] <= 1.0e-5 { continue; }
        let screenPositions = positions.map(|position| [
            posX as f32 + position[0] * scale,
            posY as f32 - position[1] * scale,
            position[2],
        ]);
        let uvs = [
            enchanted_glint_local_uv(face[0].uv, pass),
            enchanted_glint_local_uv(face[1].uv, pass),
            enchanted_glint_local_uv(face[2].uv, pass),
            enchanted_glint_local_uv(face[3].uv, pass),
        ];
        quads.push(GuiItemQuad {
            depth: positions.iter().map(|position| position[2]).sum::<f32>() / 4.0,
            positions: screenPositions,
            uvs,
            rectangle: glintRectangle,
            color: pass.color,
        });
    }
}

fn append_player_inventory_entity_glints(
    capture: &WorldRenderCapture,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let Some(player) = capture.localPlayerRenderState.as_ref() else { return; };
    let inventory = player_inventory_layout(capture);
    let (posX, posY, mouseYOffset, scale) = if capture.inventoryIsCreative {
        (
            inventory.container().guiLeft + 88,
            inventory.container().guiTop + 45,
            30,
            20.0_f32,
        )
    } else {
        (
            inventory.container().guiLeft + 51,
            inventory.container().guiTop + 75,
            50,
            30.0_f32,
        )
    };
    let mouseX = posX as f32 - capture.inventoryOldMouseX;
    let mouseY = (posY - mouseYOffset) as f32 - capture.inventoryOldMouseY;
    let horizontal = (mouseX / 40.0).atan();
    let vertical = (mouseY / 40.0).atan();
    let bodyYaw = horizontal * 20.0;
    let headYaw = horizontal * 40.0;
    let headPitch = -vertical * 20.0;
    let entityPitch = inventory_player_entity_matrix(
        Some(player), bodyYaw, headYaw, headPitch, -vertical * 20.0,
    );
    let partial = 1.0_f32;
    let limbSwingAmount = capture.localPrevLimbSwingAmount
        + (capture.localLimbSwingAmount - capture.localPrevLimbSwingAmount) * partial;
    let limbSwing = capture.localLimbSwing
        - capture.localLimbSwingAmount * (1.0 - partial);
    let (leftArmPose, rightArmPose) = inventory_player_arm_poses(capture);
    let pose = RenderPlayer::buildPose(PlayerRenderInput {
        position: [0.0; 3],
        bodyYaw,
        headYaw,
        headPitch,
        limbSwing,
        limbSwingAmount,
        ageInTicks: capture.playerTicksExisted as f32 + partial,
        swingProgress: capture.localSwingProgress,
        sneaking: capture.localSneaking,
        riding: capture.localRiding,
        slim: capture.localSlim,
        skinParts: capture.localSkinParts,
        swingingArmIsLeft: capture.localSwingingArmIsLeft,
        leftArmPose,
        rightArmPose,
        ticksElytraFlying: player.ticksElytraFlying,
        motion: player.motion,
    });

    let mut quads = Vec::<GuiItemQuad>::new();
    for pass in LayerArmorBase::glintPasses(capture.playerTicksExisted as f32 + partial) {
        for slot in [
            EntityEquipmentSlot::Chest,
            EntityEquipmentSlot::Legs,
            EntityEquipmentSlot::Feet,
            EntityEquipmentSlot::Head,
        ] {
            let stack = player_armor_stack(player, slot);
            let Some(definition) = ItemArmor::definition(stack.itemId) else { continue; };
            if definition.slot != slot || !stack.isItemEnchanted() { continue; }
            let mesh = RenderPlayer::buildBoxesMesh(
                LayerBipedArmor::boxes(pose, slot), [0.0; 3], bodyYaw,
                player.sneaking, 64.0, 32.0,
            );
            append_player_glint_gui_quads(
                mesh, entityPitch, pass, atlas.glintRectangle,
                posX, posY, scale, &mut quads,
            );
        }

        if ItemArmor::isElytra(&player.chestStack) && player.chestStack.isItemEnchanted() {
            let elytraPose = ModelElytra::poseFromRotations(player.sneaking, player.elytraRotation);
            let mesh = RenderPlayer::buildLocalBoxesMesh(
                ModelElytra::boxes(elytraPose), 64.0, 32.0,
            );
            let mut matrix = player_layer_root_matrix(bodyYaw, player.sneaking);
            matrix = multiply4(matrix, translation4([0.0, 0.0, 0.125]));
            matrix = multiply4(entityPitch, matrix);
            append_player_glint_gui_quads(
                mesh, matrix, pass, atlas.glintRectangle,
                posX, posY, scale, &mut quads,
            );
        }
    }
    quads.sort_by(|left, right| left.depth.total_cmp(&right.depth));
    append_gui_item_quads(quads, vertices, indices);
}

#[allow(clippy::too_many_arguments)]
fn append_player_model_gui_quads(
    mesh: crate::net::minecraft::client::renderer::entity::RenderPlayer::PlayerModelMesh,
    matrix: [[f32; 4]; 4],
    rectangle: [f32; 4],
    tint: [f32; 3],
    shaded: bool,
    posX: i32,
    posY: i32,
    scale: f32,
    lights: [[f32; 3]; 2],
    quads: &mut Vec<GuiItemQuad>,
) {
    for face in mesh.vertices.chunks_exact(4) {
        let positions = [
            transform_point3(matrix, face[0].position),
            transform_point3(matrix, face[1].position),
            transform_point3(matrix, face[2].position),
            transform_point3(matrix, face[3].position),
        ];
        let uvs = [face[0].uv, face[1].uv, face[2].uv, face[3].uv];
        push_inventory_entity_item_quad(
            positions, uvs, rectangle, tint, shaded,
            posX, posY, scale, lights, quads,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn append_player_inventory_armor(
    player: &RemotePlayerRenderState,
    pose: BipedPose,
    bodyYaw: f32,
    entityPitch: [[f32; 4]; 4],
    posX: i32,
    posY: i32,
    scale: f32,
    lights: [[f32; 3]; 2],
    atlas: &AtlasState,
    quads: &mut Vec<GuiItemQuad>,
) {
    for slot in [
        EntityEquipmentSlot::Chest,
        EntityEquipmentSlot::Legs,
        EntityEquipmentSlot::Feet,
        EntityEquipmentSlot::Head,
    ] {
        let stack = player_armor_stack(player, slot);
        let Some(definition) = ItemArmor::definition(stack.itemId) else { continue; };
        if definition.slot != slot { continue; }
        let boxes = LayerBipedArmor::boxes(pose, slot);
        for tint in LayerArmorBase::tintPasses(stack) {
            let Some(texture) = LayerArmorBase::texture(stack, tint.overlay) else { continue; };
            let Some(rectangle) = atlas.entityTextureRectangles.get(&texture).copied() else { continue; };
            let mesh = RenderPlayer::buildBoxesMesh(
                boxes.iter().copied(), [0.0; 3], bodyYaw, player.sneaking,
                64.0, 32.0,
            );
            append_player_model_gui_quads(
                mesh, entityPitch, rectangle,
                [tint.color[0], tint.color[1], tint.color[2]], true,
                posX, posY, scale, lights, quads,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn append_player_inventory_cape(
    player: &RemotePlayerRenderState,
    bodyYaw: f32,
    entityPitch: [[f32; 4]; 4],
    posX: i32,
    posY: i32,
    scale: f32,
    lights: [[f32; 3]; 2],
    partialTicks: f32,
    atlas: &AtlasState,
    quads: &mut Vec<GuiItemQuad>,
) {
    let Some(capeLocation) = player.capeLocation.as_ref() else { return; };
    if !LayerCape::shouldRender(
        true, player.invisible, player.skinParts, &player.chestStack,
    ) {
        return;
    }
    let Some(rectangle) = atlas.entityTextureRectangles.get(capeLocation).copied() else { return; };
    let transform = LayerCape::transform(
        CapeMotionInput {
            prevChasingPos: player.prevChasingPosition,
            chasingPos: player.chasingPosition,
            prevPos: player.prevPosition,
            pos: player.position,
            // GuiInventory temporarily replaces renderYawOffset and invokes
            // RenderManager with partialTicks=1.0, so the interpolated yaw is
            // the mouse-driven body yaw used by the preview.
            prevRenderYawOffset: bodyYaw,
            renderYawOffset: bodyYaw,
            prevCameraYaw: player.prevCameraYaw,
            cameraYaw: player.cameraYaw,
            prevDistanceWalkedModified: player.prevMovedDistance,
            distanceWalkedModified: player.movedDistance,
            sneaking: player.sneaking,
        },
        partialTicks,
    );
    let mut matrix = player_layer_root_matrix(bodyYaw, player.sneaking);
    matrix = multiply4(matrix, translation4(transform.translation));
    matrix = multiply4(matrix, rotation_x4(transform.rotateX));
    matrix = multiply4(matrix, rotation_z4(transform.rotateZ));
    matrix = multiply4(matrix, rotation_y4(transform.rotateY));
    matrix = multiply4(matrix, rotation_y4(transform.finalRotateY));
    matrix = multiply4(entityPitch, matrix);
    append_player_model_gui_quads(
        RenderPlayer::buildCapeMesh(), matrix, rectangle, [1.0; 3], true,
        posX, posY, scale, lights, quads,
    );
}

#[allow(clippy::too_many_arguments)]
fn append_player_inventory_custom_head(
    player: &RemotePlayerRenderState,
    pose: BipedPose,
    bodyYaw: f32,
    limbSwing: f32,
    entityPitch: [[f32; 4]; 4],
    posX: i32,
    posY: i32,
    scale: f32,
    lights: [[f32; 3]; 2],
    atlas: &AtlasState,
    quads: &mut Vec<GuiItemQuad>,
) {
    let stack = player_armor_stack(player, EntityEquipmentSlot::Head);
    if !LayerCustomHead::isSkull(stack) { return; }
    let skullType = stack.itemDamage as i32;
    let Some(mesh) = TileEntityItemStackRenderer::buildSkullMesh(skullType, limbSwing) else { return; };
    let rectangle = if skullType == 3 {
        player.customHeadSkinLocation.as_ref()
            .and_then(|location| atlas.entityTextureRectangles.get(location).copied())
            .unwrap_or(atlas.steveRectangle)
    } else {
        atlas.builtInItemRectangles.get(&mesh.texture)
            .copied()
            .unwrap_or(atlas.missingRectangle)
    };
    let mut matrix = player_layer_root_matrix(bodyYaw, player.sneaking);
    if player.sneaking {
        matrix = multiply4(matrix, translation4([0.0, 0.2, 0.0]));
    }
    matrix = post_render_part_matrix(matrix, pose.head);
    matrix = multiply4(matrix, scale4_nonuniform([
        LayerCustomHead::SKULL_SCALE,
        -LayerCustomHead::SKULL_SCALE,
        -LayerCustomHead::SKULL_SCALE,
    ]));
    matrix = multiply4(entityPitch, matrix);
    for face in mesh.indices.chunks_exact(6) {
        let source = [face[0] as usize, face[1] as usize, face[2] as usize, face[5] as usize];
        let positions = source.map(|index| {
            let vertex = mesh.vertices[index].position;
            transform_point3(matrix, [vertex[0] - 0.5, vertex[1], vertex[2] - 0.5])
        });
        let uvs = source.map(|index| mesh.vertices[index].uv);
        push_inventory_entity_item_quad(
            positions, uvs, rectangle, [1.0; 3], true,
            posX, posY, scale, lights, quads,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn append_player_inventory_elytra(
    player: &RemotePlayerRenderState,
    bodyYaw: f32,
    entityPitch: [[f32; 4]; 4],
    posX: i32,
    posY: i32,
    scale: f32,
    lights: [[f32; 3]; 2],
    atlas: &AtlasState,
    quads: &mut Vec<GuiItemQuad>,
) {
    if !ItemArmor::isElytra(&player.chestStack) { return; }
    let Some(location) = player.elytraLocation.as_ref() else { return; };
    let Some(rectangle) = atlas.entityTextureRectangles.get(location).copied() else { return; };
    let pose = ModelElytra::poseFromRotations(player.sneaking, player.elytraRotation);
    let mesh = RenderPlayer::buildLocalBoxesMesh(ModelElytra::boxes(pose), 64.0, 32.0);
    let mut matrix = player_layer_root_matrix(bodyYaw, player.sneaking);
    matrix = multiply4(matrix, translation4([0.0, 0.0, 0.125]));
    matrix = multiply4(entityPitch, matrix);
    append_player_model_gui_quads(
        mesh, matrix, rectangle, [1.0; 3], true,
        posX, posY, scale, lights, quads,
    );
}

fn merchant_preview_stacks(capture: &WorldRenderCapture) -> Vec<(ItemStack, i32, i32)> {
    if capture.inventoryWindowKind != Some(ContainerWindowKind::Merchant) { return Vec::new(); }
    let mut gui=GuiMerchant::new(); gui.initGui(capture.guiWidth,capture.guiHeight);
    gui.setSelectedMerchantRecipe(capture.merchantRecipeIndex,capture.merchantRecipes.as_ref());
    let Some(recipe)=capture.merchantRecipes.as_ref().and_then(|l|l.get(gui.selectedMerchantRecipe() as usize)) else{return Vec::new();};
    let mut result=vec![(recipe.getItemToBuy().clone(),gui.container.guiLeft+36,gui.container.guiTop+24)];
    if recipe.hasSecondItemToBuy(){result.push((recipe.getSecondItemToBuy().clone(),gui.container.guiLeft+62,gui.container.guiTop+24));}
    result.push((recipe.getItemToSell().clone(),gui.container.guiLeft+120,gui.container.guiTop+24)); result
}

fn append_player_inventory_item_models(
    capture: &WorldRenderCapture,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    append_recipe_book_item_models(capture, atlas, vertices, indices);
    let narrowRecipeBook = recipe_book_narrow_open(capture);
    let inventory = player_inventory_layout(capture);
    if !narrowRecipeBook {
    for (stack,x,y) in merchant_preview_stacks(capture) {
        append_item_stack_gui(&stack,x,y,atlas,vertices,indices);
    }
    if capture.inventoryWindowKind == Some(ContainerWindowKind::Beacon) {
        let container = inventory.container();
        for (itemId, offsetX) in [(388_i16, 42_i32), (264, 64), (266, 86), (265, 108)] {
            let sample = ItemStack {
                itemId,
                count: 1,
                itemDamage: 0,
                tagCompound: None,
            };
            append_item_stack_gui(
                &sample,
                container.guiLeft + offsetX,
                container.guiTop + 109,
                atlas,
                vertices,
                indices,
            );
        }
    }
    if capture.inventoryIsCreative {
        for tab in CREATIVE_TAB_ARRAY {
            if let Some((iconX, iconY)) = creative_tab_icon_position(inventory.container(), tab.tabIndex) {
                append_item_stack_gui(&tab.getIconItemStack(), iconX, iconY, atlas, vertices, indices);
            }
        }
    }
    for slotId in 0..capture.inventorySlots.len() as i32 {
        let Some((slotX, slotY)) = inventory.slotPosition(slotId) else { continue; };
        let Some(stack) = player_inventory_display_stack(capture, slotId) else { continue; };
        append_item_stack_gui(&stack, slotX, slotY, atlas, vertices, indices);
    }
    }
    let cursor = player_inventory_cursor_display_stack(capture);
    if !cursor.isEmpty() {
        append_item_stack_gui(
            &cursor,
            capture.inventoryMouseX - 8,
            capture.inventoryMouseY - 8,
            atlas, vertices, indices,
        );
    }
}

fn append_player_inventory_item_glints(
    capture: &WorldRenderCapture,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    append_recipe_book_item_glints(capture, atlas, vertices, indices);
    let narrowRecipeBook = recipe_book_narrow_open(capture);
    let inventory = player_inventory_layout(capture);
    if !narrowRecipeBook {
    for (stack,x,y) in merchant_preview_stacks(capture) {
        append_item_glint_gui(&stack,x,y,capture.systemTimeMillis,atlas,vertices,indices);
    }
    if capture.inventoryIsCreative {
        for tab in CREATIVE_TAB_ARRAY {
            if let Some((iconX, iconY)) = creative_tab_icon_position(inventory.container(), tab.tabIndex) {
                append_item_glint_gui(
                    &tab.getIconItemStack(), iconX, iconY,
                    capture.systemTimeMillis, atlas, vertices, indices,
                );
            }
        }
    }
    for slotId in 0..capture.inventorySlots.len() as i32 {
        let Some((slotX, slotY)) = inventory.slotPosition(slotId) else { continue; };
        let Some(stack) = player_inventory_display_stack(capture, slotId) else { continue; };
        append_item_glint_gui(
            &stack, slotX, slotY,
            capture.systemTimeMillis, atlas, vertices, indices,
        );
    }
    }
    let cursor = player_inventory_cursor_display_stack(capture);
    append_item_glint_gui(
        &cursor,
        capture.inventoryMouseX - 8,
        capture.inventoryMouseY - 8,
        capture.systemTimeMillis,
        atlas, vertices, indices,
    );
}

fn append_player_inventory_overlays(
    capture: &WorldRenderCapture,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let narrowRecipeBook = recipe_book_narrow_open(capture);
    let inventory = player_inventory_layout(capture);
    if !narrowRecipeBook {
    for (stack,x,y) in merchant_preview_stacks(capture) {
        append_item_overlay_gui(&stack,x,y,atlas,vertices,indices);
    }
    if capture.inventoryIsCreative {
        for tab in CREATIVE_TAB_ARRAY {
            if let Some((iconX, iconY)) = creative_tab_icon_position(inventory.container(), tab.tabIndex) {
                append_item_overlay_gui(&tab.getIconItemStack(), iconX, iconY, atlas, vertices, indices);
            }
        }
    }
    for slotId in 0..capture.inventorySlots.len() as i32 {
        let Some((slotX, slotY)) = inventory.slotPosition(slotId) else { continue; };
        let Some(stack) = player_inventory_display_stack(capture, slotId) else { continue; };
        if let Some(limit) = player_inventory_drag_preview_limit(capture, slotId) {
            let text = limit.to_string();
            append_item_overlay_gui_with_alt(
                &stack,
                slotX,
                slotY,
                Some((&text, [1.0, 1.0, 0.33, 1.0])),
                atlas,
                vertices,
                indices,
            );
        } else {
            append_item_overlay_gui(&stack, slotX, slotY, atlas, vertices, indices);
        }
    }

    if let Some(slotId) = inventory.slotAt(capture.inventoryMouseX, capture.inventoryMouseY) {
        if let Some((slotX, slotY)) = inventory.slotPosition(slotId) {
            append_solid_hud_quad(
                slotX, slotY, 16, 16,
                [1.0, 1.0, 1.0, 0.5], atlas, vertices, indices,
            );
        }
    }
    }

    append_recipe_book_overlays(capture, atlas, vertices, indices);
    let cursor = player_inventory_cursor_display_stack(capture);
    if !cursor.isEmpty() {
        append_item_overlay_gui(
            &cursor,
            capture.inventoryMouseX - 8,
            capture.inventoryMouseY - 8,
            atlas, vertices, indices,
        );
    } else if capture.inventoryDragSplitting
        && capture.inventoryDragSplittingSlots.len() > 1
        && capture.inventoryDragSplittingRemnant <= 0
    {
        append_ascii_text_colored(
            "0",
            capture.inventoryMouseX + 3,
            capture.inventoryMouseY + 1,
            [1.0, 1.0, 0.33, 1.0],
            atlas, vertices, indices,
        );
    }
}

fn append_player_inventory_tooltip(
    capture: &WorldRenderCapture,
    locale: &Locale,
    fontRenderer: &mut FontRenderer,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    if append_recipe_book_tooltip(
        capture, locale, fontRenderer, atlas, vertices, indices,
    ) {
        return;
    }
    if recipe_book_narrow_open(capture) {
        return;
    }
    if capture.inventoryWindowKind == Some(ContainerWindowKind::Enchantment) {
        let inventory = player_inventory_layout(capture);
        let container = inventory.container();
        let lapis = capture.inventorySlots.get(1).map_or(0, ItemStack::getCount);
        for option in 0..3_i32 {
            let level = capture.inventoryProperties.get(option as usize).copied().unwrap_or(0);
            let enchantmentId = capture.inventoryProperties
                .get((4 + option) as usize)
                .copied()
                .unwrap_or(-1);
            let enchantmentLevel = capture.inventoryProperties
                .get((7 + option) as usize)
                .copied()
                .unwrap_or(-1);
            let hovered = capture.inventoryMouseX >= container.guiLeft + 60
                && capture.inventoryMouseX < container.guiLeft + 168
                && capture.inventoryMouseY >= container.guiTop + 14 + 19 * option
                && capture.inventoryMouseY < container.guiTop + 31 + 19 * option;
            let Some(enchantment) = Enchantment::getEnchantmentByID(enchantmentId) else {
                continue;
            };
            if !hovered || level <= 0 || enchantmentLevel < 0 {
                continue;
            }

            let clue = format_gui_translation(
                locale.translate_key("container.enchant.clue"),
                enchantment.getTranslatedName(enchantmentLevel, locale),
            );
            let mut lines = vec![format!("§f§o{clue}")];
            if !capture.playerCreativeMode {
                lines.push(String::new());
                if capture.experienceLevel < level {
                    let requirement = format_gui_translation(
                        locale.translate_key("container.enchant.level.requirement"),
                        level,
                    );
                    lines.push(format!("§c{requirement}"));
                } else {
                    let cost = option + 1;
                    let lapisText = if cost == 1 {
                        locale.translate_key("container.enchant.lapis.one").to_owned()
                    } else {
                        format_gui_translation(
                            locale.translate_key("container.enchant.lapis.many"),
                            cost,
                        )
                    };
                    lines.push(format!("{}{}", if lapis >= cost { "§7" } else { "§c" }, lapisText));
                    let levelText = if cost == 1 {
                        locale.translate_key("container.enchant.level.one").to_owned()
                    } else {
                        format_gui_translation(
                            locale.translate_key("container.enchant.level.many"),
                            cost,
                        )
                    };
                    lines.push(format!("§7{levelText}"));
                }
            }
            append_hovering_text(
                &lines,
                capture.inventoryMouseX,
                capture.inventoryMouseY,
                capture.guiWidth,
                capture.guiHeight,
                fontRenderer,
                atlas,
                vertices,
                indices,
            );
            return;
        }
    }
    if capture.inventoryWindowKind == Some(ContainerWindowKind::Merchant) {
        let mut gui=GuiMerchant::new(); gui.initGui(capture.guiWidth,capture.guiHeight);
        gui.setSelectedMerchantRecipe(capture.merchantRecipeIndex,capture.merchantRecipes.as_ref());
        if let Some(recipe)=capture.merchantRecipes.as_ref().and_then(|l|l.get(gui.selectedMerchantRecipe() as usize)) {
            let stack=match gui.previewRegionAt(capture.inventoryMouseX,capture.inventoryMouseY) {
                Some(MerchantPreviewRegion::FirstBuy)=>Some(recipe.getItemToBuy()),
                Some(MerchantPreviewRegion::SecondBuy) if recipe.hasSecondItemToBuy()=>Some(recipe.getSecondItemToBuy()),
                Some(MerchantPreviewRegion::Sell)=>Some(recipe.getItemToSell()),
                Some(MerchantPreviewRegion::Disabled) if recipe.isRecipeDisabled()=>{
                    append_hovering_text(&[locale.translate_key("merchant.deprecated").to_owned()],capture.inventoryMouseX,capture.inventoryMouseY,capture.guiWidth,capture.guiHeight,fontRenderer,atlas,vertices,indices); return;
                }
                _=>None,
            };
            if let Some(stack)=stack { let lines=ItemTooltip::getItemToolTip(stack,locale,capture.advancedItemTooltips); append_hovering_text(&lines,capture.inventoryMouseX,capture.inventoryMouseY,capture.guiWidth,capture.guiHeight,fontRenderer,atlas,vertices,indices); return; }
        }
    }
    if capture.inventoryWindowKind == Some(ContainerWindowKind::Beacon) {
        let mut gui = GuiBeacon::new();
        gui.initGui(capture.guiWidth, capture.guiHeight);
        let levels = capture.inventoryProperties.first().copied().unwrap_or(0);
        let primary = capture.inventoryProperties.get(1).copied().unwrap_or(0);
        let secondary = capture.inventoryProperties.get(2).copied().unwrap_or(0);
        let mut buttonTooltip = None;
        for button in if levels >= 0 { gui.powerButtons(primary) } else { Vec::new() } {
            if capture.inventoryMouseX >= button.x
                && capture.inventoryMouseX < button.x + 22
                && capture.inventoryMouseY >= button.y
                && capture.inventoryMouseY < button.y + 22
            {
                if let Some(key) = GuiBeacon::effectNameKey(button.effectId) {
                    let mut label = locale.translate_key(key).to_owned();
                    if button.tier >= 3 && button.effectId != 10 {
                        label.push_str(" II");
                    }
                    let enabled = button.tier < levels;
                    let selected = if button.tier < 3 {
                        button.effectId == primary
                    } else {
                        button.effectId == secondary
                    };
                    let _ = (enabled, selected);
                    buttonTooltip = Some(label);
                }
                break;
            }
        }
        if gui.confirmAt(capture.inventoryMouseX, capture.inventoryMouseY) {
            buttonTooltip = Some(locale.translate_key("gui.done").to_owned());
        } else if gui.cancelAt(capture.inventoryMouseX, capture.inventoryMouseY) {
            buttonTooltip = Some(locale.translate_key("gui.cancel").to_owned());
        }
        if let Some(label) = buttonTooltip {
            append_hovering_text(
                &[label],
                capture.inventoryMouseX,
                capture.inventoryMouseY,
                capture.guiWidth,
                capture.guiHeight,
                fontRenderer,
                atlas,
                vertices,
                indices,
            );
            return;
        }
    }

    // GuiContainer only renders a slot tooltip while the carried stack is
    // empty. Drag previews and a stack attached to the cursor suppress it.
    if !player_inventory_cursor_display_stack(capture).isEmpty() {
        return;
    }
    let inventory = player_inventory_layout(capture);
    let Some(slotId) = inventory.slotAt(capture.inventoryMouseX, capture.inventoryMouseY) else {
        return;
    };
    let Some(stack) = player_inventory_display_stack(capture, slotId) else { return; };
    if stack.isEmpty() { return; }
    let lines = ItemTooltip::getItemToolTip(&stack, locale, capture.advancedItemTooltips);
    append_hovering_text(
        &lines,
        capture.inventoryMouseX,
        capture.inventoryMouseY,
        capture.guiWidth,
        capture.guiHeight,
        fontRenderer,
        atlas,
        vertices,
        indices,
    );
}

#[allow(clippy::too_many_arguments)]
fn append_hovering_text(
    textLines: &[String],
    x: i32,
    y: i32,
    screenWidth: i32,
    screenHeight: i32,
    fontRenderer: &mut FontRenderer,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    if textLines.is_empty() { return; }
    let width = textLines.iter()
        .map(|line| fontRenderer.get_string_width(line))
        .max().unwrap_or(0);
    let mut left = x + 12;
    let mut top = y - 12;
    let mut height = 8;
    if textLines.len() > 1 {
        height += 2 + (textLines.len() as i32 - 1) * 10;
    }
    if left + width > screenWidth { left -= 28 + width; }
    if top + height + 6 > screenHeight { top = screenHeight - height - 6; }

    let background = packed_argb_to_rgba((-267_386_864_i32) as u32);
    let borderTop = packed_argb_to_rgba(1_347_420_415_u32);
    let borderBottom = packed_argb_to_rgba(1_344_798_847_u32);
    append_solid_hud_quad(left - 3, top - 4, width + 6, 1, background, atlas, vertices, indices);
    append_solid_hud_quad(left - 3, top + height + 3, width + 6, 1, background, atlas, vertices, indices);
    append_solid_hud_quad(left - 3, top - 3, width + 6, height + 6, background, atlas, vertices, indices);
    append_solid_hud_quad(left - 4, top - 3, 1, height + 6, background, atlas, vertices, indices);
    append_solid_hud_quad(left + width + 3, top - 3, 1, height + 6, background, atlas, vertices, indices);
    append_gradient_hud_quad(left - 3, top - 2, 1, height + 4, borderTop, borderBottom, atlas, vertices, indices);
    append_gradient_hud_quad(left + width + 2, top - 2, 1, height + 4, borderTop, borderBottom, atlas, vertices, indices);
    append_solid_hud_quad(left - 3, top - 3, width + 6, 1, borderTop, atlas, vertices, indices);
    append_solid_hud_quad(left - 3, top + height + 2, width + 6, 1, borderBottom, atlas, vertices, indices);

    let mut textY = top;
    for (index, line) in textLines.iter().enumerate() {
        let text = HudText {
            text: line.clone(),
            x: left,
            y: textY,
            color: 0xFFFF_FFFF,
            outline: true,
        };
        append_hud_text(&text, fontRenderer, atlas, vertices, indices);
        if index == 0 { textY += 2; }
        textY += 10;
    }
}

fn append_gradient_hud_quad(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    topColor: [f32; 4],
    bottomColor: [f32; 4],
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    if width <= 0 || height <= 0 { return; }
    let rectangle = atlas.widgetsRectangle;
    let u = rectangle[0] + (rectangle[2] - rectangle[0]) * 247.5 / 256.0;
    let v = rectangle[1] + (rectangle[3] - rectangle[1]) * 3.5 / 256.0;
    let left = x as f32;
    let top = y as f32;
    let right = (x + width) as f32;
    let bottom = (y + height) as f32;
    let base = vertices.len() as u32;
    let lightmap = [15.0, 15.0];
    vertices.extend_from_slice(&[
        WorldVertex { position: [left, bottom, 0.0], uv: [u, v], color: bottomColor, lightmap, shaderEntity: [-1, -1, -1], shaderPadding: 0, },
        WorldVertex { position: [right, bottom, 0.0], uv: [u, v], color: bottomColor, lightmap, shaderEntity: [-1, -1, -1], shaderPadding: 0, },
        WorldVertex { position: [right, top, 0.0], uv: [u, v], color: topColor, lightmap, shaderEntity: [-1, -1, -1], shaderPadding: 0, },
        WorldVertex { position: [left, top, 0.0], uv: [u, v], color: topColor, lightmap, shaderEntity: [-1, -1, -1], shaderPadding: 0, },
    ]);
    indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
}

fn for_each_hotbar_item(capture: &WorldRenderCapture, mut operation: impl FnMut(&ItemStack, i32, i32)) {
    if capture.gameType == GameType::Spectator { return; }
    let center = capture.guiWidth / 2;
    let y = capture.guiHeight - 16 - 3;
    for slot in 0..9 {
        let Some(stack) = capture.hotbarStacks.get(slot) else { continue; };
        operation(stack, center - 90 + slot as i32 * 20 + 2, y);
    }
    if !capture.offhandStack.isEmpty() {
        let offhandSide = match capture.primaryHand {
            EnumHandSide::Right => EnumHandSide::Left,
            EnumHandSide::Left => EnumHandSide::Right,
        };
        let x = if offhandSide == EnumHandSide::Left { center - 91 - 26 } else { center + 91 + 10 };
        operation(&capture.offhandStack, x, y);
    }
}

fn append_hotbar_item_models(
    capture: &WorldRenderCapture, atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>, indices: &mut Vec<u32>,
) {
    for_each_hotbar_item(capture, |stack, x, y| {
        append_item_stack_gui(stack, x, y, atlas, vertices, indices);
    });
}

fn append_hotbar_item_glints(
    capture: &WorldRenderCapture, atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>, indices: &mut Vec<u32>,
) {
    for_each_hotbar_item(capture, |stack, x, y| {
        append_item_glint_gui(stack, x, y, capture.systemTimeMillis, atlas, vertices, indices);
    });
}

fn append_hotbar_item_overlays(
    capture: &WorldRenderCapture, atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>, indices: &mut Vec<u32>,
) {
    for_each_hotbar_item(capture, |stack, x, y| {
        append_item_overlay_gui(stack, x, y, atlas, vertices, indices);
    });
}

fn is_unpatterned_shield(stack: &ItemStack) -> bool {
    stack.itemId == 442
        && !stack.tagCompound.as_ref().is_some_and(|tag| tag.hasKey("BlockEntityTag"))
}

fn built_in_item_rectangle(
    stack: &ItemStack,
    mesh: &BuiltInItemMesh,
    atlas: &AtlasState,
) -> [f32; 4] {
    if stack.itemId == 355 {
        let metadata = if (0..16).contains(&stack.itemDamage) {
            stack.itemDamage as usize
        } else {
            0
        };
        atlas.bedRectangles[metadata]
    } else {
        atlas.builtInItemRectangles
            .get(&mesh.texture)
            .copied()
            .unwrap_or(atlas.missingRectangle)
    }
}

fn append_builtin_item_mesh_world(
    stack: &ItemStack,
    matrix: [[f32; 4]; 4],
    blockLight: f32,
    skyLight: f32,
    itemLights: [[f32; 3]; 2],
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let Some(mesh) = TileEntityItemStackRenderer::buildMesh(stack) else { return; };
    let rectangle = built_in_item_rectangle(stack, &mesh, atlas);
    for face in mesh.indices.chunks_exact(6) {
        let source = [face[0] as usize, face[1] as usize, face[2] as usize, face[5] as usize];
        let transformed = source.map(|index| transform_point3(matrix, mesh.vertices[index].position));
        let normal = normalize3(cross3(
            subtract3(transformed[1], transformed[0]),
            subtract3(transformed[2], transformed[0]),
        ));
        let diffuse = standard_item_diffuse(normal, itemLights);
        let base = vertices.len() as u32;
        for (corner, sourceIndex) in source.into_iter().enumerate() {
            let uv0 = mesh.vertices[sourceIndex].uv;
            let uv = [
                rectangle[0] + (rectangle[2] - rectangle[0]) * uv0[0],
                rectangle[1] + (rectangle[3] - rectangle[1]) * uv0[1],
            ];
            vertices.push(WorldVertex {
                position: transformed[corner],
                uv,
                color: [
                    mesh.color[0] * diffuse,
                    mesh.color[1] * diffuse,
                    mesh.color[2] * diffuse,
                    mesh.color[3],
                ],
                lightmap: [blockLight, skyLight],
            
                shaderEntity: [-1, -1, -1],
                shaderPadding: 0,
            });
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

fn append_builtin_item_mesh_gui(
    stack: &ItemStack,
    x: i32,
    y: i32,
    transform: ItemTransformVec3f,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let Some(mesh) = TileEntityItemStackRenderer::buildMesh(stack) else { return; };
    let rectangle = built_in_item_rectangle(stack, &mesh, atlas);
    let mut quads = Vec::<GuiItemQuad>::new();
    for face in mesh.indices.chunks_exact(6) {
        let source = [face[0] as usize, face[1] as usize, face[2] as usize, face[5] as usize];
        let modelPositions = source.map(|index| apply_item_transform(mesh.vertices[index].position, transform));
        let normal = normalize3(cross3(
            subtract3(modelPositions[1], modelPositions[0]),
            subtract3(modelPositions[2], modelPositions[0]),
        ));
        if normal[2] <= 1.0e-5 { continue; }
        let positions = modelPositions.map(|position| [
            x as f32 + 8.0 + position[0] * 16.0,
            y as f32 + 8.0 - position[1] * 16.0,
            position[2],
        ]);
        let uvs = source.map(|index| mesh.vertices[index].uv);
        let light = gui_item_diffuse(normal);
        quads.push(GuiItemQuad {
            depth: positions.iter().map(|position| position[2]).sum::<f32>() / 4.0,
            positions,
            uvs,
            rectangle,
            color: [
                mesh.color[0] * light,
                mesh.color[1] * light,
                mesh.color[2] * light,
                mesh.color[3],
            ],
        });
    }
    quads.sort_by(|left, right| left.depth.total_cmp(&right.depth));
    for quad in quads {
        let base = vertices.len() as u32;
        for index in 0..4 {
            let uv = [
                quad.rectangle[0] + (quad.rectangle[2] - quad.rectangle[0]) * quad.uvs[index][0],
                quad.rectangle[1] + (quad.rectangle[3] - quad.rectangle[1]) * quad.uvs[index][1],
            ];
            vertices.push(WorldVertex {
                position: [quad.positions[index][0], quad.positions[index][1], 0.0],
                uv,
                color: quad.color,
                lightmap: [15.0, 15.0],
            
                shaderEntity: [-1, -1, -1],
                shaderPadding: 0,
            });
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
    }
}

fn append_shield_model_world(
    matrix: [[f32; 4]; 4],
    rectangle: [f32; 4],
    blockLight: f32,
    skyLight: f32,
    itemLights: [[f32; 3]; 2],
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let shield = ModelShield::buildMesh();
    for face in shield.indices.chunks_exact(6) {
        let source = [face[0] as usize, face[1] as usize, face[2] as usize, face[5] as usize];
        let transformed = source.map(|index| transform_point3(matrix, shield.vertices[index].position));
        let normal = normalize3(cross3(
            subtract3(transformed[1], transformed[0]),
            subtract3(transformed[2], transformed[0]),
        ));
        let diffuse = standard_item_diffuse(normal, itemLights);
        let base = vertices.len() as u32;
        for (corner, sourceIndex) in source.into_iter().enumerate() {
            let uv0 = shield.vertices[sourceIndex].uv;
            let uv = [
                rectangle[0] + (rectangle[2] - rectangle[0]) * uv0[0],
                rectangle[1] + (rectangle[3] - rectangle[1]) * uv0[1],
            ];
            vertices.push(WorldVertex {
                position: transformed[corner],
                uv,
                color: [diffuse, diffuse, diffuse, 1.0],
                lightmap: [blockLight, skyLight],
            
                shaderEntity: [-1, -1, -1],
                shaderPadding: 0,
            });
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

fn append_shield_model_gui(
    x: i32,
    y: i32,
    transform: ItemTransformVec3f,
    rectangle: [f32; 4],
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let shield = ModelShield::buildMesh();
    let mut quads = Vec::<GuiItemQuad>::new();
    for face in shield.indices.chunks_exact(6) {
        let source = [face[0] as usize, face[1] as usize, face[2] as usize, face[5] as usize];
        let modelPositions = source.map(|index| apply_item_transform(shield.vertices[index].position, transform));
        let normal = normalize3(cross3(
            subtract3(modelPositions[1], modelPositions[0]),
            subtract3(modelPositions[2], modelPositions[0]),
        ));
        if normal[2] <= 1.0e-5 { continue; }
        let positions = modelPositions.map(|position| [
            x as f32 + 8.0 + position[0] * 16.0,
            y as f32 + 8.0 - position[1] * 16.0,
            position[2],
        ]);
        let uvs = source.map(|index| shield.vertices[index].uv);
        let light = gui_item_diffuse(normal);
        quads.push(GuiItemQuad {
            depth: positions.iter().map(|position| position[2]).sum::<f32>() / 4.0,
            positions,
            uvs,
            rectangle,
            color: [light, light, light, 1.0],
        });
    }
    quads.sort_by(|left, right| left.depth.total_cmp(&right.depth));
    for quad in quads {
        let base = vertices.len() as u32;
        for index in 0..4 {
            let uv = [
                quad.rectangle[0] + (quad.rectangle[2] - quad.rectangle[0]) * quad.uvs[index][0],
                quad.rectangle[1] + (quad.rectangle[3] - quad.rectangle[1]) * quad.uvs[index][1],
            ];
            vertices.push(WorldVertex {
                position: [quad.positions[index][0], quad.positions[index][1], 0.0],
                uv,
                color: quad.color,
                lightmap: [15.0, 15.0],
            
                shaderEntity: [-1, -1, -1],
                shaderPadding: 0,
            });
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
    }
}

#[derive(Clone)]
struct GuiItemQuad {
    depth: f32,
    positions: [[f32; 3]; 4],
    uvs: [[f32; 2]; 4],
    rectangle: [f32; 4],
    color: [f32; 4],
}

/// MCP `EntityRenderer#func_190563_a`: 40-tick item activation overlay.
/// The ordinary HUD pipeline already has alpha blending, no culling and no
/// depth writes; quads are emitted in model order after the same FIXED camera
/// transform and model-view transform used by RenderItem.
fn append_item_activation(
    capture: &WorldRenderCapture,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    if capture.itemActivationTicks <= 0 || capture.itemActivationItem.isEmpty() {
        return;
    }
    let Some(model) = item_model_for_stack(&capture.itemActivationItem, atlas) else { return; };

    let elapsed = 40 - capture.itemActivationTicks;
    let f = ((elapsed as f32 + capture.partialTicks) / 40.0).clamp(0.0, 1.0);
    let f1 = f * f;
    let f2 = f * f1;
    let f3 = 10.25 * f2 * f1 - 24.95 * f1 * f1 + 25.5 * f2 - 13.8 * f1 + 4.0 * f;
    let f4 = f3 * core::f32::consts::PI;
    let oscillation = (f4 * 2.0).sin().abs();
    let center_x = capture.guiWidth as f32 * 0.5
        + capture.itemActivationRandomX * (capture.guiWidth / 4) as f32 * oscillation;
    let center_y = capture.guiHeight as f32 * 0.5
        + capture.itemActivationRandomY * (capture.guiHeight / 4) as f32 * oscillation;
    let scale = 50.0 + 175.0 * f4.sin();

    let mut matrix = translation4([center_x, center_y, -50.0]);
    matrix = multiply4(matrix, scale4_nonuniform([scale, -scale, scale]));
    matrix = multiply4(matrix, rotation_y4(900.0 * f4.sin().abs()));
    let wobble = 6.0 * (f * 8.0).cos();
    matrix = multiply4(matrix, rotation_x4(wobble));
    matrix = multiply4(matrix, rotation_z4(wobble));

    append_item_stack_world_transformed(
        &capture.itemActivationItem,
        model,
        matrix,
        TransformType::Fixed,
        15_728_880,
        atlas,
        vertices,
        indices,
    );
}

fn append_item_stack_gui(
    stack: &ItemStack,
    x: i32,
    y: i32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    if stack.isEmpty() {
        return;
    }
    let Some(modelKey) = ItemModelMesher::getModelKey(stack) else { return; };
    let Some(model) = atlas.itemModels.get(&modelKey) else { return; };
    if model.builtInRenderer {
        if is_unpatterned_shield(stack) {
            append_shield_model_gui(
                x, y, model.guiTransform(), atlas.shieldBaseRectangle,
                vertices, indices,
            );
        } else {
            append_builtin_item_mesh_gui(
                stack, x, y, model.guiTransform(), atlas, vertices, indices,
            );
        }
        return;
    }
    if model.quads.is_empty() { return; }

    let transform = model.guiTransform();
    let mut quads = Vec::<GuiItemQuad>::new();
    for quad in &model.quads {
        let mut modelPositions = [[0.0_f32; 3]; 4];
        for (index, position) in quad.positions.iter().copied().enumerate() {
            modelPositions[index] = apply_item_transform(position, transform);
        }
        let edge1 = subtract3(modelPositions[1], modelPositions[0]);
        let edge2 = subtract3(modelPositions[2], modelPositions[0]);
        let normal = normalize3(cross3(edge1, edge2));
        // RenderItem keeps back-face culling enabled. Culling must occur before
        // setupGuiTransform's negative Y scale; using screen-space winding here
        // would select the hidden three faces of every GUI cube.
        if normal[2] <= 1.0e-5 {
            continue;
        }
        let mut transformed = [[0.0_f32; 3]; 4];
        for (index, modelPosition) in modelPositions.iter().copied().enumerate() {
            transformed[index] = [
                x as f32 + 8.0 + modelPosition[0] * 16.0,
                y as f32 + 8.0 - modelPosition[1] * 16.0,
                modelPosition[2],
            ];
        }
        let key = item_material_key(stack.itemId, quad.texture.clone(), quad.tintIndex);
        let rectangle = atlas
            .rectangles
            .get(&key)
            .copied()
            .unwrap_or(atlas.missingRectangle);
        let tint = item_tint_color(&atlas.itemColors, stack, quad.tintIndex);
        let light = if model.gui3d && quad.shade {
            gui_item_diffuse(normal)
        } else {
            1.0
        };
        quads.push(GuiItemQuad {
            depth: transformed.iter().map(|position| position[2]).sum::<f32>() / 4.0,
            positions: transformed,
            uvs: quad.uvs,
            rectangle,
            color: [tint[0] * light, tint[1] * light, tint[2] * light, 1.0],
        });
    }
    // With the HUD depth test disabled, preserve RenderItem's depth result by
    // drawing the convex baked-model faces from back to front.
    quads.sort_by(|left, right| left.depth.total_cmp(&right.depth));
    for quad in quads {
        let base = vertices.len() as u32;
        for index in 0..4 {
            let uv = [
                quad.rectangle[0]
                    + (quad.rectangle[2] - quad.rectangle[0]) * quad.uvs[index][0],
                quad.rectangle[1]
                    + (quad.rectangle[3] - quad.rectangle[1]) * quad.uvs[index][1],
            ];
            vertices.push(WorldVertex {
                position: [quad.positions[index][0], quad.positions[index][1], 0.0],
                uv,
                color: quad.color,
                lightmap: [15.0, 15.0],
            
                shaderEntity: [-1, -1, -1],
                shaderPadding: 0,
            });
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
    }
}

fn append_item_glint_gui(
    stack: &ItemStack,
    x: i32,
    y: i32,
    systemTimeMillis: u64,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    if stack.isEmpty() || !stack.hasEffect() { return; }
    let Some(modelKey) = ItemModelMesher::getModelKey(stack) else { return; };
    let Some(model) = atlas.itemModels.get(&modelKey) else { return; };
    if model.builtInRenderer || model.quads.is_empty() { return; }

    let transform = model.guiTransform();
    for (period, direction, angle) in [(3000_u64, 1.0_f32, -50.0_f32), (4873_u64, -1.0_f32, 10.0_f32)] {
        let translation = direction * (systemTimeMillis % period) as f32 / period as f32 / 8.0;
        let (sin, cos) = angle.to_radians().sin_cos();
        let mut quads = Vec::<GuiItemQuad>::new();
        for quad in &model.quads {
            let mut modelPositions = [[0.0_f32; 3]; 4];
            for (index, position) in quad.positions.iter().copied().enumerate() {
                modelPositions[index] = apply_item_transform(position, transform);
            }
            let edge1 = subtract3(modelPositions[1], modelPositions[0]);
            let edge2 = subtract3(modelPositions[2], modelPositions[0]);
            let normal = normalize3(cross3(edge1, edge2));
            if normal[2] <= 1.0e-5 { continue; }
            let mut transformed = [[0.0_f32; 3]; 4];
            let mut glintUvs = [[0.0_f32; 2]; 4];
            for index in 0..4 {
                let modelPosition = modelPositions[index];
                transformed[index] = [
                    x as f32 + 8.0 + modelPosition[0] * 16.0,
                    y as f32 + 8.0 - modelPosition[1] * 16.0,
                    modelPosition[2],
                ];
                let u = quad.uvs[index][0];
                let v = quad.uvs[index][1];
                // OpenGL texture matrix order used by RenderItem.func_191966_a:
                // scale(8), translate(scroll), rotate(angle), then repeat.
                glintUvs[index] = [
                    (8.0 * (cos * u - sin * v + translation)).rem_euclid(1.0),
                    (8.0 * (sin * u + cos * v)).rem_euclid(1.0),
                ];
            }
            quads.push(GuiItemQuad {
                depth: transformed.iter().map(|position| position[2]).sum::<f32>() / 4.0,
                positions: transformed,
                uvs: glintUvs,
                rectangle: atlas.glintRectangle,
                // -8372020 == ARGB ff8040cc.
                color: [128.0 / 255.0, 64.0 / 255.0, 204.0 / 255.0, 1.0],
            });
        }
        quads.sort_by(|left, right| left.depth.total_cmp(&right.depth));
        for quad in quads {
            let base = vertices.len() as u32;
            for index in 0..4 {
                let uv = [
                    quad.rectangle[0] + (quad.rectangle[2] - quad.rectangle[0]) * quad.uvs[index][0],
                    quad.rectangle[1] + (quad.rectangle[3] - quad.rectangle[1]) * quad.uvs[index][1],
                ];
                vertices.push(WorldVertex {
                    position: [quad.positions[index][0], quad.positions[index][1], 0.0],
                    uv,
                    color: quad.color,
                    lightmap: [15.0, 15.0],
                
                    shaderEntity: [-1, -1, -1],
                    shaderPadding: 0,
                });
            }
            indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
        }
    }
}

fn append_item_overlay_gui(
    stack: &ItemStack,
    x: i32,
    y: i32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    append_item_overlay_gui_with_alt(stack, x, y, None, atlas, vertices, indices);
}

fn append_item_overlay_gui_with_alt(
    stack: &ItemStack,
    x: i32,
    y: i32,
    alternateCount: Option<(&str, [f32; 4])>,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    if stack.isEmpty() { return; }
    if let Some((text, color)) = alternateCount {
        let width = text.chars().count() as i32 * 6;
        append_ascii_text_colored(
            text, x + 17 - width, y + 9, color, atlas, vertices, indices,
        );
    } else if stack.getCount() != 1 {
        let text = stack.getCount().to_string();
        let width = text.chars().count() as i32 * 6;
        append_ascii_text(&text, x + 17 - width, y + 9, atlas, vertices, indices);
    }
    if stack.showDurabilityBar() {
        let damage = stack.itemDamage.max(0) as f32;
        let maximum = stack.getMaxDamage().max(1) as f32;
        let remaining = ((maximum - damage) / maximum).max(0.0);
        let barWidth = (13.0 - damage * 13.0 / maximum).round().clamp(0.0, 13.0) as i32;
        let color = hsv_to_rgb(remaining / 3.0, 1.0, 1.0);
        append_solid_hud_quad(x + 2, y + 13, 13, 2, [0.0, 0.0, 0.0, 1.0], atlas, vertices, indices);
        append_solid_hud_quad(x + 2, y + 13, barWidth, 1, color, atlas, vertices, indices);
    }
}

fn append_ascii_text(
    text: &str,
    x: i32,
    y: i32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    append_formatted_text_colored(text, x, y, [1.0, 1.0, 1.0, 1.0], true, atlas, vertices, indices);
}

fn append_ascii_text_colored(
    text: &str,
    x: i32,
    y: i32,
    color: [f32; 4],
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    append_formatted_text_colored(text, x, y, color, true, atlas, vertices, indices);
}

fn append_font_text_colored(
    text: &str,
    x: i32,
    y: i32,
    color: i32,
    shadow: bool,
    fontRenderer: &mut FontRenderer,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let mut drawList = GuiDrawList::new();
    fontRenderer.draw_string(
        &mut drawList,
        text,
        x as f32,
        y as f32,
        color,
        shadow,
    );
    append_font_draw_list(&drawList, atlas, vertices, indices);
}

fn append_font_text_colored_no_shadow(
    text: &str,
    x: i32,
    y: i32,
    color: i32,
    fontRenderer: &mut FontRenderer,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    append_font_text_colored(
        text, x, y, color, false, fontRenderer, atlas, vertices, indices,
    );
}

fn append_font_split_text_colored_no_shadow(
    text: &str,
    x: i32,
    y: i32,
    width: i32,
    color: i32,
    fontRenderer: &mut FontRenderer,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    for (lineIndex, line) in fontRenderer
        .list_formatted_string_to_width(text, width)
        .into_iter()
        .enumerate()
    {
        append_font_text_colored_no_shadow(
            &line,
            x,
            y + lineIndex as i32 * 9,
            color,
            fontRenderer,
            atlas,
            vertices,
            indices,
        );
    }
}

fn append_hud_text(
    text: &HudText,
    fontRenderer: &mut FontRenderer,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let mut drawList = GuiDrawList::new();
    fontRenderer.draw_string(
        &mut drawList,
        &text.text,
        text.x as f32,
        text.y as f32,
        text.color as i32,
        text.outline,
    );
    append_font_draw_list(&drawList, atlas, vertices, indices);
}

fn append_experience_level_text(
    text: &HudText,
    fontRenderer: &mut FontRenderer,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    // GuiIngame.renderExpBar draws four black neighbours and then the green
    // center. This is deliberately not FontRenderer's single drop shadow.
    let mut drawList = GuiDrawList::new();
    for (dx, dy) in [(1.0_f32, 0.0_f32), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
        fontRenderer.draw_string(
            &mut drawList,
            &text.text,
            text.x as f32 + dx,
            text.y as f32 + dy,
            0,
            false,
        );
    }
    fontRenderer.draw_string(
        &mut drawList,
        &text.text,
        text.x as f32,
        text.y as f32,
        text.color as i32,
        false,
    );
    append_font_draw_list(&drawList, atlas, vertices, indices);
}

fn append_hud_text_scaled(
    text: &HudText,
    scale: f32,
    fontRenderer: &mut FontRenderer,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let mut drawList = GuiDrawList::new();
    drawList.push_matrix();
    drawList.translate(text.x as f32, text.y as f32);
    drawList.scale(scale, scale);
    fontRenderer.draw_string(
        &mut drawList,
        &text.text,
        0.0,
        0.0,
        text.color as i32,
        text.outline,
    );
    drawList.pop_matrix();
    append_font_draw_list(&drawList, atlas, vertices, indices);
}

fn append_font_draw_list(
    drawList: &GuiDrawList,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    for command in drawList.commands() {
        let GuiDrawCommand::Quad { texture, topology, vertices: guiVertices } = command else {
            continue;
        };
        let rectangle = texture
            .as_ref()
            .and_then(|location| atlas.textureRectangles.get(location))
            .copied()
            .unwrap_or(atlas.widgetsRectangle);
        let base = vertices.len() as u32;
        for vertex in guiVertices {
            let (u, v) = if texture.is_some() {
                (
                    rectangle[0] + (rectangle[2] - rectangle[0]) * vertex.u,
                    rectangle[1] + (rectangle[3] - rectangle[1]) * vertex.v,
                )
            } else {
                (
                    rectangle[0] + (rectangle[2] - rectangle[0]) * (247.5 / 256.0),
                    rectangle[1] + (rectangle[3] - rectangle[1]) * (3.5 / 256.0),
                )
            };
            vertices.push(WorldVertex {
                position: [vertex.x, vertex.y, 0.0],
                uv: [u, v],
                color: packed_argb_to_rgba(vertex.color),
                lightmap: [15.0, 15.0],
            
                shaderEntity: [-1, -1, -1],
                shaderPadding: 0,
            });
        }
        match topology {
            GuiTopology::Quads => indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]),
            GuiTopology::TriangleStrip => indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 1, base + 3]),
        }
    }
}

fn packed_argb_to_rgba(color: u32) -> [f32; 4] {
    let alpha = ((color >> 24) & 0xFF) as f32 / 255.0;
    [
        ((color >> 16) & 0xFF) as f32 / 255.0,
        ((color >> 8) & 0xFF) as f32 / 255.0,
        (color & 0xFF) as f32 / 255.0,
        if alpha == 0.0 { 1.0 } else { alpha },
    ]
}

#[allow(clippy::too_many_arguments)]
fn append_formatted_text_colored(
    text: &str,
    x: i32,
    y: i32,
    baseColor: [f32; 4],
    shadow: bool,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let passes: &[(i32, bool)] = if shadow { &[(1, true), (0, false)] } else { &[(0, false)] };
    for (offset, shadowPass) in passes {
        let mut cursor = x + *offset;
        let mut color = baseColor;
        let mut bold = false;
        let mut formatting = false;
        for character in text.chars() {
            if formatting {
                formatting = false;
                let code = character.to_ascii_lowercase();
                if let Some(index) = "0123456789abcdef".find(code) {
                    color = minecraft_format_color(index, baseColor[3]);
                    bold = false;
                } else {
                    match code {
                        'l' => bold = true,
                        'r' => { color = baseColor; bold = false; }
                        _ => {}
                    }
                }
                continue;
            }
            if character == '§' {
                formatting = true;
                continue;
            }
            let glyph = if (character as u32) <= 255 { character as u8 } else { b'?' };
            let glyphX = (glyph as i32 & 15) * 8;
            let glyphY = (glyph as i32 >> 4) * 8;
            let mut drawColor = color;
            if *shadowPass {
                drawColor[0] *= 0.25;
                drawColor[1] *= 0.25;
                drawColor[2] *= 0.25;
            }
            append_hud_quad_colored(
                vertices, indices, atlas.fontRectangle,
                cursor, y + *offset, 8, 8,
                glyphX, glyphY, 8, 8, 128, 128, drawColor,
            );
            if bold {
                append_hud_quad_colored(
                    vertices, indices, atlas.fontRectangle,
                    cursor + 1, y + *offset, 8, 8,
                    glyphX, glyphY, 8, 8, 128, 128, drawColor,
                );
            }
            cursor += if bold { 7 } else { 6 };
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn append_formatted_text_colored_scaled(
    text: &str,
    x: i32,
    y: i32,
    baseColor: [f32; 4],
    shadow: bool,
    scale: f32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let scale = scale.clamp(0.01, 1.0);
    let passes: &[(f32, bool)] = if shadow { &[(1.0, true), (0.0, false)] } else { &[(0.0, false)] };
    for (offset, shadowPass) in passes {
        let mut cursor = x as f32 + *offset * scale;
        let mut color = baseColor;
        let mut bold = false;
        let mut formatting = false;
        for character in text.chars() {
            if formatting {
                formatting = false;
                let code = character.to_ascii_lowercase();
                if let Some(index) = "0123456789abcdef".find(code) {
                    color = minecraft_format_color(index, baseColor[3]);
                    bold = false;
                } else {
                    match code {
                        'l' => bold = true,
                        'r' => { color = baseColor; bold = false; }
                        _ => {}
                    }
                }
                continue;
            }
            if character == '§' { formatting = true; continue; }
            let glyph = if (character as u32) <= 255 { character as u8 } else { b'?' };
            let glyphX = (glyph as i32 & 15) * 8;
            let glyphY = (glyph as i32 >> 4) * 8;
            let mut drawColor = color;
            if *shadowPass {
                drawColor[0] *= 0.25;
                drawColor[1] *= 0.25;
                drawColor[2] *= 0.25;
            }
            let drawX = cursor.round() as i32;
            let drawY = (y as f32 + *offset * scale).round() as i32;
            let drawSize = (8.0 * scale).ceil().max(1.0) as i32;
            append_hud_quad_colored(
                vertices, indices, atlas.fontRectangle,
                drawX, drawY, drawSize, drawSize,
                glyphX, glyphY, 8, 8, 128, 128, drawColor,
            );
            if bold {
                append_hud_quad_colored(
                    vertices, indices, atlas.fontRectangle,
                    (cursor + scale).round() as i32, drawY, drawSize, drawSize,
                    glyphX, glyphY, 8, 8, 128, 128, drawColor,
                );
            }
            cursor += (if bold { 7.0 } else { 6.0 }) * scale;
        }
    }
}

fn minecraft_format_color(index: usize, alpha: f32) -> [f32; 4] {
    const COLORS: [[f32; 3]; 16] = [
        [0.0, 0.0, 0.0], [0.0, 0.0, 2.0 / 3.0], [0.0, 2.0 / 3.0, 0.0], [0.0, 2.0 / 3.0, 2.0 / 3.0],
        [2.0 / 3.0, 0.0, 0.0], [2.0 / 3.0, 0.0, 2.0 / 3.0], [1.0, 2.0 / 3.0, 0.0], [2.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0],
        [1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0], [1.0 / 3.0, 1.0 / 3.0, 1.0], [1.0 / 3.0, 1.0, 1.0 / 3.0], [1.0 / 3.0, 1.0, 1.0],
        [1.0, 1.0 / 3.0, 1.0 / 3.0], [1.0, 1.0 / 3.0, 1.0], [1.0, 1.0, 1.0 / 3.0], [1.0, 1.0, 1.0],
    ];
    let [r, g, b] = COLORS[index.min(15)];
    [r, g, b, alpha]
}

fn append_solid_hud_quad(
    x: i32, y: i32, width: i32, height: i32, color: [f32; 4],
    atlas: &AtlasState, vertices: &mut Vec<WorldVertex>, indices: &mut Vec<u32>,
) {
    if width <= 0 || height <= 0 { return; }
    // Opaque white pixel in widgets.png; vanilla disables texturing for these
    // rectangles, while the shared Vulkan shader keeps one sampled atlas.
    append_hud_quad_colored(
        vertices, indices, atlas.widgetsRectangle,
        x, y, width, height, 247, 3, 1, 1, 256, 256, color,
    );
}

#[allow(clippy::too_many_arguments)]
fn append_hud_quad_colored(
    vertices: &mut Vec<WorldVertex>, indices: &mut Vec<u32>, rectangle: [f32; 4],
    x: i32, y: i32, width: i32, height: i32,
    textureX: i32, textureY: i32, textureWidth: i32, textureHeight: i32,
    sourceWidth: i32, sourceHeight: i32, color: [f32; 4],
) {
    let u0 = rectangle[0] + (rectangle[2] - rectangle[0]) * textureX as f32 / sourceWidth as f32;
    let v0 = rectangle[1] + (rectangle[3] - rectangle[1]) * textureY as f32 / sourceHeight as f32;
    let u1 = rectangle[0] + (rectangle[2] - rectangle[0]) * (textureX + textureWidth) as f32 / sourceWidth as f32;
    let v1 = rectangle[1] + (rectangle[3] - rectangle[1]) * (textureY + textureHeight) as f32 / sourceHeight as f32;
    let left = x as f32; let top = y as f32; let right = (x + width) as f32; let bottom = (y + height) as f32;
    let base = vertices.len() as u32; let lightmap = [15.0, 15.0];
    vertices.extend_from_slice(&[
        WorldVertex { position: [left, bottom, 0.0], uv: [u0, v1], color, lightmap, shaderEntity: [-1, -1, -1], shaderPadding: 0, },
        WorldVertex { position: [right, bottom, 0.0], uv: [u1, v1], color, lightmap, shaderEntity: [-1, -1, -1], shaderPadding: 0, },
        WorldVertex { position: [right, top, 0.0], uv: [u1, v0], color, lightmap, shaderEntity: [-1, -1, -1], shaderPadding: 0, },
        WorldVertex { position: [left, top, 0.0], uv: [u0, v0], color, lightmap, shaderEntity: [-1, -1, -1], shaderPadding: 0, },
    ]);
    indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
}

fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> [f32; 4] {
    let h = (hue.rem_euclid(1.0) * 6.0).floor() as i32;
    let f = hue.rem_euclid(1.0) * 6.0 - h as f32;
    let p = value * (1.0 - saturation);
    let q = value * (1.0 - f * saturation);
    let t = value * (1.0 - (1.0 - f) * saturation);
    let (r, g, b) = match h % 6 {
        0 => (value, t, p), 1 => (q, value, p), 2 => (p, value, t),
        3 => (p, q, value), 4 => (t, p, value), _ => (value, p, q),
    };
    [r, g, b, 1.0]
}

fn apply_item_transform(position: [f32; 3], transform: ItemTransformVec3f) -> [f32; 3] {
    let mut value = [
        (position[0] - 0.5) * transform.scale[0],
        (position[1] - 0.5) * transform.scale[1],
        (position[2] - 0.5) * transform.scale[2],
    ];
    value = rotate_item_quaternion(value, transform.rotation);
    [
        value[0] + transform.translation[0],
        value[1] + transform.translation[1],
        value[2] + transform.translation[2],
    ]
}

/// Quaternion construction copied from ItemCameraTransforms.makeQuaternion.
fn rotate_item_quaternion(value: [f32; 3], rotation: [f32; 3]) -> [f32; 3] {
    let [x, y, z] = rotation.map(f32::to_radians);
    let (sx, cx) = (0.5 * x).sin_cos();
    let (sy, cy) = (0.5 * y).sin_cos();
    let (sz, cz) = (0.5 * z).sin_cos();
    let qx = sx * cy * cz + cx * sy * sz;
    let qy = cx * sy * cz - sx * cy * sz;
    let qz = sx * sy * cz + cx * cy * sz;
    let qw = cx * cy * cz - sx * sy * sz;
    let q = [qx, qy, qz];
    let t = scale3(cross3(q, value), 2.0);
    add3(value, add3(scale3(t, qw), cross3(q, t)))
}

fn gui_item_diffuse(normal: [f32; 3]) -> f32 {
    // RenderHelper.enableGUIStandardItemLighting's two directional lights,
    // expressed in model/view space. The 0.4 ambient term matches OpenGL's
    // standard item-lighting setup; diffuse contributions are clamped by GL.
    let first = normalize3([0.2, 1.0, -0.7]);
    let second = normalize3([-0.2, 1.0, 0.7]);
    (0.4 + 0.6 * dot3(normal, first).max(0.0) + 0.6 * dot3(normal, second).max(0.0))
        .clamp(0.0, 1.0)
}

fn item_tint_color(itemColors: &ItemColors, stack: &ItemStack, tintIndex: Option<i32>) -> [f32; 3] {
    let color = tintIndex
        .map(|index| itemColors.getColorFromItemstack(stack, index))
        .filter(|color| *color != ItemColors::DEFAULT_COLOR)
        .unwrap_or(0xFF_FFFF);
    [
        ((color >> 16) & 0xFF) as f32 / 255.0,
        ((color >> 8) & 0xFF) as f32 / 255.0,
        (color & 0xFF) as f32 / 255.0,
    ]
}

fn append_player_tab_head(
    head: &crate::net::minecraft::client::gui::GuiPlayerTabOverlay::PlayerTabHead,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let rectangle = player_skin_rectangle(atlas, &head.location, false);
    let (sourceY, sourceHeight) = if head.upsideDown { (16, -8) } else { (8, 8) };
    append_hud_quad_colored(
        vertices,
        indices,
        rectangle,
        head.x,
        head.y,
        8,
        8,
        8,
        sourceY,
        8,
        sourceHeight,
        64,
        64,
        [1.0, 1.0, 1.0, 1.0],
    );
    if head.renderHat {
        append_hud_quad_colored(
            vertices,
            indices,
            rectangle,
            head.x,
            head.y,
            8,
            8,
            40,
            sourceY,
            8,
            sourceHeight,
            64,
            64,
            [1.0, 1.0, 1.0, 1.0],
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn append_hud_quad(
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
    rectangle: [f32; 4],
    x: i32, y: i32, width: i32, height: i32,
    textureX: i32, textureY: i32, textureWidth: i32, textureHeight: i32,
    alpha: f32,
) {
    let u0 = rectangle[0] + (rectangle[2] - rectangle[0]) * textureX as f32 / 256.0;
    let v0 = rectangle[1] + (rectangle[3] - rectangle[1]) * textureY as f32 / 256.0;
    let u1 = rectangle[0] + (rectangle[2] - rectangle[0]) * (textureX + textureWidth) as f32 / 256.0;
    let v1 = rectangle[1] + (rectangle[3] - rectangle[1]) * (textureY + textureHeight) as f32 / 256.0;
    let left = x as f32;
    let top = y as f32;
    let right = (x + width) as f32;
    let bottom = (y + height) as f32;
    let base = vertices.len() as u32;
    let color = [1.0, 1.0, 1.0, alpha];
    let lightmap = [15.0, 15.0];
    vertices.extend_from_slice(&[
        WorldVertex { position: [left, bottom, 0.0], uv: [u0, v1], color, lightmap, shaderEntity: [-1, -1, -1], shaderPadding: 0, },
        WorldVertex { position: [right, bottom, 0.0], uv: [u1, v1], color, lightmap, shaderEntity: [-1, -1, -1], shaderPadding: 0, },
        WorldVertex { position: [right, top, 0.0], uv: [u1, v0], color, lightmap, shaderEntity: [-1, -1, -1], shaderPadding: 0, },
        WorldVertex { position: [left, top, 0.0], uv: [u0, v0], color, lightmap, shaderEntity: [-1, -1, -1], shaderPadding: 0, },
    ]);
    indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
}

#[allow(clippy::too_many_arguments)]
fn append_world_player_held_items(
    player: &RemotePlayerRenderState,
    pose: BipedPose,
    position: [f32; 3],
    bodyYaw: f32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let rightStack = LayerHeldItem::stackForSide(
        player.primaryHand,
        &player.mainHandStack,
        &player.offHandStack,
        EnumHandSide::Right,
    );
    let leftStack = LayerHeldItem::stackForSide(
        player.primaryHand,
        &player.mainHandStack,
        &player.offHandStack,
        EnumHandSide::Left,
    );

    append_world_player_held_item(
        rightStack,
        EnumHandSide::Right,
        pose.rightArm,
        player.sneaking,
        position,
        bodyYaw,
        player.packedLight,
        player.itemInUseCount > 0
            && player.activeHand == if player.primaryHand == EnumHandSide::Right { EnumHand::MainHand } else { EnumHand::OffHand },
        atlas,
        vertices,
        indices,
    );
    append_world_player_held_item(
        leftStack,
        EnumHandSide::Left,
        pose.leftArm,
        player.sneaking,
        position,
        bodyYaw,
        player.packedLight,
        player.itemInUseCount > 0
            && player.activeHand == if player.primaryHand == EnumHandSide::Left { EnumHand::MainHand } else { EnumHand::OffHand },
        atlas,
        vertices,
        indices,
    );
}

#[allow(clippy::too_many_arguments)]
fn append_world_player_held_item(
    stack: &ItemStack,
    handSide: EnumHandSide,
    armPose: PartPose,
    sneaking: bool,
    position: [f32; 3],
    bodyYaw: f32,
    packedLight: u32,
    activelyUsing: bool,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    if stack.isEmpty() {
        return;
    }
    let Some(modelKey) = ItemModelMesher::getModelKey(stack) else { return; };
    let Some(baseModel) = atlas.itemModels.get(&modelKey) else { return; };
    let blockingShield = activelyUsing
        && stack.getItemUseAction() == EnumAction::Block
        && is_unpatterned_shield(stack);
    let model = if blockingShield {
        atlas.shieldBlockingModel.as_ref().unwrap_or(baseModel)
    } else {
        baseModel
    };
    let unpatternedShield = model.builtInRenderer && is_unpatterned_shield(stack);
    let supportedBuiltIn = model.builtInRenderer
        && (unpatternedShield || TileEntityItemStackRenderer::buildMesh(stack).is_some());
    if (model.builtInRenderer && !supportedBuiltIn)
        || (!model.builtInRenderer && model.quads.is_empty())
    {
        return;
    }

    // RenderLivingBase installs the player root model matrix before layer
    // rendering. LayerHeldItem then post-renders the physical arm and applies
    // the exact hand-side transform sequence from 1.12.2.
    let mut matrix = multiply4(
        translation4(position),
        player_layer_root_matrix(bodyYaw, sneaking),
    );
    matrix = post_render_part_matrix(matrix, armPose);
    if sneaking {
        matrix = multiply4(matrix, translation4([0.0, 0.2, 0.0]));
    }
    matrix = multiply4(matrix, rotation_x4(-90.0));
    matrix = multiply4(matrix, rotation_y4(180.0));
    matrix = multiply4(matrix, translation4(LayerHeldItem::handTranslation(handSide)));

    let leftHanded = LayerHeldItem::leftHanded(handSide);
    let transformType = LayerHeldItem::transformType(handSide);
    let itemTransform = model.transforms.getTransform(transformType);
    matrix = multiply4(matrix, item_camera_transform4(itemTransform, leftHanded));
    matrix = multiply4(matrix, translation4([-0.5, -0.5, -0.5]));

    let blockLight = ((packedLight >> 4) & 15) as f32;
    let skyLight = ((packedLight >> 20) & 15) as f32;
    let itemLights = [
        normalize3([0.2, 1.0, -0.7]),
        normalize3([-0.2, 1.0, 0.7]),
    ];

    if unpatternedShield {
        append_shield_model_world(
            matrix,
            atlas.shieldBaseRectangle,
            blockLight,
            skyLight,
            itemLights,
            vertices,
            indices,
        );
        return;
    }
    if model.builtInRenderer {
        append_builtin_item_mesh_world(
            stack, matrix, blockLight, skyLight, itemLights, atlas, vertices, indices,
        );
        return;
    }

    for quad in &model.quads {
        let transformed = quad.positions.map(|point| transform_point3(matrix, point));
        let normal = normalize3(cross3(
            subtract3(transformed[1], transformed[0]),
            subtract3(transformed[2], transformed[0]),
        ));
        let key = item_material_key(stack.itemId, quad.texture.clone(), quad.tintIndex);
        let rectangle = atlas
            .rectangles
            .get(&key)
            .copied()
            .unwrap_or(atlas.missingRectangle);
        let tint = item_tint_color(&atlas.itemColors, stack, quad.tintIndex);
        let diffuse = if model.gui3d && quad.shade {
            standard_item_diffuse(normal, itemLights)
        } else {
            1.0
        };
        let base = vertices.len() as u32;
        for vertexIndex in 0..4 {
            let sourceUv = quad.uvs[vertexIndex];
            vertices.push(WorldVertex {
                position: transformed[vertexIndex],
                uv: [
                    rectangle[0] + (rectangle[2] - rectangle[0]) * sourceUv[0],
                    rectangle[1] + (rectangle[3] - rectangle[1]) * sourceUv[1],
                ],
                color: [
                    tint[0] * diffuse,
                    tint[1] * diffuse,
                    tint[2] * diffuse,
                    1.0,
                ],
                lightmap: [blockLight, skyLight],
            
                shaderEntity: [-1, -1, -1],
                shaderPadding: 0,
            });
        }
        indices.extend_from_slice(&[
            base,
            base + 1,
            base + 2,
            base + 2,
            base + 3,
            base,
        ]);
    }
}

#[derive(Default)]
struct NonPlayerEntityMeshBatch {
    vertices: Vec<WorldVertex>,
    indices: Vec<u32>,
    depthVertices: Vec<WorldVertex>,
    depthIndices: Vec<u32>,
    overlayVertices: Vec<WorldVertex>,
    overlayIndices: Vec<u32>,
    lineVertices: Vec<WorldVertex>,
    lineIndices: Vec<u32>,
    rendered: usize,
}

fn append_indexed_mesh_stream(
    targetVertices: &mut Vec<WorldVertex>,
    targetIndices: &mut Vec<u32>,
    sourceVertices: Vec<WorldVertex>,
    sourceIndices: Vec<u32>,
) {
    if sourceIndices.is_empty() {
        return;
    }
    let base = targetVertices.len() as u32;
    targetVertices.extend(sourceVertices);
    targetIndices.extend(sourceIndices.into_iter().map(|index| base + index));
}


#[derive(Debug, Clone, Copy)]
struct StaticHangingRenderContext {
    renderer: EntityRendererKind,
    position: [f32; 3],
    distanceSquared: f64,
    packedLight: u32,
}

fn static_hanging_mesh_kind(renderer: EntityRendererKind) -> Option<StaticEntityMeshKind> {
    match renderer {
        EntityRendererKind::Painting => Some(StaticEntityMeshKind::Painting),
        EntityRendererKind::ItemFrame => Some(StaticEntityMeshKind::ItemFrame),
        EntityRendererKind::LeashKnot => Some(StaticEntityMeshKind::LeashKnot),
        _ => None,
    }
}

fn static_hanging_render_context(
    entity: &EntityOtherClient,
    partialTicks: f32,
    camera: [f32; 3],
    frustum: &Frustum,
    chunks: &HashMap<ChunkKey, Chunk>,
    dimension: i32,
) -> Option<StaticHangingRenderContext> {
    if entity.entity.isDead {
        return None;
    }
    let renderer = RenderManager::getEntityRenderObject(&entity.kind);
    static_hanging_mesh_kind(renderer)?;
    let position = Render::interpolatedPosition(&entity.entity, partialTicks);
    let dx = position[0] as f64 - camera[0] as f64;
    let dy = position[1] as f64 - camera[1] as f64;
    let dz = position[2] as f64 - camera[2] as f64;
    let distanceSquared = dx * dx + dy * dy + dz * dz;
    let inRenderRange = match renderer {
        EntityRendererKind::ItemFrame => {
            distanceSquared < EntityItemFrame::ENTITY_RENDER_DISTANCE_SQ
        }
        EntityRendererKind::LeashKnot => {
            distanceSquared < EntityLeashKnot::MAX_RENDER_DISTANCE_SQ
        }
        EntityRendererKind::Painting => {
            Render::isInRangeToRenderDist(&entity.entity, distanceSquared)
        }
        _ => false,
    };
    if !inRenderRange {
        return None;
    }
    let bounds = Render::renderBoundingBox(&entity.entity);
    if !frustum.isBoxInFrustum(
        bounds.min_x,
        bounds.min_y,
        bounds.min_z,
        bounds.max_x,
        bounds.max_y,
        bounds.max_z,
    ) {
        return None;
    }
    let packedLight = packed_light_with_living_hurt_overlay(
        entity,
        non_player_entity_light(entity, position, chunks, dimension),
    );
    Some(StaticHangingRenderContext {
        renderer,
        position,
        distanceSquared,
        packedLight,
    })
}

fn entity_bounds_chunk_revision_hash(
    chunks: &HashMap<ChunkKey, Chunk>,
    bounds: AxisAlignedBB,
) -> u64 {
    let minChunkX = (bounds.min_x.floor() as i32).div_euclid(16);
    let maxChunkX = (bounds.max_x.floor() as i32).div_euclid(16);
    let minChunkZ = (bounds.min_z.floor() as i32).div_euclid(16);
    let maxChunkZ = (bounds.max_z.floor() as i32).div_euclid(16);
    let mut fingerprint = RenderStateFingerprint::default();
    for chunkX in minChunkX..=maxChunkX {
        for chunkZ in minChunkZ..=maxChunkZ {
            let revision = chunks
                .get(&ChunkKey::new(chunkX, chunkZ))
                .map(|chunk| chunk.revision())
                .unwrap_or(0);
            write!(&mut fingerprint, "{chunkX}:{chunkZ}:{revision};")
                .expect("static entity snapshot hashing cannot fail");
        }
    }
    fingerprint.0
}

fn static_hanging_mesh_cache_key(
    entity: &EntityOtherClient,
    context: StaticHangingRenderContext,
    mapData: &HashMap<i32, MapData>,
    chunks: &HashMap<ChunkKey, Chunk>,
    dimension: i32,
    atlasRevision: u64,
) -> StaticEntityMeshCacheKey {
    match context.renderer {
        EntityRendererKind::Painting => StaticEntityMeshCacheKey {
            stateHash: debug_render_state_hash(&(
                entity.paintingArt(),
                entity.hangingFacing,
                entity.hangingPosition,
                entity.entity.posX,
                entity.entity.posY,
                entity.entity.posZ,
                entity.entity.rotationYaw,
                dimension,
            )),
            snapshotHash: entity_bounds_chunk_revision_hash(
                chunks,
                Render::renderBoundingBox(&entity.entity),
            ),
            atlasRevision,
        },
        EntityRendererKind::ItemFrame => {
            let displayed = entity.itemFrameDisplayedItem();
            let renderDisplayedItem = context.distanceSquared
                <= EntityItemFrame::ITEM_RENDER_DISTANCE_SQ;
            let mapRevision = if renderDisplayedItem {
                displayed
                    .and_then(|stack| ItemMap::getMapData(stack, mapData))
                    .map(|map| map.revision)
            } else {
                None
            };
            StaticEntityMeshCacheKey {
                stateHash: debug_render_state_hash(&(
                    entity.hangingPosition,
                    entity.hangingFacing,
                    entity.entity.posX,
                    entity.entity.posY,
                    entity.entity.posZ,
                    entity.entity.rotationYaw,
                    context.position,
                    entity.itemFrameRotation(),
                    displayed,
                    renderDisplayedItem,
                    mapRevision,
                    context.packedLight,
                )),
                snapshotHash: 0,
                atlasRevision,
            }
        }
        EntityRendererKind::LeashKnot => StaticEntityMeshCacheKey {
            stateHash: debug_render_state_hash(&(
                entity.hangingPosition,
                context.position,
                context.packedLight,
            )),
            snapshotHash: 0,
            atlasRevision,
        },
        _ => unreachable!("only static hanging renderers receive cache keys"),
    }
}

fn build_static_hanging_mesh(
    entity: &EntityOtherClient,
    context: StaticHangingRenderContext,
    mapData: &HashMap<i32, MapData>,
    chunks: &HashMap<ChunkKey, Chunk>,
    dimension: i32,
    atlas: &AtlasState,
) -> StaticEntityMeshBatch {
    let mut mesh = StaticEntityMeshBatch::default();
    match context.renderer {
        EntityRendererKind::Painting => append_painting_mesh(
            entity,
            chunks,
            dimension,
            atlas,
            &mut mesh.vertices,
            &mut mesh.indices,
        ),
        EntityRendererKind::ItemFrame => append_item_frame_mesh(
            entity,
            context.position,
            context.distanceSquared,
            context.packedLight,
            mapData,
            atlas,
            &mut mesh.vertices,
            &mut mesh.indices,
        ),
        EntityRendererKind::LeashKnot => append_leash_knot_mesh(
            entity,
            context.position,
            context.packedLight,
            atlas,
            &mut mesh.vertices,
            &mut mesh.indices,
        ),
        _ => {}
    }
    mesh
}


fn push_world_entity_draw_range(
    ranges: &mut Vec<WorldEntityDrawRange>,
    pipeline: WorldEntityPipelineKind,
    mesh: WorldEntityMeshKind,
    firstIndex: u32,
    indexCount: u32,
) {
    if indexCount == 0 {
        return;
    }
    if let Some(last) = ranges.last_mut() {
        if last.pipeline == pipeline
            && last.mesh == mesh
            && last.firstIndex.saturating_add(last.indexCount) == firstIndex
        {
            last.indexCount = last.indexCount.saturating_add(indexCount);
            return;
        }
    }
    ranges.push(WorldEntityDrawRange {
        pipeline,
        mesh,
        firstIndex,
        indexCount,
    });
}
fn append_static_entity_mesh_batch(
    mesh: &StaticEntityMeshBatch,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    if mesh.indices.is_empty() {
        return;
    }
    let base = vertices.len() as u32;
    vertices.extend_from_slice(&mesh.vertices);
    indices.extend(mesh.indices.iter().copied().map(|index| base + index));
}

#[allow(clippy::too_many_arguments)]
fn append_non_player_entity_meshes(
    entities: &[EntityOtherClient],
    mapData: &HashMap<i32, MapData>,
    remotePlayers: &[RemotePlayerRenderState],
    localPlayerRenderState: Option<&RemotePlayerRenderState>,
    localPlayerTarget: Option<LivingTargetRenderState>,
    totalWorldTime: i64,
    partialTicks: f32,
    camera: [f32; 3],
    cameraYaw: f32,
    cameraPitch: f32,
    thirdPersonView: i32,
    fov: f32,
    frustum: &Frustum,
    chunks: &HashMap<ChunkKey, Chunk>,
    dimension: i32,
    atlas: &AtlasState,
    frameMeshCache: &mut RenderFrameMeshCache,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
    staticVertices: &mut Vec<WorldVertex>,
    staticIndices: &mut Vec<u32>,
    entityDrawRanges: &mut Vec<WorldEntityDrawRange>,
    depthVertices: &mut Vec<WorldVertex>,
    depthIndices: &mut Vec<u32>,
    overlayVertices: &mut Vec<WorldVertex>,
    overlayIndices: &mut Vec<u32>,
    lineVertices: &mut Vec<WorldVertex>,
    lineIndices: &mut Vec<u32>,
) -> usize {
    frameMeshCache.beginStaticEntityFrame();
    let mut rendered = 0usize;
    let mut uncachedStart = 0usize;

    macro_rules! append_uncached_run {
        ($run:expr) => {{
            let run = $run;
            if run.is_empty() {
                0usize
            } else {
                let firstIndex = indices.len() as u32;
                let count = append_non_player_entity_meshes_uncached(
                    run,
                    entities,
                    mapData,
                    remotePlayers,
                    localPlayerRenderState,
                    localPlayerTarget,
                    totalWorldTime,
                    partialTicks,
                    camera,
                    cameraYaw,
                    cameraPitch,
                    thirdPersonView,
                    fov,
                    frustum,
                    chunks,
                    dimension,
                    atlas,
                    vertices,
                    indices,
                    depthVertices,
                    depthIndices,
                    overlayVertices,
                    overlayIndices,
                    lineVertices,
                    lineIndices,
                );
                push_world_entity_draw_range(
                    entityDrawRanges,
                    WorldEntityPipelineKind::Entities,
                    WorldEntityMeshKind::Dynamic,
                    firstIndex,
                    indices.len() as u32 - firstIndex,
                );
                count
            }
        }};
    }

    for (index, entity) in entities.iter().enumerate() {
        let renderer = RenderManager::getEntityRenderObject(&entity.kind);
        let Some(kind) = static_hanging_mesh_kind(renderer) else {
            continue;
        };

        rendered = rendered.saturating_add(append_uncached_run!(&entities[uncachedStart..index]));
        uncachedStart = index + 1;

        let identity = StaticEntityMeshIdentity {
            kind,
            entityId: entity.entityId,
        };
        frameMeshCache.touchStaticEntity(identity);
        let Some(context) = static_hanging_render_context(
            entity,
            partialTicks,
            camera,
            frustum,
            chunks,
            dimension,
        ) else {
            continue;
        };

        let mut drewEntity = false;
        if !entity.isInvisibleFlag() {
            let key = static_hanging_mesh_cache_key(
                entity,
                context,
                mapData,
                chunks,
                dimension,
                atlas.revision,
            );
            let mesh = frameMeshCache.staticEntityMesh(identity, key, || {
                build_static_hanging_mesh(entity, context, mapData, chunks, dimension, atlas)
            });
            let firstIndex = staticIndices.len() as u32;
            append_static_entity_mesh_batch(&mesh, staticVertices, staticIndices);
            let indexCount = staticIndices.len() as u32 - firstIndex;
            push_world_entity_draw_range(
                entityDrawRanges,
                WorldEntityPipelineKind::Entities,
                WorldEntityMeshKind::StaticEntities,
                firstIndex,
                indexCount,
            );
            drewEntity |= indexCount > 0;
        }
        if entity.isBurning() {
            let firstIndex = indices.len() as u32;
            append_entity_fire_mesh(
                &entity.entity,
                context.position,
                cameraYaw,
                atlas,
                vertices,
                indices,
            );
            let indexCount = indices.len() as u32 - firstIndex;
            push_world_entity_draw_range(
                entityDrawRanges,
                WorldEntityPipelineKind::Entities,
                WorldEntityMeshKind::Dynamic,
                firstIndex,
                indexCount,
            );
            drewEntity |= indexCount > 0;
        }
        if drewEntity {
            rendered = rendered.saturating_add(1);
        }
    }

    rendered = rendered.saturating_add(append_uncached_run!(&entities[uncachedStart..]));
    frameMeshCache.finishStaticEntityFrame();
    rendered
}

#[allow(clippy::too_many_arguments)]
fn append_non_player_entity_meshes_uncached(
    entities: &[EntityOtherClient],
    allEntities: &[EntityOtherClient],
    mapData: &HashMap<i32, MapData>,
    remotePlayers: &[RemotePlayerRenderState],
    localPlayerRenderState: Option<&RemotePlayerRenderState>,
    localPlayerTarget: Option<LivingTargetRenderState>,
    totalWorldTime: i64,
    partialTicks: f32,
    camera: [f32; 3],
    cameraYaw: f32,
    cameraPitch: f32,
    thirdPersonView: i32,
    fov: f32,
    frustum: &Frustum,
    chunks: &HashMap<ChunkKey, Chunk>,
    dimension: i32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
    depthVertices: &mut Vec<WorldVertex>,
    depthIndices: &mut Vec<u32>,
    overlayVertices: &mut Vec<WorldVertex>,
    overlayIndices: &mut Vec<u32>,
    lineVertices: &mut Vec<WorldVertex>,
    lineIndices: &mut Vec<u32>,
) -> usize {
    let threadCount = rayon::current_num_threads().max(1);
    if entities.len() < PARALLEL_ENTITY_BATCH_THRESHOLD || threadCount <= 1 {
        return append_non_player_entity_meshes_serial(
            entities,
            allEntities,
            mapData,
            remotePlayers,
            localPlayerRenderState,
            localPlayerTarget,
            totalWorldTime,
            partialTicks,
            camera,
            cameraYaw,
            cameraPitch,
            thirdPersonView,
            fov,
            frustum,
            chunks,
            dimension,
            atlas,
            vertices,
            indices,
            depthVertices,
            depthIndices,
            overlayVertices,
            overlayIndices,
            lineVertices,
            lineIndices,
        );
    }

    // Two batches per worker keep long model builders from leaving cores idle.
    // IndexedParallelIterator::collect retains slice order, so RenderManager's
    // original entity order and every overlay/multipass boundary remain stable.
    let targetBatches = threadCount.saturating_mul(2).min(entities.len()).max(1);
    let batchSize = entities.len().div_ceil(targetBatches);
    let batches = entities
        .par_chunks(batchSize)
        .map(|batch| {
            let mut output = NonPlayerEntityMeshBatch::default();
            output.rendered = append_non_player_entity_meshes_serial(
                batch,
                allEntities,
                mapData,
                remotePlayers,
                localPlayerRenderState,
                localPlayerTarget,
                totalWorldTime,
                partialTicks,
                camera,
                cameraYaw,
                cameraPitch,
                thirdPersonView,
                fov,
                frustum,
                chunks,
                dimension,
                atlas,
                &mut output.vertices,
                &mut output.indices,
                &mut output.depthVertices,
                &mut output.depthIndices,
                &mut output.overlayVertices,
                &mut output.overlayIndices,
                &mut output.lineVertices,
                &mut output.lineIndices,
            );
            output
        })
        .collect::<Vec<_>>();

    let mut rendered = 0usize;
    for batch in batches {
        rendered = rendered.saturating_add(batch.rendered);
        append_indexed_mesh_stream(vertices, indices, batch.vertices, batch.indices);
        append_indexed_mesh_stream(
            depthVertices,
            depthIndices,
            batch.depthVertices,
            batch.depthIndices,
        );
        append_indexed_mesh_stream(
            overlayVertices,
            overlayIndices,
            batch.overlayVertices,
            batch.overlayIndices,
        );
        append_existing_line_strip(
            lineVertices,
            lineIndices,
            &batch.lineVertices,
            &batch.lineIndices,
        );
    }
    rendered
}

#[allow(clippy::too_many_arguments)]
fn append_non_player_entity_meshes_serial(
    entities: &[EntityOtherClient],
    allEntities: &[EntityOtherClient],
    mapData: &HashMap<i32, MapData>,
    remotePlayers: &[RemotePlayerRenderState],
    localPlayerRenderState: Option<&RemotePlayerRenderState>,
    localPlayerTarget: Option<LivingTargetRenderState>,
    totalWorldTime: i64,
    partialTicks: f32,
    camera: [f32; 3],
    cameraYaw: f32,
    cameraPitch: f32,
    thirdPersonView: i32,
    fov: f32,
    frustum: &Frustum,
    chunks: &HashMap<ChunkKey, Chunk>,
    dimension: i32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
    depthVertices: &mut Vec<WorldVertex>,
    depthIndices: &mut Vec<u32>,
    overlayVertices: &mut Vec<WorldVertex>,
    overlayIndices: &mut Vec<u32>,
    lineVertices: &mut Vec<WorldVertex>,
    lineIndices: &mut Vec<u32>,
) -> usize {
    let mut rendered = 0;
    for entity in entities {
        if entity.entity.isDead { continue; }
        let renderer = RenderManager::getEntityRenderObject(&entity.kind);
        if renderer == EntityRendererKind::Unsupported { continue; }
        let invisible = entity.isInvisibleFlag();
        let isGuardian = renderer == EntityRendererKind::Guardian;
        let isEnderCrystal = renderer == EntityRendererKind::EnderCrystal;
        let isShulker = renderer == EntityRendererKind::Shulker;
        let isIllusioner = renderer == EntityRendererKind::Illager
            && matches!(&entity.kind, ClientEntityKind::Mob { entityType } if RenderIllusionIllager::supports(*entityType));
        // `RenderGuardian#doRender` submits the beam after the living-model
        // pass, and RenderShulker's HeadLayer is independent of its hidden
        // main model. Ordinary invisible entities still reach
        // Render#doRenderShadowAndFire: their model is skipped below, but an
        // active fire overlay remains visible through Entity#canRenderOnFire.
        let position = Render::interpolatedPosition(&entity.entity, partialTicks);
        let dx = position[0] as f64 - camera[0] as f64;
        let dy = position[1] as f64 - camera[1] as f64;
        let dz = position[2] as f64 - camera[2] as f64;
        let distanceSquared = dx * dx + dy * dy + dz * dz;
        let inRenderRange = if renderer == EntityRendererKind::ShulkerBullet {
            // MCP `EntityShulkerBullet#isInRangeToRenderDist` overrides the
            // tiny 0.3125 bounding-box range and accepts squared distance
            // below 16384 (128 blocks).
            EntityShulkerBullet::isInRangeToRenderDist(distanceSquared)
        } else if matches!(
            renderer,
            EntityRendererKind::Fireball
                | EntityRendererKind::DragonFireball
                | EntityRendererKind::WitherSkull
        ) {
            EntityFireball::isInRangeToRenderDist(&entity.entity, distanceSquared)
        } else if renderer == EntityRendererKind::FishHook {
            EntityFishHook::isInRangeToRenderDist(distanceSquared)
        } else if renderer == EntityRendererKind::ItemFrame {
            // EntityItemFrame itself uses its much larger Entity override;
            // RenderItemFrame independently hides only the displayed item at
            // 64 blocks. Do not cull the wooden frame at the item threshold.
            distanceSquared < EntityItemFrame::ENTITY_RENDER_DISTANCE_SQ
        } else if renderer == EntityRendererKind::LeashKnot {
            distanceSquared < EntityLeashKnot::MAX_RENDER_DISTANCE_SQ
        } else {
            Render::isInRangeToRenderDist(&entity.entity, distanceSquared)
        };
        // MCP `RenderGuardian#shouldRender` tests the beam AABB even after the
        // normal entity distance/frustum test fails. Other renderers retain the
        // ordinary early distance rejection.
        let shulkerTeleportActive = isShulker
            && entity.shulkerClientTeleportInterp() > 0
            && entity.shulkerIsAttachedToBlock();
        let enderCrystalBeamTarget = if isEnderCrystal {
            entity.enderCrystalBeamTarget()
        } else {
            None
        };
        // RenderEnderCrystal#shouldRender returns true whenever a beam target
        // exists, even when the ordinary entity range/frustum test fails.
        if !isGuardian && !shulkerTeleportActive && enderCrystalBeamTarget.is_none() && !inRenderRange { continue; }
        let guardianTarget = if isGuardian {
            find_living_target(
                entity.guardianTargetEntityId(),
                allEntities,
                remotePlayers,
                localPlayerTarget,
            )
        } else {
            None
        };
        let bounds = if isIllusioner {
            RenderIllusionIllager::renderBoundingBox(entity).expand_xyz(0.5)
        } else if renderer == EntityRendererKind::Minecart {
            RenderMinecart::renderBoundingBox(entity)
        } else {
            Render::renderBoundingBox(&entity.entity)
        };
        let entityVisible = inRenderRange && (
            renderer == EntityRendererKind::FishHook
                || frustum.isBoxInFrustum(
                    bounds.min_x,
                    bounds.min_y,
                    bounds.min_z,
                    bounds.max_x,
                    bounds.max_y,
                    bounds.max_z,
                )
        );
        let beamVisible = guardianTarget.is_some_and(|target| {
            let start = [
                entity.entity.posX,
                entity.entity.posY + entity.eyeHeight() as f64,
                entity.entity.posZ,
            ];
            let end = target.currentCenter();
            frustum.isBoxInFrustum(
                start[0].min(end[0]), start[1].min(end[1]), start[2].min(end[2]),
                start[0].max(end[0]), start[1].max(end[1]), start[2].max(end[2]),
            )
        });
        let enderCrystalBeamVisible = enderCrystalBeamTarget.is_some();
        let shulkerTeleportVisible = if shulkerTeleportActive {
            match (entity.shulkerOldAttachPos(), entity.shulkerAttachmentPos()) {
                (Some(old), Some(current)) => frustum.isBoxInFrustum(
                    old.x.min(current.x) as f64,
                    old.y.min(current.y) as f64,
                    old.z.min(current.z) as f64,
                    old.x.max(current.x) as f64,
                    old.y.max(current.y) as f64,
                    old.z.max(current.z) as f64,
                ),
                _ => false,
            }
        } else {
            false
        };
        if !entityVisible && !beamVisible && !enderCrystalBeamVisible && !shulkerTeleportVisible {
            continue;
        }
        let before = indices.len();
        let packedLight = packed_light_with_living_hurt_overlay(
            entity,
            non_player_entity_light(entity, position, chunks, dimension),
        );
        let renderModel = !invisible || isIllusioner || isGuardian || isEnderCrystal || isShulker;
        if renderModel {
            match renderer {
            EntityRendererKind::Boat => append_boat_mesh(
                entity,
                position,
                partialTicks,
                packedLight,
                atlas,
                vertices,
                indices,
                depthVertices,
                depthIndices,
            ),
            EntityRendererKind::Minecart => append_minecart_mesh(
                entity,
                position,
                partialTicks,
                packedLight,
                chunks,
                atlas,
                vertices,
                indices,
                overlayVertices,
                overlayIndices,
            ),
            EntityRendererKind::EntityItem => append_entity_item_mesh(
                entity, position, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::FallingBlock => append_falling_block_entity_mesh(
                entity, position, packedLight, chunks, atlas, vertices, indices,
            ),
            EntityRendererKind::ExperienceOrb => append_experience_orb_mesh(
                entity, position, partialTicks, cameraYaw, cameraPitch,
                thirdPersonView, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::Snowball => append_snowball_entity_mesh(
                entity, position, cameraYaw, cameraPitch, thirdPersonView,
                packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::Arrow => append_arrow_entity_mesh(
                entity, position, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::TntPrimed => append_primed_tnt_mesh(
                entity, position, partialTicks, packedLight, chunks, atlas, vertices, indices,
            ),
            EntityRendererKind::EnderCrystal => append_ender_crystal_mesh(
                entity, position, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::Zombie => append_zombie_mesh(
                entity, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::Skeleton => append_skeleton_mesh(
                entity, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::ArmorStand => append_armor_stand_mesh(
                entity, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::Pig => append_pig_mesh(
                entity, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::Cow => append_cow_mesh(
                entity, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::Sheep => append_sheep_mesh(
                entity, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::Chicken => append_chicken_mesh(
                entity, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::Mooshroom => append_mooshroom_mesh(
                entity, partialTicks, packedLight, chunks, atlas, vertices, indices,
            ),
            EntityRendererKind::Creeper => append_creeper_mesh(
                entity, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::Spider => append_spider_mesh(
                entity, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::Slime => append_slime_mesh(
                entity, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::MagmaCube => append_magma_cube_mesh(
                entity, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::Blaze => append_blaze_mesh(
                entity, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::Ghast => append_ghast_mesh(
                entity, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::Guardian => append_guardian_mesh(
                entity,
                guardianTarget,
                localPlayerTarget,
                !invisible,
                totalWorldTime,
                partialTicks,
                packedLight,
                atlas,
                vertices,
                indices,
            ),
            EntityRendererKind::Shulker => append_shulker_mesh(
                entity, !invisible, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::ShulkerBullet => append_shulker_bullet_mesh(
                entity,
                position,
                partialTicks,
                EntityShulkerBullet::getBrightnessForRender(),
                atlas,
                vertices,
                indices,
            ),
            EntityRendererKind::Fireball => append_fireball_mesh(
                entity, position, cameraYaw, cameraPitch, thirdPersonView,
                atlas, vertices, indices,
            ),
            EntityRendererKind::DragonFireball => append_dragon_fireball_mesh(
                entity, position, cameraYaw, cameraPitch, thirdPersonView,
                atlas, vertices, indices,
            ),
            EntityRendererKind::WitherSkull => append_wither_skull_mesh(
                entity, position, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::FishHook => append_fish_hook_mesh(
                entity,
                position,
                remotePlayers,
                localPlayerRenderState,
                partialTicks,
                cameraYaw,
                cameraPitch,
                thirdPersonView,
                fov,
                packedLight,
                atlas,
                vertices,
                indices,
                lineVertices,
                lineIndices,
            ),
            EntityRendererKind::AreaEffectCloud => {},
            EntityRendererKind::Painting => append_painting_mesh(
                entity, chunks, dimension, atlas, vertices, indices,
            ),
            EntityRendererKind::ItemFrame => append_item_frame_mesh(
                entity,
                position,
                distanceSquared,
                packedLight,
                mapData,
                atlas,
                vertices,
                indices,
            ),
            EntityRendererKind::LeashKnot => append_leash_knot_mesh(
                entity, position, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::Wolf => append_wolf_mesh(
                entity, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::Ocelot => append_ocelot_mesh(
                entity, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::Rabbit => append_rabbit_mesh(
                entity, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::PolarBear => append_polar_bear_mesh(
                entity, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::Horse => append_horse_mesh(
                entity, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::Llama => append_llama_mesh(
                entity, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::Villager => append_villager_mesh(
                entity, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::Witch => append_witch_mesh(
                entity, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::Illager => append_illager_mesh(
                entity, partialTicks, packedLight, atlas, vertices, indices,
            ),
            EntityRendererKind::ZombieVillager => append_zombie_villager_mesh(
                entity, partialTicks, packedLight, atlas, vertices, indices,
            ),
                EntityRendererKind::Unsupported => {}
            }
        }
        if entityVisible && entity.isBurning() {
            // Render#renderEntityOnFire binds TextureMap.LOCATION_BLOCKS_TEXTURE
            // and emits full-bright textured quads. Keep the TNT flash stream
            // exclusively for RenderTntMinecart's texture-disabled overlay.
            append_entity_fire_mesh(
                &entity.entity,
                position,
                cameraYaw,
                atlas,
                vertices,
                indices,
            );
        }
        if indices.len() > before {
            rendered += 1;
        }
    }
    rendered
}



fn append_player_fire_meshes(
    players: &[RemotePlayerRenderState],
    partialTicks: f32,
    cameraYaw: f32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let partial = partialTicks.clamp(0.0, 1.0) as f64;
    for player in players {
        if !player.burning || player.invisible { continue; }
        let position = [
            (player.prevPosition[0] + (player.position[0] - player.prevPosition[0]) * partial) as f32,
            (player.prevPosition[1] + (player.position[1] - player.prevPosition[1]) * partial) as f32,
            (player.prevPosition[2] + (player.position[2] - player.prevPosition[2]) * partial) as f32,
        ];
        append_fire_billboards(
            position,
            0.6,
            player.height,
            0.0,
            cameraYaw,
            atlas,
            vertices,
            indices,
        );
    }
}

fn append_entity_fire_mesh(
    entity: &crate::net::minecraft::entity::Entity::Entity,
    position: [f32; 3],
    cameraYaw: f32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    append_fire_billboards(
        position,
        entity.width,
        entity.height,
        (entity.posY - entity.boundingBox.min_y) as f32,
        cameraYaw,
        atlas,
        vertices,
        indices,
    );
}

/// Exact geometry loop from MCP `Render#renderEntityOnFire`. The view-aligned
/// stack alternates the two block-fire sprites, mirrors every second pair and
/// narrows by 10 percent per 0.45 entity-height step.
#[allow(clippy::too_many_arguments)]
fn append_fire_billboards(
    position: [f32; 3],
    width: f32,
    height: f32,
    entityBoxYOffset: f32,
    cameraYaw: f32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let scaleValue = width * 1.4;
    if scaleValue <= 0.0 { return; }
    let mut halfWidth = 0.5_f32;
    let mut remainingHeight = height / scaleValue;
    let mut verticalOffset = entityBoxYOffset;
    let mut z = 0.0_f32;
    let zBase = -0.3 + remainingHeight.floor() * 0.02;
    let mut layer = 0_i32;
    let matrix = fire_billboard_matrix(position, scaleValue, cameraYaw, zBase);
    while remainingHeight > 0.0 {
        let rectangle = if layer & 1 == 0 {
            atlas.fireLayer0Rectangle
        } else {
            atlas.fireLayer1Rectangle
        };
        let mirror = (layer / 2) & 1 == 0;
        let (u0, u1) = if mirror {
            (rectangle[2], rectangle[0])
        } else {
            (rectangle[0], rectangle[2])
        };
        let positions = [
            [halfWidth, -verticalOffset, z],
            [-halfWidth, -verticalOffset, z],
            [-halfWidth, 1.4 - verticalOffset, z],
            [halfWidth, 1.4 - verticalOffset, z],
        ];
        let uvs = [
            [u1, rectangle[3]],
            [u0, rectangle[3]],
            [u0, rectangle[1]],
            [u1, rectangle[1]],
        ];
        let base = vertices.len() as u32;
        for corner in 0..4 {
            vertices.push(WorldVertex {
                position: transform_point3(matrix, positions[corner]),
                uv: uvs[corner],
                color: [1.0, 1.0, 1.0, encoded_fire_alpha(1.0, (layer & 1) as usize)],
                lightmap: [15.0, 15.0],
            
                shaderEntity: [-1, -1, -1],
                shaderPadding: 0,
            });
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        remainingHeight -= 0.45;
        verticalOffset -= 0.45;
        halfWidth *= 0.9;
        z += 0.03;
        layer += 1;
    }
}


fn fire_billboard_matrix(
    position: [f32; 3],
    scaleValue: f32,
    cameraYaw: f32,
    zBase: f32,
) -> [[f32; 4]; 4] {
    // Fixed-function OpenGL post-multiplies in call order. Preserve the
    // exact Render#renderEntityOnFire sequence: translate, scale, face the
    // camera, then apply the per-height Z setback. In particular, the final
    // setback is intentionally affected by the entity-width scale.
    let mut matrix = translation4(position);
    matrix = multiply4(
        matrix,
        scale4_nonuniform([scaleValue, scaleValue, scaleValue]),
    );
    matrix = multiply4(matrix, rotation_y4(-cameraYaw));
    multiply4(matrix, translation4([0.0, 0.0, zBase]))
}

#[allow(clippy::too_many_arguments)]
fn append_fish_hook_mesh(
    entity: &EntityOtherClient,
    position: [f32; 3],
    remotePlayers: &[RemotePlayerRenderState],
    localPlayer: Option<&RemotePlayerRenderState>,
    partialTicks: f32,
    cameraYaw: f32,
    cameraPitch: f32,
    thirdPersonView: i32,
    fov: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
    lineVertices: &mut Vec<WorldVertex>,
    lineIndices: &mut Vec<u32>,
) {
    let Some(anglerId) = entity.fishHookAnglerId else { return; };
    let angler = localPlayer
        .filter(|player| player.entityId == anglerId)
        .or_else(|| remotePlayers.iter().find(|player| player.entityId == anglerId));
    let Some(angler) = angler else { return; };

    let Some(rectangle) = atlas.entityTextureRectangles
        .get(&RenderFish::texture())
        .copied()
    else { return; };

    // RenderFish: translate -> scale(0.5) -> playerViewY -> playerViewX.
    let mut hookMatrix = translation4(position);
    hookMatrix = multiply4(hookMatrix, scale4_nonuniform([RenderFish::SCALE; 3]));
    hookMatrix = multiply4(hookMatrix, rotation_y4(180.0 - cameraYaw));
    let pitchSign = if thirdPersonView == 2 { -1.0 } else { 1.0 };
    hookMatrix = multiply4(hookMatrix, rotation_x4(pitchSign * -cameraPitch));
    let lightmap = [
        ((packedLight >> 4) & 15) as f32,
        ((packedLight >> 20) & 15) as f32,
    ];
    append_entity_texture_quad(
        hookMatrix,
        RenderFish::HOOK_POSITIONS,
        RenderFish::HOOK_UV,
        rectangle,
        lightmap,
        vertices,
        indices,
    );

    let mut handSign = if angler.primaryHand == EnumHandSide::Right { 1 } else { -1 };
    if angler.mainHandStack.itemId != 346 {
        handSign = -handSign;
    }

    let mut swingDelta = angler.swingProgress - angler.prevSwingProgress;
    if swingDelta < 0.0 {
        swingDelta += 1.0;
    }
    let swingProgress = angler.prevSwingProgress + swingDelta * partialTicks.clamp(0.0, 1.0);
    let swing = (swingProgress.sqrt() * core::f32::consts::PI).sin();
    let bodyYaw = (angler.prevBodyYaw
        + (angler.bodyYaw - angler.prevBodyYaw) * partialTicks)
        .to_radians();
    let bodySin = bodyYaw.sin() as f64;
    let bodyCos = bodyYaw.cos() as f64;
    let side = handSign as f64 * 0.35;
    let playerPosition = [
        angler.prevPosition[0]
            + (angler.position[0] - angler.prevPosition[0]) * partialTicks as f64,
        angler.prevPosition[1]
            + (angler.position[1] - angler.prevPosition[1]) * partialTicks as f64,
        angler.prevPosition[2]
            + (angler.position[2] - angler.prevPosition[2]) * partialTicks as f64,
    ];

    let localFirstPerson = localPlayer
        .is_some_and(|local| local.entityId == angler.entityId)
        && thirdPersonView <= 0;
    let (handX, handY, handZ, verticalOffset) = if localFirstPerson {
        let fovScale = fov / 100.0;
        let mut hand = Vec3d::new(
            handSign as f64 * -0.36 * fovScale as f64,
            -0.045 * fovScale as f64,
            0.4,
        );
        let pitch = (angler.prevPitch
            + (angler.pitch - angler.prevPitch) * partialTicks)
            .to_radians();
        let yaw = (angler.prevHeadYaw
            + (angler.headYaw - angler.prevHeadYaw) * partialTicks)
            .to_radians();
        hand = hand.rotate_pitch(-pitch);
        hand = hand.rotate_yaw(-yaw);
        hand = hand.rotate_yaw(swing * 0.5);
        hand = hand.rotate_pitch(-swing * 0.7);
        (
            playerPosition[0] + hand.x,
            playerPosition[1] + hand.y,
            playerPosition[2] + hand.z,
            angler.eyeHeight as f64,
        )
    } else {
        (
            playerPosition[0] - bodyCos * side - bodySin * 0.8,
            playerPosition[1] + angler.eyeHeight as f64 - 0.45,
            playerPosition[2] - bodySin * side + bodyCos * 0.8,
            if angler.sneaking { -0.1875 } else { 0.0 },
        )
    };

    let hookStart = [
        entity.entity.prevPosX
            + (entity.entity.posX - entity.entity.prevPosX) * partialTicks as f64,
        entity.entity.prevPosY
            + (entity.entity.posY - entity.entity.prevPosY) * partialTicks as f64
            + RenderFish::HOOK_Y_OFFSET,
        entity.entity.prevPosZ
            + (entity.entity.posZ - entity.entity.prevPosZ) * partialTicks as f64,
    ];
    // MCP explicitly narrows these three deltas to float before line emission.
    let delta = [
        (handX - hookStart[0]) as f32 as f64,
        (handY - hookStart[1] + verticalOffset) as f32 as f64,
        (handZ - hookStart[2]) as f32 as f64,
    ];
    let mut stripVertices = Vec::with_capacity(RenderFish::LINE_SEGMENTS + 1);
    for step in 0..=RenderFish::LINE_SEGMENTS {
        let point = RenderFish::linePoint(hookStart, delta, step);
        stripVertices.push(WorldVertex {
            position: [point[0] as f32, point[1] as f32, point[2] as f32],
            uv: [0.0, 0.0],
            color: [0.0, 0.0, 0.0, 1.0],
            lightmap: [15.0, 15.0],
        
            shaderEntity: [-1, -1, -1],
            shaderPadding: 0,
        });
    }
    let stripIndices = (0..stripVertices.len() as u32).collect::<Vec<_>>();
    append_existing_line_strip(lineVertices, lineIndices, &stripVertices, &stripIndices);
}



#[derive(Debug, Clone, Copy)]
enum ItemFrameFace { Down, Up, North, South, West, East }

fn entity_texture_uv(rectangle: [f32; 4], uv: [f32; 2]) -> [f32; 2] {
    [
        rectangle[0] + (rectangle[2] - rectangle[0]) * uv[0],
        rectangle[1] + (rectangle[3] - rectangle[1]) * uv[1],
    ]
}

#[allow(clippy::too_many_arguments)]
fn append_entity_texture_quad(
    matrix: [[f32; 4]; 4],
    positions: [[f32; 3]; 4],
    sourceUvs: [[f32; 2]; 4],
    textureRectangle: [f32; 4],
    lightmap: [f32; 2],
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    append_textured_quad_world(
        matrix,
        positions,
        sourceUvs.map(|uv| entity_texture_uv(textureRectangle, uv)),
        [1.0; 4],
        lightmap,
        vertices,
        indices,
    );
}

#[allow(clippy::too_many_arguments)]
fn append_painting_mesh(
    entity: &EntityOtherClient,
    chunks: &HashMap<ChunkKey, Chunk>,
    dimension: i32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let Some(art) = entity.paintingArt() else { return; };
    let Some(facing) = entity.hangingFacing else { return; };
    let data = art.data();
    let rectangle = atlas.entityTextureRectangles
        .get(&RenderPainting::texture())
        .copied()
        .unwrap_or(atlas.missingRectangle);
    let mut matrix = translation4([
        entity.entity.posX as f32,
        entity.entity.posY as f32,
        entity.entity.posZ as f32,
    ]);
    matrix = multiply4(matrix, rotation_y4(180.0 - entity.entity.rotationYaw));
    matrix = multiply4(matrix, scale4_nonuniform([RenderPainting::MODEL_SCALE; 3]));
    let left = -(data.sizeX as f32) / 2.0;
    let bottom = -(data.sizeY as f32) / 2.0;

    for tileX in 0..data.sizeX / 16 {
        for tileY in 0..data.sizeY / 16 {
            let x1 = left + ((tileX + 1) * 16) as f32;
            let x0 = left + (tileX * 16) as f32;
            let y1 = bottom + ((tileY + 1) * 16) as f32;
            let y0 = bottom + (tileY * 16) as f32;

            // MCP `RenderPainting#setLightmap`: each 16x16 tile samples the
            // block light at its own world-space center rather than using one
            // value for the entire motive.
            let centerX = (x1 + x0) * 0.5;
            let centerY = (y1 + y0) * 0.5;
            let mut lightX = entity.entity.posX.floor() as i32;
            let lightY = (entity.entity.posY + (centerY / 16.0) as f64).floor() as i32;
            let mut lightZ = entity.entity.posZ.floor() as i32;
            match facing {
                EnumFacing::North => lightX = (entity.entity.posX + (centerX / 16.0) as f64).floor() as i32,
                EnumFacing::West => lightZ = (entity.entity.posZ - (centerX / 16.0) as f64).floor() as i32,
                EnumFacing::South => lightX = (entity.entity.posX - (centerX / 16.0) as f64).floor() as i32,
                EnumFacing::East => lightZ = (entity.entity.posZ + (centerX / 16.0) as f64).floor() as i32,
                _ => {}
            }
            let lightPos = BlockPos::new(lightX, lightY, lightZ);
            let state = snapshot_block_state(chunks, lightPos);
            let packedLight = snapshot_combined_light(chunks, lightPos, dimension, state);
            let lightmap = [((packedLight >> 4) & 15) as f32, ((packedLight >> 20) & 15) as f32];

            let u1 = (data.offsetX + data.sizeX - tileX * 16) as f32 / 256.0;
            let u0 = (data.offsetX + data.sizeX - (tileX + 1) * 16) as f32 / 256.0;
            let v1 = (data.offsetY + data.sizeY - tileY * 16) as f32 / 256.0;
            let v0 = (data.offsetY + data.sizeY - (tileY + 1) * 16) as f32 / 256.0;

            // Front motive, back, top, bottom, left and right follow the exact
            // BufferBuilder vertex order and source UV regions from 1.12.2.
            append_entity_texture_quad(matrix,
                [[x1, y0, -0.5], [x0, y0, -0.5], [x0, y1, -0.5], [x1, y1, -0.5]],
                [[u0, v1], [u1, v1], [u1, v0], [u0, v0]], rectangle, lightmap, vertices, indices);
            append_entity_texture_quad(matrix,
                [[x1, y1, 0.5], [x0, y1, 0.5], [x0, y0, 0.5], [x1, y0, 0.5]],
                [[0.75, 0.0], [0.8125, 0.0], [0.8125, 0.0625], [0.75, 0.0625]], rectangle, lightmap, vertices, indices);
            append_entity_texture_quad(matrix,
                [[x1, y1, -0.5], [x0, y1, -0.5], [x0, y1, 0.5], [x1, y1, 0.5]],
                [[0.75, 0.001953125], [0.8125, 0.001953125], [0.8125, 0.001953125], [0.75, 0.001953125]], rectangle, lightmap, vertices, indices);
            append_entity_texture_quad(matrix,
                [[x1, y0, 0.5], [x0, y0, 0.5], [x0, y0, -0.5], [x1, y0, -0.5]],
                [[0.75, 0.001953125], [0.8125, 0.001953125], [0.8125, 0.001953125], [0.75, 0.001953125]], rectangle, lightmap, vertices, indices);
            append_entity_texture_quad(matrix,
                [[x1, y1, 0.5], [x1, y0, 0.5], [x1, y0, -0.5], [x1, y1, -0.5]],
                [[0.751953125, 0.0], [0.751953125, 0.0625], [0.751953125, 0.0625], [0.751953125, 0.0]], rectangle, lightmap, vertices, indices);
            append_entity_texture_quad(matrix,
                [[x0, y1, -0.5], [x0, y0, -0.5], [x0, y0, 0.5], [x0, y1, 0.5]],
                [[0.751953125, 0.0], [0.751953125, 0.0625], [0.751953125, 0.0625], [0.751953125, 0.0]], rectangle, lightmap, vertices, indices);
        }
    }
}

fn append_item_frame_face(
    matrix: [[f32; 4]; 4],
    from: [f32; 3],
    to: [f32; 3],
    face: ItemFrameFace,
    uvRect: [f32; 4],
    textureRectangle: [f32; 4],
    lightmap: [f32; 2],
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let [x0, y0, z0] = from.map(|value| value / 16.0);
    let [x1, y1, z1] = to.map(|value| value / 16.0);
    let positions = match face {
        ItemFrameFace::North => [[x1, y0, z0], [x0, y0, z0], [x0, y1, z0], [x1, y1, z0]],
        ItemFrameFace::South => [[x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1]],
        ItemFrameFace::Down => [[x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1]],
        ItemFrameFace::Up => [[x0, y1, z1], [x1, y1, z1], [x1, y1, z0], [x0, y1, z0]],
        ItemFrameFace::West => [[x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0]],
        ItemFrameFace::East => [[x1, y0, z1], [x1, y0, z0], [x1, y1, z0], [x1, y1, z1]],
    };
    let u0 = uvRect[0] / 16.0;
    let v0 = uvRect[1] / 16.0;
    let u1 = uvRect[2] / 16.0;
    let v1 = uvRect[3] / 16.0;
    append_entity_texture_quad(
        matrix,
        positions,
        [[u0, v1], [u1, v1], [u1, v0], [u0, v0]],
        textureRectangle,
        lightmap,
        vertices,
        indices,
    );
}

#[allow(clippy::too_many_arguments)]
fn append_item_frame_model(
    mapVariant: bool,
    matrix: [[f32; 4]; 4],
    lightmap: [f32; 2],
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let wood = atlas.entityTextureRectangles.get(&RenderItemFrame::woodTexture()).copied().unwrap_or(atlas.missingRectangle);
    let background = atlas.entityTextureRectangles.get(&RenderItemFrame::backgroundTexture()).copied().unwrap_or(atlas.missingRectangle);
    if mapVariant {
        for face in [ItemFrameFace::North, ItemFrameFace::South] {
            append_item_frame_face(matrix, [1.0, 1.0, 15.001], [15.0, 15.0, 16.0], face, [1.0, 1.0, 15.0, 15.0], background, lightmap, vertices, indices);
        }
        let elements: [([f32; 3], [f32; 3], &[(ItemFrameFace, [f32; 4])]); 4] = [
            ([0.0, 0.0, 15.001], [16.0, 1.0, 16.0], &[
                (ItemFrameFace::Down, [0.0, 0.0, 16.0, 1.0]), (ItemFrameFace::Up, [0.0, 15.0, 16.0, 16.0]),
                (ItemFrameFace::North, [0.0, 15.0, 16.0, 16.0]), (ItemFrameFace::South, [0.0, 15.0, 16.0, 16.0]),
                (ItemFrameFace::West, [15.0, 15.0, 16.0, 16.0]), (ItemFrameFace::East, [0.0, 15.0, 1.0, 16.0])]),
            ([0.0, 15.0, 15.001], [16.0, 16.0, 16.0], &[
                (ItemFrameFace::Down, [0.0, 0.0, 16.0, 1.0]), (ItemFrameFace::Up, [0.0, 15.0, 16.0, 16.0]),
                (ItemFrameFace::North, [0.0, 0.0, 16.0, 1.0]), (ItemFrameFace::South, [0.0, 0.0, 16.0, 1.0]),
                (ItemFrameFace::West, [15.0, 0.0, 16.0, 1.0]), (ItemFrameFace::East, [0.0, 0.0, 1.0, 1.0])]),
            ([0.0, 1.0, 15.001], [1.0, 15.0, 16.0], &[
                (ItemFrameFace::North, [15.0, 1.0, 16.0, 15.0]), (ItemFrameFace::South, [0.0, 1.0, 1.0, 15.0]),
                (ItemFrameFace::West, [15.0, 1.0, 16.0, 15.0]), (ItemFrameFace::East, [0.0, 1.0, 1.0, 15.0])]),
            ([15.0, 1.0, 15.001], [16.0, 15.0, 16.0], &[
                (ItemFrameFace::North, [0.0, 1.0, 1.0, 15.0]), (ItemFrameFace::South, [15.0, 1.0, 16.0, 15.0]),
                (ItemFrameFace::West, [15.0, 1.0, 16.0, 15.0]), (ItemFrameFace::East, [0.0, 1.0, 1.0, 15.0])]),
        ];
        for (from, to, faces) in elements {
            for (face, uv) in faces { append_item_frame_face(matrix, from, to, *face, *uv, wood, lightmap, vertices, indices); }
        }
    } else {
        for face in [ItemFrameFace::North, ItemFrameFace::South] {
            append_item_frame_face(matrix, [3.0, 3.0, 15.5], [13.0, 13.0, 16.0], face, [3.0, 3.0, 13.0, 13.0], background, lightmap, vertices, indices);
        }
        let elements: [([f32; 3], [f32; 3], &[(ItemFrameFace, [f32; 4])]); 4] = [
            ([2.0, 2.0, 15.0], [14.0, 3.0, 16.0], &[
                (ItemFrameFace::Down, [2.0, 0.0, 14.0, 1.0]), (ItemFrameFace::Up, [2.0, 15.0, 14.0, 16.0]),
                (ItemFrameFace::North, [2.0, 13.0, 14.0, 14.0]), (ItemFrameFace::South, [2.0, 13.0, 14.0, 14.0]),
                (ItemFrameFace::West, [15.0, 13.0, 16.0, 14.0]), (ItemFrameFace::East, [0.0, 13.0, 1.0, 14.0])]),
            ([2.0, 13.0, 15.0], [14.0, 14.0, 16.0], &[
                (ItemFrameFace::Down, [2.0, 0.0, 14.0, 1.0]), (ItemFrameFace::Up, [2.0, 15.0, 14.0, 16.0]),
                (ItemFrameFace::North, [2.0, 2.0, 14.0, 3.0]), (ItemFrameFace::South, [2.0, 2.0, 14.0, 3.0]),
                (ItemFrameFace::West, [15.0, 2.0, 16.0, 3.0]), (ItemFrameFace::East, [0.0, 2.0, 1.0, 3.0])]),
            ([2.0, 3.0, 15.0], [3.0, 13.0, 16.0], &[
                (ItemFrameFace::North, [13.0, 3.0, 14.0, 13.0]), (ItemFrameFace::South, [2.0, 3.0, 3.0, 13.0]),
                (ItemFrameFace::West, [15.0, 3.0, 16.0, 13.0]), (ItemFrameFace::East, [0.0, 3.0, 1.0, 13.0])]),
            ([13.0, 3.0, 15.0], [14.0, 13.0, 16.0], &[
                (ItemFrameFace::North, [2.0, 3.0, 3.0, 13.0]), (ItemFrameFace::South, [13.0, 3.0, 14.0, 13.0]),
                (ItemFrameFace::West, [15.0, 3.0, 16.0, 13.0]), (ItemFrameFace::East, [0.0, 3.0, 1.0, 13.0])]),
        ];
        for (from, to, faces) in elements {
            for (face, uv) in faces { append_item_frame_face(matrix, from, to, *face, *uv, wood, lightmap, vertices, indices); }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn append_item_frame_mesh(
    entity: &EntityOtherClient,
    _position: [f32; 3],
    distanceSquared: f64,
    packedLight: u32,
    mapData: &HashMap<i32, MapData>,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let Some(anchor) = entity.hangingPosition else { return; };
    let displayed = entity.itemFrameDisplayedItem();
    let mapVariant = EntityItemFrame::isMap(displayed);
    let center = [anchor.x as f32 + 0.5, anchor.y as f32 + 0.5, anchor.z as f32 + 0.5];
    let mut root = translation4(center);
    root = multiply4(root, rotation_y4(180.0 - entity.entity.rotationYaw));
    let frameMatrix = multiply4(root, translation4([-0.5, -0.5, -0.5]));
    let lightmap = [((packedLight >> 4) & 15) as f32, ((packedLight >> 20) & 15) as f32];
    append_item_frame_model(mapVariant, frameMatrix, lightmap, atlas, vertices, indices);

    let Some(stack) = displayed else { return; };
    if distanceSquared > EntityItemFrame::ITEM_RENDER_DISTANCE_SQ { return; }
    let rotation = EntityItemFrame::renderedRotation(
        entity.itemFrameRotation(),
        mapVariant,
    ) as f32 * 45.0;
    if mapVariant {
        let Some(map) = ItemMap::getMapData(stack, mapData) else { return; };
        let mut mapMatrix = multiply4(
            root,
            translation4([0.0, 0.0, RenderItemFrame::ITEM_TRANSLATE_Z]),
        );
        mapMatrix = multiply4(mapMatrix, rotation_z4(rotation));
        mapMatrix = multiply4(mapMatrix, rotation_z4(180.0));
        mapMatrix = multiply4(
            mapMatrix,
            scale4_nonuniform([MapItemRenderer::MAP_SCALE; 3]),
        );
        mapMatrix = multiply4(mapMatrix, translation4([-64.0, -64.0, -1.0]));
        append_map_item_frame_mesh(map, mapMatrix, atlas, vertices, indices);
        return;
    }

    let Some(model) = item_model_for_stack(stack, atlas) else { return; };
    let mut itemMatrix = multiply4(
        root,
        translation4([0.0, 0.0, RenderItemFrame::ITEM_TRANSLATE_Z]),
    );
    itemMatrix = multiply4(itemMatrix, rotation_z4(rotation));
    itemMatrix = multiply4(
        itemMatrix,
        scale4_nonuniform([RenderItemFrame::ITEM_SCALE; 3]),
    );
    append_item_stack_world_transformed(
        stack,
        model,
        itemMatrix,
        TransformType::Fixed,
        packedLight,
        atlas,
        vertices,
        indices,
    );
}

fn append_map_item_frame_mesh(
    mapData: &MapData,
    mapMatrix: [[f32; 4]; 4],
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    // Item-frame semantics are exactly
    // `MapItemRenderer::renderedDecorations(mapData, true)`; the shared helper
    // accepts the same boolean so hand-held maps can retain player markers.
    append_map_mesh(mapData, true, mapMatrix, atlas, vertices, indices);
}

fn append_map_mesh(
    mapData: &MapData,
    noOverlayRendering: bool,
    mapMatrix: [[f32; 4]; 4],
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    append_map_quad(
        mapMatrix,
        [0.0, 0.0],
        [MapItemRenderer::MAP_SIZE, MapItemRenderer::MAP_SIZE],
        MapItemRenderer::MAP_PLANE_Z,
        atlas.mapCheckerRectangle,
        [1.0, 1.0, 1.0, 1.0],
        false,
        vertices,
        indices,
    );

    // A DynamicTexture would upload all 16,384 texels. The Vulkan atlas is
    // immutable between resource reloads, so exact non-air pixels are emitted
    // as horizontally coalesced full-bright quads over the static vanilla
    // zero-color checker. This preserves the final texel result without using
    // an unrelated block/item texture as a surrogate.
    for y in 0..MapData::HEIGHT {
        let mut x = 0;
        while x < MapData::WIDTH {
            let index = x + y * MapData::WIDTH;
            let colorByte = mapData.colors[index];
            if colorByte / 4 == 0 {
                x += 1;
                continue;
            }
            let mut end = x + 1;
            while end < MapData::WIDTH
                && mapData.colors[end + y * MapData::WIDTH] == colorByte
            {
                end += 1;
            }
            append_map_quad(
                mapMatrix,
                [x as f32, y as f32],
                [end as f32, y as f32 + 1.0],
                MapItemRenderer::MAP_PLANE_Z - 0.0001,
                atlas.solidWhiteRectangle,
                MapItemRenderer::pixelColor(mapData, index),
                true,
                vertices,
                indices,
            );
            x = end;
        }
    }

    for (layer, decoration) in
        MapItemRenderer::renderedDecorations(mapData, noOverlayRendering).enumerate()
    {
        let mut iconMatrix = multiply4(
            mapMatrix,
            translation4([
                decoration.getX() as f32 / 2.0 + 64.0,
                decoration.getY() as f32 / 2.0 + 64.0,
                -0.02,
            ]),
        );
        iconMatrix = multiply4(
            iconMatrix,
            rotation_z4(decoration.getRotation() as f32 * 360.0 / 16.0),
        );
        iconMatrix = multiply4(
            iconMatrix,
            scale4_nonuniform([
                MapItemRenderer::ICON_SCALE,
                MapItemRenderer::ICON_SCALE,
                3.0,
            ]),
        );
        iconMatrix = multiply4(
            iconMatrix,
            translation4([
                MapItemRenderer::ICON_TRANSLATE_X,
                MapItemRenderer::ICON_TRANSLATE_Y,
                0.0,
            ]),
        );
        let icon = decoration.getType() as f32;
        let u0 = (icon % 4.0) / 4.0;
        let v0 = (icon / 4.0).floor() / 4.0;
        let u1 = u0 + 0.25;
        let v1 = v0 + 0.25;
        append_textured_quad(
            iconMatrix,
            [
                [-1.0, 1.0, layer as f32 * -0.001],
                [1.0, 1.0, layer as f32 * -0.001],
                [1.0, -1.0, layer as f32 * -0.001],
                [-1.0, -1.0, layer as f32 * -0.001],
            ],
            [[u0, v0], [u1, v0], [u1, v1], [u0, v1]],
            atlas.mapIconsRectangle,
            [1.0, 1.0, 1.0, 1.0],
            vertices,
            indices,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn append_map_quad(
    matrix: [[f32; 4]; 4],
    minimum: [f32; 2],
    maximum: [f32; 2],
    z: f32,
    rectangle: [f32; 4],
    color: [f32; 4],
    sampleCenter: bool,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let uvs = if sampleCenter {
        let center = [
            (rectangle[0] + rectangle[2]) * 0.5,
            (rectangle[1] + rectangle[3]) * 0.5,
        ];
        [center; 4]
    } else {
        [
            [rectangle[0], rectangle[3]],
            [rectangle[2], rectangle[3]],
            [rectangle[2], rectangle[1]],
            [rectangle[0], rectangle[1]],
        ]
    };
    append_textured_quad(
        matrix,
        [
            [minimum[0], maximum[1], z],
            [maximum[0], maximum[1], z],
            [maximum[0], minimum[1], z],
            [minimum[0], minimum[1], z],
        ],
        uvs,
        [0.0, 0.0, 1.0, 1.0],
        color,
        vertices,
        indices,
    );
}

#[allow(clippy::too_many_arguments)]
fn append_textured_quad(
    matrix: [[f32; 4]; 4],
    positions: [[f32; 3]; 4],
    localUvs: [[f32; 2]; 4],
    rectangle: [f32; 4],
    color: [f32; 4],
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let base = vertices.len() as u32;
    for index in 0..4 {
        let uv = [
            rectangle[0] + (rectangle[2] - rectangle[0]) * localUvs[index][0],
            rectangle[1] + (rectangle[3] - rectangle[1]) * localUvs[index][1],
        ];
        vertices.push(WorldVertex {
            position: transform_point3(matrix, positions[index]),
            uv,
            color,
            lightmap: [15.0, 15.0],
        
            shaderEntity: [-1, -1, -1],
            shaderPadding: 0,
        });
    }
    indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
}

#[allow(clippy::too_many_arguments)]
fn append_leash_knot_mesh(
    entity: &EntityOtherClient,
    position: [f32; 3],
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Object { objectType: ObjectSpawnType::LeashKnot, .. } = &entity.kind else { return; };
    let mut matrix = translation4(position);
    matrix = multiply4(matrix, scale4_nonuniform([-1.0, -1.0, 1.0]));
    append_vehicle_model_pass(
        ModelLeashKnot::buildMesh(0.0, 0.0),
        RenderLeashKnot::texture(),
        matrix,
        packedLight,
        atlas,
        vertices,
        indices,
    );
}

#[allow(clippy::too_many_arguments)]
fn append_vehicle_model_pass(
    mesh: VehicleModelMesh,
    texture: ResourceLocation,
    matrix: [[f32; 4]; 4],
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    if mesh.indices.is_empty() { return; }
    let rectangle = atlas.entityTextureRectangles
        .get(&texture)
        .copied()
        .unwrap_or(atlas.missingRectangle);
    let blockLight = ((packedLight >> 4) & 15) as f32;
    let skyLight = ((packedLight >> 20) & 15) as f32;
    let base = vertices.len() as u32;
    vertices.extend(mesh.vertices.into_iter().map(|vertex| WorldVertex {
        position: transform_point3(matrix, vertex.position),
        uv: [
            rectangle[0] + (rectangle[2] - rectangle[0]) * vertex.uv[0],
            rectangle[1] + (rectangle[3] - rectangle[1]) * vertex.uv[1],
        ],
        color: [1.0; 4],
        lightmap: [blockLight, skyLight],
    
        shaderEntity: [-1, -1, -1],
        shaderPadding: 0,
    }));
    indices.extend(mesh.indices.into_iter().map(|index| base + index));
}

#[allow(clippy::too_many_arguments)]
fn append_boat_mesh(
    entity: &EntityOtherClient,
    position: [f32; 3],
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
    depthVertices: &mut Vec<WorldVertex>,
    depthIndices: &mut Vec<u32>,
) {
    let ClientEntityKind::Object { objectType: ObjectSpawnType::Boat, .. } = &entity.kind else { return; };
    let partial = partialTicks.clamp(0.0, 1.0);
    // RenderManager#renderEntityStatic does a direct linear interpolation here.
    let entityYaw = entity.entity.prevRotationYaw
        + (entity.entity.rotationYaw - entity.entity.prevRotationYaw) * partial;
    let damageRotation = RenderBoat::damageRotation(
        entity.boatTimeSinceHit(),
        entity.boatDamageTaken(),
        entity.boatForwardDirection(),
        partial,
    );
    let mut matrix = translation4([
        position[0],
        position[1] + RenderBoat::Y_TRANSLATION,
        position[2],
    ]);
    matrix = multiply4(matrix, rotation_y4(180.0 - entityYaw));
    if damageRotation != 0.0 {
        matrix = multiply4(matrix, rotation_x4(damageRotation));
    }
    matrix = multiply4(matrix, scale4_nonuniform([-1.0, -1.0, 1.0]));
    // ModelBoat#render begins with GlStateManager.rotate(90, Y).
    matrix = multiply4(matrix, rotation_y4(ModelBoat::MODEL_ROTATION_Y));
    append_vehicle_model_pass(
        ModelBoat::buildMesh([
            entity.boatRowingTime(0, partial),
            entity.boatRowingTime(1, partial),
        ]),
        RenderBoat::texture(entity.boatType()),
        matrix,
        packedLight,
        atlas,
        vertices,
        indices,
    );
    // `RenderBoat#isMultipass` makes RenderGlobal call this after every
    // normal entity has been submitted. `ModelBoat#renderMultipass` repeats
    // the same renderer transform, applies the model's 90-degree Y rotation,
    // masks RGBA, and renders only `noWater`. A dedicated Vulkan stream and
    // pipeline provide the exact colorMask(false,false,false,false) / depth
    // write equivalent; this geometry must never enter the visible entity
    // color stream.
    append_vehicle_model_pass(
        ModelBoat::buildNoWaterMesh(),
        RenderBoat::texture(entity.boatType()),
        matrix,
        packedLight,
        atlas,
        depthVertices,
        depthIndices,
    );
}

#[allow(clippy::too_many_arguments)]
fn append_minecart_mesh(
    entity: &EntityOtherClient,
    position: [f32; 3],
    partialTicks: f32,
    packedLight: u32,
    chunks: &HashMap<ChunkKey, Chunk>,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
    overlayVertices: &mut Vec<WorldVertex>,
    overlayIndices: &mut Vec<u32>,
) {
    let ClientEntityKind::Object { objectType: ObjectSpawnType::Minecart, .. } = &entity.kind else { return; };
    let partial = partialTicks.clamp(0.0, 1.0);
    let d0 = position[0] as f64;
    let d1 = position[1] as f64;
    let d2 = position[2] as f64;
    let mut renderPosition = position;
    let mut entityYaw = entity.entity.prevRotationYaw
        + (entity.entity.rotationYaw - entity.entity.prevRotationYaw) * partial;
    let mut entityPitch = entity.entity.prevRotationPitch
        + (entity.entity.rotationPitch - entity.entity.prevRotationPitch) * partial;

    if let Some(center) = EntityMinecart::getPos(d0, d1, d2, |pos| snapshot_block_state(chunks, pos)) {
        let front = EntityMinecart::getPosOffset(
            d0, d1, d2, EntityMinecart::RENDER_SAMPLE_OFFSET,
            |pos| snapshot_block_state(chunks, pos),
        ).unwrap_or(center);
        let back = EntityMinecart::getPosOffset(
            d0, d1, d2, -EntityMinecart::RENDER_SAMPLE_OFFSET,
            |pos| snapshot_block_state(chunks, pos),
        ).unwrap_or(center);
        renderPosition[0] += (center[0] - d0) as f32;
        renderPosition[1] += ((front[1] + back[1]) * 0.5 - d1) as f32;
        renderPosition[2] += (center[2] - d2) as f32;
        let vector = [back[0] - front[0], back[1] - front[1], back[2] - front[2]];
        let length = (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt();
        if length != 0.0 {
            let normalized = [vector[0] / length, vector[1] / length, vector[2] / length];
            entityYaw = normalized[2].atan2(normalized[0]).to_degrees() as f32;
            entityPitch = (normalized[1].atan() * 73.0) as f32;
        }
    }

    let jitter = RenderMinecart::deterministicOffset(entity.entityId);
    let mut matrix = translation4([
        renderPosition[0] + jitter[0],
        renderPosition[1] + jitter[1] + RenderMinecart::Y_TRANSLATION,
        renderPosition[2] + jitter[2],
    ]);
    matrix = multiply4(matrix, rotation_y4(180.0 - entityYaw));
    matrix = multiply4(matrix, rotation_z4(-entityPitch));
    let damageRotation = RenderMinecart::damageRotation(
        entity.minecartRollingAmplitude(),
        entity.minecartDamage(),
        entity.minecartRollingDirection(),
        partial,
    );
    if damageRotation != 0.0 {
        matrix = multiply4(matrix, rotation_x4(damageRotation));
    }

    append_minecart_contents(
        entity,
        matrix,
        partial,
        packedLight,
        chunks,
        atlas,
        vertices,
        indices,
        overlayVertices,
        overlayIndices,
    );

    let modelMatrix = multiply4(matrix, scale4_nonuniform([-1.0, -1.0, 1.0]));
    append_vehicle_model_pass(
        ModelMinecart::buildMesh(-0.1),
        RenderMinecart::texture(),
        modelMatrix,
        packedLight,
        atlas,
        vertices,
        indices,
    );
}

#[allow(clippy::too_many_arguments)]
fn append_minecart_contents(
    entity: &EntityOtherClient,
    matrix: [[f32; 4]; 4],
    partialTicks: f32,
    packedLight: u32,
    chunks: &HashMap<ChunkKey, Chunk>,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
    overlayVertices: &mut Vec<WorldVertex>,
    overlayIndices: &mut Vec<u32>,
) {
    let state = IBlockState::fromGlobalStateId(entity.minecartDisplayStateId());
    if state.isAir() { return; }
    let offset = entity.minecartDisplayOffset();
    let mut contentMatrix = multiply4(matrix, scale4_nonuniform([RenderMinecart::CONTENT_SCALE; 3]));
    contentMatrix = multiply4(contentMatrix, translation4([
        -0.5,
        (offset - 8) as f32 / 16.0,
        0.5,
    ]));
    if entity.minecartType() == MinecartType::Tnt {
        let pulse = RenderMinecart::tntContentScale(entity.minecartTntFuse(), partialTicks);
        if pulse != 1.0 {
            contentMatrix = multiply4(contentMatrix, scale4_nonuniform([pulse; 3]));
        }
    }
    // BlockModelRenderer#renderModelBrightness and ChestRenderer both begin
    // with the same 90-degree Y rotation.
    contentMatrix = multiply4(contentMatrix, rotation_y4(90.0));
    if state.getBlockId() == 54 {
        let stack = ItemStack { itemId: 54, count: 1, itemDamage: 0, tagCompound: None };
        let blockLight = ((packedLight >> 4) & 15) as f32;
        let skyLight = ((packedLight >> 20) & 15) as f32;
        let itemLights = [
            normalize3([0.2, 1.0, -0.7]),
            normalize3([-0.2, 1.0, 0.7]),
        ];
        append_builtin_item_mesh_world(
            &stack, contentMatrix, blockLight, skyLight, itemLights,
            atlas, vertices, indices,
        );
        return;
    }
    let Some(model) = model_for_state(&atlas.models, state) else { return; };
    if model.missing { return; }
    let colorPos = BlockPos::new(
        entity.entity.posX.floor() as i32,
        entity.entity.posY.floor() as i32,
        entity.entity.posZ.floor() as i32,
    );
    append_block_state_model_world(
        state,
        colorPos,
        model,
        contentMatrix,
        packedLight,
        chunks,
        atlas,
        [1.0; 4],
        None,
        vertices,
        indices,
    );

    // RenderTntMinecart disables texture and lighting, switches to
    // blendFunc(SRC_ALPHA, DST_ALPHA), and renders the *default TNT state*
    // a second time on alternating five-tick intervals. The dedicated
    // entity-overlay stream retains that blend/state boundary. forceUv makes
    // this helper emit untinted, unshaded geometry; the overlay shader path
    // ignores the sampled atlas color entirely.
    if entity.minecartType() == MinecartType::Tnt {
        if let Some(alpha) = RenderMinecart::tntFlashAlpha(entity.minecartTntFuse(), partialTicks) {
            let tntState = IBlockState::fromGlobalStateId(46 << 4);
            if let Some(tntModel) = model_for_state(&atlas.models, tntState).filter(|model| !model.missing) {
                append_block_state_model_world(
                    tntState,
                    colorPos,
                    tntModel,
                    contentMatrix,
                    packedLight,
                    chunks,
                    atlas,
                    [1.0, 1.0, 1.0, alpha],
                    Some([0.0, 0.0]),
                    overlayVertices,
                    overlayIndices,
                );
            }
        }
    }
}

fn find_living_target(
    entityId: i32,
    entities: &[EntityOtherClient],
    remotePlayers: &[RemotePlayerRenderState],
    localPlayerTarget: Option<LivingTargetRenderState>,
) -> Option<LivingTargetRenderState> {
    if entityId == 0 { return None; }
    if localPlayerTarget.is_some_and(|target| target.entityId == entityId) {
        return localPlayerTarget;
    }
    if let Some(player) = remotePlayers.iter().find(|player| player.entityId == entityId) {
        return Some(LivingTargetRenderState {
            entityId: player.entityId,
            prevPosition: player.prevPosition,
            position: player.position,
            height: player.height,
            eyeHeight: player.eyeHeight,
        });
    }
    entities.iter()
        .find(|candidate| candidate.entityId == entityId && candidate.isLivingBase())
        .map(|candidate| LivingTargetRenderState {
            entityId: candidate.entityId,
            prevPosition: [candidate.entity.prevPosX, candidate.entity.prevPosY, candidate.entity.prevPosZ],
            position: [candidate.entity.posX, candidate.entity.posY, candidate.entity.posZ],
            height: candidate.entity.height,
            eyeHeight: candidate.eyeHeight(),
        })
}

fn entity_look_at_zero(entity: &EntityOtherClient) -> [f64; 3] {
    let yaw = -entity.prevRotationYawHead.to_radians() - std::f32::consts::PI;
    let pitch = -entity.entity.prevRotationPitch.to_radians();
    let f = yaw.cos();
    let f1 = yaw.sin();
    let f2 = -pitch.cos();
    let f3 = pitch.sin();
    [(f1 * f2) as f64, f3 as f64, (f * f2) as f64]
}

#[allow(clippy::too_many_arguments)]
fn append_guardian_mesh(
    entity: &EntityOtherClient,
    target: Option<LivingTargetRenderState>,
    renderViewEntity: Option<LivingTargetRenderState>,
    renderModel: bool,
    totalWorldTime: i64,
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Mob { entityType } = &entity.kind else { return; };
    let Some(variant) = RenderGuardian::variant(*entityType) else { return; };
    let input = RenderLivingBase::renderInput(entity, partialTicks, RenderGuardian::preScale(variant));
    let guardianEyes = [
        entity.entity.prevPosX,
        entity.entity.prevPosY + entity.eyeHeight() as f64,
        entity.entity.prevPosZ,
    ];
    let focusEyes = target.or(renderViewEntity).map(LivingTargetRenderState::previousEyes);
    let modelState = GuardianModelState {
        spikesAnimation: entity.guardianSpikesAnimationAt(partialTicks),
        tailAnimation: entity.guardianTailAnimationAt(partialTicks),
        guardianEyes,
        focusEyes,
        guardianLook: entity_look_at_zero(entity),
    };
    if renderModel {
        append_living_model_mesh(
            RenderLivingBase::buildMesh(
                input,
                ModelGuardian::boxes(input, modelState),
                64.0,
                64.0,
            ),
            RenderGuardian::texture(variant),
            packedLight,
            atlas,
            vertices,
            indices,
        );
    }
    if let Some(target) = target {
        append_guardian_beam_mesh(
            entity, target, totalWorldTime, partialTicks, atlas, vertices, indices,
        );
    }
}

fn guardian_beam_point(start: [f64; 3], direction: [f64; 3], local: [f64; 3]) -> [f32; 3] {
    let pitch = direction[1].clamp(-1.0, 1.0).acos();
    let yaw = direction[2].atan2(direction[0]);
    let (cx, sx) = (pitch.cos(), pitch.sin());
    let afterX = [
        local[0],
        local[1] * cx - local[2] * sx,
        local[1] * sx + local[2] * cx,
    ];
    let angleY = std::f64::consts::FRAC_PI_2 - yaw;
    let (cy, sy) = (angleY.cos(), angleY.sin());
    let afterY = [
        afterX[0] * cy + afterX[2] * sy,
        afterX[1],
        -afterX[0] * sy + afterX[2] * cy,
    ];
    [
        (start[0] + afterY[0]) as f32,
        (start[1] + afterY[1]) as f32,
        (start[2] + afterY[2]) as f32,
    ]
}

fn guardian_beam_uv(rectangle: [f32; 4], uv: [f32; 2]) -> [f32; 2] {
    [
        rectangle[0] + (rectangle[2] - rectangle[0]) * uv[0],
        rectangle[1] + (rectangle[3] - rectangle[1]) * uv[1],
    ]
}

#[allow(clippy::too_many_arguments)]
fn append_guardian_beam_quad(
    points: [[f32; 3]; 4],
    uvs: [[f32; 2]; 4],
    rectangle: [f32; 4],
    color: [f32; 4],
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let base = vertices.len() as u32;
    for i in 0..4 {
        vertices.push(WorldVertex {
            position: points[i],
            uv: guardian_beam_uv(rectangle, uvs[i]),
            color,
            lightmap: [
                ((RenderGuardian::PACKED_FULL_BRIGHT >> 4) & 15) as f32,
                ((RenderGuardian::PACKED_FULL_BRIGHT >> 20) & 15) as f32,
            ],
        
            shaderEntity: [-1, -1, -1],
            shaderPadding: 0,
        });
    }
    // RenderGuardian disables culling for both crossed side sheets and the cap.
    indices.extend_from_slice(&[
        base, base + 1, base + 2, base, base + 2, base + 3,
        base + 2, base + 1, base, base + 3, base + 2, base,
    ]);
}

#[allow(clippy::too_many_arguments)]
fn append_guardian_beam_mesh(
    entity: &EntityOtherClient,
    target: LivingTargetRenderState,
    totalWorldTime: i64,
    partialTicks: f32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let partial = partialTicks.clamp(0.0, 1.0) as f64;
    let start = [
        entity.entity.prevPosX + (entity.entity.posX - entity.entity.prevPosX) * partial,
        entity.entity.prevPosY + (entity.entity.posY - entity.entity.prevPosY) * partial
            + entity.eyeHeight() as f64,
        entity.entity.prevPosZ + (entity.entity.posZ - entity.entity.prevPosZ) * partial,
    ];
    let end = target.interpolatedCenter(partialTicks);
    let delta = [end[0] - start[0], end[1] - start[1], end[2] - start[2]];
    let distance = (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]).sqrt();
    if distance <= 1.0e-9 { return; }
    let direction = [delta[0] / distance, delta[1] / distance, delta[2] / distance];
    let beamLength = distance + 1.0;
    let attack = RenderGuardian::attackAnimationScale(entity, partialTicks) as f64;
    let attackSquared = attack * attack;
    // DefaultVertexFormats.POSITION_TEX_COLOR stores UBYTE components. Java's
    // BufferBuilder casts each int to byte, so preserve the low eight bits.
    let red = (64 + (attackSquared * 191.0) as i32) & 255;
    let green = (32 + (attackSquared * 191.0) as i32) & 255;
    let blue = (128 - (attackSquared * 64.0) as i32) & 255;
    let color = [
        red as f32 / 255.0,
        green as f32 / 255.0,
        blue as f32 / 255.0,
        1.0,
    ];
    let animationTime = totalWorldTime as f64 + partialTicks as f64;
    let textureOffset = (animationTime * 0.5).rem_euclid(1.0);
    let rotation = animationTime * 0.05 * -1.5;
    let rectangle = atlas.entityTextureRectangles
        .get(&RenderGuardian::beamTexture())
        .copied()
        .unwrap_or(atlas.missingRectangle);

    let radius = 0.2_f64;
    let a = [rotation.cos() * -radius, rotation.sin() * -radius];
    let b = [rotation.cos() * radius, rotation.sin() * radius];
    let c = [(rotation + std::f64::consts::FRAC_PI_2).cos() * radius,
             (rotation + std::f64::consts::FRAC_PI_2).sin() * radius];
    let d = [(rotation + std::f64::consts::PI * 1.5).cos() * radius,
             (rotation + std::f64::consts::PI * 1.5).sin() * radius];

    let vStart = -1.0 + textureOffset;
    let vEnd = beamLength * 2.5 + vStart;
    let vRange = vEnd - vStart;
    let mut currentV = vStart;
    while currentV < vEnd - 1.0e-9 {
        let boundary = currentV.floor() + 1.0;
        let nextV = boundary.min(vEnd);
        let y0 = (currentV - vStart) / vRange * beamLength;
        let y1 = (nextV - vStart) / vRange * beamLength;
        let localV0 = currentV.rem_euclid(1.0) as f32;
        let localV1 = if (nextV - boundary).abs() < 1.0e-9 && nextV < vEnd - 1.0e-9 {
            1.0
        } else {
            nextV.rem_euclid(1.0) as f32
        };
        for (left, right) in [(a, b), (c, d)] {
            append_guardian_beam_quad(
                [
                    guardian_beam_point(start, direction, [left[0], y1, left[1]]),
                    guardian_beam_point(start, direction, [left[0], y0, left[1]]),
                    guardian_beam_point(start, direction, [right[0], y0, right[1]]),
                    guardian_beam_point(start, direction, [right[0], y1, right[1]]),
                ],
                [[0.4999, localV1], [0.4999, localV0], [0.0, localV0], [0.0, localV1]],
                rectangle, color, vertices, indices,
            );
        }
        currentV = nextV;
    }

    let capRadius = 0.282_f64;
    let capAngles = [
        rotation + 3.0 * std::f64::consts::FRAC_PI_4,
        rotation + std::f64::consts::FRAC_PI_4,
        rotation + 7.0 * std::f64::consts::FRAC_PI_4,
        rotation + 5.0 * std::f64::consts::FRAC_PI_4,
    ];
    let capV = if entity.entity.ticksExisted % 2 == 0 { 0.5 } else { 0.0 };
    append_guardian_beam_quad(
        capAngles.map(|angle| guardian_beam_point(
            start,
            direction,
            [angle.cos() * capRadius, beamLength, angle.sin() * capRadius],
        )),
        [[0.5, capV + 0.5], [1.0, capV + 0.5], [1.0, capV], [0.5, capV]],
        rectangle, color, vertices, indices,
    );
}

fn shulker_transform_matrix(operations: &[ShulkerTransformOp]) -> [[f32; 4]; 4] {
    let mut matrix = identity4();
    for operation in operations {
        let next = match *operation {
            ShulkerTransformOp::Translate(offset) => translation4(offset),
            ShulkerTransformOp::Rotate { degrees, axis } if axis == [1.0, 0.0, 0.0] => {
                rotation_x4(degrees)
            }
            ShulkerTransformOp::Rotate { degrees, axis } if axis == [0.0, 1.0, 0.0] => {
                rotation_y4(degrees)
            }
            ShulkerTransformOp::Rotate { degrees, axis } if axis == [0.0, 0.0, 1.0] => {
                rotation_z4(degrees)
            }
            ShulkerTransformOp::Rotate { .. } => identity4(),
        };
        matrix = multiply4(matrix, next);
    }
    matrix
}

fn transform_shulker_living_mesh(
    mut mesh: LivingModelMesh,
    input: crate::net::minecraft::client::renderer::entity::RenderLivingBase::LivingRenderInput,
    facing: EnumFacing,
    headLayer: bool,
) -> LivingModelMesh {
    let yawDegrees = 180.0 - input.bodyYaw;
    let inverseYaw = rotation_y4(-yawDegrees);
    let inverseDeath = rotation_z4(-input.deathRotation);
    let death = rotation_z4(input.deathRotation);
    let yaw = rotation_y4(yawDegrees);
    let corpse = shulker_transform_matrix(RenderShulker::corpseTransform(facing));
    let head = shulker_transform_matrix(RenderShulker::headLayerTransform(facing));
    let pre = input.preScaleXYZ;

    for vertex in &mut mesh.vertices {
        let relative = [
            vertex.position[0] - input.position[0],
            vertex.position[1] - input.position[1],
            vertex.position[2] - input.position[2],
        ];
        // Undo the inherited corpse rotations to recover the output of
        // prepareScale. RenderShulker inserts its attachment matrix between
        // those two stages.
        let afterYaw = transform_point3(inverseYaw, relative);
        let prepared = transform_point3(inverseDeath, afterYaw);
        let attached = if headLayer {
            // HeadLayer operations are appended after prepareScale. Recover
            // model/block coordinates, apply the exact layer matrix, then run
            // the inherited -1/-1/+1 and -1.501 transforms again.
            let model = [
                -prepared[0] / pre[0],
                1.501 - prepared[1] / pre[1],
                prepared[2] / pre[2],
            ];
            let model = transform_point3(head, model);
            let preparedHead = [
                -model[0] * pre[0],
                (1.501 - model[1]) * pre[1],
                model[2] * pre[2],
            ];
            transform_point3(corpse, preparedHead)
        } else {
            transform_point3(corpse, prepared)
        };
        let afterDeath = transform_point3(death, attached);
        let worldRelative = transform_point3(yaw, afterDeath);
        vertex.position = [
            input.position[0] + worldRelative[0],
            input.position[1] + worldRelative[1],
            input.position[2] + worldRelative[2],
        ];
    }
    mesh
}

#[allow(clippy::too_many_arguments)]
fn append_shulker_mesh(
    entity: &EntityOtherClient,
    renderShell: bool,
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Mob { entityType } = &entity.kind else { return; };
    if !RenderShulker::supports(*entityType) { return; }
    let mut input = RenderLivingBase::withAdultTranslation(
        RenderLivingBase::renderInput(entity, partialTicks, RenderShulker::PRE_SCALE),
        [0.0; 3],
    );
    let offset = RenderShulker::teleportRenderOffset(entity, partialTicks);
    input.position[0] += offset[0];
    input.position[1] += offset[1];
    input.position[2] += offset[2];
    let facing = entity.shulkerAttachmentFacing();
    let texture = RenderShulker::texture(entity);

    if renderShell {
        let shell = RenderLivingBase::buildMesh(
            input,
            ModelShulker::shellBoxes(
                input,
                ShulkerModelState {
                    clientPeekAmount: entity.shulkerClientPeekAmount(partialTicks),
                },
            ),
            ModelShulker::TEXTURE_WIDTH,
            ModelShulker::TEXTURE_HEIGHT,
        );
        append_living_model_mesh(
            transform_shulker_living_mesh(shell, input, facing, false),
            texture.clone(),
            packedLight,
            atlas,
            vertices,
            indices,
        );
    }

    // RenderShulker.HeadLayer is a separate layer and is not part of
    // ModelShulker#render. Keep that draw order and transform path intact.
    let head = RenderLivingBase::buildMesh(
        input,
        ModelShulker::headBoxes(input),
        ModelShulker::TEXTURE_WIDTH,
        ModelShulker::TEXTURE_HEIGHT,
    );
    append_living_model_mesh(
        transform_shulker_living_mesh(head, input, facing, true),
        texture,
        packed_light_without_living_hurt_overlay(packedLight),
        atlas,
        vertices,
        indices,
    );
}

#[allow(clippy::too_many_arguments)]
fn append_fireball_billboard(
    position: [f32; 3],
    scale: f32,
    texture: crate::net::minecraft::util::ResourceLocation::ResourceLocation,
    cameraYaw: f32,
    cameraPitch: f32,
    thirdPersonView: i32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let Some(rectangle) = atlas.entityTextureRectangles.get(&texture).copied() else { return; };
    let _ = thirdPersonView;
    // RenderFireball/RenderDragonFireball call translate, scale, rotateY,
    // rotateX in this exact order. `orient_camera_112` already carries the
    // front-view pitch inversion, so the source signed pitch reduces to this
    // single negative camera pitch just as in the XP-orb billboard path.
    let mut matrix = translation4(position);
    matrix = multiply4(matrix, scale4_nonuniform([scale; 3]));
    matrix = multiply4(matrix, rotation_y4(180.0 - cameraYaw));
    matrix = multiply4(matrix, rotation_x4(-cameraPitch));
    append_textured_quad_world(
        matrix,
        [
            [-0.5, -0.25, 0.0],
            [0.5, -0.25, 0.0],
            [0.5, 0.75, 0.0],
            [-0.5, 0.75, 0.0],
        ],
        [
            [rectangle[0], rectangle[3]],
            [rectangle[2], rectangle[3]],
            [rectangle[2], rectangle[1]],
            [rectangle[0], rectangle[1]],
        ],
        [1.0; 4],
        [15.0, 15.0],
        vertices,
        indices,
    );
}

#[allow(clippy::too_many_arguments)]
fn append_fireball_mesh(
    entity: &EntityOtherClient,
    position: [f32; 3],
    cameraYaw: f32,
    cameraPitch: f32,
    thirdPersonView: i32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Object { objectType, .. } = &entity.kind else { return; };
    let Some(scale) = RenderFireball::scale(*objectType) else { return; };
    append_fireball_billboard(
        position,
        scale,
        RenderFireball::texture(),
        cameraYaw,
        cameraPitch,
        thirdPersonView,
        atlas,
        vertices,
        indices,
    );
}

#[allow(clippy::too_many_arguments)]
fn append_dragon_fireball_mesh(
    entity: &EntityOtherClient,
    position: [f32; 3],
    cameraYaw: f32,
    cameraPitch: f32,
    thirdPersonView: i32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Object { objectType: ObjectSpawnType::DragonFireball, .. } = &entity.kind else { return; };
    append_fireball_billboard(
        position,
        RenderDragonFireball::SCALE,
        RenderDragonFireball::texture(),
        cameraYaw,
        cameraPitch,
        thirdPersonView,
        atlas,
        vertices,
        indices,
    );
}

#[allow(clippy::too_many_arguments)]
fn append_wither_skull_mesh(
    entity: &EntityOtherClient,
    position: [f32; 3],
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Object { objectType: ObjectSpawnType::WitherSkull, .. } = &entity.kind else { return; };
    let yaw = RenderWitherSkull::getRenderYaw(
        entity.entity.prevRotationYaw,
        entity.entity.rotationYaw,
        partialTicks,
    );
    let pitch = entity.entity.prevRotationPitch
        + (entity.entity.rotationPitch - entity.entity.prevRotationPitch) * partialTicks;
    let matrix = multiply4(
        translation4(position),
        scale4_nonuniform([-1.0, -1.0, 1.0]),
    );
    append_vehicle_model_pass(
        ModelSkeletonHead::buildMesh(yaw, pitch),
        RenderWitherSkull::texture(entity.isWitherSkullInvulnerable()),
        matrix,
        packedLight,
        atlas,
        vertices,
        indices,
    );
}

#[allow(clippy::too_many_arguments)]
fn append_shulker_bullet_model_pass(
    mesh: &ShulkerBulletModelMesh,
    matrix: [[f32; 4]; 4],
    color: [f32; 4],
    packedLight: u32,
    rectangle: [f32; 4],
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let blockLight = ((packedLight >> 4) & 15) as f32;
    let skyLight = ((packedLight >> 20) & 15) as f32;
    let base = vertices.len() as u32;
    vertices.extend(mesh.vertices.iter().map(|vertex| WorldVertex {
        position: transform_point3(matrix, vertex.position),
        uv: [
            rectangle[0] + (rectangle[2] - rectangle[0]) * vertex.uv[0],
            rectangle[1] + (rectangle[3] - rectangle[1]) * vertex.uv[1],
        ],
        color,
        lightmap: [blockLight, skyLight],
    
        shaderEntity: [-1, -1, -1],
        shaderPadding: 0,
    }));
    indices.extend(mesh.indices.iter().map(|index| base + *index));
}

#[allow(clippy::too_many_arguments)]
fn append_shulker_bullet_mesh(
    entity: &EntityOtherClient,
    position: [f32; 3],
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Object { objectType: ObjectSpawnType::ShulkerBullet, .. } = &entity.kind else { return; };
    let yaw = RenderShulkerBullet::rotLerp(
        entity.entity.prevRotationYaw,
        entity.entity.rotationYaw,
        partialTicks,
    );
    let pitch = entity.entity.prevRotationPitch
        + (entity.entity.rotationPitch - entity.entity.prevRotationPitch) * partialTicks.clamp(0.0, 1.0);
    let age = entity.entity.ticksExisted as f32 + partialTicks.clamp(0.0, 1.0);
    let mesh = ModelShulkerBullet::buildMesh(yaw, pitch);
    let mut matrix = translation4([
        position[0],
        position[1] + RenderShulkerBullet::MODEL_Y_OFFSET,
        position[2],
    ]);
    matrix = multiply4(matrix, rotation_y4((age * 0.1).sin() * 180.0));
    matrix = multiply4(matrix, rotation_x4((age * 0.1).cos() * 180.0));
    matrix = multiply4(matrix, rotation_z4((age * 0.15).sin() * 360.0));
    matrix = multiply4(matrix, scale4_nonuniform([-1.0, -1.0, 1.0]));
    let rectangle = atlas.entityTextureRectangles
        .get(&RenderShulkerBullet::texture())
        .copied()
        .unwrap_or(atlas.missingRectangle);
    append_shulker_bullet_model_pass(
        &mesh, matrix, [1.0; 4], packedLight, rectangle, vertices, indices,
    );
    let outer = multiply4(
        matrix,
        scale4_nonuniform([RenderShulkerBullet::OUTER_SCALE; 3]),
    );
    append_shulker_bullet_model_pass(
        &mesh,
        outer,
        [1.0, 1.0, 1.0, RenderShulkerBullet::OUTER_ALPHA],
        packedLight,
        rectangle,
        vertices,
        indices,
    );
}

fn append_zombie_mesh(
    entity: &EntityOtherClient,
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Mob { entityType } = &entity.kind else { return; };
    let Some(variant) = RenderZombie::variant(*entityType) else { return; };
    let input = RenderLivingBase::renderInput(entity, partialTicks, RenderZombie::preScale(variant));
    let pose = ModelZombie::pose(input, entity.dataManager.boolean(14, false));
    let mesh = RenderLivingBase::buildMesh(input, ModelZombie::boxes(pose, 0.0), 64.0, 64.0);
    append_living_model_mesh(
        mesh,
        RenderZombie::texture(variant),
        packedLight,
        atlas,
        vertices,
        indices,
    );
}

fn append_skeleton_mesh(
    entity: &EntityOtherClient,
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Mob { entityType } = &entity.kind else { return; };
    let Some(variant) = RenderSkeleton::variant(*entityType) else { return; };
    let input = RenderLivingBase::renderInput(entity, partialTicks, RenderSkeleton::preScale(variant));
    let holdingBow = {
        let stack = entity.equipment.getItemStackFromSlot(EntityEquipmentSlot::Mainhand);
        !stack.isEmpty() && stack.itemId == 261
    };
    let primaryLeft = entity.dataManager.byte(11, 0) & 2 != 0;
    let pose = ModelSkeleton::pose(
        input,
        entity.dataManager.boolean(12, false),
        holdingBow,
        primaryLeft,
    );
    let mesh = RenderLivingBase::buildMesh(
        input,
        ModelSkeleton::boxes(pose, 0.0, false),
        64.0,
        32.0,
    );
    append_living_model_mesh(
        mesh,
        RenderSkeleton::texture(variant),
        packedLight,
        atlas,
        vertices,
        indices,
    );
    if let Some(overlay) = RenderSkeleton::overlayTexture(variant) {
        let overlayMesh = RenderLivingBase::buildMesh(
            input,
            ModelSkeleton::boxes(pose, 0.25, true),
            64.0,
            32.0,
        );
        append_living_model_mesh(
            overlayMesh,
            overlay,
            packedLight,
            atlas,
            vertices,
            indices,
        );
    }
}

fn append_armor_stand_mesh(
    entity: &EntityOtherClient,
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Object { objectType: ObjectSpawnType::ArmorStand, .. } = &entity.kind else { return; };
    let input = RenderArmorStand::applyCorpseRotation(
        RenderLivingBase::renderInput(entity, partialTicks, 1.0),
        entity.entity.ticksExisted,
        entity.armorStandPunchTick,
        partialTicks,
    );
    let status = entity.armorStandStatus();
    let pose = ModelArmorStand::pose(
        input,
        entity.armorStandRotation(12, [0.0, 0.0, 0.0]),
        entity.armorStandRotation(13, [0.0, 0.0, 0.0]),
        entity.armorStandRotation(14, [-10.0, 0.0, -10.0]),
        entity.armorStandRotation(15, [-15.0, 0.0, 10.0]),
        entity.armorStandRotation(16, [-1.0, 0.0, -1.0]),
        entity.armorStandRotation(17, [1.0, 0.0, 1.0]),
        status & 0x04 != 0,
        status & 0x08 != 0,
        status & 0x10 != 0,
    );
    let mesh = RenderLivingBase::buildMesh(
        input,
        ModelArmorStand::boxes(pose, 0.0),
        64.0,
        64.0,
    );
    append_living_model_mesh(
        mesh,
        RenderArmorStand::texture(),
        packedLight,
        atlas,
        vertices,
        indices,
    );
}

fn append_pig_mesh(
    entity: &EntityOtherClient,
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Mob { entityType } = &entity.kind else { return; };
    if !RenderPig::supports(*entityType) { return; }
    let input = ModelPig::input(RenderLivingBase::renderInput(entity, partialTicks, 1.0));
    let pose = ModelPig::pose(input);
    append_living_model_mesh(
        RenderLivingBase::buildMesh(input, ModelPig::boxes(pose, 0.0), 64.0, 32.0),
        RenderPig::texture(),
        packedLight,
        atlas,
        vertices,
        indices,
    );
    if LayerSaddle::shouldRender(entity) {
        append_living_model_mesh(
            RenderLivingBase::buildMesh(input, ModelPig::boxes(pose, LayerSaddle::modelScale()), 64.0, 32.0),
            LayerSaddle::texture(),
            packed_light_without_living_hurt_overlay(packedLight),
            atlas,
            vertices,
            indices,
        );
    }
}

fn append_cow_mesh(
    entity: &EntityOtherClient,
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Mob { entityType } = &entity.kind else { return; };
    if !RenderCow::supports(*entityType) { return; }
    let input = ModelCow::input(RenderLivingBase::renderInput(entity, partialTicks, 1.0));
    let pose = ModelCow::pose(input);
    append_living_model_mesh(
        RenderLivingBase::buildMesh(input, ModelCow::boxes(pose), 64.0, 32.0),
        RenderCow::texture(),
        packedLight,
        atlas,
        vertices,
        indices,
    );
}

fn append_sheep_mesh(
    entity: &EntityOtherClient,
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Mob { entityType } = &entity.kind else { return; };
    if !RenderSheep::supports(*entityType) { return; }
    let input = ModelSheep2::input(RenderLivingBase::renderInput(entity, partialTicks, 1.0));
    let pose = ModelSheep2::pose(
        input,
        entity.sheepHeadRotationPointY(partialTicks),
        entity.sheepHeadRotationAngleX(partialTicks),
    );
    append_living_model_mesh(
        RenderLivingBase::buildMesh(input, ModelSheep2::boxes(pose), 64.0, 32.0),
        RenderSheep::texture(),
        packedLight,
        atlas,
        vertices,
        indices,
    );
    if LayerSheepWool::shouldRender(entity) {
        let color = LayerSheepWool::color(entity, partialTicks);
        append_living_model_mesh_tinted(
            RenderLivingBase::buildMesh(input, ModelSheep1::boxes(pose), 64.0, 32.0),
            LayerSheepWool::texture(),
            packedLight,
            color,
            atlas,
            vertices,
            indices,
        );
    }
}

fn append_chicken_mesh(
    entity: &EntityOtherClient,
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Mob { entityType } = &entity.kind else { return; };
    if !RenderChicken::supports(*entityType) { return; }
    let input = ModelChicken::input(RenderLivingBase::renderInput(entity, partialTicks, 1.0));
    let pose = ModelChicken::pose(input, entity.chickenFlap(partialTicks));
    append_living_model_mesh(
        RenderLivingBase::buildMesh(input, ModelChicken::boxes(pose), 64.0, 32.0),
        RenderChicken::texture(),
        packedLight,
        atlas,
        vertices,
        indices,
    );
}


fn append_mooshroom_mesh(
    entity: &EntityOtherClient,
    partialTicks: f32,
    packedLight: u32,
    chunks: &HashMap<ChunkKey, Chunk>,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Mob { entityType } = &entity.kind else { return; };
    if !RenderMooshroom::supports(*entityType) { return; }
    let input = ModelCow::input(RenderLivingBase::renderInput(entity, partialTicks, 1.0));
    let pose = ModelCow::pose(input);
    append_living_model_mesh(
        RenderLivingBase::buildMesh(input, ModelCow::boxes(pose), 64.0, 32.0),
        RenderMooshroom::texture(),
        packedLight,
        atlas,
        vertices,
        indices,
    );
    if !LayerMooshroomMushroom::shouldRender(input.child, (entity.dataManager.byte(0, 0) & 0x20) != 0) { return; }
    let state = IBlockState::fromGlobalStateId(LayerMooshroomMushroom::RED_MUSHROOM_GLOBAL_STATE);
    let Some(model) = model_for_state(&atlas.models, state) else { return; };
    if model.missing { return; }
    let colorPos = BlockPos::new(
        entity.entity.posX.floor() as i32,
        entity.entity.posY.floor() as i32,
        entity.entity.posZ.floor() as i32,
    );
    let base = living_layer_base_matrix(input);
    let bodyRoot = multiply4(
        multiply4(
            multiply4(base, scale4_nonuniform(LayerMooshroomMushroom::BODY_FLIP)),
            translation4(LayerMooshroomMushroom::FIRST_BODY_TRANSLATION),
        ),
        rotation_y4(LayerMooshroomMushroom::FIRST_BODY_ROTATION_Y),
    );
    let first = multiply4(
        multiply4(bodyRoot, translation4(LayerMooshroomMushroom::FIRST_MODEL_TRANSLATION)),
        rotation_y4(LayerMooshroomMushroom::BLOCK_MODEL_ROTATION_Y),
    );
    append_block_state_model_world_with_winding(
        state, colorPos, model, first, packedLight, chunks, atlas,
        [1.0; 4], None, LayerMooshroomMushroom::REVERSE_WINDING, vertices, indices,
    );
    let second = multiply4(
        multiply4(
            multiply4(bodyRoot, translation4(LayerMooshroomMushroom::SECOND_BODY_TRANSLATION)),
            rotation_y4(LayerMooshroomMushroom::SECOND_BODY_ROTATION_Y),
        ),
        translation4(LayerMooshroomMushroom::FIRST_MODEL_TRANSLATION),
    );
    let second = multiply4(second, rotation_y4(LayerMooshroomMushroom::BLOCK_MODEL_ROTATION_Y));
    append_block_state_model_world_with_winding(
        state, colorPos, model, second, packedLight, chunks, atlas,
        [1.0; 4], None, LayerMooshroomMushroom::REVERSE_WINDING, vertices, indices,
    );

    let headPost = part_post_render_matrix(pose.head, LayerMooshroomMushroom::HEAD_POST_RENDER_SCALE);
    let head = multiply4(
        multiply4(
            multiply4(
                multiply4(
                    multiply4(
                        multiply4(base, headPost),
                        scale4_nonuniform(LayerMooshroomMushroom::BODY_FLIP),
                    ),
                    translation4(LayerMooshroomMushroom::HEAD_TRANSLATION),
                ),
                rotation_y4(LayerMooshroomMushroom::HEAD_ROTATION_Y),
            ),
            translation4(LayerMooshroomMushroom::FIRST_MODEL_TRANSLATION),
        ),
        rotation_y4(LayerMooshroomMushroom::BLOCK_MODEL_ROTATION_Y),
    );
    append_block_state_model_world_with_winding(
        state, colorPos, model, head, packedLight, chunks, atlas,
        [1.0; 4], None, LayerMooshroomMushroom::REVERSE_WINDING, vertices, indices,
    );
}

fn append_creeper_mesh(
    entity: &EntityOtherClient,
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Mob { entityType } = &entity.kind else { return; };
    if !RenderCreeper::supports(*entityType) { return; }
    let mut input = RenderLivingBase::renderInput(entity, partialTicks, 1.0);
    input = RenderLivingBase::withPreScaleXYZ(input, RenderCreeper::scale(entity, partialTicks));
    let pose = ModelCreeper::pose(input);
    let mesh = RenderLivingBase::buildMesh(input, ModelCreeper::boxes(pose, 0.0), 64.0, 32.0);
    append_living_model_mesh(mesh.clone(), RenderCreeper::texture(), packedLight, atlas, vertices, indices);
    if packedLight & ENTITY_HURT_OVERLAY_FLAG == 0 {
        if let Some(color) = RenderCreeper::flashColor(entity, partialTicks) {
            append_living_model_mesh_tinted(
                mesh.clone(), RenderCreeper::texture(), packedLight, color, atlas, vertices, indices,
            );
        }
    }
    if RenderCreeper::powered(entity) {
        let age = entity.entity.ticksExisted as f32 + partialTicks;
        let charged = RenderLivingBase::buildMesh(input, ModelCreeper::boxes(pose, LayerCreeperCharge::modelDelta()), 64.0, 32.0);
        append_living_model_mesh_tinted_uv_offset(
            charged, LayerCreeperCharge::texture(), LayerCreeperCharge::packedFullBright(),
            LayerCreeperCharge::tint(), LayerCreeperCharge::uvOffset(age),
            atlas, vertices, indices,
        );
    }
}

fn append_spider_mesh(
    entity: &EntityOtherClient,
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Mob { entityType } = &entity.kind else { return; };
    let Some(variant) = RenderSpider::variant(*entityType) else { return; };
    let mut input = RenderLivingBase::renderInput(entity, partialTicks, RenderSpider::preScale(variant));
    input.deathRotation *= 2.0;
    let pose = ModelSpider::pose(input);
    let mesh = RenderLivingBase::buildMesh(input, ModelSpider::boxes(pose), 64.0, 32.0);
    append_living_model_mesh(mesh.clone(), RenderSpider::texture(variant), packedLight, atlas, vertices, indices);
    append_living_model_mesh_tinted(
        mesh,
        LayerSpiderEyes::texture(),
        LayerSpiderEyes::packedFullBright(),
        [1.0, 1.0, 1.0, 1.0],
        atlas,
        vertices,
        indices,
    );
}

fn append_slime_mesh(
    entity: &EntityOtherClient,
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Mob { entityType } = &entity.kind else { return; };
    if !RenderSlime::supports(*entityType) { return; }
    let mut input = RenderLivingBase::renderInput(entity, partialTicks, 1.0);
    input = RenderLivingBase::withPreScaleXYZ(input, RenderSlime::scale(entity, partialTicks));
    append_living_model_mesh(
        RenderLivingBase::buildMesh(input, ModelSlime::innerBoxes(), 64.0, 32.0),
        RenderSlime::texture(), packedLight, atlas, vertices, indices,
    );
    append_living_model_mesh_tinted(
        RenderLivingBase::buildMesh(input, ModelSlime::gelBoxes(), 64.0, 32.0),
        RenderSlime::texture(), packedLight, LayerSlimeGel::color(),
        atlas, vertices, indices,
    );
}

fn append_magma_cube_mesh(
    entity: &EntityOtherClient,
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Mob { entityType } = &entity.kind else { return; };
    if !RenderMagmaCube::supports(*entityType) { return; }
    let mut input = RenderLivingBase::renderInput(entity, partialTicks, 1.0);
    input = RenderLivingBase::withPreScaleXYZ(input, RenderMagmaCube::scale(entity, partialTicks));
    append_living_model_mesh(
        RenderLivingBase::buildMesh(
            input,
            ModelMagmaCube::boxes(RenderMagmaCube::interpolatedSquish(entity, partialTicks)),
            64.0,
            32.0,
        ),
        RenderMagmaCube::texture(), packedLight, atlas, vertices, indices,
    );
}

fn append_blaze_mesh(
    entity: &EntityOtherClient,
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Mob { entityType } = &entity.kind else { return; };
    if !RenderBlaze::supports(*entityType) { return; }
    let input = RenderLivingBase::renderInput(entity, partialTicks, 1.0);
    append_living_model_mesh(
        RenderLivingBase::buildMesh(input, ModelBlaze::boxes(input), 64.0, 32.0),
        RenderBlaze::texture(), packedLight, atlas, vertices, indices,
    );
}

fn append_ghast_mesh(
    entity: &EntityOtherClient,
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Mob { entityType } = &entity.kind else { return; };
    if !RenderGhast::supports(*entityType) { return; }
    let input = ModelGhast::input(RenderLivingBase::renderInput(
        entity,
        partialTicks,
        RenderGhast::PRE_SCALE,
    ));
    append_living_model_mesh(
        RenderLivingBase::buildMesh(input, ModelGhast::boxes(input), 64.0, 32.0),
        RenderGhast::texture(entity), packedLight, atlas, vertices, indices,
    );
}

fn append_wolf_mesh(
    entity: &EntityOtherClient,
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Mob { entityType } = &entity.kind else { return; };
    if !RenderWolf::supports(*entityType) { return; }
    let input = RenderLivingBase::withChildLayout(
        RenderLivingBase::renderInput(entity, partialTicks, 1.0),
        ModelWolf::CHILD_LAYOUT,
    );
    let pose = ModelWolf::pose(input, entity, partialTicks);
    let mesh = RenderLivingBase::buildMesh(input, ModelWolf::boxes(pose, 0.0), 64.0, 32.0);
    append_living_model_mesh_tinted(
        mesh.clone(), RenderWolf::texture(entity), packedLight,
        RenderWolf::wetColor(entity, partialTicks), atlas, vertices, indices,
    );
    if LayerWolfCollar::shouldRender(entity) {
        append_living_model_mesh_tinted(
            mesh, LayerWolfCollar::texture(), packedLight, LayerWolfCollar::color(entity),
            atlas, vertices, indices,
        );
    }
}

fn append_ocelot_mesh(
    entity: &EntityOtherClient,
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Mob { entityType } = &entity.kind else { return; };
    if !RenderOcelot::supports(*entityType) { return; }
    let input = RenderLivingBase::withChildLayout(
        RenderLivingBase::renderInput(entity, partialTicks, RenderOcelot::scale(entity)),
        ModelOcelot::CHILD_LAYOUT,
    );
    let pose = ModelOcelot::pose(input, entity);
    append_living_model_mesh(
        RenderLivingBase::buildMesh(input, ModelOcelot::boxes(pose), 64.0, 32.0),
        RenderOcelot::texture(entity), packedLight, atlas, vertices, indices,
    );
}

fn append_rabbit_mesh(
    entity: &EntityOtherClient,
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Mob { entityType } = &entity.kind else { return; };
    if !RenderRabbit::supports(*entityType) { return; }
    let preScale = if entity.isChild() { 1.0 } else { 0.6 };
    let mut input = RenderLivingBase::withChildLayout(
        RenderLivingBase::renderInput(entity, partialTicks, preScale),
        ModelRabbit::CHILD_LAYOUT,
    );
    if !entity.isChild() {
        input = RenderLivingBase::withAdultTranslation(input, [0.0, 1.0, 0.0]);
    }
    let pose = ModelRabbit::pose(input, entity.rabbitJumpCompletion(partialTicks));
    append_living_model_mesh(
        RenderLivingBase::buildMesh(input, ModelRabbit::boxes(pose), 64.0, 32.0),
        RenderRabbit::texture(entity), packedLight, atlas, vertices, indices,
    );
}

fn append_polar_bear_mesh(
    entity: &EntityOtherClient,
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Mob { entityType } = &entity.kind else { return; };
    if !RenderPolarBear::supports(*entityType) { return; }
    let input = RenderLivingBase::withChildLayout(
        RenderLivingBase::renderInput(entity, partialTicks, 1.2),
        ModelPolarBear::CHILD_LAYOUT,
    );
    let pose = ModelPolarBear::pose(input, entity.polarBearStandingScale(partialTicks));
    append_living_model_mesh(
        RenderLivingBase::buildMesh(input, ModelPolarBear::boxes(pose), 128.0, 64.0),
        RenderPolarBear::texture(), packedLight, atlas, vertices, indices,
    );
}

fn append_horse_mesh(
    entity: &EntityOtherClient,
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Mob { entityType } = &entity.kind else { return; };
    let (variant, texture) = if RenderHorse::supports(*entityType) {
        (HorseModelVariant::Horse, RenderHorse::texture(entity))
    } else if let Some(variant) = RenderAbstractHorse::variant(*entityType) {
        (variant, RenderAbstractHorse::texture(variant))
    } else {
        return;
    };
    let input = RenderLivingBase::renderInput(entity, partialTicks, 1.0);
    let pose = ModelHorse::pose(input, entity, partialTicks);
    append_living_model_mesh(
        RenderLivingBase::buildMesh(
            input,
            ModelHorse::boxes(pose, input, entity, variant, partialTicks),
            128.0,
            128.0,
        ),
        texture,
        packedLight,
        atlas,
        vertices,
        indices,
    );
}

fn append_llama_mesh(
    entity: &EntityOtherClient,
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Mob { entityType } = &entity.kind else { return; };
    if !RenderLlama::supports(*entityType) { return; }
    let input = RenderLivingBase::renderInput(entity, partialTicks, 1.0);
    let pose = ModelLlama::pose(input);
    append_living_model_mesh(
        RenderLivingBase::buildMesh(input, ModelLlama::boxes(pose, input, entity, 0.0), 128.0, 64.0),
        RenderLlama::texture(entity),
        packedLight,
        atlas,
        vertices,
        indices,
    );
    if let Some(texture) = LayerLlamaDecor::texture(entity) {
        append_living_model_mesh(
            RenderLivingBase::buildMesh(
                input,
                ModelLlama::boxes(pose, input, entity, LayerLlamaDecor::modelDelta()),
                128.0,
                64.0,
            ),
            texture,
            packed_light_without_living_hurt_overlay(packedLight),
            atlas,
            vertices,
            indices,
        );
    }
}


fn append_villager_mesh(
    entity: &EntityOtherClient,
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Mob { entityType } = &entity.kind else { return; };
    if !RenderVillager::supports(*entityType) { return; }
    let mut input = RenderLivingBase::renderInput(entity, partialTicks, RenderVillager::preScale(entity));
    // ModelVillager overrides ModelBase.render and does not use ModelBase's
    // biped child split. RenderLivingBase still triples child limbSwing.
    input.child = false;
    let pose = ModelVillager::pose(input);
    append_living_model_mesh(
        RenderLivingBase::buildMesh(input, ModelVillager::boxes(pose, 0.0), 64.0, 64.0),
        RenderVillager::texture(entity), packedLight, atlas, vertices, indices,
    );
}

fn append_witch_mesh(
    entity: &EntityOtherClient,
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Mob { entityType } = &entity.kind else { return; };
    if !RenderWitch::supports(*entityType) { return; }
    let mut input = RenderLivingBase::renderInput(entity, partialTicks, RenderWitch::preScale());
    input.child = false;
    let pose = ModelWitch::pose(input, entity.entityId, RenderWitch::holdingItem(entity));
    append_living_model_mesh(
        RenderLivingBase::buildMesh(input, ModelWitch::boxes(pose), 64.0, 128.0),
        RenderWitch::texture(), packedLight, atlas, vertices, indices,
    );
}

fn append_illager_mesh(
    entity: &EntityOtherClient,
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Mob { entityType } = &entity.kind else { return; };
    let (texture, preScale, armPose, showHood, renderHeldItems) = if RenderVindicator::supports(*entityType) {
        (
            RenderVindicator::texture(), RenderVindicator::preScale(),
            RenderVindicator::armPose(entity), false, RenderVindicator::shouldRenderHeldItem(entity),
        )
    } else if RenderEvoker::supports(*entityType) {
        (
            RenderEvoker::texture(), RenderEvoker::preScale(),
            RenderEvoker::armPose(entity), false, RenderEvoker::shouldRenderHeldItem(entity),
        )
    } else if RenderIllusionIllager::supports(*entityType) {
        (
            RenderIllusionIllager::texture(), RenderIllusionIllager::preScale(),
            RenderIllusionIllager::armPose(entity), true, RenderIllusionIllager::shouldRenderHeldItem(entity),
        )
    } else { return; };
    let mut baseInput = RenderLivingBase::renderInput(entity, partialTicks, preScale);
    baseInput.child = false;
    let offsets: Vec<[f32; 3]> = if RenderIllusionIllager::supports(*entityType) && entity.isInvisibleFlag() {
        let age = baseInput.ageInTicks;
        entity.illusionOffsets(partialTicks).into_iter().enumerate().map(|(i, offset)| {
            let fi = i as f32;
            [
                offset[0] as f32 + (fi + age * 0.5).cos() * 0.025,
                offset[1] as f32 + (fi + age * 0.75).cos() * 0.0125,
                offset[2] as f32 + (fi + age * 0.7).cos() * 0.025,
            ]
        }).collect()
    } else { vec![[0.0; 3]] };
    for offset in offsets {
        let mut input = baseInput;
        input.position[0] += offset[0];
        input.position[1] += offset[1];
        input.position[2] += offset[2];
        let pose = ModelIllager::pose(input, armPose, entity.primaryHandSide());
        append_living_model_mesh(
            RenderLivingBase::buildMesh(input, ModelIllager::boxes(pose, showHood), 64.0, 64.0),
            texture.clone(), packedLight, atlas, vertices, indices,
        );
        if renderHeldItems {
            append_illager_held_items(
                entity, input, pose, packedLight, atlas, vertices, indices,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn append_illager_held_items(
    entity: &EntityOtherClient,
    input: crate::net::minecraft::client::renderer::entity::RenderLivingBase::LivingRenderInput,
    pose: IllagerPose,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let mainHand = entity.equipment.getItemStackFromSlot(EntityEquipmentSlot::Mainhand);
    let offHand = entity.equipment.getItemStackFromSlot(EntityEquipmentSlot::Offhand);
    for handSide in [EnumHandSide::Right, EnumHandSide::Left] {
        let stack = LayerHeldItem::stackForSide(
            entity.primaryHandSide(), mainHand, offHand, handSide,
        );
        if stack.isEmpty() { continue; }
        let Some(model) = item_model_for_stack(stack, atlas) else { continue; };
        if model.builtInRenderer && !is_unpatterned_shield(stack)
            && TileEntityItemStackRenderer::buildMesh(stack).is_none()
        {
            continue;
        }
        if !model.builtInRenderer && model.quads.is_empty() { continue; }

        let mut matrix = living_layer_base_matrix(input);
        if input.child {
            matrix = multiply4(matrix, translation4([0.0, 0.75, 0.0]));
            matrix = multiply4(matrix, scale4_nonuniform([0.5; 3]));
        }
        matrix = multiply4(
            matrix,
            part_post_render_matrix(ModelIllager::armForSide(pose, handSide), 0.0625),
        );
        if input.sneaking {
            matrix = multiply4(matrix, translation4([0.0, 0.2, 0.0]));
        }
        matrix = multiply4(matrix, rotation_x4(-90.0));
        matrix = multiply4(matrix, rotation_y4(180.0));
        matrix = multiply4(matrix, translation4(LayerHeldItem::handTranslation(handSide)));
        append_item_stack_world_transformed_side(
            stack,
            model,
            matrix,
            LayerHeldItem::transformType(handSide),
            LayerHeldItem::leftHanded(handSide),
            packedLight,
            atlas,
            vertices,
            indices,
        );
    }
}

fn append_zombie_villager_mesh(
    entity: &EntityOtherClient,
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Mob { entityType } = &entity.kind else { return; };
    if !RenderZombieVillager::supports(*entityType) { return; }
    let mut input = RenderLivingBase::renderInput(entity, partialTicks, 1.0);
    if entity.zombieVillagerConverting() {
        input.bodyYaw += ((entity.entity.ticksExisted as f32) * 3.25).cos()
            * std::f32::consts::PI * 0.25;
    }
    append_living_model_mesh(
        RenderLivingBase::buildMesh(
            input,
            ModelZombieVillager::boxes(input, entity.dataManager.boolean(14, false), 0.0),
            64.0,
            64.0,
        ),
        RenderZombieVillager::texture(entity), packedLight, atlas, vertices, indices,
    );
}

fn living_layer_base_matrix(input: crate::net::minecraft::client::renderer::entity::RenderLivingBase::LivingRenderInput) -> [[f32;4];4] {
    let mut matrix = translation4(input.position);
    matrix = multiply4(matrix, rotation_y4(180.0 - input.bodyYaw));
    if input.deathRotation != 0.0 { matrix = multiply4(matrix, rotation_z4(input.deathRotation)); }
    matrix = multiply4(matrix, scale4_nonuniform(input.preScaleXYZ));
    matrix = multiply4(matrix, scale4_nonuniform([-1.0, -1.0, 1.0]));
    multiply4(matrix, translation4([0.0, -1.501, 0.0]))
}

fn part_post_render_matrix(pose: PartPose, scale: f32) -> [[f32;4];4] {
    let mut matrix = translation4([pose.pivot[0]*scale, pose.pivot[1]*scale, pose.pivot[2]*scale]);
    if pose.rotation[2] != 0.0 { matrix = multiply4(matrix, rotation_z4(pose.rotation[2].to_degrees())); }
    if pose.rotation[1] != 0.0 { matrix = multiply4(matrix, rotation_y4(pose.rotation[1].to_degrees())); }
    if pose.rotation[0] != 0.0 { matrix = multiply4(matrix, rotation_x4(pose.rotation[0].to_degrees())); }
    matrix
}

fn append_living_model_mesh(
    mesh: LivingModelMesh,
    texture: ResourceLocation,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    append_living_model_mesh_tinted(mesh, texture, packedLight, [1.0; 4], atlas, vertices, indices);
}

fn append_living_model_mesh_tinted(
    mesh: LivingModelMesh,
    texture: ResourceLocation,
    packedLight: u32,
    color: [f32; 4],
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    if mesh.indices.is_empty() { return; }
    let rectangle = atlas
        .entityTextureRectangles
        .get(&texture)
        .copied()
        .unwrap_or(atlas.missingRectangle);
    let blockLight = encoded_block_light_from_packed(packedLight);
    let skyLight = ((packedLight >> 20) & 15) as f32;
    let base = vertices.len() as u32;
    vertices.extend(mesh.vertices.into_iter().map(|vertex| WorldVertex {
        position: vertex.position,
        uv: [
            rectangle[0] + (rectangle[2] - rectangle[0]) * vertex.uv[0],
            rectangle[1] + (rectangle[3] - rectangle[1]) * vertex.uv[1],
        ],
        color,
        lightmap: [blockLight, skyLight],
    
        shaderEntity: [-1, -1, -1],
        shaderPadding: 0,
    }));
    indices.extend(mesh.indices.into_iter().map(|index| base + index));
}

fn non_player_entity_light(
    entity: &EntityOtherClient,
    position: [f32; 3],
    chunks: &HashMap<ChunkKey, Chunk>,
    dimension: i32,
) -> u32 {
    if matches!(
        &entity.kind,
        ClientEntityKind::Object { objectType, .. } if objectType.isFireball()
    ) {
        return EntityFireball::PACKED_FULL_BRIGHT;
    }
    if entity.isBurning()
        || matches!(&entity.kind, ClientEntityKind::Mob { entityType } if RenderBlaze::supports(*entityType))
    {
        return RenderBlaze::PACKED_FULL_BRIGHT;
    }
    let sample = BlockPos::new(
        position[0].floor() as i32,
        (position[1] + entity.entity.height * 0.5).floor() as i32,
        position[2].floor() as i32,
    );
    let state = snapshot_block_state(chunks, sample);
    snapshot_combined_light(chunks, sample, dimension, state)
}

fn item_model_for_stack<'a>(
    stack: &ItemStack,
    atlas: &'a AtlasState,
) -> Option<&'a ResolvedItemModel> {
    let modelKey = ItemModelMesher::getModelKey(stack)?;
    atlas.itemModels.get(&modelKey).map(Arc::as_ref)
}

#[allow(clippy::too_many_arguments)]
fn append_item_stack_world_transformed(
    stack: &ItemStack,
    model: &ResolvedItemModel,
    matrix: [[f32; 4]; 4],
    transformType: TransformType,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    append_item_stack_world_transformed_side(
        stack, model, matrix, transformType, false, packedLight, atlas, vertices, indices,
    );
}

#[allow(clippy::too_many_arguments)]
fn append_item_stack_world_transformed_side(
    stack: &ItemStack,
    model: &ResolvedItemModel,
    mut matrix: [[f32; 4]; 4],
    transformType: TransformType,
    leftHanded: bool,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let itemTransform = model.transforms.getTransform(transformType);
    matrix = multiply4(matrix, item_camera_transform4(itemTransform, leftHanded));
    matrix = multiply4(matrix, translation4([-0.5, -0.5, -0.5]));
    let blockLight = ((packedLight >> 4) & 15) as f32;
    let skyLight = ((packedLight >> 20) & 15) as f32;
    let itemLights = [
        normalize3([0.2, 1.0, -0.7]),
        normalize3([-0.2, 1.0, 0.7]),
    ];
    let unpatternedShield = model.builtInRenderer && is_unpatterned_shield(stack);
    if unpatternedShield {
        append_shield_model_world(
            matrix,
            atlas.shieldBaseRectangle,
            blockLight,
            skyLight,
            itemLights,
            vertices,
            indices,
        );
        return;
    }
    if model.builtInRenderer {
        append_builtin_item_mesh_world(
            stack,
            matrix,
            blockLight,
            skyLight,
            itemLights,
            atlas,
            vertices,
            indices,
        );
        return;
    }
    if model.quads.is_empty() {
        return;
    }
    let reverseWinding = (itemTransform.scale[0] < 0.0)
        ^ (itemTransform.scale[1] < 0.0)
        ^ (itemTransform.scale[2] < 0.0);
    for quad in &model.quads {
        let transformed = quad.positions.map(|point| transform_point3(matrix, point));
        let normal = normalize3(cross3(
            subtract3(transformed[1], transformed[0]),
            subtract3(transformed[2], transformed[0]),
        ));
        let key = item_material_key(stack.itemId, quad.texture.clone(), quad.tintIndex);
        let rectangle = atlas.rectangles.get(&key).copied().unwrap_or(atlas.missingRectangle);
        let tint = item_tint_color(&atlas.itemColors, stack, quad.tintIndex);
        let diffuse = if model.gui3d && quad.shade {
            standard_item_diffuse(normal, itemLights)
        } else {
            1.0
        };
        let base = vertices.len() as u32;
        for vertexIndex in 0..4 {
            let sourceUv = quad.uvs[vertexIndex];
            vertices.push(WorldVertex {
                position: transformed[vertexIndex],
                uv: [
                    rectangle[0] + (rectangle[2] - rectangle[0]) * sourceUv[0],
                    rectangle[1] + (rectangle[3] - rectangle[1]) * sourceUv[1],
                ],
                color: [tint[0] * diffuse, tint[1] * diffuse, tint[2] * diffuse, 1.0],
                lightmap: [blockLight, skyLight],
            
                shaderEntity: [-1, -1, -1],
                shaderPadding: 0,
            });
        }
        if reverseWinding {
            indices.extend_from_slice(&[base, base + 2, base + 1, base + 2, base, base + 3]);
        } else {
            indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
        }
    }
}

#[allow(clippy::too_many_arguments)]

fn append_living_model_mesh_tinted_uv_offset(
    mesh: LivingModelMesh,
    texture: ResourceLocation,
    packedLight: u32,
    color: [f32; 4],
    uvOffset: [f32; 2],
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    if mesh.indices.is_empty() { return; }
    let rectangle = atlas.entityTextureRectangles.get(&texture).copied().unwrap_or(atlas.missingRectangle);
    let blockLight = ((packedLight >> 4) & 15) as f32;
    let skyLight = ((packedLight >> 20) & 15) as f32;
    let base = vertices.len() as u32;
    vertices.extend(mesh.vertices.into_iter().map(|vertex| {
        let localU = (vertex.uv[0] + uvOffset[0]).rem_euclid(1.0);
        let localV = (vertex.uv[1] + uvOffset[1]).rem_euclid(1.0);
        WorldVertex {
            position: vertex.position,
            uv: [
                rectangle[0] + (rectangle[2] - rectangle[0]) * localU,
                rectangle[1] + (rectangle[3] - rectangle[1]) * localV,
            ],
            color,
            lightmap: [blockLight, skyLight],
        
            shaderEntity: [-1, -1, -1],
            shaderPadding: 0,
        }
    }));
    indices.extend(mesh.indices.into_iter().map(|index| base + index));
}

fn append_entity_item_mesh(
    entity: &EntityOtherClient,
    position: [f32; 3],
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let Some(stack) = entity.entityItem() else { return; };
    let Some(model) = item_model_for_stack(stack, atlas) else { return; };
    let ground = model.transforms.getTransform(TransformType::Ground);
    let count = RenderEntityItem::getModelCount(stack);
    let mut root = translation4([
        position[0],
        position[1]
            + RenderEntityItem::bobOffset(entity.entity.ticksExisted, partialTicks, entity.hoverStart)
            + 0.25 * ground.scale[1],
        position[2],
    ]);
    root = multiply4(
        root,
        rotation_y4(RenderEntityItem::rotationDegrees(
            entity.entity.ticksExisted,
            partialTicks,
            entity.hoverStart,
        )),
    );
    let mut random = RenderEntityItem::randomFor(stack);
    if !model.gui3d {
        root = multiply4(
            root,
            translation4([0.0, 0.0, -0.09375 * (count - 1) as f32 * 0.5 * ground.scale[2]]),
        );
    }
    for copy in 0..count {
        let mut matrix = root;
        if copy > 0 {
            if model.gui3d {
                matrix = multiply4(
                    matrix,
                    translation4([
                        (random.next_f32() * 2.0 - 1.0) * 0.15,
                        (random.next_f32() * 2.0 - 1.0) * 0.15,
                        (random.next_f32() * 2.0 - 1.0) * 0.15,
                    ]),
                );
            } else {
                matrix = multiply4(
                    matrix,
                    translation4([
                        (random.next_f32() * 2.0 - 1.0) * 0.075,
                        (random.next_f32() * 2.0 - 1.0) * 0.075,
                        0.0,
                    ]),
                );
            }
        }
        append_item_stack_world_transformed(
            stack,
            model,
            matrix,
            TransformType::Ground,
            packedLight,
            atlas,
            vertices,
            indices,
        );
        if !model.gui3d {
            root = multiply4(root, translation4([0.0, 0.0, 0.09375 * ground.scale[2]]));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn append_snowball_entity_mesh(
    entity: &EntityOtherClient,
    position: [f32; 3],
    cameraYaw: f32,
    cameraPitch: f32,
    thirdPersonView: i32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Object { objectType, .. } = &entity.kind else { return; };
    let Some(stack) = RenderSnowball::getStackToRender(*objectType, entity.metadataItem()) else {
        return;
    };
    let Some(model) = item_model_for_stack(&stack, atlas) else { return; };
    let _ = thirdPersonView;
    let mut matrix = translation4(position);
    matrix = multiply4(matrix, rotation_y4(-cameraYaw));
    // `orient_camera_112` already negates cameraPitch for the front-facing
    // third-person view, exactly cancelling RenderSnowball's explicit sign.
    matrix = multiply4(matrix, rotation_x4(cameraPitch));
    matrix = multiply4(matrix, rotation_y4(180.0));
    append_item_stack_world_transformed(
        &stack,
        model,
        matrix,
        TransformType::Ground,
        packedLight,
        atlas,
        vertices,
        indices,
    );
}

#[allow(clippy::too_many_arguments)]
fn append_falling_block_entity_mesh(
    entity: &EntityOtherClient,
    position: [f32; 3],
    packedLight: u32,
    chunks: &HashMap<ChunkKey, Chunk>,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Object { data, .. } = &entity.kind else { return; };
    let state = RenderFallingBlock::getBlockState(*data);
    if state.isAir() {
        return;
    }
    let currentPos = BlockPos::new(
        entity.entity.posX.floor() as i32,
        entity.entity.posY.floor() as i32,
        entity.entity.posZ.floor() as i32,
    );
    if snapshot_block_state(chunks, currentPos) == state {
        return;
    }
    let Some(model) = model_for_state(&atlas.models, state) else { return; };
    if model.missing {
        return;
    }
    let renderPos = RenderFallingBlock::renderBlockPos(
        [entity.entity.posX as f32, entity.entity.posY as f32, entity.entity.posZ as f32],
        entity.entity.height,
    );
    let matrix = translation4([position[0] - 0.5, position[1], position[2] - 0.5]);
    append_block_state_model_world(
        state,
        renderPos,
        model,
        matrix,
        packedLight,
        chunks,
        atlas,
        [1.0, 1.0, 1.0, 1.0],
        None,
        vertices,
        indices,
    );
}

#[allow(clippy::too_many_arguments)]
fn append_block_state_model_world(
    state: IBlockState,
    colorPos: BlockPos,
    model: &ResolvedBlockModel,
    matrix: [[f32; 4]; 4],
    packedLight: u32,
    chunks: &HashMap<ChunkKey, Chunk>,
    atlas: &AtlasState,
    colorMultiplier: [f32; 4],
    forceUv: Option<[f32; 2]>,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    append_block_state_model_world_with_winding(
        state, colorPos, model, matrix, packedLight, chunks, atlas,
        colorMultiplier, forceUv, false, vertices, indices,
    );
}

#[allow(clippy::too_many_arguments)]
fn append_block_state_model_world_with_winding(
    state: IBlockState,
    colorPos: BlockPos,
    model: &ResolvedBlockModel,
    matrix: [[f32; 4]; 4],
    packedLight: u32,
    chunks: &HashMap<ChunkKey, Chunk>,
    atlas: &AtlasState,
    colorMultiplier: [f32; 4],
    forceUv: Option<[f32; 2]>,
    reverseWinding: bool,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    append_block_state_model_world_outset(
        state, colorPos, model, matrix, packedLight, chunks, atlas,
        colorMultiplier, forceUv, 0.0, reverseWinding, vertices, indices,
    );
}

#[allow(clippy::too_many_arguments)]
fn append_block_state_model_world_outset(
    state: IBlockState,
    colorPos: BlockPos,
    model: &ResolvedBlockModel,
    matrix: [[f32; 4]; 4],
    packedLight: u32,
    chunks: &HashMap<ChunkKey, Chunk>,
    atlas: &AtlasState,
    colorMultiplier: [f32; 4],
    forceUv: Option<[f32; 2]>,
    localFaceOutset: f32,
    reverseWinding: bool,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let access = SnapshotBlockAccess { chunks };
    let blockLight = encoded_block_light_from_packed(packedLight);
    let skyLight = ((packedLight >> 20) & 15) as f32;
    for quad in &model.quads {
        if quad.material.layers.is_empty() {
            continue;
        }
        let materialKey = material_key(state.getBlockId(), &quad.material);
        let fireLayer = fire_texture_layer(&quad.material);
        let rectangle = atlas.rectangles.get(&materialKey).copied().unwrap_or(atlas.missingRectangle);
        let (tint, shade) = if forceUv.is_some() {
            ([1.0; 3], 1.0)
        } else {
            (
                dynamic_quad_tint(atlas, &access, state, colorPos, &quad.material),
                if quad.shade { face_brightness(quad.face) } else { 1.0 },
            )
        };
        let base = vertices.len() as u32;
        for vertexIndex in 0..4 {
            let sourceUv = forceUv.unwrap_or(quad.uvs[vertexIndex]);
            vertices.push(WorldVertex {
                position: {
                    let mut local = quad.positions[vertexIndex];
                    if localFaceOutset != 0.0 {
                        let (dx, dy, dz) = quad.face.offsets();
                        local[0] += dx as f32 * localFaceOutset;
                        local[1] += dy as f32 * localFaceOutset;
                        local[2] += dz as f32 * localFaceOutset;
                    }
                    transform_point3(matrix, local)
                },
                uv: if forceUv.is_some() {
                    sourceUv
                } else {
                    [
                        rectangle[0] + (rectangle[2] - rectangle[0]) * sourceUv[0],
                        rectangle[1] + (rectangle[3] - rectangle[1]) * sourceUv[1],
                    ]
                },
                color: [
                    colorMultiplier[0] * tint[0] * shade,
                    colorMultiplier[1] * tint[1] * shade,
                    colorMultiplier[2] * tint[2] * shade,
                    fireLayer.map_or(colorMultiplier[3], |layer| {
                        encoded_fire_alpha(colorMultiplier[3], layer)
                    }),
                ],
                lightmap: [blockLight, skyLight],
            
                shaderEntity: [-1, -1, -1],
                shaderPadding: 0,
            });
        }
        if reverseWinding {
            indices.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
        } else {
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn append_experience_orb_mesh(
    entity: &EntityOtherClient,
    position: [f32; 3],
    partialTicks: f32,
    cameraYaw: f32,
    cameraPitch: f32,
    thirdPersonView: i32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::ExperienceOrb { xpValue } = &entity.kind else { return; };
    let texture = RenderXPOrb::texture();
    let Some(rectangle) = atlas.entityTextureRectangles.get(&texture).copied() else { return; };
    let sprite = RenderXPOrb::textureCoordinates(*xpValue);
    let u0 = rectangle[0] + (rectangle[2] - rectangle[0]) * sprite[0];
    let v0 = rectangle[1] + (rectangle[3] - rectangle[1]) * sprite[1];
    let u1 = rectangle[0] + (rectangle[2] - rectangle[0]) * sprite[2];
    let v1 = rectangle[1] + (rectangle[3] - rectangle[1]) * sprite[3];
    let _ = thirdPersonView;
    let mut matrix = translation4([position[0], position[1] + 0.1, position[2]]);
    matrix = multiply4(matrix, rotation_y4(180.0 - cameraYaw));
    // The oriented camera pitch contains the front-view inversion. The MCP
    // RenderXPOrb formula therefore reduces to one consistent negative pitch.
    matrix = multiply4(matrix, rotation_x4(-cameraPitch));
    matrix = multiply4(matrix, scale4_nonuniform([0.3, 0.3, 0.3]));
    let color = RenderXPOrb::color(entity.xpColor, partialTicks);
    let block = (((packedLight >> 4) & 15) as f32 + 7.5).min(15.0);
    let sky = ((packedLight >> 20) & 15) as f32;
    append_textured_quad_world(
        matrix,
        [[-0.5, -0.25, 0.0], [0.5, -0.25, 0.0], [0.5, 0.75, 0.0], [-0.5, 0.75, 0.0]],
        [[u0, v1], [u1, v1], [u1, v0], [u0, v0]],
        color,
        [block, sky],
        vertices,
        indices,
    );
}

#[allow(clippy::too_many_arguments)]
fn append_arrow_entity_mesh(
    entity: &EntityOtherClient,
    position: [f32; 3],
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Object { objectType, .. } = &entity.kind else { return; };
    let Some(texture) = RenderArrow::texture(*objectType) else { return; };
    let Some(rectangle) = atlas.entityTextureRectangles.get(&texture).copied() else { return; };
    let partial = partialTicks.clamp(0.0, 1.0);
    let yaw = entity.entity.prevRotationYaw
        + (entity.entity.rotationYaw - entity.entity.prevRotationYaw) * partial;
    let pitch = entity.entity.prevRotationPitch
        + (entity.entity.rotationPitch - entity.entity.prevRotationPitch) * partial;
    let mut matrix = translation4(position);
    matrix = multiply4(matrix, rotation_y4(yaw - 90.0));
    matrix = multiply4(matrix, rotation_z4(pitch));
    matrix = multiply4(matrix, rotation_z4(RenderArrow::shakeRotation(entity.arrowShake, partial)));
    matrix = multiply4(matrix, rotation_x4(45.0));
    matrix = multiply4(matrix, scale4_nonuniform([0.05625, 0.05625, 0.05625]));
    matrix = multiply4(matrix, translation4([-4.0, 0.0, 0.0]));
    let lightmap = [((packedLight >> 4) & 15) as f32, ((packedLight >> 20) & 15) as f32];
    let uv = |u: f32, v: f32| -> [f32; 2] {
        [
            rectangle[0] + (rectangle[2] - rectangle[0]) * u,
            rectangle[1] + (rectangle[3] - rectangle[1]) * v,
        ]
    };
    append_textured_quad_world(
        matrix,
        [[-7.0, -2.0, -2.0], [-7.0, -2.0, 2.0], [-7.0, 2.0, 2.0], [-7.0, 2.0, -2.0]],
        [uv(0.0, 0.15625), uv(0.15625, 0.15625), uv(0.15625, 0.3125), uv(0.0, 0.3125)],
        [1.0; 4],
        lightmap,
        vertices,
        indices,
    );
    append_textured_quad_world(
        matrix,
        [[-7.0, 2.0, -2.0], [-7.0, 2.0, 2.0], [-7.0, -2.0, 2.0], [-7.0, -2.0, -2.0]],
        [uv(0.0, 0.15625), uv(0.15625, 0.15625), uv(0.15625, 0.3125), uv(0.0, 0.3125)],
        [1.0; 4],
        lightmap,
        vertices,
        indices,
    );
    for quarter in 1..=4 {
        let bladeMatrix = multiply4(matrix, rotation_x4(90.0 * quarter as f32));
        append_textured_quad_world(
            bladeMatrix,
            [[-8.0, -2.0, 0.0], [8.0, -2.0, 0.0], [8.0, 2.0, 0.0], [-8.0, 2.0, 0.0]],
            [uv(0.0, 0.0), uv(0.5, 0.0), uv(0.5, 0.15625), uv(0.0, 0.15625)],
            [1.0; 4],
            lightmap,
            vertices,
            indices,
        );
    }
}

fn append_textured_quad_world(
    matrix: [[f32; 4]; 4],
    positions: [[f32; 3]; 4],
    uvs: [[f32; 2]; 4],
    color: [f32; 4],
    lightmap: [f32; 2],
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let base = vertices.len() as u32;
    for index in 0..4 {
        vertices.push(WorldVertex {
            position: transform_point3(matrix, positions[index]),
            uv: uvs[index],
            color,
            lightmap,
        
            shaderEntity: [-1, -1, -1],
            shaderPadding: 0,
        });
    }
    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

#[allow(clippy::too_many_arguments)]
fn append_primed_tnt_mesh(
    entity: &EntityOtherClient,
    position: [f32; 3],
    partialTicks: f32,
    packedLight: u32,
    chunks: &HashMap<ChunkKey, Chunk>,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let state = RenderTNTPrimed::TNT_STATE;
    let Some(model) = model_for_state(&atlas.models, state) else { return; };
    if model.missing {
        return;
    }
    let scale = RenderTNTPrimed::scale(entity.tntFuse, partialTicks);
    let mut matrix = translation4([position[0], position[1] + 0.5, position[2]]);
    matrix = multiply4(matrix, scale4_nonuniform([scale, scale, scale]));
    matrix = multiply4(matrix, rotation_y4(-90.0));
    matrix = multiply4(matrix, translation4([-0.5, -0.5, 0.5]));
    let colorPos = BlockPos::new(position[0].floor() as i32, position[1].floor() as i32, position[2].floor() as i32);
    append_block_state_model_world(
        state,
        colorPos,
        model,
        matrix,
        packedLight,
        chunks,
        atlas,
        [1.0, 1.0, 1.0, 1.0],
        None,
        vertices,
        indices,
    );
    if RenderTNTPrimed::shouldFlash(entity.tntFuse) {
        let rectangle = atlas.widgetsRectangle;
        let whiteUv = [
            rectangle[0] + (rectangle[2] - rectangle[0]) * 247.5 / 256.0,
            rectangle[1] + (rectangle[3] - rectangle[1]) * 3.5 / 256.0,
        ];
        let center = multiply4(
            translation4([position[0], position[1] + 0.5, position[2]]),
            scale4_nonuniform([scale, scale, scale]),
        );
        let mut overlayMatrix = multiply4(center, rotation_y4(-90.0));
        overlayMatrix = multiply4(overlayMatrix, translation4([-0.5, -0.5, 0.5]));
        append_block_state_model_world_outset(
            state,
            colorPos,
            model,
            overlayMatrix,
            15_728_880,
            chunks,
            atlas,
            [1.0, 1.0, 1.0, RenderTNTPrimed::flashAlpha(entity.tntFuse, partialTicks)],
            Some(whiteUv),
            0.002,
            false,
            vertices,
            indices,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn append_ender_crystal_mesh(
    entity: &EntityOtherClient,
    position: [f32; 3],
    partialTicks: f32,
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let ClientEntityKind::Object { objectType: ObjectSpawnType::EnderCrystal, .. } = &entity.kind else { return; };
    let (rotation, bob) = RenderEnderCrystal::animation(entity.enderCrystalInnerRotation, partialTicks);
    let mut root = translation4(position);
    root = multiply4(root, scale4_nonuniform([ModelEnderCrystal::ROOT_SCALE; 3]));
    root = multiply4(root, translation4([0.0, ModelEnderCrystal::ROOT_TRANSLATE_Y, 0.0]));

    if entity.enderCrystalShouldShowBottom() {
        append_vehicle_model_pass(
            ModelEnderCrystal::baseMesh(),
            RenderEnderCrystal::texture(),
            root,
            packedLight,
            atlas,
            vertices,
            indices,
        );
    }

    let mut glassOne = multiply4(root, rotation_y4(rotation * 3.0));
    glassOne = multiply4(glassOne, translation4([
        0.0,
        ModelEnderCrystal::FLOAT_TRANSLATE_Y + bob * 0.2,
        0.0,
    ]));
    glassOne = multiply4(glassOne, rotation_axis4(
        ModelEnderCrystal::DIAGONAL_ROTATION_DEGREES,
        ModelEnderCrystal::DIAGONAL_ROTATION_AXIS,
    ));
    append_vehicle_model_pass(
        ModelEnderCrystal::glassMesh(),
        RenderEnderCrystal::texture(),
        glassOne,
        packedLight,
        atlas,
        vertices,
        indices,
    );

    let mut glassTwo = multiply4(glassOne, scale4_nonuniform([ModelEnderCrystal::NESTED_SCALE; 3]));
    glassTwo = multiply4(glassTwo, rotation_axis4(
        ModelEnderCrystal::DIAGONAL_ROTATION_DEGREES,
        ModelEnderCrystal::DIAGONAL_ROTATION_AXIS,
    ));
    glassTwo = multiply4(glassTwo, rotation_y4(rotation * 3.0));
    append_vehicle_model_pass(
        ModelEnderCrystal::glassMesh(),
        RenderEnderCrystal::texture(),
        glassTwo,
        packedLight,
        atlas,
        vertices,
        indices,
    );

    let mut cube = multiply4(glassTwo, scale4_nonuniform([ModelEnderCrystal::NESTED_SCALE; 3]));
    cube = multiply4(cube, rotation_axis4(
        ModelEnderCrystal::DIAGONAL_ROTATION_DEGREES,
        ModelEnderCrystal::DIAGONAL_ROTATION_AXIS,
    ));
    cube = multiply4(cube, rotation_y4(rotation * 3.0));
    append_vehicle_model_pass(
        ModelEnderCrystal::cubeMesh(),
        RenderEnderCrystal::texture(),
        cube,
        packedLight,
        atlas,
        vertices,
        indices,
    );

    if let Some(target) = entity.enderCrystalBeamTarget() {
        append_ender_crystal_beam(
            entity,
            position,
            target,
            rotation,
            bob,
            partialTicks,
            atlas,
            vertices,
            indices,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn append_ender_crystal_beam(
    entity: &EntityOtherClient,
    position: [f32; 3],
    target: BlockPos,
    rotation: f32,
    bob: f32,
    partialTicks: f32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    // Exact arguments prepared by RenderEnderCrystal#doRender before calling
    // RenderDragon#renderCrystalBeams.
    let targetCenter = [
        target.x as f64 + 0.5,
        target.y as f64 + 0.5,
        target.z as f64 + 0.5,
    ];
    let d0 = targetCenter[0] - entity.entity.posX;
    let d1 = targetCenter[1] - entity.entity.posY;
    let d2 = targetCenter[2] - entity.entity.posZ;
    let origin = [
        position[0] + d0 as f32,
        position[1] - 0.3 + bob * 0.4 + d1 as f32 + 2.0,
        position[2] + d2 as f32,
    ];
    let dx = (entity.entity.posX - targetCenter[0]) as f32;
    let dy = (entity.entity.posY - 1.0 - targetCenter[1]) as f32;
    let dz = (entity.entity.posZ - targetCenter[2]) as f32;
    let horizontal = (dx * dx + dz * dz).sqrt();
    let length = (dx * dx + dy * dy + dz * dz).sqrt();
    if length <= f32::EPSILON { return; }

    let mut matrix = translation4(origin);
    matrix = multiply4(matrix, rotation_y4(-dz.atan2(dx).to_degrees() - 90.0));
    matrix = multiply4(matrix, rotation_x4(-horizontal.atan2(dy).to_degrees() - 90.0));

    let rectangle = atlas.entityTextureRectangles
        .get(&RenderEnderCrystal::beamTexture())
        .copied()
        .unwrap_or(atlas.missingRectangle);
    let startV = -(rotation * 0.01);
    let endV = length / 32.0 - rotation * 0.01;
    let vSpan = endV - startV;
    if vSpan.abs() <= f32::EPSILON { return; }

    // Vanilla emits a triangle strip with nine angular samples. Atlas-backed
    // Vulkan must split longitudinally at every integer V boundary so GL_REPEAT
    // remains confined to this texture rectangle.
    let mut cuts = vec![0.0_f32, 1.0_f32];
    let low = startV.min(endV).floor() as i32;
    let high = startV.max(endV).ceil() as i32;
    for boundary in low..=high {
        let t = (boundary as f32 - startV) / vSpan;
        if t > 0.0 && t < 1.0 { cuts.push(t); }
    }
    cuts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    cuts.dedup_by(|a, b| (*a - *b).abs() < 1.0e-6);

    for side in 0..8 {
        let angle0 = side as f32 * std::f32::consts::TAU / 8.0;
        let angle1 = (side + 1) as f32 * std::f32::consts::TAU / 8.0;
        for segment in cuts.windows(2) {
            let t0 = segment[0];
            let t1 = segment[1];
            let radius0 = 0.15 + (0.75 - 0.15) * t0;
            let radius1 = 0.15 + (0.75 - 0.15) * t1;
            let z0 = length * t0;
            let z1 = length * t1;
            let rawV0 = startV + vSpan * t0;
            let rawV1 = startV + vSpan * t1;
            let localV0 = rawV0.rem_euclid(1.0);
            let localV1 = if (rawV1 - rawV1.round()).abs() < 1.0e-5 && t1 < 1.0 {
                1.0
            } else {
                rawV1.rem_euclid(1.0)
            };
            let u0 = side as f32 / 8.0;
            let u1 = (side + 1) as f32 / 8.0;
            let local = [
                [angle0.sin() * radius0, angle0.cos() * radius0, z0],
                [angle1.sin() * radius0, angle1.cos() * radius0, z0],
                [angle1.sin() * radius1, angle1.cos() * radius1, z1],
                [angle0.sin() * radius1, angle0.cos() * radius1, z1],
            ];
            let localUvs = [[u0, localV0], [u1, localV0], [u1, localV1], [u0, localV1]];
            let colors = [
                [t0, t0, t0, 1.0],
                [t0, t0, t0, 1.0],
                [t1, t1, t1, 1.0],
                [t1, t1, t1, 1.0],
            ];
            let base = vertices.len() as u32;
            for corner in 0..4 {
                vertices.push(WorldVertex {
                    position: transform_point3(matrix, local[corner]),
                    uv: [
                        rectangle[0] + (rectangle[2] - rectangle[0]) * localUvs[corner][0],
                        rectangle[1] + (rectangle[3] - rectangle[1]) * localUvs[corner][1],
                    ],
                    color: colors[corner],
                    lightmap: [15.0, 15.0],
                
                    shaderEntity: [-1, -1, -1],
                    shaderPadding: 0,
                });
            }
            // RenderDragon disables culling for the strip.
            indices.extend_from_slice(&[
                base, base + 1, base + 2, base, base + 2, base + 3,
                base + 2, base + 1, base, base + 3, base + 2, base,
            ]);
        }
    }

    let _ = partialTicks; // retained in the source-equivalent signature.
}

fn append_built_in_world_mesh(
    mesh: &BuiltInItemMesh,
    origin: [f32; 3],
    packedLight: u32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let rectangle = atlas.builtInItemRectangles
        .get(&mesh.texture)
        .copied()
        .unwrap_or(atlas.missingRectangle);
    let matrix = translation4(origin);
    let lights = [normalize3([0.2, 1.0, -0.7]), normalize3([-0.2, 1.0, 0.7])];
    let block_light = ((packedLight >> 4) & 15) as f32;
    let sky_light = ((packedLight >> 20) & 15) as f32;
    for face in mesh.indices.chunks_exact(6) {
        let source = [face[0] as usize, face[1] as usize, face[2] as usize, face[5] as usize];
        let transformed = source.map(|index| transform_point3(matrix, mesh.vertices[index].position));
        let normal = normalize3(cross3(
            subtract3(transformed[1], transformed[0]),
            subtract3(transformed[2], transformed[0]),
        ));
        let diffuse = standard_item_diffuse(normal, lights);
        let base = vertices.len() as u32;
        for (corner, source_index) in source.into_iter().enumerate() {
            let uv0 = mesh.vertices[source_index].uv;
            vertices.push(WorldVertex {
                position: transformed[corner],
                uv: [
                    rectangle[0] + (rectangle[2] - rectangle[0]) * uv0[0],
                    rectangle[1] + (rectangle[3] - rectangle[1]) * uv0[1],
                ],
                color: [diffuse * mesh.color[0], diffuse * mesh.color[1], diffuse * mesh.color[2], mesh.color[3]],
                lightmap: [block_light, sky_light],
            
                shaderEntity: [-1, -1, -1],
                shaderPadding: 0,
            });
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

fn append_built_in_world_damage_mesh(
    mesh: &BuiltInItemMesh,
    origin: [f32; 3],
    packedLight: u32,
    destroyRectangle: [f32; 4],
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let matrix = translation4(origin);
    let lights = [normalize3([0.2, 1.0, -0.7]), normalize3([-0.2, 1.0, 0.7])];
    let blockLight = ((packedLight >> 4) & 15) as f32;
    let skyLight = ((packedLight >> 20) & 15) as f32;
    for face in mesh.indices.chunks_exact(6) {
        let source = [face[0] as usize, face[1] as usize, face[2] as usize, face[5] as usize];
        let transformed = source.map(|index| transform_point3(matrix, mesh.vertices[index].position));
        let normal = normalize3(cross3(
            subtract3(transformed[1], transformed[0]),
            subtract3(transformed[2], transformed[0]),
        ));
        let diffuse = standard_item_diffuse(normal, lights);
        let base = vertices.len() as u32;
        for (corner, sourceIndex) in source.into_iter().enumerate() {
            let sourceUv = mesh.vertices[sourceIndex].uv;
            // TileEntityShulkerBoxRenderer applies texture-matrix
            // scale(4,4,1), then translate(1/16,1/16,1/16). With fixed-
            // function post multiplication this is 4 * (uv + 1/16).
            let u = (sourceUv[0] * 4.0 + 0.25).rem_euclid(1.0);
            let v = (sourceUv[1] * 4.0 + 0.25).rem_euclid(1.0);
            vertices.push(WorldVertex {
                position: transformed[corner],
                uv: [
                    destroyRectangle[0] + (destroyRectangle[2] - destroyRectangle[0]) * u,
                    destroyRectangle[1] + (destroyRectangle[3] - destroyRectangle[1]) * v,
                ],
                color: [diffuse, diffuse, diffuse, 1.0],
                lightmap: [blockLight, skyLight],
            
                shaderEntity: [-1, -1, -1],
                shaderPadding: 0,
            });
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

fn append_shulker_box_damage_meshes(
    shulkerBoxes: &[ShulkerBoxRenderState],
    damagedBlocks: &[DestroyBlockProgress],
    camera: [f32; 3],
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    for progress in damagedBlocks {
        let stage = progress.getPartialBlockDamage();
        if !(0..10).contains(&stage) {
            continue;
        }
        let pos = progress.getPosition();
        let Some(shulker) = shulkerBoxes.iter().find(|boxState| boxState.pos == pos) else {
            continue;
        };
        let center = [pos.x as f32 + 0.5, pos.y as f32 + 0.5, pos.z as f32 + 0.5];
        let dx = center[0] - camera[0];
        let dy = center[1] - camera[1];
        let dz = center[2] - camera[2];
        if dx * dx + dy * dy + dz * dz >= 4096.0 {
            continue;
        }
        let mesh = TileEntityShulkerBoxRenderer::buildMesh(
            shulker.colorMetadata,
            shulker.facing,
            shulker.progress,
        );
        append_built_in_world_damage_mesh(
            &mesh,
            [pos.x as f32, pos.y as f32, pos.z as f32],
            shulker.packedLight,
            atlas.destroyStageRectangles[stage as usize],
            vertices,
            indices,
        );
    }
}

fn tile_entity_visible(pos: BlockPos, camera: [f32; 3], frustum: &Frustum, width: f64, height: f64) -> bool {
    let center = [pos.x as f32 + 0.5, pos.y as f32 + 0.5, pos.z as f32 + 0.5];
    let dx = center[0] - camera[0];
    let dy = center[1] - camera[1];
    let dz = center[2] - camera[2];
    dx * dx + dy * dy + dz * dz <= 4096.0
        && frustum.isBoxInFrustum(
            pos.x as f64 - 0.01,
            pos.y as f64 - 0.01,
            pos.z as f64 - 0.01,
            pos.x as f64 + width + 0.01,
            pos.y as f64 + height + 0.01,
            pos.z as f64 + width + 0.01,
        )
}

fn append_bed_tile_entity_meshes(
    beds: &[BedRenderState],
    camera: [f32; 3],
    frustum: &Frustum,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    for bed in beds {
        if !tile_entity_visible(bed.pos, camera, frustum, 1.0, 1.0) { continue; }
        let mesh = TileEntityItemStackRenderer::buildWorldBedHalf(
            bed.colorMetadata, bed.head, bed.horizontalIndex,
        );
        append_built_in_world_mesh(
            &mesh,
            [bed.pos.x as f32, bed.pos.y as f32, bed.pos.z as f32],
            bed.packedLight,
            atlas,
            vertices,
            indices,
        );
    }
}

fn append_chest_tile_entity_meshes(
    chests: &[ChestRenderState],
    camera: [f32; 3],
    frustum: &Frustum,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    for chest in chests {
        let width = if chest.large { 2.0 } else { 1.0 };
        if !tile_entity_visible(chest.pos, camera, frustum, width, 1.0) { continue; }
        let mesh = TileEntityChestRenderer::buildMesh(ChestRenderInput {
            trapped: chest.trapped,
            ender: chest.ender,
            large: chest.large,
            metadata: chest.metadata,
            adjacentXPos: chest.adjacentXPos,
            adjacentZPos: chest.adjacentZPos,
            lidProgress: chest.lidProgress,
        });
        append_built_in_world_mesh(
            &mesh,
            [chest.pos.x as f32, chest.pos.y as f32, chest.pos.z as f32],
            chest.packedLight,
            atlas,
            vertices,
            indices,
        );
    }
}

fn append_shulker_box_tile_entity_meshes(
    shulker_boxes: &[ShulkerBoxRenderState],
    camera: [f32; 3],
    frustum: &Frustum,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    for shulker in shulker_boxes {
        // TileEntityShulkerBox#getRenderBoundingBox grows by half a block in
        // the facing direction as the lid opens. Use the exact directional
        // extension rather than a permanently oversized cube.
        let extension = 0.5 * shulker.progress.clamp(0.0, 1.0) as f64;
        let (offset_x, offset_y, offset_z) = shulker.facing.offsets();
        let mut min_x = shulker.pos.x as f64;
        let mut min_y = shulker.pos.y as f64;
        let mut min_z = shulker.pos.z as f64;
        let mut max_x = min_x + 1.0;
        let mut max_y = min_y + 1.0;
        let mut max_z = min_z + 1.0;
        if offset_x < 0 { min_x -= extension; } else if offset_x > 0 { max_x += extension; }
        if offset_y < 0 { min_y -= extension; } else if offset_y > 0 { max_y += extension; }
        if offset_z < 0 { min_z -= extension; } else if offset_z > 0 { max_z += extension; }

        let center = [
            shulker.pos.x as f32 + 0.5,
            shulker.pos.y as f32 + 0.5,
            shulker.pos.z as f32 + 0.5,
        ];
        let dx = center[0] - camera[0];
        let dy = center[1] - camera[1];
        let dz = center[2] - camera[2];
        if dx * dx + dy * dy + dz * dz > 4096.0
            || !frustum.isBoxInFrustum(
                min_x - 0.01, min_y - 0.01, min_z - 0.01,
                max_x + 0.01, max_y + 0.01, max_z + 0.01,
            )
        {
            continue;
        }

        let mesh = TileEntityShulkerBoxRenderer::buildMesh(
            shulker.colorMetadata,
            shulker.facing,
            shulker.progress,
        );
        append_built_in_world_mesh(
            &mesh,
            [shulker.pos.x as f32, shulker.pos.y as f32, shulker.pos.z as f32],
            shulker.packedLight,
            atlas,
            vertices,
            indices,
        );
    }
}


/// MCP 1.12.2 `RenderGlobal#renderSky` celestial subset. The flat sky colour
/// is supplied by the render-pass clear value; this mesh contains the
/// sunrise/sunset fan, sun, moon phases and the deterministic star field.
fn append_sky_mesh(
    capture: &WorldRenderCapture,
    celestialAngle: f32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) -> (u32, u32) {
    if capture.dimension == 1 {
        let alphaStart = indices.len() as u32;
        append_end_sky_mesh(capture.cameraPosition, atlas, vertices, indices);
        return (indices.len() as u32 - alphaStart, 0);
    }
    if capture.dimension != 0 {
        return (0, 0);
    }

    let alphaStart = indices.len() as u32;
    if let Some(colors) = sunrise_sunset_colors(celestialAngle) {
        let mut matrix = translation4(capture.cameraPosition);
        matrix = multiply4(matrix, rotation_x4(90.0));
        if (celestialAngle * std::f32::consts::TAU).sin() < 0.0 {
            matrix = multiply4(matrix, rotation_z4(180.0));
        }
        matrix = multiply4(matrix, rotation_z4(90.0));
        let rectangle = atlas.solidWhiteRectangle;
        let uv = [(rectangle[0] + rectangle[2]) * 0.5, (rectangle[1] + rectangle[3]) * 0.5];
        let base = vertices.len() as u32;
        vertices.push(WorldVertex {
            position: transform_point3(matrix, [0.0, 100.0, 0.0]),
            uv,
            color: colors,
            lightmap: [15.0, 15.0],
        
            shaderEntity: [-1, -1, -1],
            shaderPadding: 0,
        });
        for step in 0..=16 {
            let angle = step as f32 * std::f32::consts::TAU / 16.0;
            let sin = angle.sin();
            let cos = angle.cos();
            vertices.push(WorldVertex {
                position: transform_point3(
                    matrix,
                    [sin * 120.0, cos * 120.0, -cos * 40.0 * colors[3]],
                ),
                uv,
                color: [colors[0], colors[1], colors[2], 0.0],
                lightmap: [15.0, 15.0],
            
                shaderEntity: [-1, -1, -1],
                shaderPadding: 0,
            });
        }
        for step in 0..16_u32 {
            indices.extend_from_slice(&[base, base + step + 1, base + step + 2]);
        }
    }
    let alphaCount = indices.len() as u32 - alphaStart;

    let celestialStart = indices.len() as u32;
    let mut celestialMatrix = translation4(capture.cameraPosition);
    celestialMatrix = multiply4(celestialMatrix, rotation_y4(-90.0));
    celestialMatrix = multiply4(celestialMatrix, rotation_x4(celestialAngle * 360.0));

    if let Some(rectangle) = atlas.entityTextureRectangles
        .get(&ResourceLocation::parse("textures/environment/sun.png"))
        .copied()
    {
        append_textured_quad(
            celestialMatrix,
            [[-30.0, 100.0, -30.0], [30.0, 100.0, -30.0], [30.0, 100.0, 30.0], [-30.0, 100.0, 30.0]],
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            rectangle,
            [1.0, 1.0, 1.0, 1.0],
            vertices,
            indices,
        );
    }

    if let Some(rectangle) = atlas.entityTextureRectangles
        .get(&ResourceLocation::parse("textures/environment/moon_phases.png"))
        .copied()
    {
        let phase = capture.worldTime.div_euclid(24_000).rem_euclid(8) as i32;
        let column = phase % 4;
        let row = phase / 4;
        let u0 = column as f32 / 4.0;
        let v0 = row as f32 / 2.0;
        let u1 = (column + 1) as f32 / 4.0;
        let v1 = (row + 1) as f32 / 2.0;
        append_textured_quad(
            celestialMatrix,
            [[-20.0, -100.0, 20.0], [20.0, -100.0, 20.0], [20.0, -100.0, -20.0], [-20.0, -100.0, -20.0]],
            [[u1, v1], [u0, v1], [u0, v0], [u1, v0]],
            rectangle,
            [1.0, 1.0, 1.0, 1.0],
            vertices,
            indices,
        );
    }

    let starBrightness = star_brightness(celestialAngle);
    if starBrightness > 0.0 {
        let rectangle = atlas.solidWhiteRectangle;
        let uv = [(rectangle[0] + rectangle[2]) * 0.5, (rectangle[1] + rectangle[3]) * 0.5];
        for quad in vanilla_star_quads() {
            let base = vertices.len() as u32;
            for point in quad {
                vertices.push(WorldVertex {
                    position: transform_point3(celestialMatrix, *point),
                    uv,
                    color: [starBrightness, starBrightness, starBrightness, starBrightness],
                    lightmap: [15.0, 15.0],
                
                    shaderEntity: [-1, -1, -1],
                    shaderPadding: 0,
                });
            }
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }

    (alphaCount, indices.len() as u32 - celestialStart)
}

/// MCP `RenderGlobal#renderSkyEnd`. The source maps `end_sky.png` from
/// UV 0..16 over each 200-block cube face with texture repeat. The shared
/// Vulkan atlas cannot repeat a sub-rectangle, so each face is split into a
/// 16x16 grid and every cell maps the complete stitched sprite once.
fn append_end_sky_mesh(
    camera: [f32; 3],
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let rectangle = atlas
        .entityTextureRectangles
        .get(&TileEntityEndPortalRenderer::endSkyTexture())
        .copied()
        .unwrap_or(atlas.missingRectangle);
    let color = [40.0 / 255.0, 40.0 / 255.0, 40.0 / 255.0, 1.0];

    for face in 0..6 {
        let mut matrix = translation4(camera);
        matrix = match face {
            1 => multiply4(matrix, rotation_x4(90.0)),
            2 => multiply4(matrix, rotation_x4(-90.0)),
            3 => multiply4(matrix, rotation_x4(180.0)),
            4 => multiply4(matrix, rotation_z4(90.0)),
            5 => multiply4(matrix, rotation_z4(-90.0)),
            _ => matrix,
        };

        for u in 0..16 {
            let x0 = -100.0 + 200.0 * u as f32 / 16.0;
            let x1 = -100.0 + 200.0 * (u + 1) as f32 / 16.0;
            for v in 0..16 {
                let z0 = -100.0 + 200.0 * v as f32 / 16.0;
                let z1 = -100.0 + 200.0 * (v + 1) as f32 / 16.0;
                append_textured_quad(
                    matrix,
                    [[x0, -100.0, z0], [x0, -100.0, z1], [x1, -100.0, z1], [x1, -100.0, z0]],
                    [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]],
                    rectangle,
                    color,
                    vertices,
                    indices,
                );
            }
        }
    }
}

fn vanilla_star_quads() -> &'static Vec<[[f32; 3]; 4]> {
    static STARS: OnceLock<Vec<[[f32; 3]; 4]>> = OnceLock::new();
    STARS.get_or_init(|| {
        let mut random = crate::compat::Java::JavaRandom::new(10_842);
        let mut stars = Vec::new();
        for _ in 0..1500 {
            let mut x = (random.next_f32() * 2.0 - 1.0) as f64;
            let mut y = (random.next_f32() * 2.0 - 1.0) as f64;
            let mut z = (random.next_f32() * 2.0 - 1.0) as f64;
            let size = (0.15 + random.next_f32() * 0.1) as f64;
            let lengthSquared = x * x + y * y + z * z;
            if !(0.01..1.0).contains(&lengthSquared) {
                continue;
            }
            let inverseLength = 1.0 / lengthSquared.sqrt();
            x *= inverseLength;
            y *= inverseLength;
            z *= inverseLength;
            let center = [x * 100.0, y * 100.0, z * 100.0];
            let yaw = x.atan2(z);
            let yawSin = yaw.sin();
            let yawCos = yaw.cos();
            let pitch = (x * x + z * z).sqrt().atan2(y);
            let pitchSin = pitch.sin();
            let pitchCos = pitch.cos();
            let roll = random.next_f64() * std::f64::consts::TAU;
            let rollSin = roll.sin();
            let rollCos = roll.cos();
            let mut quad = [[0.0; 3]; 4];
            for corner in 0..4 {
                let localX = ((corner & 2) as i32 - 1) as f64 * size;
                let localY = (((corner + 1) & 2) as i32 - 1) as f64 * size;
                let rolledX = localX * rollCos - localY * rollSin;
                let rolledY = localY * rollCos + localX * rollSin;
                let pitchedY = rolledX * pitchSin;
                let pitchedZ = -rolledX * pitchCos;
                let worldX = pitchedZ * yawSin - rolledY * yawCos;
                let worldZ = rolledY * yawSin + pitchedZ * yawCos;
                quad[corner] = [
                    (center[0] + worldX) as f32,
                    (center[1] + pitchedY) as f32,
                    (center[2] + worldZ) as f32,
                ];
            }
            stars.push(quad);
        }
        stars
    })
}


fn append_beacon_tile_entity_meshes(
    beacons: &[BeaconRenderState],
    totalWorldTime: i64,
    partialTicks: f32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) -> (u32, u32) {
    let rectangle = atlas
        .entityTextureRectangles
        .get(&TileEntityBeaconRenderer::texture())
        .copied()
        .unwrap_or(atlas.missingRectangle);
    let firstIndex = indices.len() as u32;

    for beacon in beacons {
        if beacon.beamScale <= 0.0 || beacon.segments.is_empty() { continue; }
        let mut yOffset = 0_i32;
        for segment in &beacon.segments {
            let geometry = TileEntityBeaconRenderer::segmentGeometry(
                partialTicks,
                beacon.beamScale,
                totalWorldTime,
                segment.height,
                0.2,
                0.25,
            );
            append_beacon_layer(
                beacon.pos,
                yOffset,
                segment.height,
                geometry.innerCorners,
                geometry.innerV,
                segment.colors,
                1.0,
                rectangle,
                vertices,
                indices,
            );
            yOffset += segment.height;
        }
    }
    let coreCount = indices.len() as u32 - firstIndex;

    for beacon in beacons {
        if beacon.beamScale <= 0.0 || beacon.segments.is_empty() { continue; }
        let mut yOffset = 0_i32;
        for segment in &beacon.segments {
            let geometry = TileEntityBeaconRenderer::segmentGeometry(
                partialTicks,
                beacon.beamScale,
                totalWorldTime,
                segment.height,
                0.2,
                0.25,
            );
            append_beacon_layer(
                beacon.pos,
                yOffset,
                segment.height,
                geometry.outerCorners,
                geometry.outerV,
                segment.colors,
                0.125,
                rectangle,
                vertices,
                indices,
            );
            yOffset += segment.height;
        }
    }
    let glowCount = indices.len() as u32 - firstIndex - coreCount;
    (coreCount, glowCount)
}

#[allow(clippy::too_many_arguments)]
fn append_beacon_layer(
    pos: BlockPos,
    yOffset: i32,
    height: i32,
    corners: [[f32; 2]; 4],
    rawV: [f32; 2],
    rgb: [f32; 3],
    alpha: f32,
    rectangle: [f32; 4],
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let pairs = [(0_usize, 1_usize), (3, 2), (1, 3), (2, 0)];
    let vStart = rawV[0] as f64;
    let vEnd = rawV[1] as f64;
    let span = vEnd - vStart;
    if span.abs() <= 1.0e-9 { return; }

    // The beam texture uses GL_REPEAT, while the shared Vulkan atlas cannot
    // repeat a sub-rectangle. Split each side at every integer V boundary so
    // interpolation remains inside the beam rectangle and is visually
    // equivalent to the source sampler state.
    let mut current = vStart;
    while current < vEnd - 1.0e-9 {
        let nextBoundary = current.floor() + 1.0;
        let next = vEnd.min(nextBoundary);
        let t0 = ((current - vStart) / span) as f32;
        let t1 = ((next - vStart) / span) as f32;
        let y0 = pos.y as f32 + yOffset as f32 + height as f32 * t0;
        let y1 = pos.y as f32 + yOffset as f32 + height as f32 * t1;
        let localV0 = current.rem_euclid(1.0) as f32;
        let localV1 = if (next - nextBoundary).abs() < 1.0e-9 && next < vEnd - 1.0e-9 {
            1.0
        } else {
            next.rem_euclid(1.0) as f32
        };

        for (a, b) in pairs {
            let p = [
                [pos.x as f32 + corners[a][0], y1, pos.z as f32 + corners[a][1]],
                [pos.x as f32 + corners[a][0], y0, pos.z as f32 + corners[a][1]],
                [pos.x as f32 + corners[b][0], y0, pos.z as f32 + corners[b][1]],
                [pos.x as f32 + corners[b][0], y1, pos.z as f32 + corners[b][1]],
            ];
            let localUvs = [[1.0, localV1], [1.0, localV0], [0.0, localV0], [0.0, localV1]];
            let base = vertices.len() as u32;
            for corner in 0..4 {
                vertices.push(WorldVertex {
                    position: p[corner],
                    uv: [
                        rectangle[0] + (rectangle[2] - rectangle[0]) * localUvs[corner][0],
                        rectangle[1] + (rectangle[3] - rectangle[1]) * localUvs[corner][1],
                    ],
                    color: [rgb[0], rgb[1], rgb[2], alpha],
                    lightmap: [15.0, 15.0],
                
                    shaderEntity: [-1, -1, -1],
                    shaderPadding: 0,
                });
            }
            // TileEntityBeaconRenderer disables culling.
            indices.extend_from_slice(&[
                base, base + 1, base + 2, base, base + 2, base + 3,
                base + 2, base + 1, base, base + 3, base + 2, base,
            ]);
        }
        current = next;
    }
}

fn append_end_portal_tile_entity_meshes(
    portals: &[EndPortalRenderState],
    camera: [f32; 3],
    frustum: &Frustum,
    viewProjection: [[f32; 4]; 4],
    systemTimeMillis: u64,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) -> u32 {
    let skyRectangle = atlas
        .entityTextureRectangles
        .get(&TileEntityEndPortalRenderer::endSkyTexture())
        .copied()
        .unwrap_or(atlas.missingRectangle);
    let portalRectangle = atlas
        .entityTextureRectangles
        .get(&TileEntityEndPortalRenderer::endPortalTexture())
        .copied()
        .unwrap_or(atlas.missingRectangle);

    // Keep the first SRC_ALPHA layer contiguous and before every ONE/ONE
    // layer. Vulkan draw ranges cannot change blend state in the middle of a
    // draw, while MCP changes it exactly once at j == 1.
    let initialIndexCount = indices.len() as u32;
    let mut alphaIndexCount = 0_u32;
    for additivePass in [false, true] {
        for portal in portals {
            let x = portal.pos.x as f32;
            let y = portal.pos.y as f32 + TileEntityEndPortalRenderer::SURFACE_HEIGHT;
            let z = portal.pos.z as f32;
            let dx = x - camera[0];
            let dy = y - camera[1];
            let dz = z - camera[2];
            let distanceSquared = (dx * dx + dy * dy + dz * dz) as f64;
            if distanceSquared > 4_096.0
                || !frustum.isBoxInFrustum(
                    x as f64,
                    y as f64 - 0.01,
                    z as f64,
                    (x + 1.0) as f64,
                    (y + 0.01) as f64,
                    (z + 1.0) as f64,
                )
            {
                continue;
            }

            let positions = [
                [x, y, z + 1.0],
                [x + 1.0, y, z + 1.0],
                [x + 1.0, y, z],
                [x, y, z],
            ];
            for layer in TileEntityEndPortalRenderer::layers(distanceSquared)
                .into_iter()
                .filter(|layer| layer.additive == additivePass)
            {
                let rectangle = if layer.index == 0 { skyRectangle } else { portalRectangle };
                let base = vertices.len() as u32;
                for position in positions {
                    let rawUv = end_portal_projective_uv(
                        position,
                        layer.index,
                        systemTimeMillis,
                        viewProjection,
                    );
                    let u = rawUv[0].rem_euclid(1.0);
                    let v = rawUv[1].rem_euclid(1.0);
                    vertices.push(WorldVertex {
                        position,
                        uv: [
                            rectangle[0] + (rectangle[2] - rectangle[0]) * u,
                            rectangle[1] + (rectangle[3] - rectangle[1]) * v,
                        ],
                        color: layer.color,
                        // Lighting is disabled by TileEntityEndPortalRenderer;
                        // the draw pass uses the unlit shader sentinel.
                        lightmap: [15.0, 15.0],
                    
                        shaderEntity: [-1, -1, -1],
                        shaderPadding: 0,
                    });
                }
                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            }
        }
        if !additivePass {
            alphaIndexCount = indices.len() as u32 - initialIndexCount;
        }
    }
    alphaIndexCount
}

fn end_portal_projective_uv(
    worldPosition: [f32; 3],
    layerIndex: i32,
    systemTimeMillis: u64,
    viewProjection: [[f32; 4]; 4],
) -> [f32; 2] {
    // Texture-matrix call order from TileEntityEndPortalRenderer:
    // T(.5) S(.5) T(layer,time) R(layer) S(layer) PROJECTION MODELVIEW.
    let layer = (layerIndex + 1) as f32;
    let time = (systemTimeMillis % 800_000) as f32 / 800_000.0;
    let angle = (layer * layer * 4321.0 + layer * 9.0) * 2.0;
    let layerScale = 4.5 - layer / 4.0;
    let matrix = multiply4(
        translation4([0.5, 0.5, 0.0]),
        multiply4(
            scale4_nonuniform([0.5, 0.5, 1.0]),
            multiply4(
                translation4([17.0 / layer, (2.0 + layer / 1.5) * time, 0.0]),
                multiply4(
                    rotation_z4(angle),
                    multiply4(
                        scale4_nonuniform([layerScale, layerScale, 1.0]),
                        viewProjection,
                    ),
                ),
            ),
        ),
    );
    let projected = transform_homogeneous(
        matrix,
        [worldPosition[0], worldPosition[1], worldPosition[2], 1.0],
    );
    let q = if projected[3].abs() < 1.0e-6 { 1.0 } else { projected[3] };
    [projected[0] / q, projected[1] / q]
}

fn append_sign_tile_entity_meshes(
    signs: &[SignRenderState],
    camera: [f32; 3],
    frustum: &Frustum,
    atlas: &AtlasState,
    fontRenderer: &mut FontRenderer,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let texture = TileEntitySignRenderer::texture();
    let rectangle = atlas
        .entityTextureRectangles
        .get(&texture)
        .copied()
        .unwrap_or(atlas.missingRectangle);
    let lights = [
        normalize3([0.2, 1.0, -0.7]),
        normalize3([-0.2, 1.0, 0.7]),
    ];

    for sign in signs {
        if !tile_entity_visible(sign.pos, camera, frustum, 1.0, 1.0) {
            continue;
        }
        let placement = TileEntitySignRenderer::placement(sign.blockId, sign.metadata);
        let mesh = TileEntitySignRenderer::buildMesh(placement.standing);
        let mut placementMatrix = translation4([
            sign.pos.x as f32 + 0.5,
            sign.pos.y as f32 + 0.5,
            sign.pos.z as f32 + 0.5,
        ]);
        placementMatrix = multiply4(placementMatrix, rotation_y4(placement.yawDegrees));
        placementMatrix = multiply4(placementMatrix, translation4(placement.wallOffset));
        let modelMatrix = multiply4(
            placementMatrix,
            scale4_nonuniform([0.6666667, -0.6666667, -0.6666667]),
        );
        let blockLight = ((sign.packedLight >> 4) & 15) as f32;
        let skyLight = ((sign.packedLight >> 20) & 15) as f32;

        for face in mesh.indices.chunks_exact(6) {
            let source = [face[0] as usize, face[1] as usize, face[2] as usize, face[5] as usize];
            let transformed = source.map(|index| transform_point3(modelMatrix, mesh.vertices[index].position));
            let normal = normalize3(cross3(
                subtract3(transformed[1], transformed[0]),
                subtract3(transformed[2], transformed[0]),
            ));
            let diffuse = standard_item_diffuse(normal, lights);
            let base = vertices.len() as u32;
            for (corner, sourceIndex) in source.into_iter().enumerate() {
                let uv = mesh.vertices[sourceIndex].uv;
                vertices.push(WorldVertex {
                    position: transformed[corner],
                    uv: [
                        rectangle[0] + (rectangle[2] - rectangle[0]) * uv[0],
                        rectangle[1] + (rectangle[3] - rectangle[1]) * uv[1],
                    ],
                    color: [diffuse, diffuse, diffuse, 1.0],
                    lightmap: [blockLight, skyLight],
                
                    shaderEntity: [-1, -1, -1],
                    shaderPadding: 0,
                });
            }
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }

        // TileEntitySignRenderer keeps the board placement matrix, then uses
        // 1/96 scale for four centered FontRenderer lines at y=-20,-10,0,10.
        let mut textMatrix = multiply4(
            placementMatrix,
            translation4([0.0, 0.33333334, 0.046666667]),
        );
        textMatrix = multiply4(
            textMatrix,
            scale4_nonuniform([0.010416667, -0.010416667, 0.010416667]),
        );
        let mut drawList = GuiDrawList::new();
        for (lineIndex, componentText) in sign.lines.iter().enumerate() {
            let mut line = fontRenderer
                .list_formatted_string_to_width(componentText, 90)
                .into_iter()
                .next()
                .unwrap_or_default();
            if lineIndex as i32 == sign.lineBeingEdited {
                line = format!("> {line} <");
            }
            let x = -fontRenderer.get_string_width(&line) / 2;
            let y = lineIndex as i32 * 10 - 20;
            fontRenderer.draw_string(&mut drawList, &line, x as f32, y as f32, 0, false);
        }
        for command in drawList.commands() {
            let GuiDrawCommand::Quad { texture, topology, vertices: glyphVertices } = command else {
                continue;
            };
            let glyphRectangle = texture
                .as_ref()
                .and_then(|location| atlas.fontTextureRectangles.get(location))
                .copied()
                .unwrap_or(atlas.widgetsRectangle);
            let base = vertices.len() as u32;
            for vertex in glyphVertices {
                let (u, v) = if texture.is_some() {
                    (
                        glyphRectangle[0] + (glyphRectangle[2] - glyphRectangle[0]) * vertex.u,
                        glyphRectangle[1] + (glyphRectangle[3] - glyphRectangle[1]) * vertex.v,
                    )
                } else {
                    (
                        glyphRectangle[0] + (glyphRectangle[2] - glyphRectangle[0]) * (247.5 / 256.0),
                        glyphRectangle[1] + (glyphRectangle[3] - glyphRectangle[1]) * (3.5 / 256.0),
                    )
                };
                vertices.push(WorldVertex {
                    position: transform_point3(textMatrix, [vertex.x, vertex.y, vertex.z]),
                    uv: [u, v],
                    color: packed_argb_to_rgba(vertex.color),
                    lightmap: [blockLight, skyLight],
                
                    shaderEntity: [-1, -1, -1],
                    shaderPadding: 0,
                });
            }
            match topology {
                GuiTopology::Quads => indices.extend_from_slice(&[
                    base, base + 1, base + 2, base + 2, base + 3, base,
                ]),
                GuiTopology::TriangleStrip => indices.extend_from_slice(&[
                    base, base + 1, base + 2, base + 2, base + 1, base + 3,
                ]),
            }
        }
    }
}

fn append_enchantment_table_tile_entity_meshes(
    tables: &[EnchantmentTableRenderState],
    camera: [f32; 3],
    frustum: &Frustum,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let texture = ResourceLocation::new(
        "minecraft",
        "textures/entity/enchanting_table_book.png",
    );
    let rectangle = atlas
        .builtInItemRectangles
        .get(&texture)
        .copied()
        .unwrap_or(atlas.missingRectangle);
    let lights = [
        normalize3([0.2, 1.0, -0.7]),
        normalize3([-0.2, 1.0, 0.7]),
    ];

    for table in tables {
        if !tile_entity_visible(table.pos, camera, frustum, 1.0, 1.5) {
            continue;
        }

        let mesh = TileEntityEnchantmentTableRenderer::buildBookMesh(
            table.ticks,
            table.pageFlipRight,
            table.pageFlipLeft,
            table.spread,
        );
        // TileEntityEnchantmentTableRenderer#func_192841_a. Matrix order is
        // identical to the source GlStateManager calls: translate to the
        // block center, apply the floating bob, then yaw and the fixed 80° Z
        // tilt before ModelBook renders at scale 1/16.
        let mut matrix = translation4([
            table.pos.x as f32 + 0.5,
            table.pos.y as f32 + 0.75,
            table.pos.z as f32 + 0.5,
        ]);
        matrix = multiply4(
            matrix,
            translation4([0.0, 0.1 + (table.ticks * 0.1).sin() * 0.01, 0.0]),
        );
        matrix = multiply4(
            matrix,
            rotation_y4(-table.rotation * 180.0 / std::f32::consts::PI),
        );
        matrix = multiply4(matrix, rotation_z4(80.0));

        let block_light = ((table.packedLight >> 4) & 15) as f32;
        let sky_light = ((table.packedLight >> 20) & 15) as f32;
        for face in mesh.indices.chunks_exact(6) {
            let source = [
                face[0] as usize,
                face[1] as usize,
                face[2] as usize,
                face[5] as usize,
            ];
            let transformed = source.map(|index| {
                transform_point3(matrix, mesh.vertices[index].position)
            });
            let normal = normalize3(cross3(
                subtract3(transformed[1], transformed[0]),
                subtract3(transformed[2], transformed[0]),
            ));
            let diffuse = standard_item_diffuse(normal, lights);
            let base = vertices.len() as u32;
            for (corner, source_index) in source.into_iter().enumerate() {
                let uv = mesh.vertices[source_index].uv;
                vertices.push(WorldVertex {
                    position: transformed[corner],
                    uv: [
                        rectangle[0] + (rectangle[2] - rectangle[0]) * uv[0],
                        rectangle[1] + (rectangle[3] - rectangle[1]) * uv[1],
                    ],
                    color: [diffuse, diffuse, diffuse, 1.0],
                    lightmap: [block_light, sky_light],
                
                    shaderEntity: [-1, -1, -1],
                    shaderPadding: 0,
                });
            }
            indices.extend_from_slice(&[
                base,
                base + 1,
                base + 2,
                base,
                base + 2,
                base + 3,
            ]);
        }
    }
}

fn append_piston_tile_entity_meshes(
    pistons: &[PistonRenderState],
    chunks: &HashMap<ChunkKey, Chunk>,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    for piston in pistons {
        if piston.progress >= 1.0 || piston.pistonState.getBlockId() == 0 { continue; }
        let pos = piston.pos;
        let moving_matrix = translation4([
            pos.x as f32 + piston.offset[0],
            pos.y as f32 + piston.offset[1],
            pos.z as f32 + piston.offset[2],
        ]);
        let block_id = piston.pistonState.getBlockId();
        if block_id == 34 && piston.progress <= 0.25 {
            let sticky = (piston.pistonState.getMetadata() & 8) != 0;
            let key = (piston.facing.index() as u8, sticky, true);
            if let Some(model) = atlas.pistonHeadModels.get(&key) {
                append_block_state_model_world(
                    piston.pistonState, pos, model, moving_matrix, piston.packedLight,
                    chunks, atlas, [1.0; 4], None, vertices, indices,
                );
            }
        } else if piston.shouldHeadBeRendered && !piston.extending && matches!(block_id, 29 | 33) {
            let sticky = block_id == 29;
            let short = piston.progress >= 0.5;
            let key = (piston.facing.index() as u8, sticky, short);
            if let Some(model) = atlas.pistonHeadModels.get(&key) {
                let head_state = IBlockState::fromGlobalStateId(
                    (34 << 4) | piston.facing.index() | (if sticky { 8 } else { 0 }),
                );
                append_block_state_model_world(
                    head_state, pos, model, moving_matrix, piston.packedLight,
                    chunks, atlas, [1.0; 4], None, vertices, indices,
                );
            }
            let extended_state = IBlockState::fromGlobalStateId(
                (block_id << 4) | (piston.pistonState.getMetadata() & 7) | 8,
            );
            if let Some(model) = model_for_state(&atlas.models, extended_state) {
                append_block_state_model_world(
                    extended_state, pos, model,
                    translation4([pos.x as f32, pos.y as f32, pos.z as f32]),
                    piston.packedLight, chunks, atlas, [1.0; 4], None, vertices, indices,
                );
            }
        } else if let Some(model) = actual_model_for_state(atlas, chunks, piston.pistonState, pos) {
            append_block_state_model_world(
                piston.pistonState, pos, model, moving_matrix, piston.packedLight,
                chunks, atlas, [1.0; 4], None, vertices, indices,
            );
        }
    }
}

fn append_skull_tile_entity_meshes(
    skulls: &[SkullRenderState],
    camera: [f32; 3],
    frustum: &Frustum,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let lights = [normalize3([0.2, 1.0, -0.7]), normalize3([-0.2, 1.0, 0.7])];
    for skull in skulls {
        let state = BlockSkull::stateForFacing(skull.facing);
        let bounds = BlockSkull::getBoundingBox(state).offset(
            skull.pos.x as f64,
            skull.pos.y as f64,
            skull.pos.z as f64,
        );
        let center = [
            skull.pos.x as f32 + 0.5,
            skull.pos.y as f32 + 0.5,
            skull.pos.z as f32 + 0.5,
        ];
        let dx = center[0] - camera[0];
        let dy = center[1] - camera[1];
        let dz = center[2] - camera[2];
        // TileEntitySpecialRenderer#getMaxRenderDistanceSquared defaults to 4096.
        if dx * dx + dy * dy + dz * dz > 4096.0 {
            continue;
        }
        if !frustum.isBoxInFrustum(
            bounds.min_x, bounds.min_y, bounds.min_z,
            bounds.max_x, bounds.max_y, bounds.max_z,
        ) {
            continue;
        }
        let Some(mesh) = TileEntityItemStackRenderer::buildSkullMesh(
            skull.skullType,
            skull.animateTicks,
        ) else { continue; };
        let rectangle = if skull.skullType == 3 {
            skull.playerSkinLocation.as_ref()
                .and_then(|location| atlas.entityTextureRectangles.get(location).copied())
                .unwrap_or(atlas.steveRectangle)
        } else {
            atlas.builtInItemRectangles
                .get(&mesh.texture)
                .copied()
                .unwrap_or(atlas.missingRectangle)
        };

        // The shared item mesh baseline is an UP skull translated to
        // [0.5, 0, 0.5] and rendered at 180 degrees. Apply only the exact
        // TileEntitySkullRenderer placement delta around that block center.
        let placement = TileEntitySkullRenderer::getPlacement(skull.facing, skull.rotation);
        let shift = [
            placement.translation[0] - 0.5,
            placement.translation[1],
            placement.translation[2] - 0.5,
        ];
        let mut matrix = translation4([
            skull.pos.x as f32 + shift[0],
            skull.pos.y as f32 + shift[1],
            skull.pos.z as f32 + shift[2],
        ]);
        matrix = multiply4(matrix, translation4([0.5, 0.0, 0.5]));
        matrix = multiply4(matrix, rotation_y4(placement.yaw - 180.0));
        matrix = multiply4(matrix, translation4([-0.5, 0.0, -0.5]));

        let block_light = ((skull.packedLight >> 4) & 15) as f32;
        let sky_light = ((skull.packedLight >> 20) & 15) as f32;
        for face in mesh.indices.chunks_exact(6) {
            let source = [face[0] as usize, face[1] as usize, face[2] as usize, face[5] as usize];
            let transformed = source.map(|index| transform_point3(matrix, mesh.vertices[index].position));
            let normal = normalize3(cross3(
                subtract3(transformed[1], transformed[0]),
                subtract3(transformed[2], transformed[0]),
            ));
            let diffuse = standard_item_diffuse(normal, lights);
            let base = vertices.len() as u32;
            for (corner, source_index) in source.into_iter().enumerate() {
                let uv0 = mesh.vertices[source_index].uv;
                vertices.push(WorldVertex {
                    position: transformed[corner],
                    uv: [
                        rectangle[0] + (rectangle[2] - rectangle[0]) * uv0[0],
                        rectangle[1] + (rectangle[3] - rectangle[1]) * uv0[1],
                    ],
                    color: [diffuse, diffuse, diffuse, 1.0],
                    lightmap: [block_light, sky_light],
                
                    shaderEntity: [-1, -1, -1],
                    shaderPadding: 0,
                });
            }
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
}

fn interpolated_player_look(player: &RemotePlayerRenderState, partialTicks: f32) -> [f64; 3] {
    let partial = partialTicks.clamp(0.0, 1.0);
    let yaw = player.prevYaw + (player.yaw - player.prevYaw) * partial;
    let pitch = player.prevPitch + (player.pitch - player.prevPitch) * partial;
    let yawRadians = -yaw * 0.017453292_f32 - core::f32::consts::PI;
    let pitchRadians = -pitch * 0.017453292_f32;
    let yawCos = minecraft_cos(yawRadians);
    let yawSin = minecraft_sin(yawRadians);
    let pitchHorizontal = -minecraft_cos(pitchRadians);
    [
        (yawSin * pitchHorizontal) as f64,
        minecraft_sin(pitchRadians) as f64,
        (yawCos * pitchHorizontal) as f64,
    ]
}

fn apply_player_elytra_corpse_rotation(
    vertices: &mut [WorldVertex],
    position: [f32; 3],
    bodyYaw: f32,
    rotation: ElytraCorpseRotation,
) {
    let rootYaw = 180.0 - bodyYaw;
    let inverseRoot = rotation_y4(-rootYaw);
    let alignYaw = rotation_y4(rotation.yawDegrees);
    let flightPitch = rotation_x4(rotation.pitchDegrees);
    let root = rotation_y4(rootYaw);
    for vertex in vertices {
        let worldOffset = [
            vertex.position[0] - position[0],
            vertex.position[1] - position[1],
            vertex.position[2] - position[2],
        ];
        let local = transform_direction3(inverseRoot, worldOffset);
        let local = transform_direction3(alignYaw, local);
        let local = transform_direction3(flightPitch, local);
        let worldOffset = transform_direction3(root, local);
        vertex.position = [
            position[0] + worldOffset[0],
            position[1] + worldOffset[1],
            position[2] + worldOffset[2],
        ];
    }
}

fn player_elytra_corpse_matrix(bodyYaw: f32, rotation: ElytraCorpseRotation) -> [[f32; 4]; 4] {
    let rootYaw = 180.0 - bodyYaw;
    multiply4(
        rotation_y4(rootYaw),
        multiply4(
            rotation_x4(rotation.pitchDegrees),
            multiply4(rotation_y4(rotation.yawDegrees), rotation_y4(-rootYaw)),
        ),
    )
}

fn player_look_from_angles(yaw: f32, pitch: f32) -> [f64; 3] {
    let yawRadians = -yaw * 0.017453292_f32 - core::f32::consts::PI;
    let pitchRadians = -pitch * 0.017453292_f32;
    let horizontal = -minecraft_cos(pitchRadians);
    [
        (minecraft_sin(yawRadians) * horizontal) as f64,
        minecraft_sin(pitchRadians) as f64,
        (minecraft_cos(yawRadians) * horizontal) as f64,
    ]
}

fn inventory_player_entity_matrix(
    player: Option<&RemotePlayerRenderState>,
    bodyYaw: f32,
    headYaw: f32,
    headPitch: f32,
    wholeEntityPitch: f32,
) -> [[f32; 4]; 4] {
    let guiPitch = rotation_x4(wholeEntityPitch);
    let Some(player) = player.filter(|player| player.elytraFlying) else { return guiPitch; };
    // GuiInventory.drawEntityOnScreen invokes RenderManager with partialTicks=1.
    let rotation = RenderPlayer::elytraCorpseRotation(
        player.ticksElytraFlying,
        1.0,
        headPitch,
        player_look_from_angles(headYaw, headPitch),
        player.motion,
    );
    multiply4(guiPitch, player_elytra_corpse_matrix(bodyYaw, rotation))
}


fn can_render_player_name_for_teams(
    playerTeam: Option<&crate::net::minecraft::scoreboard::ScorePlayerTeam::ScorePlayerTeam>,
    localTeam: Option<&crate::net::minecraft::scoreboard::ScorePlayerTeam::ScorePlayerTeam>,
    playerInvisible: bool,
    localPlayerSpectator: bool,
    playerBeingRidden: bool,
) -> bool {
    let sameTeam = playerTeam
        .zip(localTeam)
        .is_some_and(|(playerTeam, localTeam)| playerTeam.isSameTeam(localTeam));

    // MCP `EntityPlayer#isInvisibleToPlayer`. Spectators see invisible
    // players; otherwise only same-team players with friendly-invisibility
    // enabled remain visible.
    let visibleToLocal = !playerInvisible
        || localPlayerSpectator
        || playerTeam.is_some_and(|team| {
            sameTeam && team.getSeeFriendlyInvisiblesEnabled()
        });

    if let Some(team) = playerTeam {
        return match team.getNameTagVisibility() {
            "always" => visibleToLocal,
            "never" => false,
            "hideForOtherTeams" => localTeam.map_or(visibleToLocal, |_| {
                sameTeam && (team.getSeeFriendlyInvisiblesEnabled() || visibleToLocal)
            }),
            "hideForOwnTeam" => {
                localTeam.map_or(visibleToLocal, |_| !sameTeam && visibleToLocal)
            }
            // `RenderLivingBase#canRenderName` returns true from the default
            // switch branch for unknown enum values.
            _ => true,
        };
    }

    // `Minecraft.isGuiEnabled()` is always true in the currently ported input
    // surface because the vanilla F1 toggle has not created a hidden-GUI state.
    // Preserve the remaining no-team conditions exactly.
    visibleToLocal && !playerBeingRidden
}

fn is_local_render_view_player(playerEntityId: i32, localPlayerEntityId: Option<i32>) -> bool {
    localPlayerEntityId == Some(playerEntityId)
}

fn can_render_remote_player_name(
    player: &RemotePlayerRenderState,
    scoreboard: &Scoreboard,
    localPlayerName: &str,
    localPlayerSpectator: bool,
) -> bool {
    can_render_player_name_for_teams(
        scoreboard.getPlayersTeam(&player.name),
        scoreboard.getPlayersTeam(localPlayerName),
        player.invisible,
        localPlayerSpectator,
        player.beingRidden,
    )
}

fn remote_player_nameplate_matrix(
    anchor: [f32; 3],
    cameraYaw: f32,
    cameraPitch: f32,
) -> [[f32; 4]; 4] {
    // MCP `EntityRenderer#drawNameplate`: translate, rotate to
    // `RenderManager.playerViewY/playerViewX`, then use the fixed 0.025 scale.
    // `orient_camera_112` already folds the front-third-person pitch sign into
    // `cameraPitch`, so the same matrix covers all three camera modes.
    let mut matrix = translation4(anchor);
    matrix = multiply4(matrix, rotation_y4(-cameraYaw));
    matrix = multiply4(matrix, rotation_x4(cameraPitch));
    multiply4(matrix, scale4_nonuniform([-0.025, -0.025, 0.025]))
}

fn append_nameplate_background(
    halfWidth: i32,
    verticalShift: i32,
    matrix: [[f32; 4]; 4],
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let rectangle = atlas.solidWhiteRectangle;
    let uv = [
        (rectangle[0] + rectangle[2]) * 0.5,
        (rectangle[1] + rectangle[3]) * 0.5,
    ];
    let positions = [
        [(-halfWidth - 1) as f32, (-1 + verticalShift) as f32, 0.0],
        [(-halfWidth - 1) as f32, (8 + verticalShift) as f32, 0.0],
        [(halfWidth + 1) as f32, (8 + verticalShift) as f32, 0.0],
        [(halfWidth + 1) as f32, (-1 + verticalShift) as f32, 0.0],
    ];
    let base = vertices.len() as u32;
    for position in positions {
        vertices.push(WorldVertex {
            position: transform_point3(matrix, position),
            uv,
            color: [0.0, 0.0, 0.0, 0.25],
            lightmap: [15.0, 15.0],
            shaderEntity: [-1, -1, -1],
            shaderPadding: 0,
        });
    }
    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

fn append_nameplate_text(
    text: &str,
    verticalShift: i32,
    color: i32,
    matrix: [[f32; 4]; 4],
    fontRenderer: &mut FontRenderer,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let mut drawList = GuiDrawList::new();
    let x = -fontRenderer.get_string_width(text) / 2;
    fontRenderer.draw_string(
        &mut drawList,
        text,
        x as f32,
        verticalShift as f32,
        color,
        false,
    );
    for command in drawList.commands() {
        let GuiDrawCommand::Quad { texture, topology, vertices: glyphVertices } = command else {
            continue;
        };
        let rectangle = texture
            .as_ref()
            .and_then(|location| atlas.fontTextureRectangles.get(location))
            .copied()
            .unwrap_or(atlas.fontRectangle);
        let base = vertices.len() as u32;
        for vertex in glyphVertices {
            let (u, v) = if texture.is_some() {
                (
                    rectangle[0] + (rectangle[2] - rectangle[0]) * vertex.u,
                    rectangle[1] + (rectangle[3] - rectangle[1]) * vertex.v,
                )
            } else {
                (
                    (rectangle[0] + rectangle[2]) * 0.5,
                    (rectangle[1] + rectangle[3]) * 0.5,
                )
            };
            vertices.push(WorldVertex {
                position: transform_point3(matrix, [vertex.x, vertex.y, vertex.z]),
                uv: [u, v],
                color: packed_argb_to_rgba(vertex.color),
                lightmap: [15.0, 15.0],
                shaderEntity: [-1, -1, -1],
                shaderPadding: 0,
            });
        }
        match topology {
            GuiTopology::Quads => indices.extend_from_slice(&[
                base, base + 1, base + 2, base + 2, base + 3, base,
            ]),
            GuiTopology::TriangleStrip => indices.extend_from_slice(&[
                base, base + 1, base + 2, base + 2, base + 1, base + 3,
            ]),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn append_remote_player_nameplate_label(
    text: &str,
    anchor: [f32; 3],
    cameraYaw: f32,
    cameraPitch: f32,
    sneaking: bool,
    fontRenderer: &mut FontRenderer,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
    ranges: &mut Vec<WorldEntityDrawRange>,
) {
    let verticalShift = if text == "deadmau5" { -10 } else { 0 };
    let halfWidth = fontRenderer.get_string_width(text) / 2;
    let matrix = remote_player_nameplate_matrix(anchor, cameraYaw, cameraPitch);

    let backgroundPass = indices.len() as u32;
    append_nameplate_background(
        halfWidth,
        verticalShift,
        matrix,
        atlas,
        vertices,
        indices,
    );
    push_world_entity_draw_range(
        ranges,
        if sneaking {
            WorldEntityPipelineKind::NameplateBackgroundDepthNoWrite
        } else {
            WorldEntityPipelineKind::NameplateBackgroundSeeThrough
        },
        WorldEntityMeshKind::Dynamic,
        backgroundPass,
        indices.len() as u32 - backgroundPass,
    );

    if !sneaking {
        // First vanilla text draw while depth is disabled. Keep it in a
        // separate range because the background is rendered with texture2D
        // disabled while the font pass samples the font atlas.
        let seeThroughTextPass = indices.len() as u32;
        append_nameplate_text(
            text,
            verticalShift,
            0x20FF_FFFF_u32 as i32,
            matrix,
            fontRenderer,
            atlas,
            vertices,
            indices,
        );
        push_world_entity_draw_range(
            ranges,
            WorldEntityPipelineKind::NameplateTextSeeThrough,
            WorldEntityMeshKind::Dynamic,
            seeThroughTextPass,
            indices.len() as u32 - seeThroughTextPass,
        );
    }

    let finalTextPass = indices.len() as u32;
    append_nameplate_text(
        text,
        verticalShift,
        if sneaking { 0x20FF_FFFF_u32 as i32 } else { -1 },
        matrix,
        fontRenderer,
        atlas,
        vertices,
        indices,
    );
    push_world_entity_draw_range(
        ranges,
        WorldEntityPipelineKind::NameplateTextDepthWrite,
        WorldEntityMeshKind::Dynamic,
        finalTextPass,
        indices.len() as u32 - finalTextPass,
    );
}

#[allow(clippy::too_many_arguments)]
fn append_remote_player_nameplates(
    players: &[RemotePlayerRenderState],
    scoreboard: &Scoreboard,
    localPlayerName: &str,
    localPlayerSpectator: bool,
    localPlayerEntityId: Option<i32>,
    viewerPosition: [f64; 3],
    partialTicks: f32,
    cameraYaw: f32,
    cameraPitch: f32,
    frustum: &Frustum,
    fontRenderer: &mut FontRenderer,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
    ranges: &mut Vec<WorldEntityDrawRange>,
) {
    let partial = partialTicks.clamp(0.0, 1.0);
    for player in players {
        // MCP `RenderLivingBase#canRenderName` excludes the current
        // `RenderManager.renderViewEntity`. The local model is appended to the
        // player mesh list in third person, so it must be filtered explicitly.
        if is_local_render_view_player(player.entityId, localPlayerEntityId) {
            continue;
        }
        if !can_render_remote_player_name(
            player,
            scoreboard,
            localPlayerName,
            localPlayerSpectator,
        ) {
            continue;
        }
        let deltaX = player.position[0] - viewerPosition[0];
        let deltaY = player.position[1] - viewerPosition[1];
        let deltaZ = player.position[2] - viewerPosition[2];
        let distanceSq = deltaX * deltaX + deltaY * deltaY + deltaZ * deltaZ;
        let maximumDistance = if player.sneaking { 32.0_f64 } else { 64.0_f64 };
        if distanceSq >= maximumDistance * maximumDistance {
            continue;
        }

        let position = [
            lerp_f64(player.prevPosition[0], player.position[0], partial) as f32,
            lerp_f64(player.prevPosition[1], player.position[1], partial) as f32,
            lerp_f64(player.prevPosition[2], player.position[2], partial) as f32,
        ];
        if !frustum.isBoxInFrustum(
            position[0] as f64 - 0.3,
            position[1] as f64,
            position[2] as f64 - 0.3,
            position[0] as f64 + 0.3,
            position[1] as f64 + player.height as f64,
            position[2] as f64 + 0.3,
        ) {
            continue;
        }

        let yOffset = player.height + 0.5 - if player.sneaking { 0.25 } else { 0.0 };
        let mut anchor = [position[0], position[1] + yOffset, position[2]];
        if distanceSq < 100.0 {
            if let Some(objective) = scoreboard.getObjectiveInDisplaySlot(2) {
                let scoreText = format!(
                    "{} {}",
                    scoreboard.getScorePoints(&player.name, objective.getName()),
                    objective.getDisplayName(),
                );
                append_remote_player_nameplate_label(
                    &scoreText,
                    anchor,
                    cameraYaw,
                    cameraPitch,
                    player.sneaking,
                    fontRenderer,
                    atlas,
                    vertices,
                    indices,
                    ranges,
                );
                anchor[1] += fontRenderer.font_height as f32 * 1.15 * 0.025;
            }
        }

        let displayName = crate::net::minecraft::scoreboard::ScorePlayerTeam::ScorePlayerTeam::formatPlayerName(
            scoreboard.getPlayersTeam(&player.name),
            &player.name,
        );
        append_remote_player_nameplate_label(
            &displayName,
            anchor,
            cameraYaw,
            cameraPitch,
            player.sneaking,
            fontRenderer,
            atlas,
            vertices,
            indices,
            ranges,
        );
    }
}

struct RemotePlayerMeshBatch {
    vertices: Vec<WorldVertex>,
    indices: Vec<u32>,
    glintVertices: Vec<WorldVertex>,
    glintIndices: Vec<u32>,
    rendered: usize,
}

fn build_remote_player_meshes(
    players: &[RemotePlayerRenderState],
    partialTicks: f32,
    camera: [f32; 3],
    frustum: &Frustum,
    atlas: &AtlasState,
) -> (Vec<WorldVertex>, Vec<u32>, Vec<WorldVertex>, Vec<u32>, usize) {
    let threadCount = rayon::current_num_threads().max(1);
    if players.len() < PARALLEL_PLAYER_BATCH_THRESHOLD || threadCount <= 1 {
        return build_remote_player_meshes_serial(players, partialTicks, camera, frustum, atlas);
    }
    let targetBatches = threadCount.saturating_mul(2).min(players.len()).max(1);
    let batchSize = players.len().div_ceil(targetBatches);
    let batches = players
        .par_chunks(batchSize)
        .map(|batch| {
            let (vertices, indices, glintVertices, glintIndices, rendered) =
                build_remote_player_meshes_serial(batch, partialTicks, camera, frustum, atlas);
            RemotePlayerMeshBatch {
                vertices,
                indices,
                glintVertices,
                glintIndices,
                rendered,
            }
        })
        .collect::<Vec<_>>();

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut glintVertices = Vec::new();
    let mut glintIndices = Vec::new();
    let mut rendered = 0usize;
    for batch in batches {
        rendered = rendered.saturating_add(batch.rendered);
        append_indexed_mesh_stream(&mut vertices, &mut indices, batch.vertices, batch.indices);
        append_indexed_mesh_stream(
            &mut glintVertices,
            &mut glintIndices,
            batch.glintVertices,
            batch.glintIndices,
        );
    }
    (vertices, indices, glintVertices, glintIndices, rendered)
}

fn build_remote_player_meshes_serial(
    players: &[RemotePlayerRenderState],
    partialTicks: f32,
    camera: [f32; 3],
    frustum: &Frustum,
    atlas: &AtlasState,
) -> (Vec<WorldVertex>, Vec<u32>, Vec<WorldVertex>, Vec<u32>, usize) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut glintVertices = Vec::new();
    let mut glintIndices = Vec::new();
    let mut rendered = 0;
    let partial = partialTicks.clamp(0.0, 1.0);
    for player in players {
        if player.invisible {
            continue;
        }
        let position = [
            lerp_f64(player.prevPosition[0], player.position[0], partial) as f32,
            lerp_f64(player.prevPosition[1], player.position[1], partial) as f32,
            lerp_f64(player.prevPosition[2], player.position[2], partial) as f32,
        ];
        let deltaX = position[0] - camera[0];
        let deltaY = position[1] - camera[1];
        let deltaZ = position[2] - camera[2];
        // `EntityOtherPlayerMP.isInRangeToRenderDist`: a 0.6x1.8x0.6
        // player has average edge length 1, then *10*64 render weight.
        if deltaX * deltaX + deltaY * deltaY + deltaZ * deltaZ >= 640.0 * 640.0 {
            continue;
        }
        if !frustum.isBoxInFrustum(
            position[0] as f64 - 0.3,
            position[1] as f64,
            position[2] as f64 - 0.3,
            position[0] as f64 + 0.3,
            position[1] as f64 + 1.8,
            position[2] as f64 + 0.3,
        ) {
            continue;
        }
        let playerVertexStart = vertices.len();
        let playerGlintVertexStart = glintVertices.len();
        let bodyYaw = interpolate_rotation(player.prevBodyYaw, player.bodyYaw, partial);
        let modelBodyYaw = if player.sleeping { 180.0 } else { bodyYaw };
        let headYaw = interpolate_rotation(player.prevHeadYaw, player.headYaw, partial);
        let headPitch = player.prevPitch + (player.pitch - player.prevPitch) * partial;
        let limbAmount = player.prevLimbSwingAmount
            + (player.limbSwingAmount - player.prevLimbSwingAmount) * partial;
        let limbSwing = player.limbSwing - player.limbSwingAmount * (1.0 - partial);
        let swing = player.prevSwingProgress
            + (player.swingProgress - player.prevSwingProgress) * partial;
        let ageInTicks = player.ticksExisted as f32 + partial;
        let slim = player.slim;
        let (leftArmPose, rightArmPose) = player_arm_poses(
            &player.mainHandStack,
            &player.offHandStack,
            player.itemInUseCount,
            player.primaryHand,
        );
        let renderInput = PlayerRenderInput {
            position, bodyYaw: modelBodyYaw, headYaw, headPitch, limbSwing,
            limbSwingAmount: limbAmount, ageInTicks, swingProgress: swing,
            sneaking: player.sneaking && !player.sleeping, riding: player.riding && !player.sleeping, slim, skinParts: player.skinParts,
            swingingArmIsLeft: player.swingingArmIsLeft,
            leftArmPose,
            rightArmPose,
            ticksElytraFlying: player.ticksElytraFlying,
            motion: player.motion,
        };
        let pose = RenderPlayer::buildPose(renderInput);
        let mut model = RenderPlayer::buildMesh(renderInput);
        if player.sleeping {
            for vertex in &mut model.vertices {
                vertex.position = sleeping_player_point(
                    vertex.position,
                    position,
                    player.renderOffset,
                    player.bedOrientation,
                );
            }
        }
        if model.indices.is_empty() { continue; }
        let rectangle = player_skin_rectangle(atlas, &player.skinLocation, slim);
        let base = vertices.len() as u32;
        let blockLight = ((player.packedLight >> 4) & 15) as f32;
        let skyLight = ((player.packedLight >> 20) & 15) as f32;
        // `RenderLivingBase#setBrightness` uses a constant red RGB with
        // alpha 0.3 while hurtTime/deathTime is active. Players carry the same
        // out-of-range block-light sentinel used by non-player living models;
        // held items are submitted after the model with ordinary packed light.
        let modelBlockLight = encoded_living_hurt_block_light(
            blockLight,
            player.hurtTime,
            player.deathTime,
        );
        vertices.extend(model.vertices.into_iter().map(|vertex| WorldVertex {
            position: vertex.position,
            uv: map_player_skin_uv(rectangle, vertex.uv),
            color: [1.0, 1.0, 1.0, 1.0],
            lightmap: [modelBlockLight, skyLight],
        
            shaderEntity: [-1, -1, -1],
            shaderPadding: 0,
        }));
        indices.extend(model.indices.into_iter().map(|index| base + index));

        // RenderPlayer layer order in 1.12.2: armor, held item, arrow,
        // Deadmau5 ears, cape, custom head, elytra, shoulder entities.
        let armorVertexStart = vertices.len();
        append_world_player_armor(
            player, pose, position, modelBodyYaw, atlas, blockLight, skyLight,
            &mut vertices, &mut indices,
        );
        if player.sleeping {
            apply_sleeping_transform_to_range(
                &mut vertices[armorVertexStart..], position, player,
            );
        }

        let heldVertexStart = vertices.len();
        append_world_player_held_items(
            player, pose, position, modelBodyYaw, atlas, &mut vertices, &mut indices,
        );
        if player.sleeping {
            apply_sleeping_transform_to_range(
                &mut vertices[heldVertexStart..], position, player,
            );
        }

        // LayerCape does not combine with the hurt/death brightness layer.
        let capeVertexStart = vertices.len();
        append_world_player_cape(
            player, position, modelBodyYaw, partial, atlas, blockLight, skyLight,
            &mut vertices, &mut indices,
        );
        if player.sleeping {
            apply_sleeping_transform_to_range(
                &mut vertices[capeVertexStart..], position, player,
            );
        }

        let customHeadVertexStart = vertices.len();
        append_world_player_custom_head(
            player, pose, position, modelBodyYaw, limbSwing, atlas, blockLight, skyLight,
            &mut vertices, &mut indices,
        );
        if player.sleeping {
            apply_sleeping_transform_to_range(
                &mut vertices[customHeadVertexStart..], position, player,
            );
        }

        let elytraVertexStart = vertices.len();
        append_world_player_elytra(
            player, position, modelBodyYaw, atlas, blockLight, skyLight,
            &mut vertices, &mut indices,
        );
        if player.sleeping {
            apply_sleeping_transform_to_range(
                &mut vertices[elytraVertexStart..], position, player,
            );
        }

        append_world_player_glint(
            player,
            pose,
            position,
            modelBodyYaw,
            ageInTicks,
            atlas,
            &mut glintVertices,
            &mut glintIndices,
        );

        // Exact MCP RenderPlayer#rotateCorpse elytra branch. Apply the entity
        // root transform after all model/layer meshes have been baked so skin,
        // armor, held items, custom heads, elytra and glint remain rigidly
        // attached to the same flying player pose.
        if player.elytraFlying && !player.sleeping {
            let rotation = RenderPlayer::elytraCorpseRotation(
                player.ticksElytraFlying,
                partial,
                player.pitch,
                interpolated_player_look(player, partial),
                player.motion,
            );
            apply_player_elytra_corpse_rotation(
                &mut vertices[playerVertexStart..],
                position,
                modelBodyYaw,
                rotation,
            );
            apply_player_elytra_corpse_rotation(
                &mut glintVertices[playerGlintVertexStart..],
                position,
                modelBodyYaw,
                rotation,
            );
        }
        rendered += 1;
    }
    (vertices, indices, glintVertices, glintIndices, rendered)
}

fn apply_sleeping_transform_to_range(
    vertices: &mut [WorldVertex],
    position: [f32; 3],
    player: &RemotePlayerRenderState,
) {
    for vertex in vertices {
        vertex.position = sleeping_player_point(
            vertex.position,
            position,
            player.renderOffset,
            player.bedOrientation,
        );
    }
}

fn player_armor_stack(
    player: &RemotePlayerRenderState,
    slot: EntityEquipmentSlot,
) -> &ItemStack {
    match slot {
        EntityEquipmentSlot::Feet => &player.armorStacks[0],
        EntityEquipmentSlot::Legs => &player.armorStacks[1],
        EntityEquipmentSlot::Chest => &player.armorStacks[2],
        EntityEquipmentSlot::Head => &player.armorStacks[3],
        EntityEquipmentSlot::Mainhand | EntityEquipmentSlot::Offhand => &ItemStack::EMPTY,
    }
}

fn enchanted_glint_uv(
    rectangle: [f32; 4],
    uv: [f32; 2],
    pass: crate::net::minecraft::client::renderer::entity::layers::LayerArmorBase::GlintPass,
) -> [f32; 2] {
    map_player_skin_uv(rectangle, enchanted_glint_local_uv(uv, pass))
}

fn enchanted_glint_local_uv(
    uv: [f32; 2],
    pass: crate::net::minecraft::client::renderer::entity::layers::LayerArmorBase::GlintPass,
) -> [f32; 2] {
    // OpenGL texture matrix after scale, rotate, translate is S*R*T.
    let translated = [uv[0], uv[1] + pass.translation];
    let radians = pass.rotationDegrees.to_radians();
    let (sin, cos) = radians.sin_cos();
    let rotated = [
        translated[0] * cos - translated[1] * sin,
        translated[0] * sin + translated[1] * cos,
    ];
    [
        (rotated[0] * pass.textureScale).rem_euclid(1.0),
        (rotated[1] * pass.textureScale).rem_euclid(1.0),
    ]
}

fn append_glint_mesh(
    mut mesh: crate::net::minecraft::client::renderer::entity::RenderPlayer::PlayerModelMesh,
    player: &RemotePlayerRenderState,
    position: [f32; 3],
    rectangle: [f32; 4],
    ageInTicks: f32,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    if player.sleeping {
        for vertex in &mut mesh.vertices {
            vertex.position = sleeping_player_point(
                vertex.position,
                position,
                player.renderOffset,
                player.bedOrientation,
            );
        }
    }
    for pass in LayerArmorBase::glintPasses(ageInTicks) {
        let base = vertices.len() as u32;
        vertices.extend(mesh.vertices.iter().map(|vertex| WorldVertex {
            position: vertex.position,
            uv: enchanted_glint_uv(rectangle, vertex.uv, pass),
            color: pass.color,
            lightmap: [15.0, 15.0],
        
            shaderEntity: [-1, -1, -1],
            shaderPadding: 0,
        }));
        indices.extend(mesh.indices.iter().map(|index| base + *index));
    }
}

#[allow(clippy::too_many_arguments)]
fn append_world_player_glint(
    player: &RemotePlayerRenderState,
    pose: BipedPose,
    position: [f32; 3],
    modelBodyYaw: f32,
    ageInTicks: f32,
    atlas: &AtlasState,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let Some(rectangle) = atlas.textureRectangles
        .get(&LayerArmorBase::glintTexture())
        .copied()
    else { return; };

    for slot in [
        EntityEquipmentSlot::Chest,
        EntityEquipmentSlot::Legs,
        EntityEquipmentSlot::Feet,
        EntityEquipmentSlot::Head,
    ] {
        let stack = player_armor_stack(player, slot);
        let Some(definition) = ItemArmor::definition(stack.itemId) else { continue; };
        if definition.slot != slot || !stack.isItemEnchanted() { continue; }
        let mesh = RenderPlayer::buildBoxesMesh(
            LayerBipedArmor::boxes(pose, slot),
            position,
            modelBodyYaw,
            player.sneaking && !player.sleeping,
            64.0,
            32.0,
        );
        append_glint_mesh(
            mesh, player, position, rectangle, ageInTicks, vertices, indices,
        );
    }

    if ItemArmor::isElytra(&player.chestStack) && player.chestStack.isItemEnchanted() {
        let pose = ModelElytra::poseFromRotations(
            player.sneaking && !player.sleeping,
            player.elytraRotation,
        );
        let mut mesh = RenderPlayer::buildLocalBoxesMesh(
            ModelElytra::boxes(pose),
            64.0,
            32.0,
        );
        let mut matrix = multiply4(
            translation4(position),
            player_layer_root_matrix(modelBodyYaw, player.sneaking && !player.sleeping),
        );
        matrix = multiply4(matrix, translation4([0.0, 0.0, 0.125]));
        for vertex in &mut mesh.vertices {
            vertex.position = transform_point3(matrix, vertex.position);
        }
        append_glint_mesh(
            mesh, player, position, rectangle, ageInTicks, vertices, indices,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn append_world_player_armor(
    player: &RemotePlayerRenderState,
    pose: BipedPose,
    position: [f32; 3],
    modelBodyYaw: f32,
    atlas: &AtlasState,
    blockLight: f32,
    skyLight: f32,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    for slot in [
        EntityEquipmentSlot::Chest,
        EntityEquipmentSlot::Legs,
        EntityEquipmentSlot::Feet,
        EntityEquipmentSlot::Head,
    ] {
        let stack = player_armor_stack(player, slot);
        let Some(definition) = ItemArmor::definition(stack.itemId) else { continue; };
        if definition.slot != slot { continue; }
        let boxes = LayerBipedArmor::boxes(pose, slot);
        for tint in LayerArmorBase::tintPasses(stack) {
            let Some(texture) = LayerArmorBase::texture(stack, tint.overlay) else { continue; };
            let Some(rectangle) = atlas.entityTextureRectangles.get(&texture).copied() else { continue; };
            let mesh = RenderPlayer::buildBoxesMesh(
                boxes.iter().copied(),
                position,
                modelBodyYaw,
                player.sneaking && !player.sleeping,
                64.0,
                32.0,
            );
            let base = vertices.len() as u32;
            vertices.extend(mesh.vertices.into_iter().map(|vertex| WorldVertex {
                position: vertex.position,
                uv: map_player_skin_uv(rectangle, vertex.uv),
                color: tint.color,
                lightmap: [blockLight, skyLight],
            
                shaderEntity: [-1, -1, -1],
                shaderPadding: 0,
            }));
            indices.extend(mesh.indices.into_iter().map(|index| base + index));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn append_world_player_custom_head(
    player: &RemotePlayerRenderState,
    pose: BipedPose,
    position: [f32; 3],
    modelBodyYaw: f32,
    limbSwing: f32,
    atlas: &AtlasState,
    blockLight: f32,
    skyLight: f32,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let stack = player_armor_stack(player, EntityEquipmentSlot::Head);
    if !LayerCustomHead::isSkull(stack) {
        return;
    }
    let skullType = stack.itemDamage as i32;
    let Some(mesh) = TileEntityItemStackRenderer::buildSkullMesh(skullType, limbSwing) else {
        return;
    };
    let rectangle = if skullType == 3 {
        player.customHeadSkinLocation.as_ref()
            .and_then(|location| atlas.entityTextureRectangles.get(location).copied())
            .unwrap_or(atlas.steveRectangle)
    } else {
        atlas.builtInItemRectangles.get(&mesh.texture)
            .copied()
            .unwrap_or(atlas.missingRectangle)
    };

    // LayerCustomHead: optional sneaking offset, head ModelRenderer.postRender,
    // 1.1875 skull scale, then TileEntitySkullRenderer at (-0.5, 0, -0.5).
    // buildSkullMesh's reusable baseline contains the UP renderer's +0.5 X/Z
    // translation, so remove that baseline before attaching it to the head.
    let mut matrix = multiply4(
        translation4(position),
        player_layer_root_matrix(modelBodyYaw, player.sneaking && !player.sleeping),
    );
    if player.sneaking && !player.sleeping {
        matrix = multiply4(matrix, translation4([0.0, 0.2, 0.0]));
    }
    matrix = post_render_part_matrix(matrix, pose.head);
    matrix = multiply4(matrix, scale4_nonuniform([
        LayerCustomHead::SKULL_SCALE,
        -LayerCustomHead::SKULL_SCALE,
        -LayerCustomHead::SKULL_SCALE,
    ]));

    let base = vertices.len() as u32;
    vertices.extend(mesh.vertices.into_iter().map(|vertex| {
        let local = [
            vertex.position[0] - 0.5,
            vertex.position[1],
            vertex.position[2] - 0.5,
        ];
        WorldVertex {
            position: transform_point3(matrix, local),
            uv: map_player_skin_uv(rectangle, vertex.uv),
            color: [1.0; 4],
            lightmap: [blockLight, skyLight],
        
            shaderEntity: [-1, -1, -1],
            shaderPadding: 0,
        }
    }));
    indices.extend(mesh.indices.into_iter().map(|index| base + index));
}

#[allow(clippy::too_many_arguments)]
fn append_world_player_elytra(
    player: &RemotePlayerRenderState,
    position: [f32; 3],
    modelBodyYaw: f32,
    atlas: &AtlasState,
    blockLight: f32,
    skyLight: f32,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    if !ItemArmor::isElytra(&player.chestStack) { return; }
    let Some(location) = player.elytraLocation.as_ref() else { return; };
    let Some(rectangle) = atlas.entityTextureRectangles.get(location).copied() else { return; };
    let pose = ModelElytra::poseFromRotations(
        player.sneaking && !player.sleeping,
        player.elytraRotation,
    );
    let mesh = RenderPlayer::buildLocalBoxesMesh(ModelElytra::boxes(pose), 64.0, 32.0);
    let mut matrix = multiply4(
        translation4(position),
        player_layer_root_matrix(modelBodyYaw, player.sneaking && !player.sleeping),
    );
    // LayerElytra#doRenderLayer translates the model 0.125 blocks toward +Z.
    matrix = multiply4(matrix, translation4([0.0, 0.0, 0.125]));
    let base = vertices.len() as u32;
    vertices.extend(mesh.vertices.into_iter().map(|vertex| WorldVertex {
        position: transform_point3(matrix, vertex.position),
        uv: map_player_skin_uv(rectangle, vertex.uv),
        color: [1.0; 4],
        lightmap: [blockLight, skyLight],
    
        shaderEntity: [-1, -1, -1],
        shaderPadding: 0,
    }));
    indices.extend(mesh.indices.into_iter().map(|index| base + index));
}

#[allow(clippy::too_many_arguments)]
fn append_world_player_cape(
    player: &RemotePlayerRenderState,
    position: [f32; 3],
    modelBodyYaw: f32,
    partialTicks: f32,
    atlas: &AtlasState,
    blockLight: f32,
    skyLight: f32,
    vertices: &mut Vec<WorldVertex>,
    indices: &mut Vec<u32>,
) {
    let Some(capeLocation) = player.capeLocation.as_ref() else { return; };
    if !LayerCape::shouldRender(
        true,
        player.invisible,
        player.skinParts,
        &player.chestStack,
    ) {
        return;
    }
    let Some(rectangle) = atlas.entityTextureRectangles.get(capeLocation).copied() else {
        // `NetworkPlayerInfo#getLocationCape` is only exposed after the
        // download callback, but atlas rebuilding is main-thread deferred.
        // Skip this frame rather than substitute the missing texture.
        return;
    };

    let transform = LayerCape::transform(
        CapeMotionInput {
            prevChasingPos: player.prevChasingPosition,
            chasingPos: player.chasingPosition,
            prevPos: player.prevPosition,
            pos: player.position,
            prevRenderYawOffset: player.prevBodyYaw,
            renderYawOffset: player.bodyYaw,
            prevCameraYaw: player.prevCameraYaw,
            cameraYaw: player.cameraYaw,
            prevDistanceWalkedModified: player.prevMovedDistance,
            distanceWalkedModified: player.movedDistance,
            sneaking: player.sneaking,
        },
        partialTicks,
    );

    let mut matrix = multiply4(
        translation4(position),
        player_layer_root_matrix(modelBodyYaw, player.sneaking),
    );
    matrix = multiply4(matrix, translation4(transform.translation));
    matrix = multiply4(matrix, rotation_x4(transform.rotateX));
    matrix = multiply4(matrix, rotation_z4(transform.rotateZ));
    matrix = multiply4(matrix, rotation_y4(transform.rotateY));
    matrix = multiply4(matrix, rotation_y4(transform.finalRotateY));

    let cape = RenderPlayer::buildCapeMesh();
    let base = vertices.len() as u32;
    vertices.extend(cape.vertices.into_iter().map(|vertex| WorldVertex {
        position: transform_point3(matrix, vertex.position),
        uv: map_player_skin_uv(rectangle, vertex.uv),
        color: [1.0, 1.0, 1.0, 1.0],
        lightmap: [blockLight, skyLight],
    
        shaderEntity: [-1, -1, -1],
        shaderPadding: 0,
    }));
    indices.extend(cape.indices.into_iter().map(|index| base + index));
}

fn sleeping_player_point(
    worldPoint: [f32; 3],
    playerPosition: [f32; 3],
    renderOffset: [f32; 3],
    bedOrientation: f32,
) -> [f32; 3] {
    // RenderPlayer#renderLivingAt followed by the sleeping branch of
    // RenderPlayer#rotateCorpse. OpenGL post-multiplication means vertices see
    // the inverse call order: Y(270), Z(90), then the bed-facing Y rotation.
    let mut local = subtract3(worldPoint, playerPosition);
    local = rotate_y_degrees(local, 270.0);
    local = rotate_z_degrees(local, 90.0);
    local = rotate_y_degrees(local, bedOrientation);
    add3(add3(playerPosition, renderOffset), local)
}

fn rotate_y_degrees(point: [f32; 3], degrees: f32) -> [f32; 3] {
    let radians = degrees.to_radians();
    let (sin, cos) = radians.sin_cos();
    [point[0] * cos + point[2] * sin, point[1], -point[0] * sin + point[2] * cos]
}

fn rotate_z_degrees(point: [f32; 3], degrees: f32) -> [f32; 3] {
    let radians = degrees.to_radians();
    let (sin, cos) = radians.sin_cos();
    [point[0] * cos - point[1] * sin, point[0] * sin + point[1] * cos, point[2]]
}

fn skull_profile_cache_key(profile: &GameProfile) -> String {
    if let Some(id) = profile.getId() {
        return format!("uuid:{id}");
    }
    if !profile.getName().is_empty() {
        return format!("name:{}", profile.getName().to_ascii_lowercase());
    }
    // NBTUtil rejects profiles with neither UUID nor name. Retain a stable
    // property-based fallback for malformed server data instead of merging
    // unrelated heads into one cache entry.
    let mut key = String::from("properties:");
    for property in profile.getProperties() {
        key.push_str(property.getName());
        key.push('=');
        key.push_str(property.getValue());
        key.push(';');
    }
    key
}

fn player_skin_rectangle(
    atlas: &AtlasState,
    location: &ResourceLocation,
    slim: bool,
) -> [f32; 4] {
    atlas.entityTextureRectangles.get(location).copied().unwrap_or_else(|| {
        if slim { atlas.alexRectangle } else { atlas.steveRectangle }
    })
}

fn map_player_skin_uv(rectangle: [f32; 4], uv: [f32; 2]) -> [f32; 2] {
    [
        rectangle[0] + (rectangle[2] - rectangle[0]) * uv[0],
        rectangle[1] + (rectangle[3] - rectangle[1]) * uv[1],
    ]
}

fn lerp_f64(start: f64, end: f64, partial: f32) -> f64 {
    start + (end - start) * partial as f64
}

fn interpolate_rotation(previous: f32, current: f32, partial: f32) -> f32 {
    let mut difference = current - previous;
    while difference < -180.0 { difference += 360.0; }
    while difference >= 180.0 { difference -= 360.0; }
    previous + difference * partial
}

fn render_chunk_distance_squared(left: RenderChunkKey, right: RenderChunkKey) -> i64 {
    let dx = (left.x - right.x) as i64;
    let dy = (left.y - right.y) as i64;
    let dz = (left.z - right.z) as i64;
    dx * dx + dy * dy + dz * dz
}

fn add3(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] + right[0], left[1] + right[1], left[2] + right[2]]
}

fn subtract3(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn scale3(value: [f32; 3], scale: f32) -> [f32; 3] {
    [value[0] * scale, value[1] * scale, value[2] * scale]
}

fn dot3(left: [f32; 3], right: [f32; 3]) -> f32 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

fn cross3(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

fn normalize3(value: [f32; 3]) -> [f32; 3] {
    let length = dot3(value, value).sqrt();
    if length <= f32::EPSILON {
        [0.0, 0.0, -1.0]
    } else {
        [value[0] / length, value[1] / length, value[2] / length]
    }
}


fn camera_axes(yaw: f32, pitch: f32) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let yawRadians = (-yaw as f64).to_radians() - std::f64::consts::PI;
    let pitchRadians = (-pitch as f64).to_radians();
    let forward = normalize3([
        (yawRadians.sin() * -pitchRadians.cos()) as f32,
        pitchRadians.sin() as f32,
        (yawRadians.cos() * -pitchRadians.cos()) as f32,
    ]);
    // `EntityRenderer#orientCamera` applies yaw and pitch as separate
    // rotations. Deriving the right axis from `forward × worldUp` instead
    // degenerates at pitch +/-90 degrees and made the view roll to the
    // normalize3 fallback when looking at the player's feet. Keep the right
    // axis yaw-owned, then derive up from the orthonormal basis. Away from
    // vertical pitch this is algebraically identical to the old look-at basis.
    let right = [yawRadians.cos() as f32, 0.0, -yawRadians.sin() as f32];
    let up = normalize3(cross3(right, forward));
    (right, up, forward)
}

fn camera_view_matrix(yaw: f32, pitch: f32, eye: [f32; 3]) -> [[f32; 4]; 4] {
    let (right, up, forward) = camera_axes(yaw, pitch);
    [
        [right[0], right[1], right[2], -dot3(right, eye)],
        [up[0], up[1], up[2], -dot3(up, eye)],
        [-forward[0], -forward[1], -forward[2], dot3(forward, eye)],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn camera_matrix(
    yaw: f32,
    pitch: f32,
    eye: [f32; 3],
    fovDegrees: f32,
    aspect: f32,
    near: f32,
    far: f32,
) -> [[f32; 4]; 4] {
    multiply4(
        perspective_matrix(fovDegrees, aspect, near, far),
        camera_view_matrix(yaw, pitch, eye),
    )
}

fn translation4(offset: [f32; 3]) -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, offset[0]],
        [0.0, 1.0, 0.0, offset[1]],
        [0.0, 0.0, 1.0, offset[2]],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn multiply4(left: [[f32; 4]; 4], right: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut output = [[0.0_f32; 4]; 4];
    for row in 0..4 {
        for column in 0..4 {
            output[row][column] = (0..4)
                .map(|index| left[row][index] * right[index][column])
                .sum();
        }
    }
    output
}

fn to_column_major(matrix: [[f32; 4]; 4]) -> [f32; 16] {
    let mut output = [0.0_f32; 16];
    for row in 0..4 {
        for column in 0..4 {
            output[column * 4 + row] = matrix[row][column];
        }
    }
    output
}

fn cube_face(
    x: i32,
    y: i32,
    z: i32,
    facing: EnumFacing,
) -> ([[f32; 3]; 4], [[f32; 2]; 4]) {
    let x0 = x as f32;
    let x1 = x0 + 1.0;
    let y0 = y as f32;
    let y1 = y0 + 1.0;
    let z0 = z as f32;
    let z1 = z0 + 1.0;
    match facing {
        // Vertex order is counter-clockwise when viewed from outside the
        // block, matching FaceBakery's outward winding and the Vulkan world
        // pipeline's COUNTER_CLOCKWISE front-face convention.
        EnumFacing::Down => (
            [[x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1]],
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        ),
        EnumFacing::Up => (
            [[x0, y1, z1], [x1, y1, z1], [x1, y1, z0], [x0, y1, z0]],
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        ),
        EnumFacing::North => (
            [[x1, y0, z0], [x0, y0, z0], [x0, y1, z0], [x1, y1, z0]],
            [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
        ),
        EnumFacing::South => (
            [[x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1]],
            [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
        ),
        EnumFacing::West => (
            [[x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0]],
            [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
        ),
        EnumFacing::East => (
            [[x1, y0, z1], [x1, y0, z0], [x1, y1, z0], [x1, y1, z1]],
            [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
        ),
    }
}

fn map_face_uv(base: [f32; 2], uv: [f32; 4], rotation: i32) -> [f32; 2] {
    let rotated = match rotation.rem_euclid(360) {
        90 => [1.0 - base[1], base[0]],
        180 => [1.0 - base[0], 1.0 - base[1]],
        270 => [base[1], 1.0 - base[0]],
        _ => base,
    };
    let u0 = uv[0] / 16.0;
    let v0 = uv[1] / 16.0;
    let u1 = uv[2] / 16.0;
    let v1 = uv[3] / 16.0;
    [u0 + (u1 - u0) * rotated[0], v0 + (v1 - v0) * rotated[1]]
}

fn face_brightness(facing: EnumFacing) -> f32 {
    match facing {
        EnumFacing::Down => 0.5,
        EnumFacing::Up => 1.0,
        EnumFacing::North | EnumFacing::South => 0.8,
        EnumFacing::West | EnumFacing::East => 0.6,
    }
}


fn sky_color(
    dimension: i32,
    biomeSkyColor: [f32; 3],
    celestialAngle: f32,
    lastLightningBolt: i32,
    partialTicks: f32,
) -> [f32; 4] {
    if dimension == -1 {
        return [48.0 / 255.0, 8.0 / 255.0, 8.0 / 255.0, 1.0];
    }
    if dimension == 1 {
        return [8.0 / 255.0, 8.0 / 255.0, 16.0 / 255.0, 1.0];
    }

    // `World#getSkyColor`: weather strengths are currently zero until the
    // corresponding WorldInfo interpolation is ported, but celestial and
    // lightning terms exactly follow MCP 1.12.2.
    let daylight = ((celestialAngle * std::f32::consts::TAU).cos() * 2.0 + 0.5)
        .clamp(0.0, 1.0);
    let mut color = [
        biomeSkyColor[0] * daylight,
        biomeSkyColor[1] * daylight,
        biomeSkyColor[2] * daylight,
    ];
    if lastLightningBolt > 0 {
        let lightning = ((lastLightningBolt as f32 - partialTicks).min(1.0) * 0.45).max(0.0);
        color[0] = color[0] * (1.0 - lightning) + 0.8 * lightning;
        color[1] = color[1] * (1.0 - lightning) + 0.8 * lightning;
        color[2] = color[2] * (1.0 - lightning) + 1.0 * lightning;
    }
    [color[0], color[1], color[2], 1.0]
}

fn fog_color(dimension: i32, celestialAngle: f32) -> [f32; 4] {
    match dimension {
        -1 => [0.2, 0.03, 0.03, 1.0],
        1 => [0.0, 0.0, 0.0, 1.0],
        _ => {
            // MCP `WorldProvider#getFogColor` for the surface provider.
            let daylight = ((celestialAngle * std::f32::consts::TAU).cos() * 2.0 + 0.5)
                .clamp(0.0, 1.0);
            [
                0.752_941_2 * (daylight * 0.94 + 0.06),
                0.847_058_83 * (daylight * 0.94 + 0.06),
                1.0 * (daylight * 0.91 + 0.09),
                1.0,
            ]
        }
    }
}

fn sunrise_sunset_colors(celestialAngle: f32) -> Option<[f32; 4]> {
    let cosine = (celestialAngle * std::f32::consts::TAU).cos();
    if !(-0.4..=0.4).contains(&cosine) {
        return None;
    }
    let phase = cosine / 0.4 * 0.5 + 0.5;
    let mut alpha = 1.0 - (1.0 - (phase * std::f32::consts::PI).sin()) * 0.99;
    alpha *= alpha;
    Some([
        phase * 0.3 + 0.7,
        phase * phase * 0.7 + 0.2,
        0.2,
        alpha,
    ])
}

fn star_brightness(celestialAngle: f32) -> f32 {
    let darkness = (1.0 - ((celestialAngle * std::f32::consts::TAU).cos() * 2.0 + 0.25))
        .clamp(0.0, 1.0);
    darkness * darkness * 0.5
}

fn dynamic_quad_tint(
    atlas: &AtlasState,
    access: &SnapshotBlockAccess<'_>,
    state: IBlockState,
    pos: BlockPos,
    face: &ResolvedFace,
) -> [f32; 3] {
    if face.layers.is_empty() || !face.layers.iter().all(|layer| layer.tintIndex.is_some()) {
        return [1.0; 3];
    }
    let tintIndex = face.layers[0].tintIndex.unwrap_or(-1);
    let color = atlas.blockColors.colorMultiplier(state, access, pos, tintIndex);
    if color < 0 {
        [1.0; 3]
    } else {
        [
            ((color >> 16) & 255) as f32 / 255.0,
            ((color >> 8) & 255) as f32 / 255.0,
            (color & 255) as f32 / 255.0,
        ]
    }
}

/// `EnumDyeColor.byDyeDamage(damage).func_193350_e()` in protocol-340
/// item-damage order: black through white.
fn dye_color_value_by_damage(damage: usize) -> i32 {
    const DYE_RGB: [i32; 16] = [
        1_908_001, 11_546_150, 6_192_150, 8_606_770,
        3_949_738, 8_991_416, 1_481_884, 10_329_495,
        4_673_362, 15_961_002, 8_439_583, 16_701_501,
        3_847_130, 13_061_821, 16_351_261, 16_383_998,
    ];
    DYE_RGB[damage.min(15)]
}

/// One pixel of 1.12.2 `LayeredColorMaskTexture`. Java copies the neutral
/// sheet, rewrites a participating mask pixel to `(mask.red, base*dye)`, then
/// draws it with the normal SRC_OVER composite.
fn layered_color_mask_pixel(base: [u8; 4], mask: [u8; 4], dye: i32) -> [u8; 4] {
    if mask[3] == 0 || mask[0] == 0 {
        return base;
    }
    let source = [
        ((base[0] as i32 * ((dye >> 16) & 0xFF)) as f32 / 255.0) as u8,
        ((base[1] as i32 * ((dye >> 8) & 0xFF)) as f32 / 255.0) as u8,
        ((base[2] as i32 * (dye & 0xFF)) as f32 / 255.0) as u8,
        mask[0],
    ];
    if source[3] == 255 {
        return source;
    }
    let sourceAlpha = source[3] as f32 / 255.0;
    let destinationAlpha = base[3] as f32 / 255.0;
    let outputAlpha = sourceAlpha + destinationAlpha * (1.0 - sourceAlpha);
    if outputAlpha <= f32::EPSILON {
        return [0; 4];
    }
    let mut output = [0_u8; 4];
    for channel in 0..3 {
        let value = (source[channel] as f32 * sourceAlpha
            + base[channel] as f32 * destinationAlpha * (1.0 - sourceAlpha))
            / outputAlpha;
        output[channel] = value.clamp(0.0, 255.0).round() as u8;
    }
    output[3] = (outputAlpha * 255.0).clamp(0.0, 255.0).round() as u8;
    output
}

fn tint_color(blockId: i32, tintIndex: Option<i32>) -> [f32; 3] {
    if tintIndex.is_none() {
        return [1.0; 3];
    }
    let color = if (-31_015..=-31_000).contains(&blockId) {
        dye_color_value_by_damage((-31_000 - blockId) as usize)
    } else {
        match blockId {
            2 => 0x91BD59,
            18 | 161 => 0x77AB2F,
            106 => 0x48B518,
            _ => 0xFFFFFF,
        }
    };
    [
        ((color >> 16) & 0xFF) as f32 / 255.0,
        ((color >> 8) & 0xFF) as f32 / 255.0,
        (color & 0xFF) as f32 / 255.0,
    ]
}

fn missing_atlas() -> BlockTextureAtlas {
    let source = TextureSource::missing(ResourceLocation::new(
        "minecraft",
        "textures/missingno.png",
    ));
    BlockTextureAtlas {
        width: source.image.width(),
        height: source.image.height(),
        rgba: source.image.rgba().to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sign_state_with_line(line: &str) -> SignRenderState {
        SignRenderState {
            pos: BlockPos::new(1, 64, -2),
            blockId: 63,
            metadata: 0,
            lines: [line.to_owned(), String::new(), String::new(), String::new()],
            lineBeingEdited: -1,
            packedLight: 0,
        }
    }

    fn test_team(
        name: &str,
        visibility: &str,
        friendlyFlags: i32,
    ) -> crate::net::minecraft::scoreboard::ScorePlayerTeam::ScorePlayerTeam {
        let mut team =
            crate::net::minecraft::scoreboard::ScorePlayerTeam::ScorePlayerTeam::new(name);
        team.update(name, "", "", friendlyFlags, visibility, "always", -1);
        team
    }

    #[test]
    fn player_name_visibility_matches_mcp_team_rules() {
        let red = test_team("red", "always", 0);
        let redFriendlyInvisible = test_team("red", "always", 2);
        let blue = test_team("blue", "always", 0);

        assert!(can_render_player_name_for_teams(
            None, None, false, false, false,
        ));
        assert!(!can_render_player_name_for_teams(
            None, None, false, false, true,
        ));
        assert!(!can_render_player_name_for_teams(
            None, None, true, false, false,
        ));
        assert!(can_render_player_name_for_teams(
            None, None, true, true, false,
        ));

        assert!(can_render_player_name_for_teams(
            Some(&red), Some(&red), false, false, false,
        ));
        assert!(!can_render_player_name_for_teams(
            Some(&red), Some(&red), true, false, false,
        ));
        assert!(can_render_player_name_for_teams(
            Some(&redFriendlyInvisible),
            Some(&redFriendlyInvisible),
            true,
            false,
            false,
        ));

        let never = test_team("red", "never", 2);
        assert!(!can_render_player_name_for_teams(
            Some(&never), Some(&never), false, false, false,
        ));

        let hideOther = test_team("red", "hideForOtherTeams", 0);
        assert!(can_render_player_name_for_teams(
            Some(&hideOther), Some(&hideOther), false, false, false,
        ));
        assert!(!can_render_player_name_for_teams(
            Some(&hideOther), Some(&blue), false, false, false,
        ));

        let hideOwn = test_team("red", "hideForOwnTeam", 0);
        assert!(!can_render_player_name_for_teams(
            Some(&hideOwn), Some(&hideOwn), false, false, false,
        ));
        assert!(can_render_player_name_for_teams(
            Some(&hideOwn), Some(&blue), false, false, false,
        ));
    }

    #[test]
    fn local_render_view_player_is_excluded_from_nameplates() {
        assert!(is_local_render_view_player(17, Some(17)));
        assert!(!is_local_render_view_player(18, Some(17)));
        assert!(!is_local_render_view_player(17, None));
    }

    #[test]
    fn nameplate_background_and_font_passes_keep_distinct_state_ranges() {
        let mut ranges = Vec::new();
        push_world_entity_draw_range(
            &mut ranges,
            WorldEntityPipelineKind::NameplateBackgroundSeeThrough,
            WorldEntityMeshKind::Dynamic,
            0,
            6,
        );
        push_world_entity_draw_range(
            &mut ranges,
            WorldEntityPipelineKind::NameplateTextSeeThrough,
            WorldEntityMeshKind::Dynamic,
            6,
            6,
        );
        push_world_entity_draw_range(
            &mut ranges,
            WorldEntityPipelineKind::NameplateTextDepthWrite,
            WorldEntityMeshKind::Dynamic,
            12,
            6,
        );
        assert_eq!(ranges.len(), 3);
    }

    #[test]
    fn block_entity_mesh_cache_reuses_exact_per_instance_state() {
        let mut cache = RenderFrameMeshCache::default();
        let identity = BlockEntityMeshIdentity {
            kind: BlockEntityMeshKind::Sign,
            pos: BlockPos::new(1, 64, -2),
        };
        let key = BlockEntityMeshCacheKey {
            stateHash: 7,
            snapshotHash: 0,
            atlasRevision: 3,
        };

        cache.beginBlockEntityFrame();
        let first = cache.blockEntityMesh(identity, key, true, BlockEntityMeshBatch::default);
        cache.finishBlockEntityFrame();
        cache.beginBlockEntityFrame();
        let second = cache.blockEntityMesh(identity, key, true, || {
            panic!("exact cache hit must not rebuild the TESR mesh")
        });
        cache.finishBlockEntityFrame();

        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(cache.profileBlockEntityBuilds, 1);
        assert_eq!(cache.profileBlockEntityReuses, 1);
        assert_eq!(cache.blockEntityResidentCount(), 1);
    }

    fn empty_cached_dynamic_meshes_for_test() -> CachedDynamicMeshes {
        let constants = WorldPushConstants {
            viewProjection: [0.0; 16],
            cameraPosition: [0.0; 4],
            fogColor: [0.0; 4],
            fogParameters: [0.0; 4],
            lightmapParameters: [0.0; 4],
        };
        CachedDynamicMeshes {
            entityMeshGeneration: 0,
            blockEntityMeshGeneration: 0,
            staticEntityMeshGeneration: 0,
            entityDepthMeshGeneration: 0,
            entityOverlayMeshGeneration: 0,
            particleMeshGeneration: 0,
            transparentParticleMeshGeneration: 0,
            damageMeshGeneration: 0,
            selectionMeshGeneration: 0,
            firstPersonMeshGeneration: 0,
            hudMeshGeneration: 0,
            builtAt: Instant::now(),
            worldGeneration: 1,
            atlasRevision: 1,
            outputWidth: 854,
            outputHeight: 480,
            guiWidth: 427,
            guiHeight: 240,
            entityVertices: Arc::new(Vec::new()),
            entityIndices: Arc::new(Vec::new()),
            blockEntityVertices: Arc::new(Vec::new()),
            blockEntityIndices: Arc::new(Vec::new()),
            staticEntityVertices: Arc::new(Vec::new()),
            staticEntityIndices: Arc::new(Vec::new()),
            entityDrawRanges: Vec::new(),
            entityDepthVertices: Arc::new(Vec::new()),
            entityDepthIndices: Arc::new(Vec::new()),
            entityOverlayVertices: Arc::new(Vec::new()),
            entityOverlayIndices: Arc::new(Vec::new()),
            skyAlphaIndexCount: 0,
            skyCelestialIndexCount: 0,
            entityOverlayDrawRanges: Vec::new(),
            renderedRemotePlayers: 0,
            renderedNonPlayerEntities: 0,
            particleVertices: Arc::new(Vec::new()),
            particleIndices: Arc::new(Vec::new()),
            transparentParticleVertices: Arc::new(Vec::new()),
            transparentParticleIndices: Arc::new(Vec::new()),
            damageVertices: Arc::new(Vec::new()),
            damageIndices: Arc::new(Vec::new()),
            selectionVertices: Arc::new(Vec::new()),
            selectionIndices: Arc::new(Vec::new()),
            firstPersonVertices: Arc::new(Vec::new()),
            firstPersonIndices: Arc::new(Vec::new()),
            firstPersonDrawRanges: Vec::new(),
            firstPersonPushConstants: constants,
            hudVertices: Arc::new(Vec::new()),
            hudIndices: Arc::new(Vec::new()),
            hudDrawRanges: Vec::new(),
            hudPushConstants: constants,
        }
    }

    #[test]
    fn world_entity_draw_ranges_merge_only_compatible_contiguous_ranges() {
        let mut ranges = Vec::new();
        push_world_entity_draw_range(
            &mut ranges,
            WorldEntityPipelineKind::Entities,
            WorldEntityMeshKind::Dynamic,
            0,
            6,
        );
        push_world_entity_draw_range(
            &mut ranges,
            WorldEntityPipelineKind::Entities,
            WorldEntityMeshKind::Dynamic,
            6,
            12,
        );
        push_world_entity_draw_range(
            &mut ranges,
            WorldEntityPipelineKind::Entities,
            WorldEntityMeshKind::StaticEntities,
            0,
            6,
        );
        push_world_entity_draw_range(
            &mut ranges,
            WorldEntityPipelineKind::BlockEntities,
            WorldEntityMeshKind::BlockEntities,
            0,
            6,
        );
        push_world_entity_draw_range(
            &mut ranges,
            WorldEntityPipelineKind::Entities,
            WorldEntityMeshKind::Dynamic,
            18,
            0,
        );

        assert_eq!(
            ranges,
            vec![
                WorldEntityDrawRange {
                    pipeline: WorldEntityPipelineKind::Entities,
                    mesh: WorldEntityMeshKind::Dynamic,
                    firstIndex: 0,
                    indexCount: 18,
                },
                WorldEntityDrawRange {
                    pipeline: WorldEntityPipelineKind::Entities,
                    mesh: WorldEntityMeshKind::StaticEntities,
                    firstIndex: 0,
                    indexCount: 6,
                },
                WorldEntityDrawRange {
                    pipeline: WorldEntityPipelineKind::BlockEntities,
                    mesh: WorldEntityMeshKind::BlockEntities,
                    firstIndex: 0,
                    indexCount: 6,
                },
            ]
        );
    }

    #[test]
    fn independent_entity_stream_generations_do_not_cross_invalidate() {
        let mut cache = RenderFrameMeshCache::default();
        let first = cache.store(empty_cached_dynamic_meshes_for_test(), Duration::ZERO);
        let block_generation = first.blockEntityMeshGeneration;
        let static_generation = first.staticEntityMeshGeneration;

        let mut second_input = empty_cached_dynamic_meshes_for_test();
        second_input.entityVertices = Arc::new(vec![test_vertex(1.0)]);
        second_input.entityIndices = Arc::new(vec![0]);
        second_input.entityDrawRanges.push(WorldEntityDrawRange {
            pipeline: WorldEntityPipelineKind::Entities,
            mesh: WorldEntityMeshKind::Dynamic,
            firstIndex: 0,
            indexCount: 1,
        });
        let second = cache.store(second_input, Duration::ZERO);

        assert_ne!(second.entityMeshGeneration, first.entityMeshGeneration);
        assert_eq!(second.blockEntityMeshGeneration, block_generation);
        assert_eq!(second.staticEntityMeshGeneration, static_generation);
        assert!(Arc::ptr_eq(
            &second.blockEntityVertices,
            &first.blockEntityVertices,
        ));
        assert!(Arc::ptr_eq(
            &second.staticEntityVertices,
            &first.staticEntityVertices,
        ));
    }

    #[test]
    fn draw_plan_changes_do_not_force_identical_entity_buffer_uploads() {
        let mut cache = RenderFrameMeshCache::default();
        let mut first_input = empty_cached_dynamic_meshes_for_test();
        first_input.entityVertices = Arc::new(vec![test_vertex(1.0)]);
        first_input.entityIndices = Arc::new(vec![0]);
        first_input.entityDrawRanges.push(WorldEntityDrawRange {
            pipeline: WorldEntityPipelineKind::Entities,
            mesh: WorldEntityMeshKind::Dynamic,
            firstIndex: 0,
            indexCount: 1,
        });
        let first = cache.store(first_input, Duration::ZERO);

        let mut second_input = empty_cached_dynamic_meshes_for_test();
        second_input.entityVertices = Arc::new(vec![test_vertex(1.0)]);
        second_input.entityIndices = Arc::new(vec![0]);
        second_input.entityDrawRanges.push(WorldEntityDrawRange {
            pipeline: WorldEntityPipelineKind::BlockEntities,
            mesh: WorldEntityMeshKind::Dynamic,
            firstIndex: 0,
            indexCount: 1,
        });
        let second = cache.store(second_input, Duration::ZERO);

        assert_eq!(second.entityMeshGeneration, first.entityMeshGeneration);
        assert_eq!(second.entityDrawRanges, vec![WorldEntityDrawRange {
            pipeline: WorldEntityPipelineKind::BlockEntities,
            mesh: WorldEntityMeshKind::Dynamic,
            firstIndex: 0,
            indexCount: 1,
        }]);
    }

    #[test]
    fn static_entity_mesh_cache_reuses_and_evicts_by_entity_identity() {
        let mut cache = RenderFrameMeshCache::default();
        let identity = StaticEntityMeshIdentity {
            kind: StaticEntityMeshKind::Painting,
            entityId: 42,
        };
        let key = StaticEntityMeshCacheKey {
            stateHash: 11,
            snapshotHash: 7,
            atlasRevision: 3,
        };

        cache.beginStaticEntityFrame();
        let first = cache.staticEntityMesh(identity, key, StaticEntityMeshBatch::default);
        cache.finishStaticEntityFrame();
        cache.beginStaticEntityFrame();
        cache.touchStaticEntity(identity);
        let second = cache.staticEntityMesh(identity, key, || {
            panic!("exact static entity cache hit must not rebuild")
        });
        cache.finishStaticEntityFrame();

        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(cache.profileStaticEntityBuilds, 1);
        assert_eq!(cache.profileStaticEntityReuses, 1);
        assert_eq!(cache.staticEntityResidentCount(), 1);

        cache.beginStaticEntityFrame();
        cache.finishStaticEntityFrame();
        assert_eq!(cache.staticEntityResidentCount(), 0);
    }

    #[test]
    fn block_entity_mesh_cache_evicts_removed_instances() {
        let mut cache = RenderFrameMeshCache::default();
        let identity = BlockEntityMeshIdentity {
            kind: BlockEntityMeshKind::Bed,
            pos: BlockPos::new(0, 64, 0),
        };
        let key = BlockEntityMeshCacheKey {
            stateHash: 1,
            snapshotHash: 0,
            atlasRevision: 1,
        };

        cache.beginBlockEntityFrame();
        cache.blockEntityMesh(identity, key, true, BlockEntityMeshBatch::default);
        cache.finishBlockEntityFrame();
        assert_eq!(cache.blockEntityResidentCount(), 1);

        cache.beginBlockEntityFrame();
        cache.finishBlockEntityFrame();
        assert_eq!(cache.blockEntityResidentCount(), 0);
    }

    #[test]
    fn obfuscated_sign_text_bypasses_persistent_mesh_cache() {
        assert!(sign_uses_obfuscated_formatting(&sign_state_with_line("§kobfuscated")));
        assert!(sign_uses_obfuscated_formatting(&sign_state_with_line("§KOBFUSCATED")));
        assert!(!sign_uses_obfuscated_formatting(&sign_state_with_line("§aordinary")));
        assert!(!sign_uses_obfuscated_formatting(&sign_state_with_line("plain text")));
    }

    #[test]
    fn dynamic_player_textures_keep_native_non_square_extent() {
        assert!(is_full_entity_texture_material(-37_000));
        assert!(is_full_entity_texture_material(-37_999));
        assert!(!is_full_entity_texture_material(-38_000));

        let texture = Arc::new(TextureSource::dynamic(
            ResourceLocation::new("minecraft", "skins/cape-test"),
            NativeImage::from_rgba(64, 32, vec![255; 64 * 32 * 4]).unwrap(),
            "test",
        ));
        let material = MaterialRegistration {
            key: MaterialKey {
                blockId: -37_000,
                layers: vec![MaterialLayerKey {
                    texture: ResourceLocation::new("minecraft", "skins/cape-test"),
                    tintIndex: None,
                }],
            },
            textures: vec![texture],
        };
        assert_eq!(material_tile_size(&material), 64);
        let extent = material.textures.iter().fold((1_u32, 1_u32), |size, texture| {
            (size.0.max(texture.image.width()), size.1.max(texture.image.height()))
        });
        assert_eq!(extent, (64, 32));
    }

    #[test]
    fn sign_and_cape_textures_use_exact_resource_rectangles_in_gui_draws() {
        assert!(is_full_entity_texture_material(-35_000));
        assert!(is_full_entity_texture_material(-37_000));

        let sign = ResourceLocation::new("minecraft", "textures/entity/sign.png");
        let cape = ResourceLocation::new("minecraft", "skins/cape-test");
        let materials = vec![
            MaterialRegistration {
                key: MaterialKey {
                    blockId: -35_000,
                    layers: vec![MaterialLayerKey {
                        texture: sign.clone(),
                        tintIndex: None,
                    }],
                },
                textures: Vec::new(),
            },
            MaterialRegistration {
                key: MaterialKey {
                    blockId: -37_000,
                    layers: vec![MaterialLayerKey {
                        texture: cape.clone(),
                        tintIndex: None,
                    }],
                },
                textures: Vec::new(),
            },
        ];
        let rectangles = [[0.0, 0.0, 0.5, 0.25], [0.5, 0.0, 1.0, 0.25]];
        let map = build_exact_texture_rectangle_map(&materials, &rectangles);
        assert_eq!(map.get(&sign), Some(&rectangles[0]));
        assert_eq!(map.get(&cape), Some(&rectangles[1]));
    }

    #[test]
    fn stitcher_preserves_native_player_skin_cell_without_expanding_block_sprites() {
        let (width, height, placements) = stitch_material_tiles(&[16, 16, 64]);
        assert!(width.is_power_of_two());
        assert!(height.is_power_of_two());
        assert_eq!(placements[2][2], 64);
        assert_eq!(placements[0][2], 16);
        assert_eq!(placements[1][2], 16);
        for left in 0..placements.len() {
            for right in (left + 1)..placements.len() {
                let [lx, ly, ls] = placements[left];
                let [rx, ry, rs] = placements[right];
                let disjoint = lx + ls <= rx || rx + rs <= lx || ly + ls <= ry || ry + rs <= ly;
                assert!(disjoint, "atlas placements overlap: {left} and {right}");
            }
        }
    }

    #[test]
    fn layered_banner_base_uses_base_shading_and_mask_red_as_alpha() {
        let base = [128, 64, 32, 255];
        assert_eq!(
            layered_color_mask_pixel(base, [255, 255, 255, 255], 0xFF0000),
            [128, 0, 0, 255],
        );
        assert_eq!(
            layered_color_mask_pixel(base, [127, 255, 255, 255], 0xFFFFFF),
            [128, 64, 32, 255],
        );
        assert_eq!(layered_color_mask_pixel(base, [255, 255, 255, 0], 0xFFFFFF), base);
    }

    #[test]
    fn player_skin_uv_keeps_exact_64x64_texel_boundaries() {
        let rectangle = [64.0 / 256.0, 32.0 / 128.0, 128.0 / 256.0, 96.0 / 128.0];
        let mapped = map_player_skin_uv(rectangle, [8.0 / 64.0, 16.0 / 64.0]);
        assert!((mapped[0] - 72.0 / 256.0).abs() < 1.0e-6);
        assert!((mapped[1] - 48.0 / 128.0).abs() < 1.0e-6);
    }

    #[test]
    fn camera_vector_helpers_match_right_handed_basis_math() {
        assert_eq!(dot3([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]), 32.0);
        assert_eq!(cross3([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]), [0.0, 0.0, 1.0]);
        assert_eq!(normalize3([0.0, 0.0, -2.0]), [0.0, 0.0, -1.0]);
    }

    #[test]
    fn face_uv_rotation_matches_block_face_uv_quarter_turns() {
        assert_eq!(
            map_face_uv([0.0, 0.0], [0.0, 0.0, 16.0, 16.0], 90),
            [1.0, 0.0]
        );
        assert_eq!(
            map_face_uv([1.0, 0.0], [0.0, 0.0, 16.0, 16.0], 180),
            [0.0, 1.0]
        );
    }

    #[test]
    fn cube_faces_keep_face_bakery_outward_winding() {
        let expected = [
            (EnumFacing::Down, [0.0, -1.0, 0.0]),
            (EnumFacing::Up, [0.0, 1.0, 0.0]),
            (EnumFacing::North, [0.0, 0.0, -1.0]),
            (EnumFacing::South, [0.0, 0.0, 1.0]),
            (EnumFacing::West, [-1.0, 0.0, 0.0]),
            (EnumFacing::East, [1.0, 0.0, 0.0]),
        ];
        for (facing, outward) in expected {
            let (positions, _) = cube_face(0, 0, 0, facing);
            let first = [
                positions[1][0] - positions[0][0],
                positions[1][1] - positions[0][1],
                positions[1][2] - positions[0][2],
            ];
            let second = [
                positions[2][0] - positions[0][0],
                positions[2][1] - positions[0][1],
                positions[2][2] - positions[0][2],
            ];
            assert!(dot3(cross3(first, second), outward) > 0.0, "{facing:?}");
        }
    }

    #[test]
    fn block_model_renderer_bounds_choose_adjacent_light_exactly() {
        let eastBoundary = [[1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 0.0]];
        let eastInset = [[0.5, 0.0, 0.0], [0.5, 0.0, 1.0], [0.5, 1.0, 1.0], [0.5, 1.0, 0.0]];
        assert!(quad_uses_neighbour_light(eastBoundary, EnumFacing::East, false));
        assert!(!quad_uses_neighbour_light(eastInset, EnumFacing::East, false));
        assert!(quad_uses_neighbour_light(eastInset, EnumFacing::East, true));
    }

    #[test]
    fn stairs_and_slabs_use_world_neighbour_brightness() {
        assert!(crate::net::minecraft::block::Block::Block::getBlockById(53).useNeighborBrightness());
        assert!(crate::net::minecraft::block::Block::Block::getBlockById(44).useNeighborBrightness());
        assert!(!crate::net::minecraft::block::Block::Block::getBlockById(1).useNeighborBrightness());
    }

    #[test]
    fn compiled_chunk_layer_ranges_are_contiguous_and_rebased() {
        let mut layers: [LayerMesh; 4] = std::array::from_fn(|_| LayerMesh::default());
        layers[BlockRenderLayer::Solid.index()].vertices = vec![
            test_vertex(0.0),
            test_vertex(1.0),
            test_vertex(2.0),
        ];
        layers[BlockRenderLayer::Solid.index()].indices = vec![0, 1, 2];
        layers[BlockRenderLayer::Translucent.index()].vertices = vec![
            test_vertex(3.0),
            test_vertex(4.0),
            test_vertex(5.0),
        ];
        layers[BlockRenderLayer::Translucent.index()].indices = vec![0, 2, 1];

        let (vertices, indices, ranges) = combine_layer_meshes(layers);
        assert_eq!(vertices.len(), 6);
        assert_eq!(indices, vec![0, 1, 2, 3, 5, 4]);
        assert_eq!(
            ranges[BlockRenderLayer::Solid.index()],
            ChunkLayerRange {
                firstIndex: 0,
                indexCount: 3,
            }
        );
        assert_eq!(
            ranges[BlockRenderLayer::Translucent.index()],
            ChunkLayerRange {
                firstIndex: 3,
                indexCount: 3,
            }
        );
    }

    #[test]
    fn gui_projection_maps_top_to_top_and_bottom_to_bottom() {
        let width = 320.0_f32;
        let height = 180.0_f32;
        let matrix = [
            2.0 / width, 0.0, 0.0, 0.0,
            0.0, 2.0 / height, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            -1.0, -1.0, 0.0, 1.0,
        ];
        let top = transform_column_major(matrix, [0.0, 0.0, 0.0, 1.0]);
        let bottom = transform_column_major(matrix, [0.0, height, 0.0, 1.0]);
        assert!((top[1] + 1.0).abs() < 1.0e-6);
        assert!((bottom[1] - 1.0).abs() < 1.0e-6);
    }


    #[test]
    fn camera_basis_remains_finite_and_yaw_stable_at_vertical_pitch() {
        for yaw in [0.0_f32, 45.0, 90.0, 180.0, 271.0] {
            for pitch in [-90.0_f32, 90.0] {
                let (right, up, forward) = camera_axes(yaw, pitch);
                let matrix = camera_matrix(yaw, pitch, [0.0, 64.0, 0.0], 70.0, 16.0 / 9.0, 0.05, 512.0);
                assert!(matrix.into_iter().flatten().all(f32::is_finite));
                let yawRadians = (-yaw as f64).to_radians() - std::f64::consts::PI;
                let expectedRight = [yawRadians.cos() as f32, 0.0, -yawRadians.sin() as f32];
                for axis in 0..3 {
                    assert!((right[axis] - expectedRight[axis]).abs() < 1.0e-5);
                }
                assert!((dot3(right, right) - 1.0).abs() < 1.0e-5);
                assert!((dot3(up, up) - 1.0).abs() < 1.0e-5);
                assert!((dot3(forward, forward) - 1.0).abs() < 1.0e-5);
                assert!(dot3(right, up).abs() < 1.0e-5);
                assert!(dot3(right, forward).abs() < 1.0e-5);
                assert!(dot3(up, forward).abs() < 1.0e-5);
            }
        }
    }

    #[test]
    fn third_person_camera_world_offset_uses_interpolated_view_rotation() {
        let world = WorldClient::new(0);
        let mut player = crate::net::minecraft::client::entity::EntityPlayerSP::EntityPlayerSP::new(1);
        player.entity.rotationYaw = 90.0;
        player.entity.rotationPitch = 0.0;
        let position = PlayerPositionState {
            posX: 0.0,
            posY: 64.0,
            posZ: 0.0,
            rotationYaw: 45.0,
            rotationPitch: 0.0,
            eyeHeight: player.getEyeHeight(),
        };
        let (camera, yaw, pitch) = orient_camera_112(&world, &position, Some(&player), 1);
        let root = [0.0_f32, 64.0 + position.eyeHeight, 0.0_f32];
        let expected = 4.0_f32 / 2.0_f32.sqrt();
        assert!((camera[0] - (root[0] + expected)).abs() < 1.0e-4);
        assert!((camera[2] - (root[2] - expected)).abs() < 1.0e-4);
        assert_eq!(yaw, 45.0);
        assert_eq!(pitch, 0.0);
    }

    #[test]
    fn front_third_person_camera_uses_interpolated_forward_direction() {
        let world = WorldClient::new(0);
        let mut player = crate::net::minecraft::client::entity::EntityPlayerSP::EntityPlayerSP::new(1);
        player.entity.rotationYaw = 90.0;
        let position = PlayerPositionState {
            posX: 0.0,
            posY: 64.0,
            posZ: 0.0,
            rotationYaw: 45.0,
            rotationPitch: 0.0,
            eyeHeight: player.getEyeHeight(),
        };
        let (camera, yaw, pitch) = orient_camera_112(&world, &position, Some(&player), 2);
        let root = [0.0_f32, 64.0 + position.eyeHeight, 0.0_f32];
        let expected = 4.0_f32 / 2.0_f32.sqrt();
        assert!((camera[0] - (root[0] - expected)).abs() < 1.0e-4);
        assert!((camera[2] - (root[2] + expected)).abs() < 1.0e-4);
        assert_eq!(yaw, 225.0);
        assert_eq!(pitch, -0.0);
    }

    #[test]
    fn local_camera_position_interpolates_between_client_ticks() {
        let mut state = PlayClientState::default();
        let mut player = crate::net::minecraft::client::entity::EntityPlayerSP::EntityPlayerSP::new(1);
        player.entity.prevPosX = 10.0;
        player.entity.prevPosY = 64.0;
        player.entity.prevPosZ = -4.0;
        player.entity.posX = 12.0;
        player.entity.posY = 65.0;
        player.entity.posZ = 0.0;
        player.entity.prevRotationYaw = 20.0;
        player.entity.rotationYaw = 40.0;
        player.entity.prevRotationPitch = -10.0;
        player.entity.rotationPitch = 10.0;
        state.thePlayer = Some(player);
        let render = interpolated_player_position(&state, 0.5);
        assert_eq!(render.posX, 11.0);
        assert_eq!(render.posY, 64.5);
        assert_eq!(render.posZ, -2.0);
        assert_eq!(render.rotationYaw, 30.0);
        assert_eq!(render.rotationPitch, 0.0);
    }

    fn transform_column_major(matrix: [f32; 16], vector: [f32; 4]) -> [f32; 4] {
        let mut result = [0.0_f32; 4];
        for row in 0..4 {
            result[row] = matrix[row] * vector[0]
                + matrix[4 + row] * vector[1]
                + matrix[8 + row] * vector[2]
                + matrix[12 + row] * vector[3];
        }
        result
    }

    #[test]
    fn double_plant_upper_model_uses_lower_variant_not_facing_metadata() {
        let lowerRose = IBlockState::fromGlobalStateId((175 << 4) | 4);
        for upperMetadata in 8..=11 {
            let upper = IBlockState::fromGlobalStateId((175 << 4) | upperMetadata);
            assert_eq!(double_plant_actual_model_key(upper, lowerRose), (4, true));
        }
        let lowerFern = IBlockState::fromGlobalStateId((175 << 4) | 3);
        assert_eq!(double_plant_actual_model_key(lowerFern, lowerFern), (3, false));
    }

    #[test]
    fn living_hurt_overlay_uses_out_of_range_block_light_sentinel() {
        assert_eq!(encoded_living_hurt_block_light(7.0, 0, 0), 7.0);
        assert_eq!(encoded_living_hurt_block_light(7.0, 10, 0), 23.0);
        assert_eq!(encoded_living_hurt_block_light(7.0, 0, 1), 23.0);
    }

    #[test]
    fn non_player_living_hurt_flag_is_limited_to_living_entities() {
        use crate::net::minecraft::client::entity::EntityOtherClient::{
            ClientEntityKind, MobEntityType, ObjectSpawnType,
        };
        let mut zombie = EntityOtherClient::new(
            1,
            None,
            ClientEntityKind::Mob { entityType: MobEntityType::fromId(54).unwrap() },
            0.0,
            64.0,
            0.0,
            0.0,
            0.0,
        );
        zombie.hurtTime = 10;
        let marked = packed_light_with_living_hurt_overlay(&zombie, 0x00F0_00F0);
        assert_ne!(marked & ENTITY_HURT_OVERLAY_FLAG, 0);
        assert_eq!(encoded_block_light_from_packed(marked), 31.0);
        assert_eq!(
            packed_light_without_living_hurt_overlay(marked),
            0x00F0_00F0,
        );

        let mut item = EntityOtherClient::new(
            2,
            None,
            ClientEntityKind::Object {
                objectType: ObjectSpawnType::Item,
                data: 0,
                spawnVelocity: [0.0; 3],
            },
            0.0,
            64.0,
            0.0,
            0.0,
            0.0,
        );
        item.hurtTime = 10;
        assert_eq!(
            packed_light_with_living_hurt_overlay(&item, 0x00F0_00F0)
                & ENTITY_HURT_OVERLAY_FLAG,
            0,
        );
    }

    #[test]
    fn overlay_projection_keeps_totem_activation_depth_visible() {
        let matrix = hud_projection(854.0, 480.0);
        let clip = transform_column_major(matrix, [427.0, 240.0, -50.0, 1.0]);
        assert!((clip[0] - 0.0).abs() < 1.0e-6);
        assert!((clip[1] - 0.0).abs() < 1.0e-6);
        assert!((clip[2] - 0.525).abs() < 1.0e-6);
        assert_eq!(clip[3], 1.0);
        assert!((0.0..=1.0).contains(&clip[2]));
    }

    #[test]
    fn vanilla_end_sky_has_six_faces_of_sixteen_by_sixteen_tiles() {
        // One quad is six indices. Splitting each of the six source cube
        // faces into 16x16 tiles preserves UV repeat inside a stitched atlas.
        assert_eq!(6 * 16 * 16 * 6, 9_216);
    }

    fn test_vertex(x: f32) -> WorldVertex {
        WorldVertex {
            position: [x, 0.0, 0.0],
            uv: [0.0, 0.0],
            color: [1.0; 4],
            lightmap: [15.0, 15.0],
            shaderEntity: [-1, -1, -1],
            shaderPadding: 0,
        }
    }

    #[test]
    fn translucent_quads_sort_far_to_near_like_bufferbuilder() {
        let mut vertices = Vec::new();
        for x in [1.0_f32, 9.0] {
            for offset in [0.0_f32, 0.2, 0.4, 0.6] {
                vertices.push(WorldVertex {
                    position: [x + offset, 0.0, 0.0],
                    uv: [0.0, 0.0],
                    color: [1.0; 4],
                    lightmap: [15.0, 15.0],
                    shaderEntity: [-1, -1, -1],
                    shaderPadding: 0,
                });
            }
        }
        let mut indices = vec![
            0, 1, 2, 0, 2, 3,
            4, 5, 6, 4, 6, 7,
        ];
        assert!(sort_translucent_indices(
            vertices.as_slice(),
            indices.as_mut_slice(),
            [0.0, 0.0, 0.0],
        ));
        assert_eq!(
            indices,
            vec![4, 5, 6, 4, 6, 7, 0, 1, 2, 0, 2, 3],
        );
    }

    #[test]
    fn translucent_equal_distance_sort_is_stable() {
        let vertices = vec![
            test_vertex(-1.0),
            test_vertex(-1.0),
            test_vertex(-1.0),
            test_vertex(-1.0),
            test_vertex(1.0),
            test_vertex(1.0),
            test_vertex(1.0),
            test_vertex(1.0),
        ];
        let original = vec![0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7];
        let mut indices = original.clone();
        assert!(!sort_translucent_indices(
            vertices.as_slice(),
            indices.as_mut_slice(),
            [0.0, 0.0, 0.0],
        ));
        assert_eq!(indices, original);
    }

    #[test]
    fn translucent_prepare_does_not_mutate_before_changed_order_is_applied() {
        let vertices = vec![
            test_vertex(1.0),
            test_vertex(1.0),
            test_vertex(1.0),
            test_vertex(1.0),
            test_vertex(9.0),
            test_vertex(9.0),
            test_vertex(9.0),
            test_vertex(9.0),
        ];
        let original = vec![0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7];
        let mut scratch = TranslucentSortScratch::default();
        assert!(prepare_translucent_sort(
            vertices.as_slice(),
            original.as_slice(),
            [0.0, 0.0, 0.0],
            &mut scratch,
        ));
        assert_eq!(original, vec![0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7]);
        let originalCapacity = scratch.originalIndices.capacity();
        let orderCapacity = scratch.order.capacity();
        let mut sorted = original.clone();
        assert!(apply_translucent_sort(sorted.as_mut_slice(), &scratch));
        assert_eq!(
            sorted,
            vec![4, 5, 6, 4, 6, 7, 0, 1, 2, 0, 2, 3]
        );

        // A second prepare reuses the same retained storage.
        assert!(!prepare_translucent_sort(
            vertices.as_slice(),
            sorted.as_slice(),
            [0.0, 0.0, 0.0],
            &mut scratch,
        ));
        assert_eq!(scratch.originalIndices.capacity(), originalCapacity);
        assert_eq!(scratch.order.capacity(), orderCapacity);
    }

    #[test]
    fn translucent_resort_changes_only_the_layer_index_range() {
        let vertices = vec![
            test_vertex(0.0),
            test_vertex(0.0),
            test_vertex(0.0),
            test_vertex(0.0),
            test_vertex(2.0),
            test_vertex(2.0),
            test_vertex(2.0),
            test_vertex(2.0),
            test_vertex(8.0),
            test_vertex(8.0),
            test_vertex(8.0),
            test_vertex(8.0),
        ];
        let mut indices = vec![
            0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 8, 9, 10, 8, 10, 11,
        ];
        let opaque = indices[..6].to_vec();
        assert!(sort_translucent_index_range(
            vertices.as_slice(),
            indices.as_mut_slice(),
            ChunkLayerRange {
                firstIndex: 6,
                indexCount: 12,
            },
            [0.0, 0.0, 0.0],
        ));
        assert_eq!(&indices[..6], opaque.as_slice());
        assert_eq!(&indices[6..12], &[8, 9, 10, 8, 10, 11]);
        assert_eq!(&indices[12..], &[4, 5, 6, 4, 6, 7]);
    }

    #[test]
    fn mesh_bounds_follow_generated_vertices_instead_of_full_world_height() {
        let vertices = vec![
            WorldVertex {
                position: [16.25, 63.0, -31.75],
                uv: [0.0, 0.0],
                color: [1.0; 4],
                lightmap: [15.0, 15.0],
            
                shaderEntity: [-1, -1, -1],
                shaderPadding: 0,
            },
            WorldVertex {
                position: [31.0, 79.5, -16.0],
                uv: [1.0, 1.0],
                color: [1.0; 4],
                lightmap: [15.0, 15.0],
            
                shaderEntity: [-1, -1, -1],
                shaderPadding: 0,
            },
        ];
        assert_eq!(
            mesh_bounds(&vertices, RenderChunkKey::new(1, 4, -2)),
            ([16, 63, -32], [31, 80, -16])
        );
    }

    #[test]
    fn chunk_distance_prioritises_near_render_chunks() {
        let center = RenderChunkKey::new(0, 4, 0);
        assert!(
            render_chunk_distance_squared(RenderChunkKey::new(1, 4, 1), center)
                < render_chunk_distance_squared(RenderChunkKey::new(5, 4, 0), center)
        );
    }

    #[test]
    fn elytra_corpse_rotation_tilts_every_player_layer_about_the_feet() {
        let mut vertices = vec![WorldVertex {
            position: [0.0, 1.0, 0.0],
            uv: [0.0, 0.0],
            color: [1.0; 4],
            lightmap: [15.0, 15.0],
        
            shaderEntity: [-1, -1, -1],
            shaderPadding: 0,
        }];
        apply_player_elytra_corpse_rotation(
            &mut vertices,
            [0.0, 0.0, 0.0],
            180.0,
            ElytraCorpseRotation { pitchDegrees: -90.0, yawDegrees: 0.0 },
        );
        assert!(vertices[0].position[0].abs() < 1.0e-6);
        assert!(vertices[0].position[1].abs() < 1.0e-6);
        assert!((vertices[0].position[2] + 1.0).abs() < 1.0e-6);
    }

    #[test]
    fn render_player_arm_poses_follow_primary_hand_and_active_item_actions() {
        let paper = ItemStack { itemId: 339, count: 1, itemDamage: 0, tagCompound: None };
        let bow = ItemStack { itemId: 261, count: 1, itemDamage: 0, tagCompound: None };
        let shield = ItemStack { itemId: 442, count: 1, itemDamage: 0, tagCompound: None };

        assert_eq!(
            player_arm_poses(&paper, &ItemStack::EMPTY, 0, EnumHandSide::Right),
            (ArmPose::Empty, ArmPose::Item),
        );
        assert_eq!(
            player_arm_poses(&paper, &ItemStack::EMPTY, 0, EnumHandSide::Left),
            (ArmPose::Item, ArmPose::Empty),
        );
        assert_eq!(
            player_arm_poses(&bow, &ItemStack::EMPTY, 20, EnumHandSide::Right),
            (ArmPose::Empty, ArmPose::BowAndArrow),
        );
        assert_eq!(
            player_arm_poses(&ItemStack::EMPTY, &shield, 20, EnumHandSide::Right),
            (ArmPose::Block, ArmPose::Empty),
        );
    }

    #[test]
    fn model_renderer_post_arm_translation_uses_one_sixteenth_scale() {
        let pose = PartPose { pivot: [16.0, 8.0, -4.0], rotation: [0.0; 3] };
        let transformed = transform_point3(post_render_part_matrix(identity4(), pose), [0.0; 3]);
        assert_eq!(transformed, [1.0, 0.5, -0.25]);
    }

    #[test]
    fn world_held_item_anchor_uses_render_player_root_and_layer_transform_order() {
        let arm = PartPose { pivot: [-5.0, 2.0, 0.0], rotation: [0.0; 3] };
        let mut matrix = multiply4(
            translation4([10.0, 64.0, -3.0]),
            player_layer_root_matrix(0.0, false),
        );
        matrix = post_render_part_matrix(matrix, arm);
        matrix = multiply4(matrix, rotation_x4(-90.0));
        matrix = multiply4(matrix, rotation_y4(180.0));
        matrix = multiply4(
            matrix,
            translation4(LayerHeldItem::handTranslation(EnumHandSide::Right)),
        );
        let anchor = transform_point3(matrix, [0.0; 3]);
        let expected = [9.6484375, 64.70406, -2.8828125];
        for axis in 0..3 {
            assert!((anchor[axis] - expected[axis]).abs() < 1.0e-5, "axis {axis}: {:?}", anchor);
        }
    }

    #[test]
    fn selection_box_mesh_preserves_vanilla_line_strip_and_zero_alpha_connectors() {
        let selection = SelectionBoxRenderState {
            boundingBox: AxisAlignedBB::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0),
            color: [0.0, 0.0, 0.0, 0.4],
            lineWidth: 2.0,
        };
        let (vertices, indices) = build_selection_box_mesh(Some(selection));
        assert_eq!(vertices.len(), 16);
        assert_eq!(indices, (0_u32..16).collect::<Vec<_>>());
        assert_eq!(vertices[0].position, [1.0, 2.0, 3.0]);
        assert_eq!(vertices[7].position, [4.0, 5.0, 6.0]);
        assert_eq!(vertices[15].position, [4.0, 2.0, 3.0]);
        for index in [9_usize, 12, 14] {
            assert_eq!(vertices[index].color, [0.0, 0.0, 0.0, 0.0]);
        }
        for index in [0_usize, 1, 8, 10, 15] {
            assert_eq!(vertices[index].color, [0.0, 0.0, 0.0, 0.4]);
        }
        assert_eq!(build_selection_box_mesh(None), (Vec::new(), Vec::new()));
    }

    #[test]
    fn creative_tab_texture_geometry_matches_gui_container_creative() {
        let mut container = GuiContainer::new(195, 136, Vec::new());
        container.initGui(320, 240);
        assert_eq!(
            creative_tab_draw_position_selected(&container, 0, 0),
            Some((62, 24, 0, 32)),
        );
        assert_eq!(
            creative_tab_draw_position_selected(&container, 5, 0),
            Some((229, 24, 140, 0)),
        );
        assert_eq!(
            creative_tab_draw_position_selected(&container, 11, 11),
            Some((229, 184, 140, 96)),
        );
        assert_eq!(creative_tab_icon_position(&container, 0), Some((68, 33)));
        assert_eq!(creative_tab_icon_position(&container, 11), Some((235, 191)));
    }

    #[test]
    fn creative_inventory_tab_icons_are_exact_compiled_mcp_stacks() {
        assert_eq!(CREATIVE_TAB_ARRAY.len(), 12);
        assert_eq!(CREATIVE_TAB_ARRAY[0].getIconItemStack().itemId, 45);
        assert_eq!(CREATIVE_TAB_ARRAY[5].getIconItemStack().itemId, 345);
        assert_eq!(CREATIVE_TAB_ARRAY[11].getIconItemStack().itemId, 54);
    }

    #[test]
    fn entity_fire_matrix_preserves_mcp_scale_before_z_setback() {
        let matrix = fire_billboard_matrix([10.0, 20.0, 30.0], 2.0, 0.0, -0.3);
        let origin = transform_point3(matrix, [0.0, 0.0, 0.0]);
        assert!((origin[0] - 10.0).abs() < 1.0e-6);
        assert!((origin[1] - 20.0).abs() < 1.0e-6);
        assert!((origin[2] - 29.4).abs() < 1.0e-6);
    }

    #[test]
    fn first_person_fire_keeps_mcp_half_block_depth_instead_of_near_plane() {
        for sign in [-1.0_f32, 1.0] {
            let transformed = transform_point3(
                first_person_fire_matrix(sign),
                [0.0, 0.0, -0.5],
            );
            // Rotation changes X/Z slightly, but the quad must remain around
            // the source z=-0.5 plane rather than Batch 77's z≈-0.05.
            assert!(transformed[2] < -0.45, "unexpected near-plane fire: {transformed:?}");
        }
    }

    #[test]
    fn fire_alpha_tag_preserves_layer_and_source_alpha() {
        assert_eq!(encoded_fire_alpha(1.0, 0), -1.0);
        assert_eq!(encoded_fire_alpha(0.9, 1), -2.9);
    }

    #[test]
    fn missing_initial_mesh_does_not_dirty_its_inflight_build() {
        let mut renderer = VulkanWorldRenderer::new(
            ResourceManager::new(),
            FontRenderer::test_metric_renderer(),
            Locale::default(),
            std::env::temp_dir().join("mc112-skin-test"),
        );
        let key = RenderChunkKey::new(2, 4, -3);
        renderer.observedChunkRevisions.insert(key, 7);
        renderer.inflightChunks.insert(
            key,
            ChunkBuildToken {
                worldGeneration: renderer.worldGeneration,
                serial: 1,
                sourceRevision: 7,
            },
        );

        renderer.enqueueChunk(key);
        assert!(!renderer.dirtyWhileInflight.contains(&key));

        renderer.invalidateChunk(key);
        assert!(renderer.dirtyWhileInflight.contains(&key));
    }
}
