use crate::net::minecraft::client::entity::EntityOtherClient::EntityOtherClient;
use crate::net::minecraft::client::model::ModelBiped::PartPose;
use crate::net::minecraft::client::model::ModelZombie::model_box;
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingModelBox, LivingModelGroup, LivingPartTransform, LivingRenderInput,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorseModelVariant {
    Horse,
    Donkey,
    Mule,
    Skeleton,
    Zombie,
}

impl HorseModelVariant {
    pub const fn chestHorse(self) -> bool {
        matches!(self, Self::Donkey | Self::Mule)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HorsePose {
    body: PartPose,
    tailBase: PartPose,
    tailMiddle: PartPose,
    tailTip: PartPose,
    backLeftLeg: PartPose,
    backLeftShin: PartPose,
    backLeftHoof: PartPose,
    backRightLeg: PartPose,
    backRightShin: PartPose,
    backRightHoof: PartPose,
    frontLeftLeg: PartPose,
    frontLeftShin: PartPose,
    frontLeftHoof: PartPose,
    frontRightLeg: PartPose,
    frontRightShin: PartPose,
    frontRightHoof: PartPose,
    head: PartPose,
    upperMouth: PartPose,
    lowerMouth: PartPose,
    horseLeftEar: PartPose,
    horseRightEar: PartPose,
    muleLeftEar: PartPose,
    muleRightEar: PartPose,
    neck: PartPose,
    muleLeftChest: PartPose,
    muleRightChest: PartPose,
    saddleBottom: PartPose,
    saddleFront: PartPose,
    saddleBack: PartPose,
    leftSaddleRope: PartPose,
    leftSaddleMetal: PartPose,
    rightSaddleRope: PartPose,
    rightSaddleMetal: PartPose,
    leftFaceMetal: PartPose,
    rightFaceMetal: PartPose,
    leftRein: PartPose,
    rightRein: PartPose,
    mane: PartPose,
    faceRopes: PartPose,
}

pub struct ModelHorse;

impl ModelHorse {
    pub fn pose(
        input: LivingRenderInput,
        entity: &EntityOtherClient,
        partialTicks: f32,
    ) -> HorsePose {
        let f3 = (input.headYaw - input.bodyYaw).clamp(-20.0, 20.0);
        let mut f4 = input.headPitch.to_radians();
        if input.limbSwingAmount > 0.2 {
            f4 += (input.limbSwing * 0.4).cos() * 0.15 * input.limbSwingAmount;
        }
        let f5 = entity.horseGrassEatingAmount(partialTicks);
        let f6 = entity.horseRearingAmount(partialTicks);
        let f7 = 1.0 - f6;
        let f8 = entity.horseMouthOpennessAngle(partialTicks);
        let tailMoving = entity.horseTailCounter != 0;
        let saddled = entity.horseSaddled();
        let ridden = entity.horseBeingRidden();
        let f9 = input.ageInTicks;
        let f10 = (input.limbSwing * 0.6662 + std::f32::consts::PI).cos();
        let f11 = f10 * 0.8 * input.limbSwingAmount;

        let mut head = PartPose {
            pivot: [0.0, 4.0, -10.0],
            rotation: [0.5235988 + f4, f3.to_radians(), 0.0],
        };
        let dominant = f6.max(f5);
        head.rotation[0] =
            f6 * (0.2617994 + f4) + f5 * 2.1816616 + (1.0 - dominant) * head.rotation[0];
        head.rotation[1] = f6 * f3.to_radians() + (1.0 - dominant) * head.rotation[1];
        head.pivot[1] = f6 * -6.0 + f5 * 11.0 + (1.0 - dominant) * head.pivot[1];
        head.pivot[2] = f6 * -1.0 + f5 * -10.0 + (1.0 - dominant) * head.pivot[2];

        let mut tailBase = PartPose {
            pivot: [0.0, f6 * 9.0 + f7 * 3.0, 14.0],
            rotation: [0.0; 3],
        };
        let body = PartPose {
            pivot: [0.0, 11.0, 9.0],
            rotation: [-f6 * std::f32::consts::FRAC_PI_4, 0.0, 0.0],
        };
        let muleRightChest = PartPose {
            pivot: [4.5, f6 * 5.5 + f7 * 3.0, f6 * 15.0 + f7 * 10.0],
            rotation: [-f11 / 5.0, std::f32::consts::FRAC_PI_2, 0.0],
        };
        let mut muleLeftChest = PartPose {
            pivot: [-7.5, 3.0, 10.0],
            rotation: [f11 / 5.0, std::f32::consts::FRAC_PI_2, 0.0],
        };

        let f12_rear = 0.2617994 * f6;
        let f13 = (f9 * 0.6 + std::f32::consts::PI).cos();
        let frontY = -2.0 * f6 + 9.0 * f7;
        let frontZ = -2.0 * f6 - 8.0 * f7;
        let backLeftLeg = PartPose {
            pivot: [4.0, 9.0, 11.0],
            rotation: [f12_rear - f10 * 0.5 * input.limbSwingAmount * f7, 0.0, 0.0],
        };
        let backRightLeg = PartPose {
            pivot: [-4.0, 9.0, 11.0],
            rotation: [f12_rear + f10 * 0.5 * input.limbSwingAmount * f7, 0.0, 0.0],
        };
        let backLeftShinAngle = -0.08726646 * f6
            + (-f10 * 0.5 * input.limbSwingAmount - (f10 * 0.5 * input.limbSwingAmount).max(0.0))
                * f7;
        let backRightShinAngle = -0.08726646 * f6
            + (f10 * 0.5 * input.limbSwingAmount - (-f10 * 0.5 * input.limbSwingAmount).max(0.0))
                * f7;
        let backLeftShin = PartPose {
            pivot: [
                4.0,
                9.0 + (std::f32::consts::FRAC_PI_2 + f12_rear
                    - f7 * f10 * 0.5 * input.limbSwingAmount)
                    .sin()
                    * 7.0,
                11.0 + (-std::f32::consts::FRAC_PI_2 + f12_rear
                    - f7 * f10 * 0.5 * input.limbSwingAmount)
                    .cos()
                    * 7.0,
            ],
            rotation: [backLeftShinAngle, 0.0, 0.0],
        };
        let backRightShin = PartPose {
            pivot: [
                -4.0,
                9.0 + (std::f32::consts::FRAC_PI_2
                    + f12_rear
                    + f7 * f10 * 0.5 * input.limbSwingAmount)
                    .sin()
                    * 7.0,
                11.0 + (-std::f32::consts::FRAC_PI_2
                    + f12_rear
                    + f7 * f10 * 0.5 * input.limbSwingAmount)
                    .cos()
                    * 7.0,
            ],
            rotation: [backRightShinAngle, 0.0, 0.0],
        };
        let f14 = (-1.0471976 + f13) * f6 + f11 * f7;
        let f15 = (-1.0471976 - f13) * f6 - f11 * f7;
        let frontLeftLeg = PartPose {
            pivot: [4.0, frontY, frontZ],
            rotation: [f14, 0.0, 0.0],
        };
        let frontRightLeg = PartPose {
            pivot: [-4.0, frontY, frontZ],
            rotation: [f15, 0.0, 0.0],
        };
        let frontLeftShinAngle = (f14 + std::f32::consts::PI * (0.2 + f13 * 0.2).max(0.0)) * f6
            + (f11 + (f10 * 0.5 * input.limbSwingAmount).max(0.0)) * f7;
        let frontRightShinAngle = (f15 + std::f32::consts::PI * (0.2 - f13 * 0.2).max(0.0)) * f6
            + (-f11 + (-f10 * 0.5 * input.limbSwingAmount).max(0.0)) * f7;
        let frontLeftShin = PartPose {
            pivot: [
                4.0,
                frontY + (std::f32::consts::FRAC_PI_2 + f14).sin() * 7.0,
                frontZ + (-std::f32::consts::FRAC_PI_2 + f14).cos() * 7.0,
            ],
            rotation: [frontLeftShinAngle, 0.0, 0.0],
        };
        let frontRightShin = PartPose {
            pivot: [
                -4.0,
                frontY + (std::f32::consts::FRAC_PI_2 + f15).sin() * 7.0,
                frontZ + (-std::f32::consts::FRAC_PI_2 + f15).cos() * 7.0,
            ],
            rotation: [frontRightShinAngle, 0.0, 0.0],
        };

        let headFollower = |z_rotation: f32| PartPose {
            pivot: head.pivot,
            rotation: [head.rotation[0], head.rotation[1], z_rotation],
        };
        let mut saddleBottom = PartPose {
            pivot: [0.0, 2.0, 2.0],
            rotation: [0.0; 3],
        };
        let mut saddleFront = PartPose {
            pivot: [0.0, 2.0, 2.0],
            rotation: [0.0; 3],
        };
        let mut saddleBack = saddleFront;
        let mut leftSaddleRope = PartPose {
            pivot: [5.0, 3.0, 2.0],
            rotation: [0.0; 3],
        };
        let mut leftSaddleMetal = leftSaddleRope;
        let mut rightSaddleRope = PartPose {
            pivot: [-5.0, 3.0, 2.0],
            rotation: [0.0; 3],
        };
        let mut rightSaddleMetal = rightSaddleRope;
        let mut leftFaceMetal = headFollower(0.0);
        let mut rightFaceMetal = headFollower(0.0);
        let mut faceRopes = headFollower(0.0);
        let mut leftRein = PartPose {
            pivot: head.pivot,
            rotation: [f4, head.rotation[1], 0.0],
        };
        let mut rightRein = leftRein;
        if saddled {
            saddleBottom.pivot = [0.0, f6 * 0.5 + f7 * 2.0, f6 * 11.0 + f7 * 2.0];
            saddleBottom.rotation[0] = body.rotation[0];
            saddleFront.pivot = saddleBottom.pivot;
            saddleFront.rotation[0] = body.rotation[0];
            saddleBack = saddleFront;
            leftSaddleRope.pivot[1] = saddleBottom.pivot[1];
            leftSaddleRope.pivot[2] = saddleBottom.pivot[2];
            leftSaddleMetal.pivot = leftSaddleRope.pivot;
            rightSaddleRope.pivot[1] = saddleBottom.pivot[1];
            rightSaddleRope.pivot[2] = saddleBottom.pivot[2];
            rightSaddleMetal.pivot = rightSaddleRope.pivot;
            muleLeftChest.pivot[1] = muleRightChest.pivot[1];
            muleLeftChest.pivot[2] = muleRightChest.pivot[2];
            if ridden {
                leftSaddleRope.rotation[0] = -1.0471976;
                leftSaddleMetal.rotation[0] = -1.0471976;
                rightSaddleRope.rotation[0] = -1.0471976;
                rightSaddleMetal.rotation[0] = -1.0471976;
            } else {
                leftSaddleRope.rotation = [f11 / 3.0, 0.0, f11 / 5.0];
                leftSaddleMetal.rotation = leftSaddleRope.rotation;
                rightSaddleRope.rotation = [f11 / 3.0, 0.0, -f11 / 5.0];
                rightSaddleMetal.rotation = rightSaddleRope.rotation;
            }
            faceRopes = headFollower(0.0);
            leftFaceMetal = headFollower(0.0);
            rightFaceMetal = headFollower(0.0);
            leftRein = PartPose {
                pivot: head.pivot,
                rotation: [f4, head.rotation[1], 0.0],
            };
            rightRein = leftRein;
        }

        let mut tailAngle = -1.3089969 + input.limbSwingAmount * 1.5;
        if tailAngle > 0.0 {
            tailAngle = 0.0;
        }
        if tailMoving {
            tailBase.rotation[1] = (f9 * 0.7).cos();
            tailAngle = 0.0;
        }
        tailBase.rotation[0] = tailAngle;
        let tailMiddle = PartPose {
            // MCP assigns the rearing-time value and later overwrites it with
            // tailBase.rotationPointZ. Preserve that final 1.12.2 behavior.
            pivot: [0.0, tailBase.pivot[1], tailBase.pivot[2]],
            rotation: [tailAngle, tailBase.rotation[1], 0.0],
        };
        let tailTip = PartPose {
            pivot: [0.0, tailBase.pivot[1], tailBase.pivot[2]],
            rotation: [-0.2617994 + tailAngle, tailBase.rotation[1], 0.0],
        };

        HorsePose {
            body,
            tailBase,
            tailMiddle,
            tailTip,
            backLeftLeg,
            backLeftShin,
            backLeftHoof: PartPose {
                pivot: backLeftShin.pivot,
                rotation: [backLeftShinAngle, 0.0, 0.0],
            },
            backRightLeg,
            backRightShin,
            backRightHoof: PartPose {
                pivot: backRightShin.pivot,
                rotation: [backRightShinAngle, 0.0, 0.0],
            },
            frontLeftLeg,
            frontLeftShin,
            frontLeftHoof: PartPose {
                pivot: frontLeftShin.pivot,
                rotation: [frontLeftShinAngle, 0.0, 0.0],
            },
            frontRightLeg,
            frontRightShin,
            frontRightHoof: PartPose {
                pivot: frontRightShin.pivot,
                rotation: [frontRightShinAngle, 0.0, 0.0],
            },
            head,
            upperMouth: PartPose {
                pivot: [0.0, 0.02, 0.02 - f8],
                rotation: [-0.09424778 * f8, 0.0, 0.0],
            },
            lowerMouth: PartPose {
                pivot: [0.0, 0.0, f8],
                rotation: [0.15707964 * f8, 0.0, 0.0],
            },
            horseLeftEar: headFollower(0.0),
            horseRightEar: headFollower(0.0),
            muleLeftEar: headFollower(0.2617994),
            muleRightEar: headFollower(-0.2617994),
            neck: headFollower(0.0),
            muleLeftChest,
            muleRightChest,
            saddleBottom,
            saddleFront,
            saddleBack,
            leftSaddleRope,
            leftSaddleMetal,
            rightSaddleRope,
            rightSaddleMetal,
            leftFaceMetal,
            rightFaceMetal,
            leftRein,
            rightRein,
            mane: headFollower(0.0),
            faceRopes,
        }
    }

    pub fn boxes(
        pose: HorsePose,
        input: LivingRenderInput,
        entity: &EntityOtherClient,
        variant: HorseModelVariant,
        partialTicks: f32,
    ) -> Vec<LivingModelBox> {
        let mut boxes = Vec::with_capacity(42);
        let child = input.child;
        let size = 0.5_f32;
        let legTransform = child.then_some(LivingPartTransform {
            scale: [size, 0.5 + size * 0.5, size],
            translation: [0.0, 0.95 * (1.0 - size), 0.0],
        });
        let bodyTransform = child.then_some(LivingPartTransform {
            scale: [size; 3],
            translation: [0.0, 1.35 * (1.0 - size), 0.0],
        });
        let grass = entity.horseGrassEatingAmount(partialTicks);
        let headScale = 0.5 + size * size * 0.5;
        let headTransform = child.then_some(LivingPartTransform {
            scale: [headScale; 3],
            translation: if grass <= 0.0 {
                [0.0, 1.35 * (1.0 - size), 0.0]
            } else {
                [
                    0.0,
                    0.9 * (1.0 - size) * grass + 1.35 * (1.0 - size) * (1.0 - grass),
                    0.15 * (1.0 - size) * grass,
                ]
            },
        });
        let mut push = |texture, origin, dimensions, delta, mirror, partPose, transform, parent| {
            let mut modelBox = model_box(
                texture,
                origin,
                dimensions,
                delta,
                mirror,
                partPose,
                LivingModelGroup::Body,
            );
            modelBox.childTransform = transform;
            modelBox.parentPose = parent;
            boxes.push(modelBox);
        };

        push(
            [0, 34],
            [-5.0, -8.0, -19.0],
            [10, 10, 24],
            0.0,
            false,
            pose.body,
            bodyTransform,
            None,
        );
        push(
            [44, 0],
            [-1.0, -1.0, 0.0],
            [2, 2, 3],
            0.0,
            false,
            pose.tailBase,
            bodyTransform,
            None,
        );
        push(
            [38, 7],
            [-1.5, -2.0, 3.0],
            [3, 4, 7],
            0.0,
            false,
            pose.tailMiddle,
            bodyTransform,
            None,
        );
        push(
            [24, 3],
            [-1.5, -4.5, 9.0],
            [3, 4, 7],
            0.0,
            false,
            pose.tailTip,
            bodyTransform,
            None,
        );
        push(
            [78, 29],
            [-2.5, -2.0, -2.5],
            [4, 9, 5],
            0.0,
            false,
            pose.backLeftLeg,
            legTransform,
            None,
        );
        push(
            [78, 43],
            [-2.0, 0.0, -1.5],
            [3, 5, 3],
            0.0,
            false,
            pose.backLeftShin,
            legTransform,
            None,
        );
        push(
            [78, 51],
            [-2.5, 5.1, -2.0],
            [4, 3, 4],
            0.0,
            false,
            pose.backLeftHoof,
            legTransform,
            None,
        );
        push(
            [96, 29],
            [-1.5, -2.0, -2.5],
            [4, 9, 5],
            0.0,
            false,
            pose.backRightLeg,
            legTransform,
            None,
        );
        push(
            [96, 43],
            [-1.0, 0.0, -1.5],
            [3, 5, 3],
            0.0,
            false,
            pose.backRightShin,
            legTransform,
            None,
        );
        push(
            [96, 51],
            [-1.5, 5.1, -2.0],
            [4, 3, 4],
            0.0,
            false,
            pose.backRightHoof,
            legTransform,
            None,
        );
        push(
            [44, 29],
            [-1.9, -1.0, -2.1],
            [3, 8, 4],
            0.0,
            false,
            pose.frontLeftLeg,
            legTransform,
            None,
        );
        push(
            [44, 41],
            [-1.9, 0.0, -1.6],
            [3, 5, 3],
            0.0,
            false,
            pose.frontLeftShin,
            legTransform,
            None,
        );
        push(
            [44, 51],
            [-2.4, 5.1, -2.1],
            [4, 3, 4],
            0.0,
            false,
            pose.frontLeftHoof,
            legTransform,
            None,
        );
        push(
            [60, 29],
            [-1.1, -1.0, -2.1],
            [3, 8, 4],
            0.0,
            false,
            pose.frontRightLeg,
            legTransform,
            None,
        );
        push(
            [60, 41],
            [-1.1, 0.0, -1.6],
            [3, 5, 3],
            0.0,
            false,
            pose.frontRightShin,
            legTransform,
            None,
        );
        push(
            [60, 51],
            [-1.6, 5.1, -2.1],
            [4, 3, 4],
            0.0,
            false,
            pose.frontRightHoof,
            legTransform,
            None,
        );
        push(
            [0, 0],
            [-2.5, -10.0, -1.5],
            [5, 5, 7],
            0.0,
            false,
            pose.head,
            headTransform,
            None,
        );
        push(
            [24, 18],
            [-2.0, -10.0, -7.0],
            [4, 3, 6],
            0.0,
            false,
            pose.upperMouth,
            headTransform,
            Some(pose.head),
        );
        push(
            [24, 27],
            [-2.0, -7.0, -6.5],
            [4, 2, 5],
            0.0,
            false,
            pose.lowerMouth,
            headTransform,
            Some(pose.head),
        );
        if variant.chestHorse() {
            push(
                [0, 12],
                [-2.0, -16.0, 4.0],
                [2, 7, 1],
                0.0,
                false,
                pose.muleLeftEar,
                headTransform,
                None,
            );
            push(
                [0, 12],
                [0.0, -16.0, 4.0],
                [2, 7, 1],
                0.0,
                false,
                pose.muleRightEar,
                headTransform,
                None,
            );
        } else {
            push(
                [0, 0],
                [0.45, -12.0, 4.0],
                [2, 3, 1],
                0.0,
                false,
                pose.horseLeftEar,
                headTransform,
                None,
            );
            push(
                [0, 0],
                [-2.45, -12.0, 4.0],
                [2, 3, 1],
                0.0,
                false,
                pose.horseRightEar,
                headTransform,
                None,
            );
        }
        push(
            [0, 12],
            [-2.05, -9.8, -2.0],
            [4, 14, 8],
            0.0,
            false,
            pose.neck,
            bodyTransform,
            None,
        );
        push(
            [58, 0],
            [-1.0, -11.5, 5.0],
            [2, 16, 4],
            0.0,
            false,
            pose.mane,
            bodyTransform,
            None,
        );

        if !child && entity.horseSaddled() {
            push(
                [80, 12],
                [-2.5, -10.1, -7.0],
                [5, 5, 12],
                0.2,
                false,
                pose.faceRopes,
                None,
                None,
            );
            push(
                [80, 0],
                [-5.0, 0.0, -3.0],
                [10, 1, 8],
                0.0,
                false,
                pose.saddleBottom,
                None,
                None,
            );
            push(
                [106, 9],
                [-1.5, -1.0, -3.0],
                [3, 1, 2],
                0.0,
                false,
                pose.saddleFront,
                None,
                None,
            );
            push(
                [80, 9],
                [-4.0, -1.0, 3.0],
                [8, 1, 2],
                0.0,
                false,
                pose.saddleBack,
                None,
                None,
            );
            push(
                [70, 0],
                [-0.5, 0.0, -0.5],
                [1, 6, 1],
                0.0,
                false,
                pose.leftSaddleRope,
                None,
                None,
            );
            push(
                [74, 0],
                [-0.5, 6.0, -1.0],
                [1, 2, 2],
                0.0,
                false,
                pose.leftSaddleMetal,
                None,
                None,
            );
            push(
                [80, 0],
                [-0.5, 0.0, -0.5],
                [1, 6, 1],
                0.0,
                false,
                pose.rightSaddleRope,
                None,
                None,
            );
            push(
                [74, 4],
                [-0.5, 6.0, -1.0],
                [1, 2, 2],
                0.0,
                false,
                pose.rightSaddleMetal,
                None,
                None,
            );
            push(
                [74, 13],
                [1.5, -8.0, -4.0],
                [1, 2, 2],
                0.0,
                false,
                pose.leftFaceMetal,
                None,
                None,
            );
            push(
                [74, 13],
                [-2.5, -8.0, -4.0],
                [1, 2, 2],
                0.0,
                false,
                pose.rightFaceMetal,
                None,
                None,
            );
            if entity.horseBeingRidden() {
                push(
                    [44, 10],
                    [2.6, -6.0, -6.0],
                    [0, 3, 16],
                    0.0,
                    false,
                    pose.leftRein,
                    None,
                    None,
                );
                push(
                    [44, 5],
                    [-2.6, -6.0, -6.0],
                    [0, 3, 16],
                    0.0,
                    false,
                    pose.rightRein,
                    None,
                    None,
                );
            }
        }
        if !child && variant.chestHorse() && entity.horseChested() {
            push(
                [0, 34],
                [-3.0, 0.0, 0.0],
                [8, 8, 3],
                0.0,
                false,
                pose.muleLeftChest,
                None,
                None,
            );
            push(
                [0, 47],
                [-3.0, 0.0, 0.0],
                [8, 8, 3],
                0.0,
                false,
                pose.muleRightChest,
                None,
                None,
            );
        }
        boxes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::client::entity::EntityOtherClient::{
        ClientEntityKind, MobEntityType,
    };
    use crate::net::minecraft::client::renderer::entity::RenderLivingBase::RenderLivingBase;
    use crate::net::minecraft::network::datasync::DataSerializers::DataValue;

    #[test]
    fn donkey_uses_long_ears_and_chest_only_when_synced() {
        let mut entity = EntityOtherClient::new(
            1,
            None,
            ClientEntityKind::Mob {
                entityType: MobEntityType::fromId(31).unwrap(),
            },
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );
        entity.applyMetadata([(15, DataValue::Boolean(true))]);
        let input = RenderLivingBase::renderInput(&entity, 0.0, 1.0);
        let boxes = ModelHorse::boxes(
            ModelHorse::pose(input, &entity, 0.0),
            input,
            &entity,
            HorseModelVariant::Donkey,
            0.0,
        );
        assert!(boxes.iter().any(|b| b.texture == [0, 47]));
        assert!(boxes.iter().any(|b| b.size == [2, 7, 1]));
    }

    #[test]
    fn horse_child_has_distinct_leg_body_and_head_transforms() {
        let mut entity = EntityOtherClient::new(
            2,
            None,
            ClientEntityKind::Mob {
                entityType: MobEntityType::fromId(100).unwrap(),
            },
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );
        entity.applyMetadata([(12, DataValue::Boolean(true))]);
        let input = RenderLivingBase::renderInput(&entity, 0.0, 1.0);
        let boxes = ModelHorse::boxes(
            ModelHorse::pose(input, &entity, 0.0),
            input,
            &entity,
            HorseModelVariant::Horse,
            0.0,
        );
        let leg = boxes
            .iter()
            .find(|b| b.texture == [78, 29])
            .unwrap()
            .childTransform
            .unwrap();
        let body = boxes
            .iter()
            .find(|b| b.texture == [0, 34])
            .unwrap()
            .childTransform
            .unwrap();
        let head = boxes
            .iter()
            .find(|b| b.texture == [0, 0] && b.size == [5, 5, 7])
            .unwrap()
            .childTransform
            .unwrap();
        assert_ne!(leg.scale, body.scale);
        assert_ne!(head.scale, body.scale);
    }
}
