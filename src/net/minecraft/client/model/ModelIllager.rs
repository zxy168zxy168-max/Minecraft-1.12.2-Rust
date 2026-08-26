use crate::net::minecraft::client::model::ModelBiped::PartPose;
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingModelBox, LivingModelGroup, LivingRenderInput,
};
use crate::net::minecraft::util::EnumHandSide::EnumHandSide;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IllagerArmPose {
    Crossed,
    Attacking,
    Spellcasting,
    BowAndArrow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IllagerPose {
    pub head: PartPose,
    pub hood: PartPose,
    pub nose: PartPose,
    pub body: PartPose,
    pub crossedArms: PartPose,
    pub rightLeg: PartPose,
    pub leftLeg: PartPose,
    pub rightArm: PartPose,
    pub leftArm: PartPose,
    pub armPose: IllagerArmPose,
}

pub struct ModelIllager;

impl ModelIllager {
    pub fn pose(
        input: LivingRenderInput,
        armPose: IllagerArmPose,
        primaryHand: EnumHandSide,
    ) -> IllagerPose {
        let head = PartPose {
            pivot: [0.0, 0.0, 0.0],
            rotation: [
                input.headPitch.to_radians(),
                (input.headYaw - input.bodyYaw).to_radians(),
                0.0,
            ],
        };
        let mut rightArm = PartPose {
            pivot: [-5.0, 2.0, 0.0],
            rotation: [0.0; 3],
        };
        let mut leftArm = PartPose {
            pivot: [5.0, 2.0, 0.0],
            rotation: [0.0; 3],
        };
        match armPose {
            IllagerArmPose::Attacking => {
                let f = (input.swingProgress * std::f32::consts::PI).sin();
                let f1 = ((1.0 - (1.0 - input.swingProgress).powi(2)) * std::f32::consts::PI).sin();
                rightArm.rotation[1] = 0.15707964;
                leftArm.rotation[1] = -0.15707964;
                if primaryHand == EnumHandSide::Right {
                    rightArm.rotation[0] =
                        -1.8849558 + (input.ageInTicks * 0.09).cos() * 0.15 + f * 2.2 - f1 * 0.4;
                    leftArm.rotation[0] =
                        (input.ageInTicks * 0.19).cos() * 0.5 + f * 1.2 - f1 * 0.4;
                } else {
                    rightArm.rotation[0] =
                        (input.ageInTicks * 0.19).cos() * 0.5 + f * 1.2 - f1 * 0.4;
                    leftArm.rotation[0] =
                        -1.8849558 + (input.ageInTicks * 0.09).cos() * 0.15 + f * 2.2 - f1 * 0.4;
                }
                rightArm.rotation[2] += (input.ageInTicks * 0.09).cos() * 0.05 + 0.05;
                leftArm.rotation[2] -= (input.ageInTicks * 0.09).cos() * 0.05 + 0.05;
                rightArm.rotation[0] += (input.ageInTicks * 0.067).sin() * 0.05;
                leftArm.rotation[0] -= (input.ageInTicks * 0.067).sin() * 0.05;
            }
            IllagerArmPose::Spellcasting => {
                rightArm.pivot = [-5.0, 2.0, 0.0];
                leftArm.pivot = [5.0, 2.0, 0.0];
                rightArm.rotation = [(input.ageInTicks * 0.6662).cos() * 0.25, 0.0, 2.3561945];
                leftArm.rotation = [(input.ageInTicks * 0.6662).cos() * 0.25, 0.0, -2.3561945];
            }
            IllagerArmPose::BowAndArrow => {
                rightArm.rotation[1] = -0.1 + head.rotation[1];
                rightArm.rotation[0] = -std::f32::consts::FRAC_PI_2 + head.rotation[0];
                leftArm.rotation[0] = -0.9424779 + head.rotation[0];
                leftArm.rotation[1] = head.rotation[1] - 0.4;
                leftArm.rotation[2] = std::f32::consts::FRAC_PI_2;
            }
            IllagerArmPose::Crossed => {}
        }
        IllagerPose {
            head,
            hood: PartPose {
                pivot: [0.0; 3],
                rotation: [0.0; 3],
            },
            nose: PartPose {
                pivot: [0.0, -2.0, 0.0],
                rotation: [0.0; 3],
            },
            body: PartPose {
                pivot: [0.0, 0.0, 0.0],
                rotation: [0.0; 3],
            },
            crossedArms: PartPose {
                pivot: [0.0, 3.0, -1.0],
                rotation: [-0.75, 0.0, 0.0],
            },
            rightLeg: PartPose {
                pivot: [-2.0, 12.0, 0.0],
                rotation: [
                    (input.limbSwing * 0.6662).cos() * 1.4 * input.limbSwingAmount * 0.5,
                    0.0,
                    0.0,
                ],
            },
            leftLeg: PartPose {
                pivot: [2.0, 12.0, 0.0],
                rotation: [
                    (input.limbSwing * 0.6662 + std::f32::consts::PI).cos()
                        * 1.4
                        * input.limbSwingAmount
                        * 0.5,
                    0.0,
                    0.0,
                ],
            },
            rightArm,
            leftArm,
            armPose,
        }
    }

    pub fn boxes(p: IllagerPose, showHood: bool) -> Vec<LivingModelBox> {
        let mut result = vec![
            model_box(
                [0, 0],
                [-4.0, -10.0, -4.0],
                [8, 10, 8],
                0.0,
                false,
                p.head,
                LivingModelGroup::Head,
            ),
            model_box(
                [16, 20],
                [-4.0, 0.0, -3.0],
                [8, 12, 6],
                0.0,
                false,
                p.body,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 38],
                [-4.0, 0.0, -3.0],
                [8, 18, 6],
                0.5,
                false,
                p.body,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 22],
                [-2.0, 0.0, -2.0],
                [4, 12, 4],
                0.0,
                false,
                p.rightLeg,
                LivingModelGroup::Body,
            ),
            model_box(
                [0, 22],
                [-2.0, 0.0, -2.0],
                [4, 12, 4],
                0.0,
                true,
                p.leftLeg,
                LivingModelGroup::Body,
            ),
        ];
        let mut nose = model_box(
            [24, 0],
            [-1.0, -1.0, -6.0],
            [2, 4, 2],
            0.0,
            false,
            p.nose,
            LivingModelGroup::Head,
        );
        nose.parentPose = Some(p.head);
        result.push(nose);
        if showHood {
            let mut hood = model_box(
                [32, 0],
                [-4.0, -10.0, -4.0],
                [8, 12, 8],
                0.45,
                false,
                p.hood,
                LivingModelGroup::Head,
            );
            hood.parentPose = Some(p.head);
            result.push(hood);
        }
        if p.armPose == IllagerArmPose::Crossed {
            result.extend([
                model_box(
                    [44, 22],
                    [-8.0, -2.0, -2.0],
                    [4, 8, 4],
                    0.0,
                    false,
                    p.crossedArms,
                    LivingModelGroup::Body,
                ),
                model_box(
                    [44, 22],
                    [4.0, -2.0, -2.0],
                    [4, 8, 4],
                    0.0,
                    true,
                    p.crossedArms,
                    LivingModelGroup::Body,
                ),
                model_box(
                    [40, 38],
                    [-4.0, 2.0, -2.0],
                    [8, 4, 4],
                    0.0,
                    false,
                    p.crossedArms,
                    LivingModelGroup::Body,
                ),
            ]);
        } else {
            result.extend([
                model_box(
                    [40, 46],
                    [-3.0, -2.0, -2.0],
                    [4, 12, 4],
                    0.0,
                    false,
                    p.rightArm,
                    LivingModelGroup::Body,
                ),
                model_box(
                    [40, 46],
                    [-1.0, -2.0, -2.0],
                    [4, 12, 4],
                    0.0,
                    true,
                    p.leftArm,
                    LivingModelGroup::Body,
                ),
            ]);
        }
        result
    }

    pub const fn armForSide(pose: IllagerPose, side: EnumHandSide) -> PartPose {
        match side {
            EnumHandSide::Left => pose.leftArm,
            EnumHandSide::Right => pose.rightArm,
        }
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
            headYaw: 15.0,
            headPitch: 10.0,
            limbSwing: 0.0,
            limbSwingAmount: 0.0,
            ageInTicks: 1.0,
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
    fn spellcasting_arms_use_mcp_diagonal_angles() {
        let p = ModelIllager::pose(input(), IllagerArmPose::Spellcasting, EnumHandSide::Right);
        assert!((p.rightArm.rotation[2] - 2.3561945).abs() < 1e-6);
        assert!((p.leftArm.rotation[2] + 2.3561945).abs() < 1e-6);
    }
}
