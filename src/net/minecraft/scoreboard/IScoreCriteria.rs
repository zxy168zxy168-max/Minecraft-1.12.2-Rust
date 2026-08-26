#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnumRenderType {
    #[default]
    Integer,
    Hearts,
}

impl EnumRenderType {
    pub fn getByName(name: &str) -> Self {
        if name.eq_ignore_ascii_case("hearts") {
            Self::Hearts
        } else {
            Self::Integer
        }
    }

    pub const fn getRenderType(self) -> &'static str {
        match self {
            Self::Integer => "integer",
            Self::Hearts => "hearts",
        }
    }
}
