use uuid::Uuid;

use crate::com::mojang::authlib::properties::Property::Property;

/// MCP/Authlib-compatible representation of `GameProfile` used by login and
/// `SPacketPlayerListItem`. Property order is retained because vanilla treats
/// the profile property multimap as insertion ordered for serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameProfile {
    id: Option<Uuid>,
    name: String,
    properties: Vec<Property>,
}

impl GameProfile {
    pub fn new(id: Option<Uuid>, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            properties: Vec::new(),
        }
    }

    pub const fn getId(&self) -> Option<Uuid> {
        self.id
    }
    pub fn getName(&self) -> &str {
        &self.name
    }
    pub fn getProperties(&self) -> &[Property] {
        &self.properties
    }
    pub fn addProperty(&mut self, property: Property) {
        self.properties.push(property);
    }
    pub fn isComplete(&self) -> bool {
        self.id.is_some() && !self.name.is_empty()
    }
}
