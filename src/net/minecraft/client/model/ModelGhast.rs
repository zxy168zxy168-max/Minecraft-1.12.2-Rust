use crate::compat::Java::JavaRandom;
use crate::net::minecraft::client::model::ModelBiped::PartPose;
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingModelBox, LivingModelGroup, LivingRenderInput, RenderLivingBase,
};

/// MCP 1.12.2 `ModelGhast`.
pub struct ModelGhast;

impl ModelGhast {
    /// `ModelGhast.render` applies this after the inherited living transforms.
    pub const MODEL_TRANSLATION: [f32; 3] = [0.0, 0.6, 0.0];

    pub fn input(input: LivingRenderInput) -> LivingRenderInput {
        RenderLivingBase::withAdultTranslation(input, Self::MODEL_TRANSLATION)
    }

    pub fn boxes(input: LivingRenderInput) -> Vec<LivingModelBox> {
        let mut boxes = Vec::with_capacity(10);
        boxes.push(model_box(
            [0, 0],
            [-8.0, -8.0, -8.0],
            [16, 16, 16],
            0.0,
            false,
            PartPose {
                pivot: [0.0, 8.0, 0.0],
                rotation: [0.0; 3],
            },
            LivingModelGroup::Body,
        ));

        let mut random = JavaRandom::new(1660);
        for i in 0_i32..9 {
            let x = ((i % 3) as f32 - (i / 3 % 2) as f32 * 0.5 + 0.25 - 1.0) * 5.0;
            let z = ((i / 3) as f32 - 1.0) * 5.0;
            let length = random.next_i32_bound(7) + 8;
            boxes.push(model_box(
                [0, 0],
                [-1.0, 0.0, -1.0],
                [2, length, 2],
                0.0,
                false,
                PartPose {
                    pivot: [x, 15.0, z],
                    rotation: [
                        0.2 * (input.ageInTicks * 0.3 + i as f32).sin() + 0.4,
                        0.0,
                        0.0,
                    ],
                },
                LivingModelGroup::Body,
            ));
        }
        boxes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::client::renderer::entity::RenderLivingBase::LivingChildLayout;

    fn input() -> LivingRenderInput {
        LivingRenderInput {
            position: [0.0; 3],
            bodyYaw: 0.0,
            headYaw: 0.0,
            headPitch: 0.0,
            limbSwing: 0.0,
            limbSwingAmount: 0.0,
            ageInTicks: 0.0,
            swingProgress: 0.0,
            sneaking: false,
            child: false,
            deathRotation: 0.0,
            preScale: 4.5,
            preScaleXYZ: [4.5; 3],
            childLayout: LivingChildLayout::BIPED,
            adultTranslation: [0.0; 3],
        }
    }

    #[test]
    fn seeded_tentacle_lengths_match_java_random() {
        let boxes = ModelGhast::boxes(input());
        let lengths = boxes[1..]
            .iter()
            .map(|model_box| model_box.size[1])
            .collect::<Vec<_>>();
        assert_eq!(lengths, vec![8, 13, 9, 11, 11, 10, 12, 9, 12]);
    }

    #[test]
    fn tentacles_use_original_three_by_three_positions() {
        let boxes = ModelGhast::boxes(input());
        assert_eq!(boxes[1].pose.pivot, [-3.75, 15.0, -5.0]);
        assert_eq!(boxes[5].pose.pivot, [-1.25, 15.0, 0.0]);
        assert_eq!(boxes[9].pose.pivot, [6.25, 15.0, 5.0]);
    }
}
