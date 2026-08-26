use crate::compat::Java::JavaRandom;
use crate::net::minecraft::client::renderer::entity::Render::RenderProperties;
use crate::net::minecraft::item::ItemStack::ItemStack;

pub struct RenderEntityItem;

impl RenderEntityItem {
    pub const PROPERTIES: RenderProperties = RenderProperties::new(0.15, 0.75);

    pub const fn getModelCount(stack: &ItemStack) -> i32 {
        let count = stack.getCount();
        if count > 48 {
            5
        } else if count > 32 {
            4
        } else if count > 16 {
            3
        } else if count > 1 {
            2
        } else {
            1
        }
    }

    pub fn bobOffset(age: i32, partialTicks: f32, hoverStart: f32) -> f32 {
        (((age as f32 + partialTicks) / 10.0 + hoverStart).sin() * 0.1) + 0.1
    }

    pub fn rotationDegrees(age: i32, partialTicks: f32, hoverStart: f32) -> f32 {
        ((age as f32 + partialTicks) / 20.0 + hoverStart) * (180.0 / std::f32::consts::PI)
    }

    /// RenderEntityItem seeds java.util.Random with item ID + metadata before
    /// positioning additional copies.
    pub fn randomFor(stack: &ItemStack) -> JavaRandom {
        JavaRandom::new((stack.itemId as i32 + stack.itemDamage as i32) as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(count: u8) -> ItemStack {
        ItemStack {
            itemId: 264,
            count,
            itemDamage: 0,
            tagCompound: None,
        }
    }

    #[test]
    fn stack_copy_thresholds_match_mcp() {
        assert_eq!(RenderEntityItem::getModelCount(&stack(1)), 1);
        assert_eq!(RenderEntityItem::getModelCount(&stack(2)), 2);
        assert_eq!(RenderEntityItem::getModelCount(&stack(17)), 3);
        assert_eq!(RenderEntityItem::getModelCount(&stack(33)), 4);
        assert_eq!(RenderEntityItem::getModelCount(&stack(49)), 5);
    }
}
