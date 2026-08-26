use crate::net::minecraft::util::math::MathHelper::{
    cos as minecraft_cos, sin as minecraft_sin, DEG_2_RAD,
};

/// Rotation/translation state used by the Rust equivalent of MCP 1.12.2
/// `ModelBiped.setRotationAngles`. Units are model pixels and radians.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PartPose {
    pub pivot: [f32; 3],
    pub rotation: [f32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BipedPose {
    pub head: PartPose,
    pub body: PartPose,
    pub rightArm: PartPose,
    pub leftArm: PartPose,
    pub rightLeg: PartPose,
    pub leftLeg: PartPose,
}

/// Exact MCP 1.12.2 `ModelBiped.ArmPose` states.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ArmPose {
    #[default]
    Empty,
    Item,
    Block,
    BowAndArrow,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BipedMotionInput {
    pub ticksElytraFlying: i32,
    pub motion: [f64; 3],
}

impl BipedMotionInput {
    pub const fn isElytraFlying(self) -> bool {
        self.ticksElytraFlying > 4
    }

    pub fn swingDivisor(self) -> f32 {
        if !self.isElytraFlying() {
            return 1.0;
        }
        let speedSquared = (self.motion[0] * self.motion[0]
            + self.motion[1] * self.motion[1]
            + self.motion[2] * self.motion[2]) as f32;
        let value = speedSquared / 0.2;
        (value * value * value).max(1.0)
    }
}

pub struct ModelBiped;

impl ModelBiped {
    #[allow(clippy::too_many_arguments)]
    pub fn setRotationAngles(
        limbSwing: f32,
        limbSwingAmount: f32,
        ageInTicks: f32,
        netHeadYaw: f32,
        headPitch: f32,
        swingProgress: f32,
        sneaking: bool,
        riding: bool,
        slimArms: bool,
        swingingArmIsLeft: bool,
        leftArmPose: ArmPose,
        rightArmPose: ArmPose,
    ) -> BipedPose {
        Self::setRotationAnglesWithMotion(
            limbSwing,
            limbSwingAmount,
            ageInTicks,
            netHeadYaw,
            headPitch,
            swingProgress,
            sneaking,
            riding,
            slimArms,
            swingingArmIsLeft,
            leftArmPose,
            rightArmPose,
            BipedMotionInput::default(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn setRotationAnglesWithMotion(
        limbSwing: f32,
        limbSwingAmount: f32,
        ageInTicks: f32,
        netHeadYaw: f32,
        headPitch: f32,
        swingProgress: f32,
        sneaking: bool,
        riding: bool,
        slimArms: bool,
        swingingArmIsLeft: bool,
        leftArmPose: ArmPose,
        rightArmPose: ArmPose,
        motionInput: BipedMotionInput,
    ) -> BipedPose {
        let swingDivisor = motionInput.swingDivisor();
        let mut pose = BipedPose {
            head: PartPose {
                pivot: [0.0, 0.0, 0.0],
                rotation: [
                    if motionInput.isElytraFlying() {
                        -std::f32::consts::FRAC_PI_4
                    } else {
                        headPitch * DEG_2_RAD
                    },
                    netHeadYaw * DEG_2_RAD,
                    0.0,
                ],
            },
            body: PartPose {
                pivot: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
            },
            rightArm: PartPose {
                pivot: [-5.0, if slimArms { 2.5 } else { 2.0 }, 0.0],
                rotation: [
                    minecraft_cos(limbSwing * 0.6662 + std::f32::consts::PI) * limbSwingAmount
                        / swingDivisor,
                    0.0,
                    0.0,
                ],
            },
            leftArm: PartPose {
                pivot: [5.0, if slimArms { 2.5 } else { 2.0 }, 0.0],
                rotation: [
                    minecraft_cos(limbSwing * 0.6662) * limbSwingAmount / swingDivisor,
                    0.0,
                    0.0,
                ],
            },
            rightLeg: PartPose {
                pivot: [-1.9, 12.0, 0.1],
                rotation: [
                    minecraft_cos(limbSwing * 0.6662) * 1.4 * limbSwingAmount / swingDivisor,
                    0.0,
                    0.0,
                ],
            },
            leftLeg: PartPose {
                pivot: [1.9, 12.0, 0.1],
                rotation: [
                    minecraft_cos(limbSwing * 0.6662 + std::f32::consts::PI)
                        * 1.4
                        * limbSwingAmount
                        / swingDivisor,
                    0.0,
                    0.0,
                ],
            },
        };

        if riding {
            pose.rightArm.rotation[0] += -std::f32::consts::PI / 5.0;
            pose.leftArm.rotation[0] += -std::f32::consts::PI / 5.0;
            pose.rightLeg.rotation = [-1.4137167, std::f32::consts::PI / 10.0, 0.07853982];
            pose.leftLeg.rotation = [-1.4137167, -std::f32::consts::PI / 10.0, -0.07853982];
        }

        // MCP applies held-item poses before the swing-progress body/arm pass.
        match leftArmPose {
            ArmPose::Empty => pose.leftArm.rotation[1] = 0.0,
            ArmPose::Block => {
                pose.leftArm.rotation[0] = pose.leftArm.rotation[0] * 0.5 - 0.9424779;
                pose.leftArm.rotation[1] = 0.5235988;
            }
            ArmPose::Item => {
                pose.leftArm.rotation[0] =
                    pose.leftArm.rotation[0] * 0.5 - std::f32::consts::PI / 10.0;
                pose.leftArm.rotation[1] = 0.0;
            }
            ArmPose::BowAndArrow => {}
        }

        match rightArmPose {
            ArmPose::Empty => pose.rightArm.rotation[1] = 0.0,
            ArmPose::Block => {
                pose.rightArm.rotation[0] = pose.rightArm.rotation[0] * 0.5 - 0.9424779;
                pose.rightArm.rotation[1] = -0.5235988;
            }
            ArmPose::Item => {
                pose.rightArm.rotation[0] =
                    pose.rightArm.rotation[0] * 0.5 - std::f32::consts::PI / 10.0;
                pose.rightArm.rotation[1] = 0.0;
            }
            ArmPose::BowAndArrow => {}
        }

        if swingProgress > 0.0 {
            let mut bodyYaw = minecraft_sin(swingProgress.sqrt() * std::f32::consts::TAU) * 0.2;
            if swingingArmIsLeft {
                bodyYaw = -bodyYaw;
            }
            pose.body.rotation[1] = bodyYaw;
            pose.rightArm.pivot[2] = minecraft_sin(bodyYaw) * 5.0;
            pose.rightArm.pivot[0] = -minecraft_cos(bodyYaw) * 5.0;
            pose.leftArm.pivot[2] = -minecraft_sin(bodyYaw) * 5.0;
            pose.leftArm.pivot[0] = minecraft_cos(bodyYaw) * 5.0;
            pose.rightArm.rotation[1] += bodyYaw;
            pose.leftArm.rotation[1] += bodyYaw;
            pose.leftArm.rotation[0] += bodyYaw;
            let mut progress = 1.0 - swingProgress;
            progress *= progress;
            progress *= progress;
            progress = 1.0 - progress;
            let f2 = minecraft_sin(progress * std::f32::consts::PI);
            let f3 = minecraft_sin(swingProgress * std::f32::consts::PI)
                * -(pose.head.rotation[0] - 0.7)
                * 0.75;
            let activeArm = if swingingArmIsLeft {
                &mut pose.leftArm
            } else {
                &mut pose.rightArm
            };
            activeArm.rotation[0] -= f2 * 1.2 + f3;
            activeArm.rotation[1] += bodyYaw * 2.0;
            activeArm.rotation[2] += minecraft_sin(swingProgress * std::f32::consts::PI) * -0.4;
        }

        if sneaking {
            pose.body.rotation[0] = 0.5;
            pose.rightArm.rotation[0] += 0.4;
            pose.leftArm.rotation[0] += 0.4;
            pose.rightLeg.pivot = [-1.9, 9.0, 4.0];
            pose.leftLeg.pivot = [1.9, 9.0, 4.0];
            pose.head.pivot[1] = 1.0;
        }

        pose.rightArm.rotation[2] += minecraft_cos(ageInTicks * 0.09) * 0.05 + 0.05;
        pose.leftArm.rotation[2] -= minecraft_cos(ageInTicks * 0.09) * 0.05 + 0.05;
        pose.rightArm.rotation[0] += minecraft_sin(ageInTicks * 0.067) * 0.05;
        pose.leftArm.rotation[0] -= minecraft_sin(ageInTicks * 0.067) * 0.05;

        // Bow pose is the final arm override in MCP ModelBiped.
        if rightArmPose == ArmPose::BowAndArrow {
            pose.rightArm.rotation[1] = -0.1 + pose.head.rotation[1];
            pose.leftArm.rotation[1] = 0.1 + pose.head.rotation[1] + 0.4;
            pose.rightArm.rotation[0] = -std::f32::consts::FRAC_PI_2 + pose.head.rotation[0];
            pose.leftArm.rotation[0] = -std::f32::consts::FRAC_PI_2 + pose.head.rotation[0];
        } else if leftArmPose == ArmPose::BowAndArrow {
            pose.rightArm.rotation[1] = -0.1 + pose.head.rotation[1] - 0.4;
            pose.leftArm.rotation[1] = 0.1 + pose.head.rotation[1];
            pose.rightArm.rotation[0] = -std::f32::consts::FRAC_PI_2 + pose.head.rotation[0];
            pose.leftArm.rotation[0] = -std::f32::consts::FRAC_PI_2 + pose.head.rotation[0];
        }

        pose
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pose(left: ArmPose, right: ArmPose) -> BipedPose {
        ModelBiped::setRotationAngles(
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, false, false, false, false, left, right,
        )
    }

    #[test]
    fn item_pose_lowers_the_held_arm_by_pi_over_ten() {
        let result = pose(ArmPose::Empty, ArmPose::Item);
        assert!((result.rightArm.rotation[0] + std::f32::consts::PI / 10.0).abs() < 1.0e-6);
    }

    #[test]
    fn block_pose_uses_side_specific_yaw() {
        let result = pose(ArmPose::Block, ArmPose::Block);
        assert!((result.leftArm.rotation[1] - 0.5235988).abs() < 1.0e-6);
        assert!((result.rightArm.rotation[1] + 0.5235988).abs() < 1.0e-6);
    }

    #[test]
    fn riding_pose_matches_mcp_arm_and_leg_angles() {
        let result = ModelBiped::setRotationAngles(
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            false,
            true,
            false,
            false,
            ArmPose::Empty,
            ArmPose::Empty,
        );
        assert!((result.rightArm.rotation[0] + std::f32::consts::PI / 5.0).abs() < 1.0e-6);
        assert!((result.leftArm.rotation[0] + std::f32::consts::PI / 5.0).abs() < 1.0e-6);
        assert_eq!(
            result.rightLeg.rotation,
            [-1.4137167, std::f32::consts::PI / 10.0, 0.07853982]
        );
        assert_eq!(
            result.leftLeg.rotation,
            [-1.4137167, -std::f32::consts::PI / 10.0, -0.07853982]
        );
    }

    #[test]
    fn bow_pose_tracks_head_rotation_for_both_arms() {
        let result = ModelBiped::setRotationAngles(
            0.0,
            0.0,
            0.0,
            20.0,
            10.0,
            0.0,
            false,
            false,
            false,
            false,
            ArmPose::Empty,
            ArmPose::BowAndArrow,
        );
        assert!((result.rightArm.rotation[1] - (20.0_f32.to_radians() - 0.1)).abs() < 1.0e-6);
        assert!(
            (result.leftArm.rotation[0] - (10.0_f32.to_radians() - std::f32::consts::FRAC_PI_2))
                .abs()
                < 1.0e-6
        );
    }

    #[test]
    fn walk_cycle_uses_mathhelper_lookup_trigonometry() {
        let limb_swing = 0.75_f32;
        let limb_amount = 0.8_f32;
        let result = ModelBiped::setRotationAngles(
            limb_swing,
            limb_amount,
            0.0,
            0.0,
            0.0,
            0.0,
            false,
            false,
            false,
            false,
            ArmPose::Empty,
            ArmPose::Empty,
        );
        let expected = minecraft_cos(limb_swing * 0.6662 + std::f32::consts::PI) * limb_amount;
        assert!((result.rightArm.rotation[0] - expected).abs() < 1.0e-7);
    }

    #[test]
    fn elytra_flight_uses_vanilla_head_pitch_and_speed_damping() {
        let normal = ModelBiped::setRotationAnglesWithMotion(
            1.0,
            1.0,
            0.0,
            0.0,
            30.0,
            0.0,
            false,
            false,
            false,
            false,
            ArmPose::Empty,
            ArmPose::Empty,
            BipedMotionInput {
                ticksElytraFlying: 5,
                motion: [1.0, 0.0, 0.0],
            },
        );
        assert!((normal.head.rotation[0] + std::f32::consts::FRAC_PI_4).abs() < 1.0e-6);
        let undamped = (1.0_f32 * 0.6662 + std::f32::consts::PI).cos();
        assert!(normal.rightArm.rotation[0].abs() < undamped.abs());
    }
}
