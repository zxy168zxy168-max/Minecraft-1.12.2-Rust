use crate::net::minecraft::scoreboard::IScoreCriteria::EnumRenderType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreObjective {
    name: String,
    displayName: String,
    renderType: EnumRenderType,
}

impl ScoreObjective {
    pub fn new(
        name: impl Into<String>,
        displayName: impl Into<String>,
        renderType: EnumRenderType,
    ) -> Self {
        Self {
            name: name.into(),
            displayName: displayName.into(),
            renderType,
        }
    }
    pub fn getName(&self) -> &str {
        &self.name
    }
    pub fn getDisplayName(&self) -> &str {
        &self.displayName
    }
    pub const fn getRenderType(&self) -> EnumRenderType {
        self.renderType
    }
    pub fn setDisplayName(&mut self, value: impl Into<String>) {
        self.displayName = value.into();
    }
    pub fn setRenderType(&mut self, value: EnumRenderType) {
        self.renderType = value;
    }
}
