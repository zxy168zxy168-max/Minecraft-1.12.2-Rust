/// Exact MCP 1.12.2 `EnumAction` ordering and names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EnumAction {
    #[default]
    None,
    Eat,
    Drink,
    Block,
    Bow,
}

impl EnumAction {
    pub const fn ordinal(self) -> i32 {
        match self {
            Self::None => 0,
            Self::Eat => 1,
            Self::Drink => 2,
            Self::Block => 3,
            Self::Bow => 4,
        }
    }
}
