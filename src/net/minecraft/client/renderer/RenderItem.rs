use std::collections::HashMap;
use std::sync::Arc;

use crate::net::minecraft::client::renderer::block::model::BlockFaceUV::BlockFaceUV;
use crate::net::minecraft::client::renderer::block::model::FaceBakery::FaceBakery;
use crate::net::minecraft::client::renderer::block::model::ItemCameraTransforms::{
    ItemCameraTransforms, ItemTransformVec3f, TransformType,
};
use crate::net::minecraft::client::renderer::block::model::ItemModelGenerator::ItemModelGenerator;
use crate::net::minecraft::client::renderer::block::model::ModelBlock::{BlockPart, ModelBlock};
use crate::net::minecraft::client::renderer::block::model::ModelRotation::ModelRotation;
use crate::net::minecraft::client::renderer::ItemModelMesher::registeredItemModels;
use crate::net::minecraft::client::resources::SimpleReloadableResourceManager::ResourceManager;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::vulkan::TextureSource::TextureSource;

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedItemQuad {
    pub positions: [[f32; 3]; 4],
    pub uvs: [[f32; 2]; 4],
    pub face: EnumFacing,
    pub texture: ResourceLocation,
    pub tintIndex: Option<i32>,
    pub shade: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedItemModel {
    pub quads: Vec<ResolvedItemQuad>,
    pub transforms: ItemCameraTransforms,
    pub gui3d: bool,
    pub builtInRenderer: bool,
    pub missing: bool,
}

impl ResolvedItemModel {
    pub fn guiTransform(&self) -> ItemTransformVec3f {
        self.transforms.getTransform(TransformType::Gui)
    }
}

/// MCP-facing item-model resolver. Vulkan consumes the resolved quads but model
/// registration, parent resolution, generated-item extrusion and camera
/// transforms remain owned by the original RenderItem/ItemModelMesher classes.
pub struct RenderItem {
    resourceManager: ResourceManager,
}

fn is_expected_unbaked_item_model(location: &str) -> bool {
    matches!(
        location,
        "minecraft:old_wood_slab" | "minecraft:purpur_double_slab"
    )
}

impl RenderItem {
    pub fn new(resourceManager: ResourceManager) -> Self {
        Self { resourceManager }
    }

    pub fn loadRegisteredModels(&self) -> HashMap<(i16, i16), Arc<ResolvedItemModel>> {
        let mut byLocation = HashMap::<(String, String), Arc<ResolvedItemModel>>::new();
        let mut models = HashMap::new();
        for &(itemId, metadata, location, variant) in registeredItemModels() {
            let cacheKey = (location.to_owned(), variant.to_owned());
            let model = if let Some(model) = byLocation.get(&cacheKey) {
                Arc::clone(model)
            } else {
                let resolved = self
                    .resolveModel(location, variant)
                    .unwrap_or_else(|error| {
                        // MCP `RenderItem` registers these legacy/non-item
                        // locations in ItemModelMesher, but
                        // `ModelBakery#registerVariantNames` deliberately does
                        // not bake them. Vanilla therefore resolves the
                        // ModelManager missing model without emitting a model
                        // load warning. Preserve that behavior while still
                        // warning for genuinely unexpected resource failures.
                        if !is_expected_unbaked_item_model(location) {
                            log::warn!("failed loading item model {location}#{variant}: {error}");
                        }
                        missing_item_model()
                    });
                let resolved = Arc::new(resolved);
                byLocation.insert(cacheKey, Arc::clone(&resolved));
                resolved
            };
            models.insert((itemId, metadata), model);
        }
        models
    }

    pub fn resolveModel(
        &self,
        location: &str,
        _variant: &str,
    ) -> anyhow::Result<ResolvedItemModel> {
        let location = ResourceLocation::parse(location);
        let itemModel = ResourceLocation::new(
            location.getNamespace(),
            format!("item/{}", location.getPath()),
        );
        let source = ModelBlock::load(&self.resourceManager, &itemModel)?;
        if source.builtInRenderer {
            return Ok(ResolvedItemModel {
                quads: Vec::new(),
                transforms: source.transforms,
                gui3d: source.gui3d,
                builtInRenderer: true,
                missing: false,
            });
        }

        let bakedSource = if source.generated {
            let mut layers = Vec::new();
            for layerName in ItemModelGenerator::LAYERS {
                let Some(textureName) = source.resolveTextureName(&format!("#{layerName}")) else {
                    break;
                };
                if textureName.getPath() == "missingno" {
                    break;
                }
                let textureLocation = ResourceLocation::new(
                    textureName.getNamespace(),
                    format!("textures/{}.png", textureName.getPath()),
                );
                let texture = Arc::new(
                    TextureSource::load(&self.resourceManager, &textureLocation)
                        .unwrap_or_else(|_| TextureSource::missing(textureLocation)),
                );
                layers.push((layerName.to_owned(), textureName, texture));
            }
            ItemModelGenerator::makeItemModel(&source, &layers).unwrap_or(source)
        } else {
            source
        };

        let mut quads = Vec::new();
        for part in &bakedSource.elements {
            bake_part(&bakedSource, part, &mut quads);
        }
        let missing = quads.is_empty();
        Ok(ResolvedItemModel {
            quads,
            transforms: bakedSource.transforms,
            gui3d: bakedSource.gui3d,
            builtInRenderer: bakedSource.builtInRenderer,
            missing,
        })
    }
}

fn bake_part(model: &ModelBlock, part: &BlockPart, output: &mut Vec<ResolvedItemQuad>) {
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
            face.rotation.rem_euclid(360),
        );
        let baked = FaceBakery::makeBakedQuad(
            part,
            sourceFacing,
            faceUv,
            ModelRotation::new(0, 0),
            part.rotation.as_ref(),
            false,
        );
        output.push(ResolvedItemQuad {
            positions: baked.positions,
            uvs: baked.uvs,
            face: baked.face,
            texture,
            tintIndex: face.tintindex,
            shade: part.shade,
        });
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

fn default_face_uv(part: &BlockPart, facing: EnumFacing) -> [f32; 4] {
    match facing {
        EnumFacing::Down => [
            part.from[0],
            16.0 - part.to[2],
            part.to[0],
            16.0 - part.from[2],
        ],
        EnumFacing::Up => [part.from[0], part.from[2], part.to[0], part.to[2]],
        EnumFacing::North => [
            16.0 - part.to[0],
            16.0 - part.to[1],
            16.0 - part.from[0],
            16.0 - part.from[1],
        ],
        EnumFacing::South => [
            part.from[0],
            16.0 - part.to[1],
            part.to[0],
            16.0 - part.from[1],
        ],
        EnumFacing::West => [
            part.from[2],
            16.0 - part.to[1],
            part.to[2],
            16.0 - part.from[1],
        ],
        EnumFacing::East => [
            16.0 - part.to[2],
            16.0 - part.to[1],
            16.0 - part.from[2],
            16.0 - part.from[1],
        ],
    }
}

fn missing_item_model() -> ResolvedItemModel {
    let part = BlockPart {
        from: [0.0, 0.0, 0.0],
        to: [16.0, 16.0, 16.0],
        rotation: None,
        shade: true,
        faces: ["down", "up", "north", "south", "west", "east"]
            .into_iter()
            .map(|name| {
                (
                name.to_owned(),
                crate::net::minecraft::client::renderer::block::model::ModelBlock::BlockPartFace {
                    texture: "minecraft:missingno".to_owned(),
                    tintindex: None,
                    cullface: None,
                    uv: None,
                    rotation: 0,
                },
            )
            })
            .collect(),
    };
    let model = ModelBlock {
        textures: HashMap::new(),
        elements: vec![part],
        ambientOcclusion: false,
        gui3d: true,
        transforms: ItemCameraTransforms::default(),
        generated: false,
        builtInRenderer: false,
        namespace: "minecraft".to_owned(),
    };
    let mut quads = Vec::new();
    bake_part(&model, &model.elements[0], &mut quads);
    ResolvedItemModel {
        quads,
        transforms: ItemCameraTransforms::default(),
        gui3d: true,
        builtInRenderer: false,
        missing: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_model_contains_a_complete_cube() {
        let model = missing_item_model();
        assert_eq!(model.quads.len(), 6);
        assert!(model.missing);
    }

    #[test]
    fn legacy_unbaked_item_models_match_mcp_model_bakery() {
        assert!(is_expected_unbaked_item_model("minecraft:old_wood_slab"));
        assert!(is_expected_unbaked_item_model(
            "minecraft:purpur_double_slab"
        ));
        assert!(!is_expected_unbaked_item_model("minecraft:stone_slab"));
    }
}
