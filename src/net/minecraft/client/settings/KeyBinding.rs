use serde::{Deserialize, Serialize};

/// Rust representation of MCP 1.12.2 `KeyBinding`.
///
/// `keyCode` deliberately keeps the LWJGL 2 integer namespace used by
/// vanilla `options.txt`: positive values are keyboard codes, zero means
/// unbound and negative values encode mouse buttons (`-100 + button`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeyBinding {
    pub keyDescription: String,
    pub keyCodeDefault: i32,
    pub keyCategory: String,
    pub keyCode: i32,
    #[serde(skip)]
    pub pressed: bool,
    #[serde(skip)]
    pub pressTime: i32,
}

impl KeyBinding {
    pub fn new(description: impl Into<String>, keyCode: i32, category: impl Into<String>) -> Self {
        Self {
            keyDescription: description.into(),
            keyCodeDefault: keyCode,
            keyCategory: category.into(),
            keyCode,
            pressed: false,
            pressTime: 0,
        }
    }

    pub fn setKeyCode(&mut self, keyCode: i32) {
        self.keyCode = keyCode;
    }
    pub fn isKeyDown(&self) -> bool {
        self.pressed
    }

    /// MCP `KeyBinding#isPressed`: consume exactly one queued press.
    pub fn isPressed(&mut self) -> bool {
        if self.pressTime == 0 {
            return false;
        }
        self.pressTime -= 1;
        true
    }

