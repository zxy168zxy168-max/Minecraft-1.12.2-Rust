/// MCP 1.12.2 `DimensionType`.
///
/// Java stores the provider class object in each enum value. Rust keeps the
/// same three vanilla identities and lets `WorldProvider::forDimensionType`
/// perform the equivalent constructor dispatch without reflection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DimensionType {
    Overworld,
    Nether,
    TheEnd,
}

impl DimensionType {
    pub const VALUES: [DimensionType; 3] = [DimensionType::Overworld, DimensionType::Nether, DimensionType::TheEnd];

    pub const fn getId(self) -> i32 {
        match self { Self::Overworld => 0, Self::Nether => -1, Self::TheEnd => 1 }
    }
    pub const fn getName(self) -> &'static str {
        match self { Self::Overworld => "overworld", Self::Nether => "the_nether", Self::TheEnd => "the_end" }
    }
    pub const fn getSuffix(self) -> &'static str {
        match self { Self::Overworld => "", Self::Nether => "_nether", Self::TheEnd => "_end" }
    }
    pub fn getById(id: i32) -> Result<Self, String> {
        Self::VALUES.into_iter().find(|value| value.getId() == id)
            .ok_or_else(|| format!("Invalid dimension id {id}"))
    }
    pub fn func_193417_a(name: &str) -> Result<Self, String> {
        Self::VALUES.into_iter().find(|value| value.getName() == name)
            .ok_or_else(|| format!("Invalid dimension {name}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vanilla_dimension_ids_names_and_suffixes_match_mcp() {
        assert_eq!(DimensionType::Overworld.getId(), 0);
        assert_eq!(DimensionType::Nether.getName(), "the_nether");
        assert_eq!(DimensionType::TheEnd.getSuffix(), "_end");
        assert_eq!(DimensionType::getById(-1).unwrap(), DimensionType::Nether);
        assert!(DimensionType::getById(2).is_err());
    }
}
