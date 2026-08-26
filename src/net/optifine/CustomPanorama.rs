use std::collections::HashMap;

use crate::compat::Java::JavaRandom;
use crate::compat::JavaProperties::parse_java_properties;
use crate::net::minecraft::client::resources::SimpleReloadableResourceManager::ResourceManager;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use thiserror::Error;

pub const VANILLA_PANORAMA_PATH: &str = "textures/gui/title/background";

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CustomPanoramaError {
    #[error("OptiFine custom panorama weights sum to {0}; java.util.Random.nextInt requires a positive bound")]
    InvalidWeightSum(i32),
}

use crate::net::optifine::CustomPanoramaProperties::CustomPanoramaProperties;

/// OptiFine C6 custom-panorama selection, preserving folder discovery,
/// property fallback, weights, and Java RNG behavior.
pub fn select_custom_panorama(
    resources: &ResourceManager,
    random: &mut JavaRandom,
) -> Result<Option<CustomPanoramaProperties>, CustomPanoramaError> {
    let folders = panorama_folders(resources);
    if folders.len() <= 1 {
        return Ok(None);
    }

    let properties = folders
        .iter()
        .enumerate()
        .map(|(index, folder)| {
            let property_folder = if index == 0 {
                "optifine/gui"
            } else {
                folder.as_str()
            };
            read_properties(
                resources,
                &ResourceLocation::parse(format!("{property_folder}/background.properties")),
            )
        })
        .collect::<Vec<_>>();

    let weights = properties
        .iter()
        .map(|candidate| {
            let effective = candidate
                .as_ref()
                .or(properties.first().and_then(Option::as_ref));
            effective
                .map(|values| parse_i32(values.get("weight"), 1))
                .unwrap_or(1)
        })
        .collect::<Vec<_>>();
    let total = weights
        .iter()
        .fold(0_i32, |sum, value| sum.wrapping_add(*value));
    if total <= 0 {
        return Err(CustomPanoramaError::InvalidWeightSum(total));
    }

    let target = random.next_i32_bound(total);
    let mut cumulative = 0_i32;
    let selected_index = weights
        .iter()
        .position(|weight| {
            cumulative = cumulative.wrapping_add(*weight);
            cumulative > target
        })
        .unwrap_or(weights.len() - 1);

    let selected_properties = properties[selected_index]
        .as_ref()
        .or(properties[0].as_ref())
        .cloned()
        .unwrap_or_default();
    Ok(Some(CustomPanoramaProperties::new(
        folders[selected_index].clone(),
        &selected_properties,
    )))
}

fn panorama_folders(resources: &ResourceManager) -> Vec<String> {
    let mut folders = vec![VANILLA_PANORAMA_PATH.to_owned()];
    for index in 0..100 {
        let path = format!("optifine/gui/background{index}");
        if resources.resource_exists(&ResourceLocation::parse(format!("{path}/panorama_0.png"))) {
            folders.push(path);
        }
    }
    folders
}

fn read_properties(
    resources: &ResourceManager,
    location: &ResourceLocation,
) -> Option<HashMap<String, String>> {
    let resource = resources.get_resource(location).ok()?;
    Some(parse_properties(&resource.bytes))
}

fn parse_properties(bytes: &[u8]) -> HashMap<String, String> {
    parse_java_properties(bytes)
}

fn parse_i32(value: Option<&String>, default: i32) -> i32 {
    value
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(default)
}
