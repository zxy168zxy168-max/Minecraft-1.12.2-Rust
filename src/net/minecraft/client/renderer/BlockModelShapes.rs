use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;

use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::block::BlockEndPortalFrame::BlockEndPortalFrame;
use crate::net::minecraft::client::renderer::block::model::BlockFaceUV::BlockFaceUV;
use crate::net::minecraft::client::renderer::block::model::FaceBakery::FaceBakery;
use crate::net::minecraft::client::renderer::block::model::ModelBlock::{BlockPart, ModelBlock};
use crate::net::minecraft::client::renderer::block::model::ModelResourceLocation::ModelResourceLocation;
use crate::net::minecraft::client::renderer::block::model::ModelRotation::ModelRotation;
use crate::net::minecraft::client::resources::SimpleReloadableResourceManager::ResourceManager;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

const EPSILON: f32 = 1.0e-5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureLayer {
    pub texture: ResourceLocation,
    pub tintIndex: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedFace {
    pub layers: Vec<TextureLayer>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedQuad {
    /// Block-local coordinates after MCP `BlockPartRotation` and
    /// `ModelRotation` have been applied.
    pub positions: [[f32; 3]; 4],
    /// Normalised texture coordinates inside the material atlas rectangle.
    pub uvs: [[f32; 2]; 4],
    /// Facing reconstructed from the baked vertex normal, matching
    /// `FaceBakery.getFacingFromVertexData`.
    pub face: EnumFacing,
    /// The model face used for neighbour culling. This is deliberately kept
    /// separate from `face`, as in `IBakedModel.getQuads(state, side, rand)`.
    pub cullFace: Option<EnumFacing>,
    pub material: ResolvedFace,
    pub shade: bool,
    /// True when all four vertices lie on the corresponding block boundary.
    /// `BlockModelRenderer` uses this distinction when selecting light from
    /// the neighbour or from the block itself.
    pub boundary: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedBlockModel {
    pub quads: Vec<ResolvedQuad>,
    /// MCP `IBakedModel#getParticleTexture`, resolved from the model's `particle` texture key.
    pub particleTexture: Option<ResourceLocation>,
    pub fullCube: bool,
    pub opaqueCube: bool,
    pub ambientOcclusion: bool,
    pub missing: bool,
}

#[derive(Debug, Clone)]
struct VariantModel {
    model: ResourceLocation,
    x: i32,
    y: i32,
    uvlock: bool,
}

#[derive(Debug, Clone)]
pub struct BlockModelShapes {
    resourceManager: ResourceManager,
    bakedModelStore: HashMap<i32, Option<Arc<ResolvedBlockModel>>>,
    bakedVariantStore: HashMap<(i32, String), Option<Arc<ResolvedBlockModel>>>,
}

impl BlockModelShapes {
    pub fn new(resourceManager: ResourceManager) -> Self {
        Self {
            resourceManager,
            bakedModelStore: HashMap::new(),
            bakedVariantStore: HashMap::new(),
        }
    }

    pub fn getModelForState(&mut self, state: IBlockState) -> Option<Arc<ResolvedBlockModel>> {
        if state.isAir() {
            return None;
        }
        if let Some(cached) = self.bakedModelStore.get(&state.getGlobalStateId()) {
            return cached.clone();
        }
        let resolved = self.resolveState(state).ok().flatten().map(Arc::new);
        self.bakedModelStore
            .insert(state.getGlobalStateId(), resolved.clone());
        resolved
    }

    /// MCP 1.12.2 `BlockModelShapes#getTexture`. The returned location is
    /// the concrete PNG resource consumed by the Vulkan TextureMap atlas.
    pub fn getTexture(&mut self, state: IBlockState) -> ResourceLocation {
        if let Some(model) = self.getModelForState(state) {
            if !model.missing {
                if let Some(texture) = &model.particleTexture {
                    return texture.clone();
                }
            }
        }

        let path = match state.getBlockId() {
            54 | 63 | 68 | 146 | 176 | 177 | 26 => "blocks/planks_oak",
            130 => "blocks/obsidian",
            8 | 9 => "blocks/water_still",
            10 | 11 => "blocks/lava_still",
            144 => "blocks/soul_sand",
            166 => "items/barrier",
            217 => "items/structure_void",
            219 => "blocks/shulker_top_white",
            220 => "blocks/shulker_top_orange",
            221 => "blocks/shulker_top_magenta",
            222 => "blocks/shulker_top_light_blue",
            223 => "blocks/shulker_top_yellow",
            224 => "blocks/shulker_top_lime",
            225 => "blocks/shulker_top_pink",
            226 => "blocks/shulker_top_gray",
            227 => "blocks/shulker_top_silver",
            228 => "blocks/shulker_top_cyan",
            229 => "blocks/shulker_top_purple",
            230 => "blocks/shulker_top_blue",
            231 => "blocks/shulker_top_brown",
            232 => "blocks/shulker_top_green",
            233 => "blocks/shulker_top_red",
            234 => "blocks/shulker_top_black",
            _ => "missingno",
        };
        sprite_texture_location(&ResourceLocation::new("minecraft", path))
    }

    pub fn getModelForVariant(
        &mut self,
        state: IBlockState,
        variant: impl Into<String>,
    ) -> Option<Arc<ResolvedBlockModel>> {
        if state.isAir() {
            return None;
        }
        let variant = canonical_variant_key(&variant.into());
        let key = (state.getGlobalStateId(), variant.clone());
        if let Some(cached) = self.bakedVariantStore.get(&key) {
            return cached.clone();
        }
        let base = self.getModelResourceLocation(state)?;
        let location = ModelResourceLocation::new(base.getPath(), variant);
        let resolved = self
            .resolveLocation(state, &location)
            .ok()
            .flatten()
            .map(Arc::new);
        self.bakedVariantStore.insert(key, resolved.clone());
        resolved
    }

    fn resolveState(&self, state: IBlockState) -> anyhow::Result<Option<ResolvedBlockModel>> {
        let Some(modelLocation) = self.getModelResourceLocation(state) else {
            return Ok(None);
        };
        self.resolveLocation(state, &modelLocation)
    }

    fn resolveLocation(
        &self,
        state: IBlockState,
        modelLocation: &ModelResourceLocation,
    ) -> anyhow::Result<Option<ResolvedBlockModel>> {
        let variants = self.resolveVariantModels(modelLocation)?;
        if variants.is_empty() {
            return Ok(None);
        }

        let mut quads = Vec::<ResolvedQuad>::new();
        let mut fullCube = true;
        let mut ambientOcclusion = true;
        let mut loadedAny = false;
        let mut particleTexture = None;
        for variant in &variants {
            let model = ModelBlock::load(&self.resourceManager, &variant.model)?;
            loadedAny = true;
            if particleTexture.is_none() {
                particleTexture = model
                    .resolveTextureName("#particle")
                    .map(|location| sprite_texture_location(&location));
            }
            fullCube &= model_is_full_cube(&model);
            ambientOcclusion &= model.ambientOcclusion;
            for part in &model.elements {
                self.bakePart(&model, part, variant, &mut quads);
            }
        }

        if !loadedAny {
            return Ok(None);
        }
        let missing = quads.is_empty();
        if particleTexture.is_none() {
            particleTexture = quads
                .first()
                .and_then(|quad| quad.material.layers.first())
                .map(|layer| layer.texture.clone());
        }
        Ok(Some(ResolvedBlockModel {
            quads,
            particleTexture,
            fullCube: fullCube && !missing,
            opaqueCube: fullCube && !missing && state.getBlock().isOpaqueCube(),
            ambientOcclusion,
            missing,
        }))
    }

    fn bakePart(
        &self,
        model: &ModelBlock,
        part: &BlockPart,
        variant: &VariantModel,
        output: &mut Vec<ResolvedQuad>,
    ) {
        let modelRotation = ModelRotation::new(variant.x, variant.y);
        for (name, face) in &part.faces {
            let Some(sourceFacing) = face_name(name) else {
                continue;
            };
            let Some(textureName) = model.resolveTextureName(&face.texture) else {
                continue;
            };
            let texture = ResourceLocation::new(
                textureName.getNamespace(),
                format!("textures/{}.png", textureName.getPath()),
            );
            let faceUv = BlockFaceUV::new(
                face.uv
                    .unwrap_or_else(|| default_face_uv(part, sourceFacing)),
                normalize_rotation(face.rotation),
            );
            let baked = FaceBakery::makeBakedQuad(
                part,
                sourceFacing,
                faceUv,
                modelRotation,
                part.rotation.as_ref(),
                variant.uvlock,
            );
            let cullFace = face
                .cullface
                .as_deref()
                .and_then(face_name)
                .map(|facing| modelRotation.rotateFace(facing));

            let quad = ResolvedQuad {
                positions: baked.positions,
                uvs: baked.uvs,
                face: baked.face,
                cullFace,
                material: ResolvedFace {
                    layers: vec![TextureLayer {
                        texture,
                        tintIndex: face.tintindex,
                    }],
                },
                shade: part.shade,
                boundary: quad_on_boundary(baked.positions, baked.face),
            };
            push_or_merge_quad(output, quad);
        }
    }

    fn getModelResourceLocation(&self, state: IBlockState) -> Option<ModelResourceLocation> {
        let id = state.getBlockId();
        let meta = state.getMetadata().clamp(0, 15) as usize;
        let wood = ["oak", "spruce", "birch", "jungle", "acacia", "dark_oak"];
        let colors = [
            "white",
            "orange",
            "magenta",
            "light_blue",
            "yellow",
            "lime",
            "pink",
            "gray",
            "silver",
            "cyan",
            "purple",
            "blue",
            "brown",
            "green",
            "red",
            "black",
        ];
        let stoneSlabs = [
            "stone",
            "sandstone",
            "wood_old",
            "cobblestone",
            "brick",
            "stone_brick",
            "nether_brick",
            "quartz",
        ];

        let direct = match id {
            // Fluids, tile-entity-only renderers and blocks whose actual state
            // cannot yet be reconstructed from protocol metadata alone remain
            // explicit instead of being replaced with an invented cube.
            0
            | 8
            | 9
            | 10
            | 11
            | 26
            | 54
            | 63
            | 68
            | 119
            | 130
            | 137
            | 144
            | 146
            | 166
            | 176
            | 177
            | 209
            | 210
            | 211
            | 217
            | 219..=234 => {
                return None;
            }
            1 => Some(ModelResourceLocation::new(
                [
                    "stone",
                    "granite",
                    "smooth_granite",
                    "diorite",
                    "smooth_diorite",
                    "andesite",
                    "smooth_andesite",
                ]
                .get(meta)
                .copied()
                .unwrap_or("stone"),
                "normal",
            )),
            2 => Some(ModelResourceLocation::new("grass", "snowy=false")),
            3 => Some(ModelResourceLocation::new(
                ["dirt", "coarse_dirt", "podzol"]
                    .get(meta)
                    .copied()
                    .unwrap_or("dirt"),
                if meta == 2 { "snowy=false" } else { "normal" },
            )),
            5 => Some(ModelResourceLocation::new(
                format!("{}_planks", wood.get(meta).copied().unwrap_or("oak")),
                "normal",
            )),
            6 => Some(ModelResourceLocation::new(
                format!("{}_sapling", wood.get(meta & 7).copied().unwrap_or("oak")),
                format!("stage={}", (meta >> 3) & 1),
            )),
            12 => Some(ModelResourceLocation::new(
                if meta == 1 { "red_sand" } else { "sand" },
                "normal",
            )),
            17 => {
                let species = wood.get(meta & 3).copied().unwrap_or("oak");
                Some(ModelResourceLocation::new(
                    format!("{species}_log"),
                    axis_variant(meta),
                ))
            }
            18 => Some(ModelResourceLocation::new(
                format!("{}_leaves", wood.get(meta & 3).copied().unwrap_or("oak")),
                "normal",
            )),
            19 => Some(ModelResourceLocation::new(
                "sponge",
                if meta == 1 { "wet=true" } else { "wet=false" },
            )),
            24 => Some(ModelResourceLocation::new(
                ["sandstone", "chiseled_sandstone", "smooth_sandstone"]
                    .get(meta)
                    .copied()
                    .unwrap_or("sandstone"),
                "normal",
            )),
            31 => Some(ModelResourceLocation::new(
                ["dead_bush", "tall_grass", "fern"]
                    .get(meta & 3)
                    .copied()
                    .unwrap_or("tall_grass"),
                "normal",
            )),
            34 => {
                let facing = match meta & 7 {
                    0 => "down",
                    1 => "up",
                    2 => "north",
                    3 => "south",
                    4 => "west",
                    5 => "east",
                    _ => "north",
                };
                let pistonType = if meta & 8 != 0 { "sticky" } else { "normal" };
                Some(ModelResourceLocation::new(
                    "piston_head",
                    format!("facing={facing},short=false,type={pistonType}"),
                ))
            }
            35 => Some(ModelResourceLocation::new(
                format!("{}_wool", colors[meta]),
                "normal",
            )),
            37 => Some(ModelResourceLocation::new("dandelion", "normal")),
            38 => Some(ModelResourceLocation::new(
                [
                    "poppy",
                    "blue_orchid",
                    "allium",
                    "houstonia",
                    "red_tulip",
                    "orange_tulip",
                    "white_tulip",
                    "pink_tulip",
                    "oxeye_daisy",
                ]
                .get(meta)
                .copied()
                .unwrap_or("poppy"),
                "normal",
            )),
            43 | 44 => {
                let material = stoneSlabs[meta & 7];
                let name = if id == 43 {
                    format!("{material}_double_slab")
                } else {
                    format!("{material}_slab")
                };
                let variant = if id == 43 {
                    if meta & 8 != 0 {
                        "all"
                    } else {
                        "normal"
                    }
                } else if meta & 8 != 0 {
                    "half=top"
                } else {
                    "half=bottom"
                };
                Some(ModelResourceLocation::new(name, variant))
            }
            53 | 67 | 108 | 109 | 114 | 128 | 134..=136 | 156 | 163 | 164 | 180 | 203 => Some(
                ModelResourceLocation::new(state.getBlock().getRegistryPath(), stair_variant(meta)),
            ),
            78 => Some(ModelResourceLocation::new(
                "snow_layer",
                format!("layers={}", (meta & 7) + 1),
            )),
            90 => Some(ModelResourceLocation::new(
                "portal",
                if meta & 3 == 2 { "axis=z" } else { "axis=x" },
            )),
            95 => Some(ModelResourceLocation::new(
                format!("{}_stained_glass", colors[meta]),
                "normal",
            )),
            97 => Some(ModelResourceLocation::new(
                [
                    "stone_monster_egg",
                    "cobblestone_monster_egg",
                    "stone_brick_monster_egg",
                    "mossy_brick_monster_egg",
                    "cracked_brick_monster_egg",
                    "chiseled_brick_monster_egg",
                ]
                .get(meta)
                .copied()
                .unwrap_or("stone_monster_egg"),
                "normal",
            )),
            98 => Some(ModelResourceLocation::new(
                [
                    "stonebrick",
                    "mossy_stonebrick",
                    "cracked_stonebrick",
                    "chiseled_stonebrick",
                ]
                .get(meta)
                .copied()
                .unwrap_or("stonebrick"),
                "normal",
            )),
            120 => Some(ModelResourceLocation::new(
                "end_portal_frame",
                BlockEndPortalFrame::modelVariant(state),
            )),
            125 | 126 => {
                let species = wood.get(meta & 7).copied().unwrap_or("oak");
                Some(ModelResourceLocation::new(
                    if id == 125 {
                        format!("{species}_double_slab")
                    } else {
                        format!("{species}_slab")
                    },
                    if id == 125 {
                        "normal"
                    } else if meta & 8 != 0 {
                        "half=top"
                    } else {
                        "half=bottom"
                    },
                ))
            }
            155 => Some(match meta {
                1 => ModelResourceLocation::new("chiseled_quartz_block", "normal"),
                2 => ModelResourceLocation::new("quartz_column", "axis=y"),
                3 => ModelResourceLocation::new("quartz_column", "axis=x"),
                4 => ModelResourceLocation::new("quartz_column", "axis=z"),
                _ => ModelResourceLocation::new("quartz_block", "normal"),
            }),
            159 => Some(ModelResourceLocation::new(
                format!("{}_stained_hardened_clay", colors[meta]),
                "normal",
            )),
            160 => Some(ModelResourceLocation::new(
                format!("{}_stained_glass_pane", colors[meta]),
                "normal",
            )),
            161 => Some(ModelResourceLocation::new(
                format!(
                    "{}_leaves",
                    wood.get(4 + (meta & 1)).copied().unwrap_or("acacia")
                ),
                "normal",
            )),
            162 => {
                let species = wood.get(4 + (meta & 1)).copied().unwrap_or("acacia");
                Some(ModelResourceLocation::new(
                    format!("{species}_log"),
                    axis_variant(meta),
                ))
            }
            168 => Some(ModelResourceLocation::new(
                ["prismarine", "prismarine_bricks", "dark_prismarine"]
                    .get(meta)
                    .copied()
                    .unwrap_or("prismarine"),
                "normal",
            )),
            170 => Some(ModelResourceLocation::new(
                "hay_block",
                rotated_pillar_axis_variant(meta),
            )),
            171 => Some(ModelResourceLocation::new(
                format!("{}_carpet", colors[meta]),
                "normal",
            )),
            175 => {
                // MCP `BlockDoublePlant$1#getModelResourceLocation`: the
                // extended state is mapped to six independent blockstate
                // resources. Upper-half metadata does not retain VARIANT, so
                // callers that need the actual state pass a representative
                // state carrying the lower-half variant.
                let plant = [
                    "sunflower",
                    "syringa",
                    "double_grass",
                    "double_fern",
                    "double_rose",
                    "paeonia",
                ]
                .get(meta & 7)
                .copied()
                .unwrap_or("sunflower");
                Some(ModelResourceLocation::new(
                    plant,
                    if meta & 8 != 0 {
                        "half=upper"
                    } else {
                        "half=lower"
                    },
                ))
            }
            179 => Some(ModelResourceLocation::new(
                [
                    "red_sandstone",
                    "chiseled_red_sandstone",
                    "smooth_red_sandstone",
                ]
                .get(meta)
                .copied()
                .unwrap_or("red_sandstone"),
                "normal",
            )),
            181 | 182 => Some(ModelResourceLocation::new(
                if id == 181 {
                    "red_sandstone_double_slab"
                } else {
                    "red_sandstone_slab"
                },
                if id == 181 {
                    "normal"
                } else if meta & 8 != 0 {
                    "half=top"
                } else {
                    "half=bottom"
                },
            )),
            202 | 216 => Some(ModelResourceLocation::new(
                state.getBlock().getRegistryPath(),
                rotated_pillar_axis_variant(meta),
            )),
            204 | 205 => Some(ModelResourceLocation::new(
                if id == 204 {
                    "purpur_double_slab"
                } else {
                    "purpur_slab"
                },
                if id == 204 {
                    "variant=default"
                } else if meta & 8 != 0 {
                    "half=top,variant=default"
                } else {
                    "half=bottom,variant=default"
                },
            )),
            218 => Some(ModelResourceLocation::new(
                "observer",
                format!("facing={},powered=false", legacy_facing(meta)),
            )),
            235..=250 => Some(ModelResourceLocation::new(
                state.getBlock().getRegistryPath(),
                format!("facing={}", horizontal_facing(meta)),
            )),
            251 => Some(ModelResourceLocation::new(
                format!("{}_concrete", colors[meta]),
                "normal",
            )),
            252 => Some(ModelResourceLocation::new(
                format!("{}_concrete_powder", colors[meta]),
                "normal",
            )),
            _ => None,
        };
        direct.or_else(|| {
            Some(ModelResourceLocation::fromLocation(
                state.getBlock().getRegistryName(),
                default_variant_for_metadata(id, meta),
            ))
        })
    }

    fn resolveVariantModels(
        &self,
        location: &ModelResourceLocation,
    ) -> anyhow::Result<Vec<VariantModel>> {
        let blockstate = ResourceLocation::new(
            location.getNamespace(),
            format!("blockstates/{}.json", location.getPath()),
        );
        if let Ok(resource) = self.resourceManager.get_resource(&blockstate) {
            let root: Value = serde_json::from_slice(&resource.bytes)?;
            if let Some(variants) = root.get("variants").and_then(Value::as_object) {
                let requested = canonical_variant_key(location.getVariant());
                let selected = variants.iter().find_map(|(key, value)| {
                    (canonical_variant_key(key) == requested).then_some(value)
                });
                if let Some(selected) = selected.and_then(first_variant) {
                    if let Some(variant) = parse_variant(location.getNamespace(), selected) {
                        return Ok(vec![variant]);
                    }
                }
            }

            if let Some(parts) = root.get("multipart").and_then(Value::as_array) {
                let properties = parse_variant_properties(location.getVariant());
                let mut models = Vec::new();
                for part in parts {
                    let applies = part
                        .get("when")
                        .map_or(true, |when| multipart_condition_matches(when, &properties));
                    if !applies {
                        continue;
                    }
                    if let Some(apply) = part.get("apply").and_then(first_variant) {
                        if let Some(variant) = parse_variant(location.getNamespace(), apply) {
                            models.push(variant);
                        }
                    }
                }
                if !models.is_empty() {
                    return Ok(models);
                }
            }
        }

        let candidate = ResourceLocation::new(
            location.getNamespace(),
            format!("block/{}", location.getPath()),
        );
        let modelJson = ResourceLocation::new(
            candidate.getNamespace(),
            format!("models/{}.json", candidate.getPath()),
        );
        Ok(self
            .resourceManager
            .resource_exists(&modelJson)
            .then_some(VariantModel {
                model: candidate,
                x: 0,
                y: 0,
                uvlock: false,
            })
            .into_iter()
            .collect())
    }
}

fn parse_variant_properties(variant: &str) -> HashMap<String, String> {
    variant
        .split(',')
        .filter_map(|entry| entry.split_once('='))
        .map(|(key, value)| (key.trim().to_owned(), value.trim().to_ascii_lowercase()))
        .collect()
}

fn multipart_condition_matches(condition: &Value, properties: &HashMap<String, String>) -> bool {
    let Some(object) = condition.as_object() else {
        return false;
    };
    if let Some(or_conditions) = object.get("OR").and_then(Value::as_array) {
        return or_conditions
            .iter()
            .any(|entry| multipart_condition_matches(entry, properties));
    }
    object.iter().all(|(key, expected)| {
        let expected = match expected {
            Value::Bool(value) => value.to_string(),
            Value::String(value) => value.to_ascii_lowercase(),
            _ => return false,
        };
        let Some(actual) = properties.get(key) else {
            return false;
        };
        expected.split('|').any(|candidate| candidate == actual)
    })
}

fn first_variant(value: &Value) -> Option<&Value> {
    value
        .as_array()
        .and_then(|array| array.first())
        .or_else(|| value.as_object().map(|_| value))
}

fn parse_variant(namespace: &str, value: &Value) -> Option<VariantModel> {
    let model = value.get("model").and_then(Value::as_str)?;
    Some(VariantModel {
        model: model_location(namespace, model),
        x: value.get("x").and_then(Value::as_i64).unwrap_or(0) as i32,
        y: value.get("y").and_then(Value::as_i64).unwrap_or(0) as i32,
        uvlock: value
            .get("uvlock")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

fn canonical_variant_key(key: &str) -> String {
    let mut parts = key.split(',').map(str::trim).collect::<Vec<_>>();
    parts.sort_unstable();
    parts.join(",")
}

fn model_location(namespace: &str, value: &str) -> ResourceLocation {
    let location = if value.contains(':') {
        ResourceLocation::parse(value)
    } else {
        ResourceLocation::new(namespace, value)
    };
    if location.getPath().starts_with("block/") {
        location
    } else {
        ResourceLocation::new(
            location.getNamespace(),
            format!("block/{}", location.getPath()),
        )
    }
}

fn sprite_texture_location(sprite: &ResourceLocation) -> ResourceLocation {
    let path = sprite.getPath();
    if path.starts_with("textures/") && path.ends_with(".png") {
        sprite.clone()
    } else {
        ResourceLocation::new(sprite.getNamespace(), format!("textures/{path}.png"))
    }
}

fn face_name(name: &str) -> Option<EnumFacing> {
    match name {
        "down" => Some(EnumFacing::Down),
        "up" => Some(EnumFacing::Up),
        "north" => Some(EnumFacing::North),
        "south" => Some(EnumFacing::South),
        "west" => Some(EnumFacing::West),
        "east" => Some(EnumFacing::East),
        _ => None,
    }
}

fn normalize_rotation(rotation: i32) -> i32 {
    rotation.rem_euclid(360) / 90 * 90
}

fn axis_variant(meta: usize) -> &'static str {
    match meta & 12 {
        0 => "axis=y",
        4 => "axis=x",
        8 => "axis=z",
        _ => "axis=none",
    }
}

fn rotated_pillar_axis_variant(meta: usize) -> &'static str {
    match meta & 12 {
        4 => "axis=x",
        8 => "axis=z",
        _ => "axis=y",
    }
}

fn stair_variant(meta: usize) -> String {
    let facing = match meta & 3 {
        0 => "east",
        1 => "west",
        2 => "south",
        _ => "north",
    };
    let half = if meta & 4 != 0 { "top" } else { "bottom" };
    format!("facing={facing},half={half},shape=straight")
}

fn default_variant_for_metadata(blockId: i32, meta: usize) -> String {
    match blockId {
        23 | 158 => format!("facing={}", legacy_facing(meta)),
        29 | 33 => format!("extended={},facing={}", meta & 8 != 0, legacy_facing(meta),),
        27 | 28 => format!(
            "powered={},shape={}",
            meta & 8 != 0,
            powered_rail_shape(meta & 7),
        ),
        66 => format!("shape={}", rail_shape(meta)),
        61 | 62 => format!("facing={}", horizontal_or_north_facing(meta)),
        50 | 75 | 76 => format!("facing={}", torch_facing(meta)),
        59 => format!("age={}", meta & 7),
        60 => format!("moisture={}", meta & 7),
        65 => format!("facing={}", wall_facing(meta)),
        69 => format!(
            "facing={},powered={}",
            lever_orientation(meta),
            meta & 8 != 0,
        ),
        70 | 72 => format!("powered={}", meta != 0),
        77 | 143 => format!("facing={},powered={}", button_facing(meta), meta & 8 != 0,),
        86 | 91 => format!("facing={}", horizontal_facing(meta)),
        93 | 94 => format!(
            "delay={},facing={},locked=false",
            1 + (meta >> 2),
            horizontal_facing(meta),
        ),
        96 | 167 => {
            let facing = ["north", "south", "west", "east"][meta & 3];
            let half = if meta & 8 != 0 { "top" } else { "bottom" };
            let open = meta & 4 != 0;
            format!("facing={facing},half={half},open={open}")
        }
        104 | 105 => format!("age={},facing=up", meta & 7),
        106 => vine_variant(meta),
        115 => format!("age={}", meta.min(3)),
        141 | 142 => format!("age={}", meta & 7),
        145 => format!(
            "damage={},facing={}",
            ((meta & 15) >> 2).min(2),
            horizontal_facing(meta),
        ),
        149 | 150 => format!(
            "facing={},mode={},powered={}",
            horizontal_facing(meta),
            if meta & 4 != 0 { "subtract" } else { "compare" },
            meta & 8 != 0,
        ),
        154 => format!("facing={}", hopper_facing(meta)),
        198 => format!("facing={}", legacy_facing(meta)),
        207 => format!("age={}", meta & 3),
        208 => "snowy=false".to_owned(),
        _ => "normal".to_owned(),
    }
}

fn vine_variant(meta: usize) -> String {
    format!(
        "east={},north={},south={},up=false,west={}",
        meta & 8 != 0,
        meta & 4 != 0,
        meta & 1 != 0,
        meta & 2 != 0,
    )
}

fn legacy_facing(meta: usize) -> &'static str {
    // EnumFacing.getFront(meta & 7) indexes D-U-N-S-W-E modulo six.
    match (meta & 7) % 6 {
        0 => "down",
        1 => "up",
        2 => "north",
        3 => "south",
        4 => "west",
        _ => "east",
    }
}

fn horizontal_facing(meta: usize) -> &'static str {
    match meta & 3 {
        0 => "south",
        1 => "west",
        2 => "north",
        _ => "east",
    }
}

fn wall_facing(meta: usize) -> &'static str {
    match meta & 7 {
        2 => "north",
        3 => "south",
        4 => "west",
        _ => "east",
    }
}

fn torch_facing(meta: usize) -> &'static str {
    match meta & 7 {
        1 => "east",
        2 => "west",
        3 => "south",
        4 => "north",
        _ => "up",
    }
}

