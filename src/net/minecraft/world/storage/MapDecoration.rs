#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MapDecorationType {
    Player = 0,
    Frame = 1,
    RedMarker = 2,
    BlueMarker = 3,
    TargetX = 4,
    TargetPoint = 5,
    PlayerOffMap = 6,
    PlayerOffLimits = 7,
    Mansion = 8,
    Monument = 9,
}

impl MapDecorationType {
    /// MCP `MapDecoration.Type#func_191159_a`: clamp, rather than wrap, the
    /// packet nibble to the available enum range.
    pub fn fromId(id: u8) -> Self {
        match id.min(9) {
            0 => Self::Player,
            1 => Self::Frame,
            2 => Self::RedMarker,
            3 => Self::BlueMarker,
            4 => Self::TargetX,
            5 => Self::TargetPoint,
            6 => Self::PlayerOffMap,
            7 => Self::PlayerOffLimits,
            8 => Self::Mansion,
            _ => Self::Monument,
        }
    }

    pub const fn id(self) -> u8 {
        self as u8
    }

    /// `func_191160_b`, used when `MapItemRenderer#renderMap` is invoked with
    /// `noOverlayRendering=true` by an item frame.
    pub const fn isRenderedOnFrame(self) -> bool {
        matches!(
            self,
            Self::Frame | Self::TargetX | Self::TargetPoint | Self::Mansion | Self::Monument
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MapDecoration {
    decorationType: MapDecorationType,
    x: i8,
    y: i8,
    rotation: i8,
}

impl MapDecoration {
    pub const fn new(decorationType: MapDecorationType, x: i8, y: i8, rotation: i8) -> Self {
        Self {
            decorationType,
            x,
            y,
            rotation,
        }
    }

    pub const fn getType(&self) -> u8 {
        self.decorationType.id()
    }
    pub const fn decorationType(&self) -> MapDecorationType {
        self.decorationType
    }
    pub const fn getX(&self) -> i8 {
        self.x
    }
    pub const fn getY(&self) -> i8 {
        self.y
    }
    pub const fn getRotation(&self) -> i8 {
        self.rotation
    }
    pub const fn isRenderedOnFrame(&self) -> bool {
        self.decorationType.isRenderedOnFrame()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_overlay_filter_matches_enum_flags() {
        assert!(!MapDecorationType::Player.isRenderedOnFrame());
        assert!(MapDecorationType::Frame.isRenderedOnFrame());
        assert!(MapDecorationType::Mansion.isRenderedOnFrame());
        assert_eq!(MapDecorationType::fromId(15), MapDecorationType::Monument);
    }
}
