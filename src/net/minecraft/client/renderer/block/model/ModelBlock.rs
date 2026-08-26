use std::collections::{HashMap, HashSet};

use serde::Deserialize;

use crate::net::minecraft::client::renderer::block::model::ItemCameraTransforms::{
    ItemCameraTransforms, ItemTransformJson,
};
use crate::net::minecraft::client::resources::SimpleReloadableResourceManager::ResourceManager;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct BlockPartFace {
    pub texture: String,
    #[serde(default)]
    pub tintindex: Option<i32>,
    #[serde(default)]
    pub cullface: Option<String>,
    #[serde(default)]
    pub uv: Option<[f32; 4]>,
    #[serde(default)]
    pub rotation: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlockPartRotation {
    pub origin: [f32; 3],
    pub axis: String,
    pub angle: f32,
    #[serde(default)]
    pub rescale: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlockPart {
    pub from: [f32; 3],
    pub to: [f32; 3],
    #[serde(default)]
    pub rotation: Option<BlockPartRotation>,
    #[serde(default = "default_true")]
    pub shade: bool,
    #[serde(default)]
    pub faces: HashMap<String, BlockPartFace>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ModelBlockJson {
    #[serde(default)]
    parent: Option<String>,
    #[serde(default)]
    textures: HashMap<String, String>,
    #[serde(default)]
    elements: Option<Vec<BlockPart>>,
    #[serde(default)]
    ambientocclusion: Option<bool>,
    #[serde(default)]
    display: HashMap<String, ItemTransformJson>,
}

#[derive(Debug, Clone)]
pub struct ModelBlock {
    pub textures: HashMap<String, String>,
    pub elements: Vec<BlockPart>,
    pub ambientOcclusion: bool,
    pub gui3d: bool,
    pub transforms: ItemCameraTransforms,
    /// True when the root parent is MCP `ModelBakery.MODEL_GENERATED`
    /// (`builtin/generated`). Such models are expanded by ItemModelGenerator.
    pub generated: bool,
    /// True when the root parent is `builtin/entity`; those item stacks are
    /// rendered by TileEntityItemStackRenderer rather than normal baked quads.
    pub builtInRenderer: bool,
    pub(crate) namespace: String,
}

impl ModelBlock {
    pub fn load(manager: &ResourceManager, model: &ResourceLocation) -> anyhow::Result<Self> {
        let mut visiting = HashSet::new();
        load_recursive(manager, model, &mut visiting)
    }

    pub fn resolveTextureName(&self, name: &str) -> Option<ResourceLocation> {
        let mut current = name;
        let mut seen = HashSet::new();
        while let Some(key) = current.strip_prefix('#') {
            if !seen.insert(key.to_owned()) {
                return None;
            }
            current = self.textures.get(key)?.as_str();
        }
        Some(if current.contains(':') {
            ResourceLocation::parse(current)
        } else {
            ResourceLocation::new(&self.namespace, current)
        })
    }

    pub fn texturePresent(&self, name: &str) -> bool {
        self.resolveTextureName(name)
            .is_some_and(|location| location.getPath() != "missingno")
    }
}

fn default_true() -> bool {
    true
}

fn load_recursive(
    manager: &ResourceManager,
    model: &ResourceLocation,
    visiting: &mut HashSet<ResourceLocation>,
) -> anyhow::Result<ModelBlock> {
    anyhow::ensure!(
        visiting.insert(model.clone()),
        "cyclic model parent: {model}"
    );

    if model.getNamespace() == "minecraft" && model.getPath().starts_with("builtin/") {
        visiting.remove(model);
        return Ok(builtin_model(model));
    }

    let jsonLocation = ResourceLocation::new(
        model.getNamespace(),
        format!("models/{}.json", model.getPath()),
    );
    let resource = manager.get_resource(&jsonLocation)?;
    let json: ModelBlockJson = serde_json::from_slice(&resource.bytes)?;

    let mut result = if let Some(parent) = json.parent.as_deref() {
        let parent = if parent.contains(':') {
            ResourceLocation::parse(parent)
        } else {
            ResourceLocation::new(model.getNamespace(), parent)
        };
        load_recursive(manager, &parent, visiting)?
    } else {
        ModelBlock {
            textures: HashMap::new(),
            elements: Vec::new(),
            ambientOcclusion: true,
            gui3d: true,
            transforms: ItemCameraTransforms::default(),
            generated: false,
            builtInRenderer: false,
            namespace: model.getNamespace().to_owned(),
        }
    };

    result.namespace = model.getNamespace().to_owned();
    result.textures.extend(json.textures);
    if let Some(elements) = json.elements {
        result.elements = elements;
    }
    if let Some(ambientOcclusion) = json.ambientocclusion {
        result.ambientOcclusion = ambientOcclusion;
    }

    let mut childTransforms = ItemCameraTransforms::from_json(json.display);
    childTransforms.inheritMissingFrom(&result.transforms);
    result.transforms = childTransforms;

    visiting.remove(model);
    Ok(result)
}

fn builtin_model(model: &ResourceLocation) -> ModelBlock {
    let generated = model.getPath() == "builtin/generated";
    let builtInRenderer = model.getPath() == "builtin/entity";
    ModelBlock {
        textures: HashMap::new(),
        elements: Vec::new(),
        ambientOcclusion: true,
        gui3d: !generated,
        transforms: ItemCameraTransforms::default(),
        generated,
        builtInRenderer,
        namespace: model.getNamespace().to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::client::renderer::block::model::ItemCameraTransforms::TransformType;

    #[test]
    fn builtin_generated_is_flat_and_uses_item_model_generator() {
        let model = builtin_model(&ResourceLocation::new("minecraft", "builtin/generated"));
        assert!(model.generated);
        assert!(!model.gui3d);
        assert!(!model.builtInRenderer);
    }

    #[test]
    fn child_display_inherits_unspecified_parent_transforms() {
        let mut parent = ItemCameraTransforms::from_json(
            serde_json::from_str(r#"{"gui":{"scale":[.625,.625,.625]}}"#).unwrap(),
        );
        let mut child = ItemCameraTransforms::from_json(
            serde_json::from_str(r#"{"ground":{"translation":[0,3,0]}}"#).unwrap(),
        );
        child.inheritMissingFrom(&parent);
        assert_eq!(child.getTransform(TransformType::Gui).scale, [0.625; 3]);
        assert_eq!(
            child.getTransform(TransformType::Ground).translation,
            [0.0, 0.1875, 0.0]
        );
        parent.inheritMissingFrom(&child);
    }
}
