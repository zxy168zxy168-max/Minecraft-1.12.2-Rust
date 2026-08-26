use crate::net::minecraft::client::entity::EntityOtherClient::EntityOtherClient;
use crate::net::minecraft::client::model::ModelBiped::PartPose;
use crate::net::minecraft::client::model::ModelBoxGeometry::{
    model_box_geometry, ModelBoxRotation, MODEL_BOX_FACE_INDICES,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LivingModelGroup {
    Head,
    Body,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LivingModelBox {
    pub texture: [i32; 2],
    pub origin: [f32; 3],
    pub size: [i32; 3],
    pub delta: f32,
    pub mirror: bool,
    pub pose: PartPose,
    pub group: LivingModelGroup,
    /// Parent ModelRenderer transforms for exact Java addChild semantics.
    /// Witch hats are four levels deep in 1.12.2, while most models only use
    /// the first entry. Transforms are applied from immediate parent outward.
    pub parentPose: Option<PartPose>,
    pub parentPose2: Option<PartPose>,
    pub parentPose3: Option<PartPose>,
    pub parentPose4: Option<PartPose>,
    /// ModelRenderer offsets are applied outside the part rotation and before
    /// the parent transform. Values are stored in model pixels (1/16 block)
    /// so they can share the exact ModelRenderer transform path.
    pub poseOffset: [f32; 3],
    pub parentOffset: [f32; 3],
    pub parentOffset2: [f32; 3],
    pub parentOffset3: [f32; 3],
    pub parentOffset4: [f32; 3],
    /// Optional child-only outer GlStateManager transform. Horse and llama
    /// models use per-part non-uniform age transforms instead of ModelBase's
    /// generic head/body split. Translation is expressed in world/model scale
    /// units exactly like the existing LivingChildLayout translations.
    pub childTransform: Option<LivingPartTransform>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LivingPartTransform {
    pub scale: [f32; 3],
    pub translation: [f32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LivingChildLayout {
    pub headScale: f32,
    pub headTranslation: [f32; 3],
    pub bodyScale: f32,
    pub bodyTranslation: [f32; 3],
}

impl LivingChildLayout {
    pub const BIPED: Self = Self {
        headScale: 0.75,
        headTranslation: [0.0, 1.0, 0.0],
        bodyScale: 0.5,
        bodyTranslation: [0.0, 1.5, 0.0],
    };

    pub const fn quadruped(childYOffset: f32, childZOffset: f32) -> Self {
        Self {
            headScale: 1.0,
            headTranslation: [0.0, childYOffset * 0.0625, childZOffset * 0.0625],
            bodyScale: 0.5,
            bodyTranslation: [0.0, 1.5, 0.0],
        }
    }

    pub const CHICKEN: Self = Self {
        headScale: 1.0,
        headTranslation: [0.0, 5.0 * 0.0625, 2.0 * 0.0625],
        bodyScale: 0.5,
        bodyTranslation: [0.0, 1.5, 0.0],
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LivingRenderInput {
    pub position: [f32; 3],
    pub bodyYaw: f32,
    pub headYaw: f32,
    pub headPitch: f32,
    pub limbSwing: f32,
    pub limbSwingAmount: f32,
    pub ageInTicks: f32,
    pub swingProgress: f32,
    pub sneaking: bool,
    pub child: bool,
    pub deathRotation: f32,
    pub preScale: f32,
    /// Non-uniform pre-render scale applied by concrete RenderLivingBase
    /// subclasses such as creepers and slimes. Uniform renderers keep all
    /// components equal to `preScale`.
    pub preScaleXYZ: [f32; 3],
    pub childLayout: LivingChildLayout,
    /// Model-local translation used by renderers whose ModelBase overrides
    /// the adult render path, notably `ModelRabbit`.
    pub adultTranslation: [f32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LivingModelVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LivingModelMesh {
    pub vertices: Vec<LivingModelVertex>,
    pub indices: Vec<u32>,
}

/// Renderer-independent subset of MCP 1.12.2 `RenderLivingBase`. Vulkan owns
/// buffer submission, while this type owns interpolation, corpse rotation and
/// ModelRenderer box transforms.
pub struct RenderLivingBase;

impl RenderLivingBase {
    /// MCP `RenderLivingBase#setBrightness` selects the hurt/death red
    /// combiner before rendering the main model. Concrete Vulkan submission
    /// keeps the decision here and only encodes the backend transport flag.
    pub fn shouldApplyHurtBrightness(entity: &EntityOtherClient) -> bool {
        entity.hurtTime > 0 || entity.deathTime > 0
    }

    pub fn interpolateRotation(previous: f32, current: f32, partialTicks: f32) -> f32 {
        let mut difference = current - previous;
        while difference < -180.0 {
            difference += 360.0;
        }
        while difference >= 180.0 {
            difference -= 360.0;
        }
        previous + partialTicks.clamp(0.0, 1.0) * difference
    }

    pub fn renderInput(
        entity: &EntityOtherClient,
        partialTicks: f32,
        preScale: f32,
    ) -> LivingRenderInput {
        let partial = partialTicks.clamp(0.0, 1.0);
        let bodyYaw =
            Self::interpolateRotation(entity.prevRenderYawOffset, entity.renderYawOffset, partial);
        let headYaw =
            Self::interpolateRotation(entity.prevRotationYawHead, entity.rotationYawHead, partial);
        let headPitch = entity.entity.prevRotationPitch
            + (entity.entity.rotationPitch - entity.entity.prevRotationPitch) * partial;
        let limbSwingAmount = (entity.prevLimbSwingAmount
            + (entity.limbSwingAmount - entity.prevLimbSwingAmount) * partial)
            .min(1.0);
        let mut limbSwing = entity.limbSwing - entity.limbSwingAmount * (1.0 - partial);
        let child = entity.isChild();
        if child {
            limbSwing *= 3.0;
        }
        let swingProgress = entity.getSwingProgress(partial);
        let deathRotation = if entity.deathTime > 0 {
            let mut progress = ((entity.deathTime as f32 + partial - 1.0) / 20.0 * 1.6).sqrt();
            if progress > 1.0 {
                progress = 1.0;
            }
            progress * 90.0
        } else {
            0.0
        };
        LivingRenderInput {
            position: [
                (entity.entity.prevPosX
                    + (entity.entity.posX - entity.entity.prevPosX) * partial as f64)
                    as f32,
                (entity.entity.prevPosY
                    + (entity.entity.posY - entity.entity.prevPosY) * partial as f64)
                    as f32,
                (entity.entity.prevPosZ
                    + (entity.entity.posZ - entity.entity.prevPosZ) * partial as f64)
                    as f32,
            ],
            bodyYaw,
            headYaw,
            headPitch,
            limbSwing,
            limbSwingAmount,
            ageInTicks: entity.entity.ticksExisted as f32 + partial,
            swingProgress,
            sneaking: entity.entity.sneaking,
            child,
            deathRotation,
            preScale,
            preScaleXYZ: [preScale; 3],
            childLayout: LivingChildLayout::BIPED,
            adultTranslation: [0.0, if entity.entity.sneaking { 0.2 } else { 0.0 }, 0.0],
        }
    }

    pub fn withChildLayout(
        mut input: LivingRenderInput,
        childLayout: LivingChildLayout,
    ) -> LivingRenderInput {
        input.childLayout = childLayout;
        input
    }

    pub fn withPreScaleXYZ(mut input: LivingRenderInput, scale: [f32; 3]) -> LivingRenderInput {
        input.preScaleXYZ = scale;
        input
    }

    pub fn withAdultTranslation(
        mut input: LivingRenderInput,
        translation: [f32; 3],
    ) -> LivingRenderInput {
        input.adultTranslation = translation;
        input
    }

    pub fn buildMesh(
        input: LivingRenderInput,
        boxes: impl IntoIterator<Item = LivingModelBox>,
        textureWidth: f32,
        textureHeight: f32,
    ) -> LivingModelMesh {
        let boxes = boxes.into_iter();
        let (minimumBoxes, _) = boxes.size_hint();
        let mut mesh = LivingModelMesh {
            vertices: Vec::with_capacity(minimumBoxes.saturating_mul(24)),
            indices: Vec::with_capacity(minimumBoxes.saturating_mul(36)),
        };
        for modelBox in boxes {
            append_box(&mut mesh, modelBox, input, textureWidth, textureHeight);
        }
        mesh
    }
}

#[derive(Debug, Clone, Copy)]
struct LivingPoseTransform {
    rotation: ModelBoxRotation,
    translation: [f32; 3],
}

#[derive(Debug, Clone, Copy)]
struct LivingBoxTransform {
    part: LivingPoseTransform,
    parents: [Option<LivingPoseTransform>; 4],
    modelScale: f32,
    partScale: [f32; 3],
    partTranslation: [f32; 3],
    preScale: [f32; 3],
    deathRotation: Option<(f32, f32)>,
    yawSin: f32,
    yawCos: f32,
    worldPosition: [f32; 3],
}

impl LivingBoxTransform {
    fn new(spec: LivingModelBox, input: LivingRenderInput) -> Self {
        let parentPoses = [
            spec.parentPose,
            spec.parentPose2,
            spec.parentPose3,
            spec.parentPose4,
        ];
        let parentOffsets = [
            spec.parentOffset,
            spec.parentOffset2,
            spec.parentOffset3,
            spec.parentOffset4,
        ];
        let parents = std::array::from_fn(|index| {
            parentPoses[index].map(|pose| LivingPoseTransform {
                rotation: ModelBoxRotation::new(pose.rotation),
                translation: [
                    pose.pivot[0] + parentOffsets[index][0],
                    pose.pivot[1] + parentOffsets[index][1],
                    pose.pivot[2] + parentOffsets[index][2],
                ],
            })
        });
        let (partScale, partTranslation) = if input.child {
            if let Some(transform) = spec.childTransform {
                (transform.scale, transform.translation)
            } else {
                match spec.group {
                    LivingModelGroup::Head => (
                        [input.childLayout.headScale; 3],
                        input.childLayout.headTranslation,
                    ),
                    LivingModelGroup::Body => (
                        [input.childLayout.bodyScale; 3],
                        input.childLayout.bodyTranslation,
                    ),
                }
            }
        } else {
            ([1.0; 3], input.adultTranslation)
        };
        let deathRotation = if input.deathRotation == 0.0 {
            None
        } else {
            Some(input.deathRotation.to_radians().sin_cos())
        };
        let (yawSin, yawCos) = (180.0 - input.bodyYaw).to_radians().sin_cos();
        Self {
            part: LivingPoseTransform {
                rotation: ModelBoxRotation::new(spec.pose.rotation),
                translation: [
                    spec.pose.pivot[0] + spec.poseOffset[0],
                    spec.pose.pivot[1] + spec.poseOffset[1],
                    spec.pose.pivot[2] + spec.poseOffset[2],
                ],
            },
            parents,
            modelScale: 0.0625,
            partScale,
            partTranslation,
            preScale: input.preScaleXYZ,
            deathRotation,
            yawSin,
            yawCos,
            worldPosition: input.position,
        }
    }

    fn apply(self, mut point: [f32; 3]) -> [f32; 3] {
        point = self.part.rotation.apply(point);
        for axis in 0..3 {
            point[axis] += self.part.translation[axis];
        }
        for parent in self.parents.into_iter().flatten() {
            point = parent.rotation.apply(point);
            for axis in 0..3 {
                point[axis] += parent.translation[axis];
            }
        }
        let mut local = [
            -(point[0] * self.modelScale + self.partTranslation[0])
                * self.partScale[0]
                * self.preScale[0],
            (1.501 - (point[1] * self.modelScale + self.partTranslation[1]) * self.partScale[1])
                * self.preScale[1],
            (point[2] * self.modelScale + self.partTranslation[2])
                * self.partScale[2]
                * self.preScale[2],
        ];
        if let Some((sin, cos)) = self.deathRotation {
            local = [
                local[0] * cos - local[1] * sin,
                local[0] * sin + local[1] * cos,
                local[2],
            ];
        }
        let x = local[0] * self.yawCos + local[2] * self.yawSin;
        let z = -local[0] * self.yawSin + local[2] * self.yawCos;
        [
            self.worldPosition[0] + x,
            self.worldPosition[1] + local[1],
            self.worldPosition[2] + z,
        ]
    }
}

fn append_box(
    mesh: &mut LivingModelMesh,
    spec: LivingModelBox,
    input: LivingRenderInput,
    textureWidth: f32,
    textureHeight: f32,
) {
    let geometry = model_box_geometry(
        spec.texture,
        spec.origin,
        spec.size,
        spec.delta,
        spec.mirror,
        textureWidth,
        textureHeight,
    );
    let transform = LivingBoxTransform::new(spec, input);
    let base = mesh.vertices.len() as u32;
    mesh.vertices.reserve(geometry.len());
    for vertex in geometry.iter() {
        mesh.vertices.push(LivingModelVertex {
            position: transform.apply(vertex.position),
            uv: vertex.uv,
        });
    }
    mesh.indices.reserve(MODEL_BOX_FACE_INDICES.len());
    mesh.indices
        .extend(MODEL_BOX_FACE_INDICES.iter().map(|index| base + index));
}

fn model_to_world(
    mut point: [f32; 3],
    pose: PartPose,
    group: LivingModelGroup,
    input: LivingRenderInput,
    parentPoses: [Option<PartPose>; 4],
    offsets: [[f32; 3]; 5],
    childTransform: Option<LivingPartTransform>,
) -> [f32; 3] {
    point = rotate_x(point, pose.rotation[0]);
    point = rotate_y(point, pose.rotation[1]);
    point = rotate_z(point, pose.rotation[2]);
    point[0] += pose.pivot[0] + offsets[0][0];
    point[1] += pose.pivot[1] + offsets[0][1];
    point[2] += pose.pivot[2] + offsets[0][2];
    for (index, parent) in parentPoses.into_iter().enumerate() {
        let Some(parent) = parent else {
            continue;
        };
        point = rotate_x(point, parent.rotation[0]);
        point = rotate_y(point, parent.rotation[1]);
        point = rotate_z(point, parent.rotation[2]);
        point[0] += parent.pivot[0] + offsets[index + 1][0];
        point[1] += parent.pivot[1] + offsets[index + 1][1];
        point[2] += parent.pivot[2] + offsets[index + 1][2];
    }

    let modelScale = 0.0625;
    let (partScale, partTranslation) = if input.child {
        if let Some(transform) = childTransform {
            (transform.scale, transform.translation)
        } else {
            match group {
                LivingModelGroup::Head => (
                    [input.childLayout.headScale; 3],
                    input.childLayout.headTranslation,
                ),
                LivingModelGroup::Body => (
                    [input.childLayout.bodyScale; 3],
                    input.childLayout.bodyTranslation,
                ),
            }
        }
    } else {
        ([1.0; 3], input.adultTranslation)
    };
    let preScale = input.preScaleXYZ;
    let mut local = [
        -(point[0] * modelScale + partTranslation[0]) * partScale[0] * preScale[0],
        (1.501 - (point[1] * modelScale + partTranslation[1]) * partScale[1]) * preScale[1],
        (point[2] * modelScale + partTranslation[2]) * partScale[2] * preScale[2],
    ];
    if input.deathRotation != 0.0 {
        local = rotate_z(local, input.deathRotation.to_radians());
    }
    let yaw = (180.0 - input.bodyYaw).to_radians();
    let x = local[0] * yaw.cos() + local[2] * yaw.sin();
    let z = -local[0] * yaw.sin() + local[2] * yaw.cos();
    [
        input.position[0] + x,
        input.position[1] + local[1],
        input.position[2] + z,
    ]
}

fn rotate_x(p: [f32; 3], a: f32) -> [f32; 3] {
    let (c, s) = (a.cos(), a.sin());
    [p[0], p[1] * c - p[2] * s, p[1] * s + p[2] * c]
}
fn rotate_y(p: [f32; 3], a: f32) -> [f32; 3] {
    let (c, s) = (a.cos(), a.sin());
    [p[0] * c + p[2] * s, p[1], -p[0] * s + p[2] * c]
}
fn rotate_z(p: [f32; 3], a: f32) -> [f32; 3] {
    let (c, s) = (a.cos(), a.sin());
    [p[0] * c - p[1] * s, p[0] * s + p[1] * c, p[2]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_interpolation_uses_shortest_arc() {
        assert!((RenderLivingBase::interpolateRotation(170.0, -170.0, 0.5) - 180.0).abs() < 1.0e-5);
    }

    #[test]
    fn model_renderer_offset_is_not_rotated_with_the_part() {
        let input = LivingRenderInput {
            position: [0.0; 3],
            bodyYaw: 180.0,
            headYaw: 180.0,
            headPitch: 0.0,
            limbSwing: 0.0,
            limbSwingAmount: 0.0,
            ageInTicks: 0.0,
            swingProgress: 0.0,
            sneaking: false,
            child: false,
            deathRotation: 0.0,
            preScale: 1.0,
            preScaleXYZ: [1.0; 3],
            childLayout: LivingChildLayout::BIPED,
            adultTranslation: [0.0; 3],
        };
        let transformed = model_to_world(
            [1.0, 0.0, 0.0],
            PartPose {
                pivot: [0.0; 3],
                rotation: [0.0, 0.0, std::f32::consts::FRAC_PI_2],
            },
            LivingModelGroup::Head,
            input,
            [None; 4],
            [[0.0, 2.0, 0.0], [0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3]],
            None,
        );
        assert!(transformed[0].abs() < 1.0e-6);
        assert!((transformed[1] - (1.501 - 3.0 / 16.0)).abs() < 1.0e-6);
    }

    #[test]
    fn precomputed_living_transform_matches_legacy_modelrenderer_path() {
        let input = LivingRenderInput {
            position: [12.5, 64.25, -8.75],
            bodyYaw: 37.0,
            headYaw: 14.0,
            headPitch: -9.0,
            limbSwing: 0.0,
            limbSwingAmount: 0.0,
            ageInTicks: 0.0,
            swingProgress: 0.0,
            sneaking: false,
            child: true,
            deathRotation: 43.0,
            preScale: 1.0,
            preScaleXYZ: [1.08, 0.94, 1.03],
            childLayout: LivingChildLayout::BIPED,
            adultTranslation: [0.0; 3],
        };
        let spec = LivingModelBox {
            texture: [4, 8],
            origin: [-2.0, -3.0, -1.0],
            size: [4, 6, 2],
            delta: 0.25,
            mirror: true,
            pose: PartPose {
                pivot: [1.0, 2.0, -0.5],
                rotation: [0.31, -0.22, 0.47],
            },
            group: LivingModelGroup::Body,
            parentPose: Some(PartPose {
                pivot: [0.5, 1.5, 0.25],
                rotation: [-0.18, 0.27, -0.39],
            }),
            parentPose2: Some(PartPose {
                pivot: [-0.25, 0.75, 1.0],
                rotation: [0.11, -0.07, 0.19],
            }),
            parentPose3: None,
            parentPose4: None,
            poseOffset: [0.2, -0.1, 0.3],
            parentOffset: [-0.4, 0.2, 0.1],
            parentOffset2: [0.15, -0.3, 0.05],
            parentOffset3: [0.0; 3],
            parentOffset4: [0.0; 3],
            childTransform: Some(LivingPartTransform {
                scale: [0.55, 0.62, 0.58],
                translation: [0.1, 1.25, -0.2],
            }),
        };
        let point = [1.25, -2.0, 0.75];
        let expected = model_to_world(
            point,
            spec.pose,
            spec.group,
            input,
            [
                spec.parentPose,
                spec.parentPose2,
                spec.parentPose3,
                spec.parentPose4,
            ],
            [
                spec.poseOffset,
                spec.parentOffset,
                spec.parentOffset2,
                spec.parentOffset3,
                spec.parentOffset4,
            ],
            spec.childTransform,
        );
        let actual = LivingBoxTransform::new(spec, input).apply(point);
        for axis in 0..3 {
            assert!((actual[axis] - expected[axis]).abs() < 1.0e-5);
        }
    }

    #[test]
    fn hurt_brightness_tracks_entity_living_base_timers() {
        use crate::net::minecraft::client::entity::EntityOtherClient::{
            ClientEntityKind, MobEntityType,
        };
        let mut entity = EntityOtherClient::new(
            1,
            None,
            ClientEntityKind::Mob {
                entityType: MobEntityType::fromId(54).unwrap(),
            },
            0.0,
            64.0,
            0.0,
            0.0,
            0.0,
        );
        assert!(!RenderLivingBase::shouldApplyHurtBrightness(&entity));
        entity.hurtTime = 10;
        assert!(RenderLivingBase::shouldApplyHurtBrightness(&entity));
        entity.hurtTime = 0;
        entity.deathTime = 1;
        assert!(RenderLivingBase::shouldApplyHurtBrightness(&entity));
    }

    #[test]
    fn death_rotation_reaches_ninety_degrees() {
        use crate::net::minecraft::client::entity::EntityOtherClient::{
            ClientEntityKind, MobEntityType,
        };
        let mut entity = EntityOtherClient::new(
            1,
            None,
            ClientEntityKind::Mob {
                entityType: MobEntityType::fromId(54).unwrap(),
            },
            0.0,
            64.0,
            0.0,
            0.0,
            0.0,
        );
        entity.deathTime = 20;
        assert_eq!(
            RenderLivingBase::renderInput(&entity, 1.0, 1.0).deathRotation,
            90.0
        );
    }
}