fn button_facing(meta: usize) -> &'static str {
    match meta & 7 {
        0 => "down",
        1 => "east",
        2 => "west",
        3 => "south",
        4 => "north",
        _ => "up",
    }
}

fn lever_orientation(meta: usize) -> &'static str {
    match meta & 7 {
        0 => "down_x",
        1 => "east",
        2 => "west",
        3 => "south",
        4 => "north",
        5 => "up_z",
        6 => "up_x",
        _ => "down_z",
    }
}

fn rail_shape(meta: usize) -> &'static str {
    match meta.min(9) {
        0 => "north_south",
        1 => "east_west",
        2 => "ascending_east",
        3 => "ascending_west",
        4 => "ascending_north",
        5 => "ascending_south",
        6 => "south_east",
        7 => "south_west",
        8 => "north_west",
        _ => "north_east",
    }
}

fn powered_rail_shape(meta: usize) -> &'static str {
    match meta & 7 {
        0 => "north_south",
        1 => "east_west",
        2 => "ascending_east",
        3 => "ascending_west",
        4 => "ascending_north",
        5 => "ascending_south",
        _ => "north_south",
    }
}

fn horizontal_or_north_facing(meta: usize) -> &'static str {
    match meta & 7 {
        2 => "north",
        3 => "south",
        4 => "west",
        5 => "east",
        _ => "north",
    }
}