    pub fn setPressed(&mut self, pressed: bool) {
        self.pressed = pressed;
    }
    pub fn onTick(&mut self) {
        if self.keyCode != 0 {
            self.pressTime = self.pressTime.saturating_add(1);
        }
    }
    pub fn unpressKey(&mut self) {
        self.pressTime = 0;
        self.pressed = false;
    }
    pub fn isDefault(&self) -> bool {
        self.keyCode == self.keyCodeDefault
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyBindingId {
    Attack,
    UseItem,
    Forward,
    Left,
    Back,
    Right,
    Jump,
    Sneak,
    Sprint,
    Drop,
    Inventory,
    Chat,
    PlayerList,
    PickBlock,
    Command,
    Screenshot,
    TogglePerspective,
    SmoothCamera,
    Fullscreen,
    SpectatorOutlines,
    SwapHands,
    SaveToolbar,
    LoadToolbar,
    Advancements,
    Hotbar1,
    Hotbar2,
    Hotbar3,
    Hotbar4,
    Hotbar5,
    Hotbar6,
    Hotbar7,
    Hotbar8,
    Hotbar9,
    OptifineZoom,
}

impl KeyBindingId {
    pub const ALL: [Self; 34] = [
        Self::Attack,
        Self::UseItem,
        Self::Forward,
        Self::Left,
        Self::Back,
        Self::Right,
        Self::Jump,
        Self::Sneak,
        Self::Sprint,
        Self::Drop,
        Self::Inventory,
        Self::Chat,
        Self::PlayerList,
        Self::PickBlock,
        Self::Command,
        Self::Screenshot,
        Self::TogglePerspective,
        Self::SmoothCamera,
        Self::Fullscreen,
        Self::SpectatorOutlines,
        Self::SwapHands,
        Self::SaveToolbar,
        Self::LoadToolbar,
        Self::Advancements,
        Self::Hotbar1,
        Self::Hotbar2,
        Self::Hotbar3,
        Self::Hotbar4,
        Self::Hotbar5,
        Self::Hotbar6,
        Self::Hotbar7,
        Self::Hotbar8,
        Self::Hotbar9,
        Self::OptifineZoom,
    ];

    pub const fn index(self) -> usize {
        match self {
            Self::Attack => 0,
            Self::UseItem => 1,
            Self::Forward => 2,
            Self::Left => 3,
            Self::Back => 4,
            Self::Right => 5,
            Self::Jump => 6,
            Self::Sneak => 7,
            Self::Sprint => 8,
            Self::Drop => 9,
            Self::Inventory => 10,
            Self::Chat => 11,
            Self::PlayerList => 12,
            Self::PickBlock => 13,
            Self::Command => 14,
            Self::Screenshot => 15,
            Self::TogglePerspective => 16,
            Self::SmoothCamera => 17,
            Self::Fullscreen => 18,
            Self::SpectatorOutlines => 19,
            Self::SwapHands => 20,
            Self::SaveToolbar => 21,
            Self::LoadToolbar => 22,
            Self::Advancements => 23,
            Self::Hotbar1 => 24,
            Self::Hotbar2 => 25,
            Self::Hotbar3 => 26,
            Self::Hotbar4 => 27,
            Self::Hotbar5 => 28,
            Self::Hotbar6 => 29,
            Self::Hotbar7 => 30,
            Self::Hotbar8 => 31,
            Self::Hotbar9 => 32,
            Self::OptifineZoom => 33,
        }
    }
}

pub fn vanilla_key_bindings() -> Vec<KeyBinding> {
    use KeyBindingId::*;
    let mut bindings = Vec::with_capacity(KeyBindingId::ALL.len());
    let mut push = |id: KeyBindingId, desc: &str, code: i32, category: &str| {
        debug_assert_eq!(bindings.len(), id.index());
        bindings.push(KeyBinding::new(desc, code, category));
    };
    push(Attack, "key.attack", -100, "key.categories.gameplay");
    push(UseItem, "key.use", -99, "key.categories.gameplay");
    push(Forward, "key.forward", 17, "key.categories.movement");
    push(Left, "key.left", 30, "key.categories.movement");
    push(Back, "key.back", 31, "key.categories.movement");
    push(Right, "key.right", 32, "key.categories.movement");
    push(Jump, "key.jump", 57, "key.categories.movement");
    push(Sneak, "key.sneak", 42, "key.categories.movement");
    push(Sprint, "key.sprint", 29, "key.categories.movement");
    push(Drop, "key.drop", 16, "key.categories.inventory");
    push(Inventory, "key.inventory", 18, "key.categories.inventory");
    push(Chat, "key.chat", 20, "key.categories.multiplayer");
    push(
        PlayerList,
        "key.playerlist",
        15,
        "key.categories.multiplayer",
    );
    push(PickBlock, "key.pickItem", -98, "key.categories.gameplay");
    push(Command, "key.command", 53, "key.categories.multiplayer");
    push(Screenshot, "key.screenshot", 60, "key.categories.misc");
    push(
        TogglePerspective,
        "key.togglePerspective",
        63,
        "key.categories.misc",
    );
    push(SmoothCamera, "key.smoothCamera", 0, "key.categories.misc");
    push(Fullscreen, "key.fullscreen", 87, "key.categories.misc");
    push(
        SpectatorOutlines,
        "key.spectatorOutlines",
        0,
        "key.categories.misc",
    );
    push(SwapHands, "key.swapHands", 33, "key.categories.inventory");
    push(
        SaveToolbar,
        "key.saveToolbarActivator",
        46,
        "key.categories.creative",
    );
    push(
        LoadToolbar,
        "key.loadToolbarActivator",
        45,
        "key.categories.creative",
    );
    push(Advancements, "key.advancements", 38, "key.categories.misc");
    push(Hotbar1, "key.hotbar.1", 2, "key.categories.inventory");
    push(Hotbar2, "key.hotbar.2", 3, "key.categories.inventory");
    push(Hotbar3, "key.hotbar.3", 4, "key.categories.inventory");
    push(Hotbar4, "key.hotbar.4", 5, "key.categories.inventory");
    push(Hotbar5, "key.hotbar.5", 6, "key.categories.inventory");
    push(Hotbar6, "key.hotbar.6", 7, "key.categories.inventory");
    push(Hotbar7, "key.hotbar.7", 8, "key.categories.inventory");
    push(Hotbar8, "key.hotbar.8", 9, "key.categories.inventory");
    push(Hotbar9, "key.hotbar.9", 10, "key.categories.inventory");
    // OptiFine 1.12.2 C6 appends this binding after vanilla construction.
    push(OptifineZoom, "of.key.zoom", 46, "key.categories.misc");
    bindings
}

pub const CATEGORY_ORDER: [&str; 7] = [
    "key.categories.movement",
    "key.categories.gameplay",
    "key.categories.inventory",
    "key.categories.creative",
    "key.categories.multiplayer",
    "key.categories.ui",
    "key.categories.misc",
];

pub fn category_order(category: &str) -> usize {
    CATEGORY_ORDER
        .iter()
        .position(|candidate| *candidate == category)
        .unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_mcp_and_optifine_c6() {
        let bindings = vanilla_key_bindings();
        assert_eq!(bindings.len(), 34);
        assert_eq!(bindings[KeyBindingId::Forward.index()].keyCode, 17);
        assert_eq!(bindings[KeyBindingId::Attack.index()].keyCode, -100);
        assert_eq!(
            bindings[KeyBindingId::TogglePerspective.index()].keyCode,
            63
        );
        assert_eq!(bindings[KeyBindingId::Hotbar9.index()].keyCode, 10);
        assert_eq!(
            bindings[KeyBindingId::OptifineZoom.index()].keyDescription,
            "of.key.zoom"
        );
    }
}
