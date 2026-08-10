use crate::net::minecraft::client::model::ModelBiped::PartPose;
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{LivingModelBox, LivingModelGroup};

pub struct ModelSquid;

impl ModelSquid {
    /// Exact ModelSquid body and eight tentacle boxes. `tentacleAngle` is the
    /// third setRotationAngles parameter supplied by RenderSquid.
    pub fn boxes(tentacleAngle: f32) -> Vec<LivingModelBox> {
        let mut boxes = Vec::with_capacity(9);
        boxes.push(model_box(
            [0, 0], [-6.0, -8.0, -6.0], [12, 16, 12], 0.0, false,
            PartPose { pivot: [0.0, 8.0, 0.0], rotation: [0.0;3] }, LivingModelGroup::Body,
        ));
        for j in 0..8 {
            let d0 = j as f32 * std::f32::consts::TAU / 8.0;
            let x = d0.cos() * 5.0;
            let z = d0.sin() * 5.0;
            let yaw = j as f32 * -std::f32::consts::TAU / 8.0 + std::f32::consts::FRAC_PI_2;
            boxes.push(model_box(
                [48, 0], [-1.0, 0.0, -1.0], [2, 18, 2], 0.0, false,
                PartPose { pivot: [x, 15.0, z], rotation: [tentacleAngle, yaw, 0.0] },
                LivingModelGroup::Body,
            ));
        }
        boxes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn has_one_body_and_eight_tentacles() { assert_eq!(ModelSquid::boxes(0.25).len(), 9); }
}
