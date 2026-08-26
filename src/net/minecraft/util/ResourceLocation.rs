use core::fmt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceLocation {
    namespace: String,
    path: String,
}

impl ResourceLocation {
    pub fn parse(resource_name: impl AsRef<str>) -> Self {
        let value = resource_name.as_ref();
        let colon = value.find(':');
        let (namespace, path) = match colon {
            Some(index) => {
                let namespace = if index > 1 {
                    &value[..index]
                } else {
                    "minecraft"
                };
                (namespace, &value[index + 1..])
            }
            None => ("minecraft", value),
        };
        Self::new(namespace, path)
    }

    pub fn new(namespace: impl AsRef<str>, path: impl AsRef<str>) -> Self {
        let namespace = namespace.as_ref();
        Self {
            namespace: if namespace.is_empty() {
                "minecraft".to_owned()
            } else {
                namespace.to_lowercase()
            },
            // MCP 1.12.2 validates non-null, not non-empty.
            path: path.as_ref().to_lowercase(),
        }
    }

    pub fn getNamespace(&self) -> &str {
        &self.namespace
    }
    pub fn getPath(&self) -> &str {
        &self.path
    }
}

impl fmt::Display for ResourceLocation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.namespace, self.path)
    }
}

impl Serialize for ResourceLocation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ResourceLocation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::parse(String::deserialize(deserializer)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_namespace_is_minecraft() {
        let value = ResourceLocation::parse("textures/gui/title/minecraft.png");
        assert_eq!(value.getNamespace(), "minecraft");
        assert_eq!(value.getPath(), "textures/gui/title/minecraft.png");
    }

    #[test]
    fn preserves_mcp_112_colon_behavior() {
        assert_eq!(
            ResourceLocation::parse("modid:block").to_string(),
            "modid:block"
        );
        assert_eq!(
            ResourceLocation::parse(":stone").to_string(),
            "minecraft:stone"
        );
        assert_eq!(
            ResourceLocation::parse("a:stone").to_string(),
            "minecraft:stone"
        );
        assert_eq!(
            ResourceLocation::parse("minecraft:").to_string(),
            "minecraft:"
        );
    }
}
