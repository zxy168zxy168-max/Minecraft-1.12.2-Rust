use crate::net::minecraft::client::model::ModelBiped::PartPose;
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingChildLayout, LivingModelBox, LivingModelGroup, LivingRenderInput,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuadrupedPose {
    pub head: PartPose,
    pub body: PartPose,
    pub leg1: PartPose,
    pub leg2: PartPose,
    pub leg3: PartPose,
    pub leg4: PartPose,
}

/// Exact common geometry/animation owned by MCP 1.12.2 `ModelQuadruped`.
pub struct ModelQuadruped;

impl ModelQuadruped {
    pub fn pose(input: LivingRenderInput, height: i32) -> QuadrupedPose {
        let phase = input.limbSwing * 0.6662;
        QuadrupedPose {
            head: PartPose {
                pivot: [0.0, (18 - height) as f32, -6.0],
                rotation: [
                    input.headPitch.to_radians(),
                    (input.headYaw - input.bodyYaw).to_radians(),
                    0.0,
                ],
            },
            body: PartPose {
                pivot: [0.0, (17 - height) as f32, 2.0],
                rotation: [std::f32::consts::FRAC_PI_2, 0.0, 0.0],
            },
            leg1: leg_pose(
                [-3.0, (24 - height) as f32, 7.0],
                phase.cos() * 1.4 * input.limbSwingAmount,
            ),
            leg2: leg_pose(
                [3.0, (24 - height) as f32, 7.0],
                (phase + std::f32::consts::PI).cos() * 1.4 * input.limbSwingAmount,
            ),
            leg3: leg_pose(
                [-3.0, (24 - height) as f32, -5.0],
                (phase + std::f32::consts::PI).cos() * 1.4 * input.limbSwingAmount,
            ),
            leg4: leg_pose(
                [3.0, (24 - height) as f32, -5.0],
                phase.cos() * 1.4 * input.limbSwingAmount,
            ),
        }
    }

    pub fn boxes(pose: QuadrupedPose, height: i32, scale: f32) -> Vec<LivingModelBox> {
        vec![
            model_box(
                [0, 0],
                [-4.0, -4.0, -8.0],
                [8, 8, 8],
                scale,
                false,
                pose.head,
                LivingModelGroup::Head,
            ),
            model_box(
                [28, 8],
                [-5.0, -10.0, -7.0],
                [10, 16, 8],
                scale,
                false,
                pose.body,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 16],
                [-2.0, 0.0, -2.0],
                [4, height, 4],
                scale,
                false,
                pose.leg1,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 16],
                [-2.0, 0.0, -2.0],
                [4, height, 4],
                scale,
                false,
                pose.leg2,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 16],
                [-2.0, 0.0, -2.0],
                [4, height, 4],
                scale,
                false,
                pose.leg3,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 16],
                [-2.0, 0.0, -2.0],
                [4, height, 4],
                scale,
                false,
                pose.leg4,
                LivingModelGroup::Body,
            ),
        ]
    }

    pub const fn childLayout(childYOffset: f32, childZOffset: f32) -> LivingChildLayout {
        LivingChildLayout::quadruped(childYOffset, childZOffset)
    }
}

const fn leg_pose(pivot: [f32; 3], rotateX: f32) -> PartPose {
    PartPose {
        pivot,
        rotation: [rotateX, 0.0, 0.0],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> LivingRenderInput {
        LivingRenderInput {
            position: [0.0; 3],
            bodyYaw: 0.0,
            headYaw: 30.0,
            headPitch: 10.0,
            limbSwing: 0.0,
            limbSwingAmount: 1.0,
            ageInTicks: 0.0,
            swingProgress: 0.0,
            sneaking: false,
            child: false,
            deathRotation: 0.0,
            preScale: 1.0,
            preScaleXYZ: [1.0; 3],
            childLayout: LivingChildLayout::BIPED,
            adultTranslation: [0.0; 3],
        }
    }

    #[test]
    fn opposite_legs_use_mcp_phase_pairing() {
        let pose = ModelQuadruped::pose(input(), 6);
        assert!((pose.leg1.rotation[0] - 1.4).abs() < 1.0e-6);
        assert!((pose.leg2.rotation[0] + 1.4).abs() < 1.0e-6);
        assert!((pose.leg3.rotation[0] + 1.4).abs() < 1.0e-6);
        assert!((pose.leg4.rotation[0] - 1.4).abs() < 1.0e-6);
    }
}
