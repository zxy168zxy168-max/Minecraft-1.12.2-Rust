use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModelResourceLocation {
    location: ResourceLocation,
    variant: String,
}
impl ModelResourceLocation {
    pub fn new(location: impl AsRef<str>, variant: impl Into<String>) -> Self {
        Self {
            location: ResourceLocation::parse(location),
            variant: variant.into(),
        }
    }
    pub fn fromLocation(location: ResourceLocation, variant: impl Into<String>) -> Self {
        Self {
            location,
            variant: variant.into(),
        }
    }
    pub fn getNamespace(&self) -> &str {
        self.location.getNamespace()
    }
    pub fn getPath(&self) -> &str {
        self.location.getPath()
    }
    pub fn getVariant(&self) -> &str {
        &self.variant
    }
    pub fn getLocation(&self) -> &ResourceLocation {
        &self.location
    }
}