fn hopper_facing(meta: usize) -> &'static str {
    match meta & 7 {
        2 => "north",
        3 => "south",
        4 => "west",
        5 => "east",
        _ => "down",
    }
}

fn model_is_full_cube(model: &ModelBlock) -> bool {
    !model.elements.is_empty()
        && model.elements.iter().all(|part| {
            part.rotation.is_none()
                && approximately_array(part.from, [0.0, 0.0, 0.0])
                && approximately_array(part.to, [16.0, 16.0, 16.0])
        })
}

fn default_face_uv(part: &BlockPart, facing: EnumFacing) -> [f32; 4] {
    let from = part.from;
    let to = part.to;
    match facing {
        EnumFacing::Down => [from[0], 16.0 - to[2], to[0], 16.0 - from[2]],
        EnumFacing::Up => [from[0], from[2], to[0], to[2]],
        EnumFacing::North => [16.0 - to[0], 16.0 - to[1], 16.0 - from[0], 16.0 - from[1]],
        EnumFacing::South => [from[0], 16.0 - to[1], to[0], 16.0 - from[1]],
        EnumFacing::West => [from[2], 16.0 - to[1], to[2], 16.0 - from[1]],
        EnumFacing::East => [16.0 - to[2], 16.0 - to[1], 16.0 - from[2], 16.0 - from[1]],
    }
}

