use crate::net::minecraft::nbt::NBTBase::{TAG_BYTE, TAG_COMPOUND};
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;

/// Rust port of MCP `net.minecraft.entity.player.PlayerCapabilities`.
///
/// These fields are server-authoritative. `GameType` configures the defaults
/// while `SPacketPlayerAbilities` may replace them at any time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerCapabilities {
    pub disableDamage: bool,
    pub isFlying: bool,
    pub allowFlying: bool,
    pub isCreativeMode: bool,
    pub allowEdit: bool,
    flySpeed: f32,
    walkSpeed: f32,
}

impl Default for PlayerCapabilities {
    fn default() -> Self {
        Self {
            disableDamage: false,
            isFlying: false,
            allowFlying: false,
            isCreativeMode: false,
            allowEdit: true,
            flySpeed: 0.05,
            walkSpeed: 0.1,
        }
    }
}

impl PlayerCapabilities {
    /// MCP `PlayerCapabilities.writeCapabilitiesToNBT`.
    pub fn writeCapabilitiesToNBT(&self, tagCompound: &mut NBTTagCompound) {
        let mut abilities = NBTTagCompound::new();
        abilities.setBoolean("invulnerable", self.disableDamage);
        abilities.setBoolean("flying", self.isFlying);
        abilities.setBoolean("mayfly", self.allowFlying);
        abilities.setBoolean("instabuild", self.isCreativeMode);
        abilities.setBoolean("mayBuild", self.allowEdit);
        abilities.setFloat("flySpeed", self.flySpeed);
        abilities.setFloat("walkSpeed", self.walkSpeed);
        tagCompound.setCompoundTag("abilities", abilities);
    }

    /// MCP `PlayerCapabilities.readCapabilitiesFromNBT` including the legacy
    /// absence rules for `flySpeed`, `walkSpeed` and `mayBuild`.
    pub fn readCapabilitiesFromNBT(&mut self, tagCompound: &NBTTagCompound) {
        if !tagCompound.hasKeyWithType("abilities", TAG_COMPOUND) {
            return;
        }
        let abilities = tagCompound.getCompoundTag("abilities");
        self.disableDamage = abilities.getBoolean("invulnerable");
        self.isFlying = abilities.getBoolean("flying");
        self.allowFlying = abilities.getBoolean("mayfly");
        self.isCreativeMode = abilities.getBoolean("instabuild");

        if abilities.hasKeyWithType("flySpeed", 99) {
            self.flySpeed = abilities.getFloat("flySpeed");
            self.walkSpeed = abilities.getFloat("walkSpeed");
        }
        if abilities.hasKeyWithType("mayBuild", TAG_BYTE) {
            self.allowEdit = abilities.getBoolean("mayBuild");
        }
    }

    pub const fn getFlySpeed(&self) -> f32 {
        self.flySpeed
    }
    pub fn setFlySpeed(&mut self, speed: f32) {
        self.flySpeed = speed;
    }
    pub const fn getWalkSpeed(&self) -> f32 {
        self.walkSpeed
    }
    pub fn setPlayerWalkSpeed(&mut self, speed: f32) {
        self.walkSpeed = speed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_defaults_match_mcp() {
        let capabilities = PlayerCapabilities::default();
        assert!(!capabilities.disableDamage);
        assert!(!capabilities.isFlying);
        assert!(!capabilities.allowFlying);
        assert!(!capabilities.isCreativeMode);
        assert!(capabilities.allowEdit);
        assert!((capabilities.getFlySpeed() - 0.05).abs() < f32::EPSILON);
        assert!((capabilities.getWalkSpeed() - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn nbt_round_trip_matches_mcp_ability_keys() {
        let mut original = PlayerCapabilities::default();
        original.disableDamage = true;
        original.isFlying = true;
        original.allowFlying = true;
        original.isCreativeMode = true;
        original.allowEdit = false;
        original.setFlySpeed(0.125);
        original.setPlayerWalkSpeed(0.2);

        let mut root = NBTTagCompound::new();
        original.writeCapabilitiesToNBT(&mut root);
        let abilities = root.getCompoundTag("abilities");
        assert!(abilities.getBoolean("invulnerable"));
        assert!(abilities.getBoolean("flying"));
        assert!(abilities.getBoolean("mayfly"));
        assert!(abilities.getBoolean("instabuild"));
        assert!(!abilities.getBoolean("mayBuild"));
        assert_eq!(abilities.getFloat("flySpeed"), 0.125);
        assert_eq!(abilities.getFloat("walkSpeed"), 0.2);

        let mut decoded = PlayerCapabilities::default();
        decoded.readCapabilitiesFromNBT(&root);
        assert_eq!(decoded, original);
    }
}
