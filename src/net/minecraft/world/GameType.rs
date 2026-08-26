use crate::net::minecraft::entity::player::PlayerCapabilities::PlayerCapabilities;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameType {
    NotSet,
    Survival,
    Creative,
    Adventure,
    Spectator,
}
impl GameType {
    pub const fn getByID(id: i32) -> Self {
        match id {
            1 => Self::Creative,
            2 => Self::Adventure,
            3 => Self::Spectator,
            -1 => Self::NotSet,
            _ => Self::Survival,
        }
    }
    pub const fn getID(self) -> i32 {
        match self {
            Self::NotSet => -1,
            Self::Survival => 0,
            Self::Creative => 1,
            Self::Adventure => 2,
            Self::Spectator => 3,
        }
    }
    pub const fn isCreative(self) -> bool {
        matches!(self, Self::Creative)
    }
    pub const fn isAdventure(self) -> bool {
        matches!(self, Self::Adventure | Self::Spectator)
    }
    pub const fn isSurvivalOrAdventure(self) -> bool {
        matches!(self, Self::Survival | Self::Adventure)
    }
    pub const fn getName(self) -> &'static str {
        match self {
            Self::NotSet => "not_set",
            Self::Survival => "survival",
            Self::Creative => "creative",
            Self::Adventure => "adventure",
            Self::Spectator => "spectator",
        }
    }
    pub fn getByName(name: &str) -> Self {
        match name {
            "creative" => Self::Creative,
            "adventure" => Self::Adventure,
            "spectator" => Self::Spectator,
            _ => Self::Survival,
        }
    }

    /// MCP `GameType.configurePlayerCapabilities`.
    pub fn configurePlayerCapabilities(self, capabilities: &mut PlayerCapabilities) {
        if self == Self::Creative {
            capabilities.allowFlying = true;
            capabilities.isCreativeMode = true;
            capabilities.disableDamage = true;
        } else if self == Self::Spectator {
            capabilities.allowFlying = true;
            capabilities.isCreativeMode = false;
            capabilities.disableDamage = true;
            capabilities.isFlying = true;
        } else {
            capabilities.allowFlying = false;
            capabilities.isCreativeMode = false;
            capabilities.disableDamage = false;
            capabilities.isFlying = false;
        }
        capabilities.allowEdit = !self.isAdventure();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creative_and_spectator_capabilities_match_mcp() {
        let mut creative = PlayerCapabilities::default();
        GameType::Creative.configurePlayerCapabilities(&mut creative);
        assert!(creative.allowFlying);
        assert!(creative.isCreativeMode);
        assert!(creative.disableDamage);
        assert!(!creative.isFlying);
        assert!(creative.allowEdit);

        let mut spectator = PlayerCapabilities::default();
        GameType::Spectator.configurePlayerCapabilities(&mut spectator);
        assert!(spectator.allowFlying);
        assert!(!spectator.isCreativeMode);
        assert!(spectator.disableDamage);
        assert!(spectator.isFlying);
        assert!(!spectator.allowEdit);
    }
}