fn quad_on_boundary(positions: [[f32; 3]; 4], facing: EnumFacing) -> bool {
    let (axis, boundary) = match facing {
        EnumFacing::Down => (1, 0.0),
        EnumFacing::Up => (1, 1.0),
        EnumFacing::North => (2, 0.0),
        EnumFacing::South => (2, 1.0),
        EnumFacing::West => (0, 0.0),
        EnumFacing::East => (0, 1.0),
    };
    positions
        .iter()
        .all(|position| (position[axis] - boundary).abs() <= EPSILON)
}

fn push_or_merge_quad(output: &mut Vec<ResolvedQuad>, quad: ResolvedQuad) {
    let quadTint = uniform_tint_index(&quad.material);
    if let Some(existing) = output.iter_mut().find(|existing| {
        positions_equal(existing.positions, quad.positions)
            && uvs_equal(existing.uvs, quad.uvs)
            && existing.face == quad.face
            && existing.cullFace == quad.cullFace
            && existing.shade == quad.shade
            && existing.boundary == quad.boundary
            // Vanilla retains one BakedQuad per tint index. Combining a
            // tintable redstone line with its untinted white overlay leaves
            // Vulkan only one vertex colour and turns the complete wire
            // white. Different non-negative tint indices must likewise stay
            // separate instead of inheriting the first layer's colour.
            && quadTint.is_some()
            && uniform_tint_index(&existing.material) == quadTint
    }) {
        existing.material.layers.extend(quad.material.layers);
    } else {
        output.push(quad);
    }
}

