use crate::net::minecraft::util::text::ITextComponent::ITextComponent;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Pink,
    Blue,
    Red,
    Green,
    Yellow,
    Purple,
    White,
}
impl Color {
    pub const VALUES: [Self; 7] = [
        Self::Pink,
        Self::Blue,
        Self::Red,
        Self::Green,
        Self::Yellow,
        Self::Purple,
        Self::White,
    ];
    pub const fn ordinal(self) -> i32 {
        match self {
            Self::Pink => 0,
            Self::Blue => 1,
            Self::Red => 2,
            Self::Green => 3,
            Self::Yellow => 4,
            Self::Purple => 5,
            Self::White => 6,
        }
    }
    pub fn byOrdinal(value: i32) -> Option<Self> {
        Self::VALUES.get(value as usize).copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    Progress,
    Notched6,
    Notched10,
    Notched12,
    Notched20,
}
impl Overlay {
    pub const VALUES: [Self; 5] = [
        Self::Progress,
        Self::Notched6,
        Self::Notched10,
        Self::Notched12,
        Self::Notched20,
    ];
    pub const fn ordinal(self) -> i32 {
        match self {
            Self::Progress => 0,
            Self::Notched6 => 1,
            Self::Notched10 => 2,
            Self::Notched12 => 3,
            Self::Notched20 => 4,
        }
    }
    pub fn byOrdinal(value: i32) -> Option<Self> {
        Self::VALUES.get(value as usize).copied()
    }
}

/// MCP 1.12.2 `BossInfo` state, independent of the Vulkan backend.
#[derive(Debug, Clone, PartialEq)]
pub struct BossInfo {
    uniqueId: Uuid,
    name: ITextComponent,
    percent: f32,
    color: Color,
    overlay: Overlay,
    darkenSky: bool,
    playEndBossMusic: bool,
    createFog: bool,
}

impl BossInfo {
    pub fn new(
        uniqueIdIn: Uuid,
        nameIn: ITextComponent,
        colorIn: Color,
        overlayIn: Overlay,
    ) -> Self {
        Self {
            uniqueId: uniqueIdIn,
            name: nameIn,
            percent: 1.0,
            color: colorIn,
            overlay: overlayIn,
            darkenSky: false,
            playEndBossMusic: false,
            createFog: false,
        }
    }
    pub const fn getUniqueId(&self) -> Uuid {
        self.uniqueId
    }
    pub fn getName(&self) -> &ITextComponent {
        &self.name
    }
    pub fn setName(&mut self, nameIn: ITextComponent) {
        self.name = nameIn;
    }
    pub const fn getPercent(&self) -> f32 {
        self.percent
    }
    pub fn setPercent(&mut self, percentIn: f32) {
        self.percent = percentIn;
    }
    pub const fn getColor(&self) -> Color {
        self.color
    }
    pub fn setColor(&mut self, colorIn: Color) {
        self.color = colorIn;
    }
    pub const fn getOverlay(&self) -> Overlay {
        self.overlay
    }
    pub fn setOverlay(&mut self, overlayIn: Overlay) {
        self.overlay = overlayIn;
    }
    pub const fn shouldDarkenSky(&self) -> bool {
        self.darkenSky
    }
    pub fn setDarkenSky(&mut self, value: bool) {
        self.darkenSky = value;
    }
    pub const fn shouldPlayEndBossMusic(&self) -> bool {
        self.playEndBossMusic
    }
    pub fn setPlayEndBossMusic(&mut self, value: bool) {
        self.playEndBossMusic = value;
    }
    pub const fn shouldCreateFog(&self) -> bool {
        self.createFog
    }
    pub fn setCreateFog(&mut self, value: bool) {
        self.createFog = value;
    }
}
