use crate::net::minecraft::entity::Entity::Entity;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;

/// MCP-facing common state owned by `Render<T>`. Vulkan replaces the GL draw
/// calls, not culling, interpolation or renderer-specific shadow dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderProperties {
    pub shadowSize: f32,
    pub shadowOpaque: f32,
}

impl RenderProperties {
    pub const fn new(shadowSize: f32, shadowOpaque: f32) -> Self {
        Self {
            shadowSize,
            shadowOpaque,
        }
    }
}

pub struct Render;

impl Render {
    /// `Render.shouldRender` expands the entity render box by 0.5 before the
    /// camera test and substitutes a four-block cube for a degenerate box.
    pub fn renderBoundingBox(entity: &Entity) -> AxisAlignedBB {
        let expanded = entity.boundingBox.expand_xyz(0.5);
        let average = expanded.average_edge_length();
        if !average.is_finite() || average == 0.0 {
            AxisAlignedBB::new(
                entity.posX - 2.0,
                entity.posY - 2.0,
                entity.posZ - 2.0,
                entity.posX + 2.0,
                entity.posY + 2.0,
                entity.posZ + 2.0,
            )
        } else {
            expanded
        }
    }

    /// MCP `Entity.isInRangeToRenderDist` with the default render-distance
    /// weight of one. Concrete renderers still perform the frustum test.
    pub fn isInRangeToRenderDist(entity: &Entity, distanceSquared: f64) -> bool {
        let mut edge = entity.boundingBox.average_edge_length();
        if edge.is_nan() {
            edge = 1.0;
        }
        let range = edge * 64.0;
        distanceSquared < range * range
    }

    pub fn interpolatedPosition(entity: &Entity, partialTicks: f32) -> [f32; 3] {
        let partial = partialTicks.clamp(0.0, 1.0) as f64;
        [
            (entity.prevPosX + (entity.posX - entity.prevPosX) * partial) as f32,
            (entity.prevPosY + (entity.posY - entity.prevPosY) * partial) as f32,
            (entity.prevPosZ + (entity.posZ - entity.prevPosZ) * partial) as f32,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_box_expands_by_half_block() {
        let mut entity = Entity::default();
        entity.width = 0.25;
        entity.height = 0.25;
        entity.setPosition(1.0, 2.0, 3.0);
        let bounds = Render::renderBoundingBox(&entity);
        assert_eq!(bounds.min_y, 1.5);
        assert_eq!(bounds.max_y, 2.75);
    }
}
