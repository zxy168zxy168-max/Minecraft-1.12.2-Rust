use crate::net::minecraft::item::ItemStack::ItemStack;

/// Interpolated source state consumed by MCP 1.12.2 `LayerCape#doRenderLayer`.
/// Rendering remains outside this class; all vanilla visibility and motion
/// calculations live here so the Vulkan backend only applies the returned
/// transform and submits `ModelPlayer#renderCape` geometry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CapeMotionInput {
    pub prevChasingPos: [f64; 3],
    pub chasingPos: [f64; 3],
    pub prevPos: [f64; 3],
    pub pos: [f64; 3],
    pub prevRenderYawOffset: f32,
    pub renderYawOffset: f32,
    pub prevCameraYaw: f32,
    pub cameraYaw: f32,
    pub prevDistanceWalkedModified: f32,
    pub distanceWalkedModified: f32,
    pub sneaking: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CapeTransform {
    /// The initial Z=0.125 translation plus the sneaking adjustment.
    pub translation: [f32; 3],
    pub rotateX: f32,
    pub rotateZ: f32,
    /// First Y rotation. The final 180 degree Y rotation remains distinct so
    /// matrix order exactly matches `LayerCape`.
    pub rotateY: f32,
    pub finalRotateY: f32,
}

pub struct LayerCape;

impl LayerCape {
    pub const ELYTRA_ITEM_ID: i16 = 443;
    pub const CAPE_PART_MASK: u8 = 1;

    /// Exact guard from MCP `LayerCape#doRenderLayer` after `hasPlayerInfo`.
    pub fn shouldRender(
        hasCapeTexture: bool,
        invisible: bool,
        skinParts: u8,
        chestStack: &ItemStack,
    ) -> bool {
        hasCapeTexture
            && !invisible
            && skinParts & Self::CAPE_PART_MASK != 0
            && chestStack.itemId != Self::ELYTRA_ITEM_ID
    }

    pub fn transform(input: CapeMotionInput, partialTicks: f32) -> CapeTransform {
        // Render partial ticks are normally in [0, 1], but MCP uses the value
        // directly. Do not introduce an extra clamp absent from LayerCape.
        let partial = partialTicks;
        let partial64 = partial as f64;
        let chasing = [
            lerp_f64(input.prevChasingPos[0], input.chasingPos[0], partial64),
            lerp_f64(input.prevChasingPos[1], input.chasingPos[1], partial64),
            lerp_f64(input.prevChasingPos[2], input.chasingPos[2], partial64),
        ];
        let position = [
            lerp_f64(input.prevPos[0], input.pos[0], partial64),
            lerp_f64(input.prevPos[1], input.pos[1], partial64),
            lerp_f64(input.prevPos[2], input.pos[2], partial64),
        ];
        let d0 = chasing[0] - position[0];
        let d1 = chasing[1] - position[1];
        let d2 = chasing[2] - position[2];
        let yaw = input.prevRenderYawOffset
            + (input.renderYawOffset - input.prevRenderYawOffset) * partial;
        let yawRadians = yaw * std::f32::consts::PI / 180.0;
        let d3 = yawRadians.sin() as f64;
        let d4 = -yawRadians.cos() as f64;

        let mut f1 = (d1 as f32 * 10.0).clamp(-6.0, 32.0);
        let mut f2 = (d0 * d3 + d2 * d4) as f32 * 100.0;
        let f3 = (d0 * d4 - d2 * d3) as f32 * 100.0;
        // Keep the explicit mutable value to mirror the MCP branch and make
        // future NaN/edge behaviour review local to this class.
        if f2 < 0.0 {
            f2 = 0.0;
        }
        if f2 > 165.0 {
            f2 = 165.0;
        }

        let f4 = input.prevCameraYaw + (input.cameraYaw - input.prevCameraYaw) * partial;
        let walked = input.prevDistanceWalkedModified
            + (input.distanceWalkedModified - input.prevDistanceWalkedModified) * partial;
        f1 += (walked * 6.0).sin() * 32.0 * f4;

        let mut translation = [0.0, 0.0, 0.125];
        if input.sneaking {
            f1 += 25.0;
            translation[1] += 0.142;
            translation[2] -= 0.0178;
        }

        CapeTransform {
            translation,
            rotateX: 6.0 + f2 / 2.0 + f1,
            rotateZ: f3 / 2.0,
            rotateY: -f3 / 2.0,
            finalRotateY: 180.0,
        }
    }

    pub const fn shouldCombineTextures() -> bool {
        false
    }
}

fn lerp_f64(start: f64, end: f64, partial: f64) -> f64 {
    start + (end - start) * partial
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_motion() -> CapeMotionInput {
        CapeMotionInput {
            prevChasingPos: [0.0; 3],
            chasingPos: [0.0; 3],
            prevPos: [0.0; 3],
            pos: [0.0; 3],
            prevRenderYawOffset: 0.0,
            renderYawOffset: 0.0,
            prevCameraYaw: 0.0,
            cameraYaw: 0.0,
            prevDistanceWalkedModified: 0.0,
            distanceWalkedModified: 0.0,
            sneaking: false,
        }
    }

    #[test]
    fn stationary_cape_keeps_vanilla_six_degree_rest_angle() {
        let transform = LayerCape::transform(empty_motion(), 0.5);
        assert_eq!(transform.translation, [0.0, 0.0, 0.125]);
        assert!((transform.rotateX - 6.0).abs() < 1.0e-6);
        assert_eq!(transform.rotateZ, 0.0);
        assert_eq!(transform.rotateY, 0.0);
        assert_eq!(transform.finalRotateY, 180.0);
    }

    #[test]
    fn sneaking_adds_exact_layer_cape_offsets_and_pitch() {
        let mut input = empty_motion();
        input.sneaking = true;
        let transform = LayerCape::transform(input, 1.0);
        assert_eq!(transform.translation[0], 0.0);
        assert!((transform.translation[1] - 0.142).abs() < 1.0e-6);
        assert!((transform.translation[2] - 0.1072).abs() < 1.0e-6);
        assert!((transform.rotateX - 31.0).abs() < 1.0e-6);
    }

    #[test]
    fn cape_is_hidden_by_model_part_or_elytra() {
        let empty = ItemStack::EMPTY;
        let elytra = ItemStack {
            itemId: 443,
            count: 1,
            itemDamage: 0,
            tagCompound: None,
        };
        assert!(LayerCape::shouldRender(true, false, 1, &empty));
        assert!(!LayerCape::shouldRender(true, false, 0, &empty));
        assert!(!LayerCape::shouldRender(true, true, 1, &empty));
        assert!(!LayerCape::shouldRender(true, false, 1, &elytra));
        assert!(!LayerCape::shouldCombineTextures());
    }
}