/// Returns the one tint semantic shared by every texture layer. The nested
/// option distinguishes a uniformly untinted material (`Some(None)`) from a
/// material with no layers or mixed tint indices (`None`).
fn uniform_tint_index(face: &ResolvedFace) -> Option<Option<i32>> {
    let first = face.layers.first()?.tintIndex;
    face.layers
        .iter()
        .all(|layer| layer.tintIndex == first)
        .then_some(first)
}

fn positions_equal(left: [[f32; 3]; 4], right: [[f32; 3]; 4]) -> bool {
    left.iter().zip(right).all(|(left, right)| {
        left.iter()
            .zip(right)
            .all(|(left, right)| (*left - right).abs() <= EPSILON)
    })
}

fn uvs_equal(left: [[f32; 2]; 4], right: [[f32; 2]; 4]) -> bool {
    left.iter().zip(right).all(|(left, right)| {
        left.iter()
            .zip(right)
            .all(|(left, right)| (*left - right).abs() <= EPSILON)
    })
}

fn approximately_array(left: [f32; 3], right: [f32; 3]) -> bool {
    left.iter()
        .zip(right)
        .all(|(left, right)| (*left - right).abs() <= EPSILON)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::client::renderer::block::model::ItemCameraTransforms::ItemCameraTransforms;

    #[test]
    fn common_state_mapper_names_match_mcp() {
        let manager = ResourceManager::new();
        let shapes = BlockModelShapes::new(manager);
        assert_eq!(
            shapes
                .getModelResourceLocation(IBlockState::fromGlobalStateId((1 << 4) | 1))
                .unwrap()
                .getPath(),
            "granite"
        );
        assert_eq!(
            shapes
                .getModelResourceLocation(IBlockState::fromGlobalStateId((5 << 4) | 5))
                .unwrap()
                .getPath(),
            "dark_oak_planks"
        );
        assert_eq!(
            shapes
                .getModelResourceLocation(IBlockState::fromGlobalStateId((17 << 4) | 4))
                .unwrap()
                .getVariant(),
            "axis=x"
        );
    }

    #[test]
    fn directional_metadata_variants_match_mcp() {
        assert_eq!(default_variant_for_metadata(61, 2), "facing=north");
        assert_eq!(
            default_variant_for_metadata(33, 10),
            "extended=true,facing=north"
        );
        assert_eq!(
            default_variant_for_metadata(69, 7),
            "facing=down_z,powered=false"
        );
        assert_eq!(default_variant_for_metadata(145, 9), "damage=2,facing=west");
        assert_eq!(default_variant_for_metadata(66, 9), "shape=north_east");
        assert_eq!(default_variant_for_metadata(23, 6), "facing=down");
        assert_eq!(default_variant_for_metadata(218, 7), "facing=up");
        assert_eq!(stair_variant(0), "facing=east,half=bottom,shape=straight");
        assert_eq!(stair_variant(7), "facing=north,half=top,shape=straight");
        assert_eq!(rotated_pillar_axis_variant(12), "axis=y");
    }

    #[test]
    fn double_plant_state_mapper_uses_only_the_half_property() {
        let shapes = BlockModelShapes::new(ResourceManager::new());
        let lower = shapes
            .getModelResourceLocation(IBlockState::fromGlobalStateId((175 << 4) | 4))
            .unwrap();
        assert_eq!(lower.getPath(), "double_rose");
        assert_eq!(lower.getVariant(), "half=lower");

        let upper = shapes
            .getModelResourceLocation(IBlockState::fromGlobalStateId((175 << 4) | 8 | 5))
            .unwrap();
        assert_eq!(upper.getPath(), "paeonia");
        assert_eq!(upper.getVariant(), "half=upper");
        assert!(!upper.getVariant().contains("variant="));
    }

    #[test]
    fn coincident_tinted_and_untinted_quads_remain_separate() {
        let make_quad = |tint_index| ResolvedQuad {
            positions: [[0.0, 0.0, 0.0]; 4],
            uvs: [[0.0, 0.0]; 4],
            face: EnumFacing::Up,
            cullFace: None,
            material: ResolvedFace {
                layers: vec![TextureLayer {
                    texture: ResourceLocation::new(
                        "minecraft",
                        "textures/blocks/redstone_dust_line.png",
                    ),
                    tintIndex: tint_index,
                }],
            },
            shade: false,
            boundary: true,
        };
        let mut output = Vec::new();
        push_or_merge_quad(&mut output, make_quad(Some(0)));
        push_or_merge_quad(&mut output, make_quad(None));
        push_or_merge_quad(&mut output, make_quad(Some(1)));
        assert_eq!(output.len(), 3);
    }

    #[test]
    fn default_uvs_match_block_part_get_face_uvs() {
        let part = BlockPart {
            from: [0.0, 0.0, 0.0],
            to: [16.0, 8.0, 16.0],
            rotation: None,
            shade: true,
            faces: HashMap::new(),
        };
        assert_eq!(
            default_face_uv(&part, EnumFacing::Up),
            [0.0, 0.0, 16.0, 16.0]
        );
        assert_eq!(
            default_face_uv(&part, EnumFacing::North),
            [0.0, 8.0, 16.0, 16.0]
        );
    }

    #[test]
    fn half_slab_geometry_is_not_classified_as_full_cube() {
        let model = ModelBlock {
            textures: HashMap::new(),
            elements: vec![BlockPart {
                from: [0.0, 0.0, 0.0],
                to: [16.0, 8.0, 16.0],
                rotation: None,
                shade: true,
                faces: HashMap::new(),
            }],
            ambientOcclusion: true,
            gui3d: false,
            transforms: ItemCameraTransforms::default(),
            generated: false,
            builtInRenderer: false,
            namespace: "minecraft".to_owned(),
        };
        assert!(!model_is_full_cube(&model));
    }
}
