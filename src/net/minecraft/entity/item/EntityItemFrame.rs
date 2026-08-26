use crate::net::minecraft::item::ItemStack::ItemStack;

/// Client-visible constants and metadata access semantics from MCP 1.12.2
/// `EntityItemFrame`.
pub struct EntityItemFrame;

impl EntityItemFrame {
    pub const WIDTH_PIXELS: i32 = 12;
    pub const HEIGHT_PIXELS: i32 = 12;
    pub const ITEM_DATA_INDEX: u8 = 6;
    pub const ROTATION_DATA_INDEX: u8 = 7;
    pub const FILLED_MAP_ITEM_ID: i16 = 358;
    /// EntityItemFrame#isInRangeToRenderDist: (16 * 64 * renderDistanceWeight)^2.
    /// The current client keeps the source default renderDistanceWeight of 1.0.
    pub const ENTITY_RENDER_DISTANCE_SQ: f64 = 1_048_576.0;
    /// RenderItemFrame displayed-item cull, separate from the frame entity range.
    pub const ITEM_RENDER_DISTANCE_SQ: f64 = 4096.0;

    pub fn isMap(stack: Option<&ItemStack>) -> bool {
        stack.is_some_and(|value| !value.isEmpty() && value.itemId == Self::FILLED_MAP_ITEM_ID)
    }

    pub fn normalizedRotation(rotation: i32) -> i32 {
        rotation.rem_euclid(8)
    }
    pub fn renderedRotation(rotation: i32, isMap: bool) -> i32 {
        if isMap {
            rotation.rem_euclid(4) * 2
        } else {
            rotation.rem_euclid(8)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_use_four_quarter_turn_states_while_items_use_eight() {
        assert_eq!(EntityItemFrame::renderedRotation(3, false), 3);
        assert_eq!(EntityItemFrame::renderedRotation(3, true), 6);
        assert_eq!(EntityItemFrame::normalizedRotation(-1), 7);
    }
}
