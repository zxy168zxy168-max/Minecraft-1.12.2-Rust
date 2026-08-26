use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemTransformVec3f {
    pub rotation: [f32; 3],
    pub translation: [f32; 3],
    pub scale: [f32; 3],
}

impl Default for ItemTransformVec3f {
    fn default() -> Self {
        Self {
            rotation: [0.0; 3],
            translation: [0.0; 3],
            scale: [1.0; 3],
        }
    }
}

impl ItemTransformVec3f {
    fn from_json(value: ItemTransformJson) -> Self {
        let mut translation = value.translation.unwrap_or([0.0; 3]);
        for component in &mut translation {
            *component = (*component * 0.0625).clamp(-5.0, 5.0);
        }
        let mut scale = value.scale.unwrap_or([1.0; 3]);
        for component in &mut scale {
            *component = component.clamp(-4.0, 4.0);
        }
        Self {
            rotation: value.rotation.unwrap_or([0.0; 3]),
            translation,
            scale,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransformType {
    ThirdPersonLeftHand,
    ThirdPersonRightHand,
    FirstPersonLeftHand,
    FirstPersonRightHand,
    Head,
    Gui,
    Ground,
    Fixed,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ItemCameraTransforms {
    transforms: HashMap<TransformType, ItemTransformVec3f>,
}

impl ItemCameraTransforms {
    pub fn getTransform(&self, transformType: TransformType) -> ItemTransformVec3f {
        self.transforms
            .get(&transformType)
            .copied()
            .unwrap_or_default()
    }

    pub fn hasCustomTransform(&self, transformType: TransformType) -> bool {
        self.transforms.contains_key(&transformType)
    }

    pub fn inheritMissingFrom(&mut self, parent: &ItemCameraTransforms) {
        for transformType in [
            TransformType::ThirdPersonLeftHand,
            TransformType::ThirdPersonRightHand,
            TransformType::FirstPersonLeftHand,
            TransformType::FirstPersonRightHand,
            TransformType::Head,
            TransformType::Gui,
            TransformType::Ground,
            TransformType::Fixed,
        ] {
            if !self.transforms.contains_key(&transformType) {
                if let Some(value) = parent.transforms.get(&transformType) {
                    self.transforms.insert(transformType, *value);
                }
            }
        }
    }

    pub(crate) fn from_json(display: HashMap<String, ItemTransformJson>) -> Self {
        let mut transforms = HashMap::new();
        let mut insert = |name: &str, transformType: TransformType| {
            if let Some(value) = display.get(name).cloned() {
                transforms.insert(transformType, ItemTransformVec3f::from_json(value));
            }
        };
        insert("thirdperson_righthand", TransformType::ThirdPersonRightHand);
        insert("thirdperson_lefthand", TransformType::ThirdPersonLeftHand);
        insert("firstperson_righthand", TransformType::FirstPersonRightHand);
        insert("firstperson_lefthand", TransformType::FirstPersonLeftHand);
        insert("head", TransformType::Head);
        insert("gui", TransformType::Gui);
        insert("ground", TransformType::Ground);
        insert("fixed", TransformType::Fixed);

        // MCP ItemCameraTransforms.Deserializer copies the right-hand transform
        // when a corresponding left-hand transform is absent.
        if !transforms.contains_key(&TransformType::ThirdPersonLeftHand) {
            if let Some(value) = transforms
                .get(&TransformType::ThirdPersonRightHand)
                .copied()
            {
                transforms.insert(TransformType::ThirdPersonLeftHand, value);
            }
        }
        if !transforms.contains_key(&TransformType::FirstPersonLeftHand) {
            if let Some(value) = transforms
                .get(&TransformType::FirstPersonRightHand)
                .copied()
            {
                transforms.insert(TransformType::FirstPersonLeftHand, value);
            }
        }
        Self { transforms }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ItemTransformJson {
    #[serde(default)]
    rotation: Option<[f32; 3]>,
    #[serde(default)]
    translation: Option<[f32; 3]>,
    #[serde(default)]
    scale: Option<[f32; 3]>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_translation_is_divided_by_sixteen_and_clamped() {
        let json: HashMap<String, ItemTransformJson> = serde_json::from_str(
            r#"{"gui":{"rotation":[30,225,0],"translation":[0,16,160],"scale":[.625,.625,.625]}}"#,
        )
        .unwrap();
        let transforms = ItemCameraTransforms::from_json(json);
        let gui = transforms.getTransform(TransformType::Gui);
        assert_eq!(gui.rotation, [30.0, 225.0, 0.0]);
        assert_eq!(gui.translation, [0.0, 1.0, 5.0]);
        assert_eq!(gui.scale, [0.625; 3]);
    }
}
