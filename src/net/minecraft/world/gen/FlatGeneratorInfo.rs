use std::collections::HashMap;

use crate::net::minecraft::block::Block::Block;
use crate::net::minecraft::world::gen::FlatLayerInfo::FlatLayerInfo;

/// MCP 1.12.2 `FlatGeneratorInfo` parser and default preset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatGeneratorInfo {
    flatLayers: Vec<FlatLayerInfo>,
    worldFeatures: HashMap<String, HashMap<String, String>>,
    biomeToUse: i32,
}

impl FlatGeneratorInfo {
    pub fn new() -> Self {
        Self {
            flatLayers: Vec::new(),
            worldFeatures: HashMap::new(),
            biomeToUse: 0,
        }
    }

    pub const fn getBiome(&self) -> i32 {
        self.biomeToUse
    }
    pub fn setBiome(&mut self, biome: i32) {
        self.biomeToUse = biome;
    }
    pub fn getWorldFeatures(&self) -> &HashMap<String, HashMap<String, String>> {
        &self.worldFeatures
    }
    pub fn getWorldFeaturesMut(&mut self) -> &mut HashMap<String, HashMap<String, String>> {
        &mut self.worldFeatures
    }
    pub fn getFlatLayers(&self) -> &[FlatLayerInfo] {
        &self.flatLayers
    }
    pub fn getFlatLayersMut(&mut self) -> &mut Vec<FlatLayerInfo> {
        &mut self.flatLayers
    }

    pub fn updateLayers(&mut self) {
        let mut y = 0;
        for layer in &mut self.flatLayers {
            layer.setMinY(y);
            y += layer.getLayerCount();
        }
    }

    fn getLayerFromString(version: i32, text: &str, minY: i32) -> Option<FlatLayerInfo> {
        let separator = if version >= 3 { '*' } else { 'x' };
        let mut split = text.splitn(2, separator);
        let first = split.next()?;
        let second = split.next();
        let (mut count, blockText) = if let Some(blockText) = second {
            let mut count = first.parse::<i32>().ok()?;
            if minY.wrapping_add(count) >= 256 {
                count = 256 - minY;
            }
            if count < 0 {
                count = 0;
            }
            (count, blockText)
        } else {
            (1, first)
        };
        if minY >= 256 {
            count = 0;
        }

        let (block, mut metadata) = if version < 3 {
            let mut parts = blockText.splitn(2, ':');
            let id = parts.next()?.parse::<i32>().ok()?;
            let metadata = parts
                .next()
                .and_then(|value| value.parse::<i32>().ok())
                .unwrap_or(0);
            (Some(Block::getBlockById(id)), metadata)
        } else {
            let parts = blockText.split(':').collect::<Vec<_>>();
            let mut block = None;
            let mut metadata = 0;
            if parts.len() > 1 {
                block = Block::getBlockFromName(&format!("{}:{}", parts[0], parts[1]));
                if block.is_some() && parts.len() > 2 {
                    metadata = parts[2].parse::<i32>().ok()?;
                }
            }
            if block.is_none() {
                block = Block::getBlockFromName(parts.first().copied().unwrap_or_default());
                if block.is_some() && parts.len() > 1 {
                    metadata = parts[1].parse::<i32>().ok()?;
                }
            }
            (block, metadata)
        };
        let block = block?;
        if block.isAir() {
            metadata = 0;
        }
        if !(0..=15).contains(&metadata) {
            metadata = 0;
        }
        let mut layer = FlatLayerInfo::newVersioned(version, count, block, metadata);
        layer.setMinY(minY);
        Some(layer)
    }

    fn getLayersFromString(version: i32, text: &str) -> Option<Vec<FlatLayerInfo>> {
        if text.is_empty() {
            return None;
        }
        let mut layers = Vec::new();
        let mut y = 0;
        for element in text.split(',') {
            let layer = Self::getLayerFromString(version, element, y)?;
            y += layer.getLayerCount();
            layers.push(layer);
        }
        Some(layers)
    }

    pub fn createFlatGeneratorFromString(settings: &str) -> Self {
        let fields = settings.split(';').collect::<Vec<_>>();
        let version = if fields.len() == 1 {
            0
        } else {
            fields[0].parse::<i32>().unwrap_or(0)
        };
        if !(0..=3).contains(&version) {
            return Self::getDefaultFlatGenerator();
        }
        let mut field = if fields.len() == 1 { 0 } else { 1 };
        let Some(layerText) = fields.get(field) else {
            return Self::getDefaultFlatGenerator();
        };
        field += 1;
        let Some(layers) = Self::getLayersFromString(version, layerText) else {
            return Self::getDefaultFlatGenerator();
        };
        if layers.is_empty() {
            return Self::getDefaultFlatGenerator();
        }

        let mut result = Self::new();
        result.flatLayers.extend(layers);
        result.updateLayers();
        let mut biome = 1; // Biomes.PLAINS
        if version > 0 {
            if let Some(value) = fields.get(field) {
                biome = value.parse::<i32>().unwrap_or(biome);
                field += 1;
            }
        }
        result.setBiome(biome);

        if version > 0 && field < fields.len() {
            let features = fields[field].to_lowercase();
            for feature in features.split(',') {
                let mut split = feature.splitn(2, '(');
                let name = split.next().unwrap_or_default();
                if name.is_empty() {
                    continue;
                }
                let mut options = HashMap::new();
                if let Some(rest) = split.next() {
                    if rest.ends_with(')') && rest.len() > 1 {
                        for option in rest[..rest.len() - 1].split(' ') {
                            let mut pair = option.splitn(2, '=');
                            if let (Some(key), Some(value)) = (pair.next(), pair.next()) {
                                options.insert(key.to_owned(), value.to_owned());
                            }
                        }
                    }
                }
                result.worldFeatures.insert(name.to_owned(), options);
            }
        } else {
            result
                .worldFeatures
                .insert("village".to_owned(), HashMap::new());
        }
        result
    }

    pub fn getDefaultFlatGenerator() -> Self {
        let mut result = Self::new();
        result.setBiome(1);
        result
            .flatLayers
            .push(FlatLayerInfo::new(1, Block::getBlockById(7))); // bedrock
        result
            .flatLayers
            .push(FlatLayerInfo::new(2, Block::getBlockById(3))); // dirt
        result
            .flatLayers
            .push(FlatLayerInfo::new(1, Block::getBlockById(2))); // grass
        result.updateLayers();
        result
            .worldFeatures
            .insert("village".to_owned(), HashMap::new());
        result
    }

    pub fn toGeneratorString(&self) -> String {
        let layers = self
            .flatLayers
            .iter()
            .map(FlatLayerInfo::toGeneratorString)
            .collect::<Vec<_>>()
            .join(",");
        let mut features = self
            .worldFeatures
            .iter()
            .map(|(name, values)| {
                if values.is_empty() {
                    name.to_lowercase()
                } else {
                    let args = values
                        .iter()
                        .map(|(k, v)| format!("{k}={v}"))
                        .collect::<Vec<_>>()
                        .join(" ");
                    format!("{}({})", name.to_lowercase(), args)
                }
            })
            .collect::<Vec<_>>();
        features.sort();
        format!("3;{};{};{}", layers, self.biomeToUse, features.join(","))
    }
}

impl Default for FlatGeneratorInfo {
    fn default() -> Self {
        Self::getDefaultFlatGenerator()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_settings_fall_back_to_the_vanilla_default_flat_preset() {
        let info = FlatGeneratorInfo::createFlatGeneratorFromString("");
        assert_eq!(info.getBiome(), 1);
        assert_eq!(info.getFlatLayers().len(), 3);
        assert_eq!(info.getFlatLayers()[0].getLayerMaterial().getBlockId(), 7);
        assert_eq!(info.getFlatLayers()[1].getLayerCount(), 2);
        assert!(info.getWorldFeatures().contains_key("village"));
    }

    #[test]
    fn version_three_registry_names_and_features_parse_like_mcp() {
        let info = FlatGeneratorInfo::createFlatGeneratorFromString(
            "3;minecraft:bedrock,2*minecraft:dirt,minecraft:grass;1;village(size=1),lake",
        );
        assert_eq!(info.getFlatLayers()[2].getMinY(), 3);
        assert_eq!(info.getFlatLayers()[2].getLayerMaterial().getBlockId(), 2);
        assert_eq!(info.getWorldFeatures()["village"]["size"], "1");
        assert!(info.getWorldFeatures().contains_key("lake"));
    }
}
