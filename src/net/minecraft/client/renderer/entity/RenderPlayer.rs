use crate::net::minecraft::client::model::ModelBiped::{
    ArmPose, BipedMotionInput, BipedPose, ModelBiped,
};
use crate::net::minecraft::client::model::ModelBoxGeometry::{
    model_box_geometry, ModelBoxRotation, MODEL_BOX_FACE_INDICES,
};
use crate::net::minecraft::client::model::ModelPlayer::{ModelBoxSpec, ModelPlayer};
use crate::net::minecraft::util::EnumHandSide::EnumHandSide;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerRenderInput {
    pub position: [f32; 3],
    pub bodyYaw: f32,
    pub headYaw: f32,
    pub headPitch: f32,
    pub limbSwing: f32,
    pub limbSwingAmount: f32,
    pub ageInTicks: f32,
    pub swingProgress: f32,
    pub sneaking: bool,
    pub riding: bool,
    pub slim: bool,
    pub skinParts: u8,
    pub swingingArmIsLeft: bool,
    pub leftArmPose: ArmPose,
    pub rightArmPose: ArmPose,
    pub ticksElytraFlying: i32,
    pub motion: [f64; 3],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerModelVertex {
    pub position: [f32; 3],
    /// 64x64 player skin coordinates, normalised to 0..1.
    pub uv: [f32; 2],
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PlayerModelMesh {
    pub vertices: Vec<PlayerModelVertex>,
    pub indices: Vec<u32>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ElytraCorpseRotation {
    /// Additional MCP RenderPlayer#rotateCorpse X rotation in degrees.
    pub pitchDegrees: f32,
    /// Additional horizontal look/motion alignment in degrees.
    pub yawDegrees: f32,
}

/// CPU model baking equivalent to `RenderPlayer` + `ModelPlayer`. Vulkan only
/// replaces the OpenGL buffer submission; dimensions, UVs and animation are
/// sourced from MCP 1.12.2.
pub struct RenderPlayer;

impl RenderPlayer {
    /// Exact non-sleeping elytra branch of MCP 1.12.2
    /// `RenderPlayer#rotateCorpse`. The ordinary `180-bodyYaw` root rotation is
    /// still owned by the renderer; these are the two rotations applied after
    /// that root transform.
    pub fn elytraCorpseRotation(
        ticksElytraFlying: i32,
        partialTicks: f32,
        rotationPitch: f32,
        look: [f64; 3],
        motion: [f64; 3],
    ) -> ElytraCorpseRotation {
        let flightTicks = ticksElytraFlying as f32 + partialTicks;
        let progress = (flightTicks * flightTicks / 100.0).clamp(0.0, 1.0);
        let pitchDegrees = progress * (-90.0 - rotationPitch);

        let motionHorizontalSquared = motion[0] * motion[0] + motion[2] * motion[2];
        let lookHorizontalSquared = look[0] * look[0] + look[2] * look[2];
        let mut yawDegrees = 0.0_f32;
        if motionHorizontalSquared > 0.0 && lookHorizontalSquared > 0.0 {
            let cosine = (motion[0] * look[0] + motion[2] * look[2])
                / (motionHorizontalSquared.sqrt() * lookHorizontalSquared.sqrt());
            let cross = motion[0] * look[2] - motion[2] * look[0];
            let sign = if cross > 0.0 {
                1.0
            } else if cross < 0.0 {
                -1.0
            } else {
                0.0
            };
            yawDegrees = (sign * cosine.acos()).to_degrees() as f32;
        }
        ElytraCorpseRotation {
            pitchDegrees,
            yawDegrees,
        }
    }

    pub fn buildPose(input: PlayerRenderInput) -> BipedPose {
        ModelBiped::setRotationAnglesWithMotion(
            input.limbSwing,
            input.limbSwingAmount,
            input.ageInTicks,
            wrap_degrees(input.headYaw - input.bodyYaw),
            input.headPitch,
            input.swingProgress,
            input.sneaking,
            input.riding,
            input.slim,
            input.swingingArmIsLeft,
            input.leftArmPose,
            input.rightArmPose,
            BipedMotionInput {
                ticksElytraFlying: input.ticksElytraFlying,
                motion: input.motion,
            },
        )
    }

    pub fn buildMesh(input: PlayerRenderInput) -> PlayerModelMesh {
        let pose = Self::buildPose(input);
        let boxes = ModelPlayer::boxes(pose, input.slim, input.skinParts);
        let mut mesh = PlayerModelMesh {
            vertices: Vec::with_capacity(boxes.len().saturating_mul(24)),
            indices: Vec::with_capacity(boxes.len().saturating_mul(36)),
        };
        for modelBox in boxes {
            add_model_box(
                &mut mesh,
                modelBox,
                input.position,
                input.bodyYaw,
                input.sneaking,
            );
        }
        mesh
    }

    /// Builds arbitrary MCP `ModelRenderer` boxes in the same living-player
    /// root transform used by `RenderPlayer`. Armor layers use 64x32 UVs while
    /// the base player uses 64x64; keeping the dimensions explicit prevents
    /// the left-side mirroring and texture-height errors common in ad-hoc ports.
    pub fn buildBoxesMesh(
        boxes: impl IntoIterator<Item = ModelBoxSpec>,
        position: [f32; 3],
        bodyYaw: f32,
        sneaking: bool,
        textureWidth: f32,
        textureHeight: f32,
    ) -> PlayerModelMesh {
        let boxes = boxes.into_iter();
        let (minimumBoxes, _) = boxes.size_hint();
        let mut mesh = PlayerModelMesh {
            vertices: Vec::with_capacity(minimumBoxes.saturating_mul(24)),
            indices: Vec::with_capacity(minimumBoxes.saturating_mul(36)),
        };
        for modelBox in boxes {
            add_model_box_with_texture(
                &mut mesh,
                modelBox,
                position,
                bodyYaw,
                sneaking,
                textureWidth,
                textureHeight,
            );
        }
        mesh
    }

    pub fn buildLocalBoxesMesh(
        boxes: impl IntoIterator<Item = ModelBoxSpec>,
        textureWidth: f32,
        textureHeight: f32,
    ) -> PlayerModelMesh {
        let boxes = boxes.into_iter();
        let (minimumBoxes, _) = boxes.size_hint();
        let mut mesh = PlayerModelMesh {
            vertices: Vec::with_capacity(minimumBoxes.saturating_mul(24)),
            indices: Vec::with_capacity(minimumBoxes.saturating_mul(36)),
        };
        for modelBox in boxes {
            add_model_box_local_with_texture(&mut mesh, modelBox, textureWidth, textureHeight);
        }
        mesh
    }

    /// Exact MCP 1.12.2 `ModelPlayer#renderCape` geometry. The cape keeps
    /// its dedicated 64x32 texture size instead of inheriting the 64x64 skin
    /// dimensions used by the body and first-person arms.
    pub fn buildCapeMesh() -> PlayerModelMesh {
        let mut mesh = PlayerModelMesh::default();
        add_model_box_local_with_texture(
            &mut mesh,
            ModelBoxSpec {
                texture: [0, 0],
                origin: [-5.0, 0.0, -1.0],
                size: [10, 16, 1],
                delta: 0.0,
                mirror: false,
                pose: Default::default(),
            },
            64.0,
            32.0,
        );
        mesh
    }

    /// CPU equivalent of MCP 1.12.2 `RenderPlayer.renderRightArm` /
    /// `renderLeftArm`. The returned positions remain in ModelRenderer local
    /// space after the original 1/16 scale so ItemRenderer can apply its hand
    /// matrix exactly once.
    pub fn buildFirstPersonArmMesh(
        slim: bool,
        side: EnumHandSide,
        skinParts: u8,
    ) -> PlayerModelMesh {
        let pose = ModelBiped::setRotationAngles(
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            false,
            false,
            slim,
            false,
            ArmPose::Empty,
            ArmPose::Empty,
        );
        let mut armPose = match side {
            EnumHandSide::Right => pose.rightArm,
            EnumHandSide::Left => pose.leftArm,
        };
        // RenderPlayer explicitly clears only rotateAngleX after
        // setRotationAngles; the age-zero Z idle offset remains vanilla.
        armPose.rotation[0] = 0.0;
        let wearMask = match side {
            EnumHandSide::Right => 0x08,
            EnumHandSide::Left => 0x04,
        };
        let mut mesh = PlayerModelMesh::default();
        for modelBox in
            ModelPlayer::firstPersonArmBoxes(armPose, slim, side, skinParts & wearMask != 0)
        {
            add_model_box_local(&mut mesh, modelBox);
        }
        mesh
    }
}

#[derive(Debug, Clone, Copy)]
struct PlayerLocalBoxTransform {
    rotation: ModelBoxRotation,
    pivot: [f32; 3],
}

impl PlayerLocalBoxTransform {
    fn new(spec: ModelBoxSpec) -> Self {
        Self {
            rotation: ModelBoxRotation::new(spec.pose.rotation),
            pivot: spec.pose.pivot,
        }
    }

    fn apply(self, point: [f32; 3]) -> [f32; 3] {
        let point = self.rotation.apply(point);
        let scale = 0.0625;
        [
            (point[0] + self.pivot[0]) * scale,
            (point[1] + self.pivot[1]) * scale,
            (point[2] + self.pivot[2]) * scale,
        ]
    }
}

#[derive(Debug, Clone, Copy)]
struct PlayerWorldBoxTransform {
    local: PlayerLocalBoxTransform,
    position: [f32; 3],
    yawSin: f32,
    yawCos: f32,
    sneaking: bool,
}

impl PlayerWorldBoxTransform {
    fn new(spec: ModelBoxSpec, position: [f32; 3], bodyYaw: f32, sneaking: bool) -> Self {
        let (yawSin, yawCos) = (180.0 - bodyYaw).to_radians().sin_cos();
        Self {
            local: PlayerLocalBoxTransform::new(spec),
            position,
            yawSin,
            yawCos,
            sneaking,
        }
    }

    fn apply(self, point: [f32; 3]) -> [f32; 3] {
        let point = self.local.rotation.apply(point);
        let pivot = self.local.pivot;
        let scale = 0.0625 * 0.9375;
        let mut local = [
            -(point[0] + pivot[0]) * scale,
            1.501 * 0.9375 - (point[1] + pivot[1]) * scale,
            (point[2] + pivot[2]) * scale,
        ];
        if self.sneaking {
            local[1] -= 0.125 + 0.2 * 0.9375;
        }
        let x = local[0] * self.yawCos + local[2] * self.yawSin;
        let z = -local[0] * self.yawSin + local[2] * self.yawCos;
        [
            self.position[0] + x,
            self.position[1] + local[1],
            self.position[2] + z,
        ]
    }
}

fn add_model_box(
    mesh: &mut PlayerModelMesh,
    spec: ModelBoxSpec,
    position: [f32; 3],
    bodyYaw: f32,
    sneaking: bool,
) {
    add_model_box_with_texture(mesh, spec, position, bodyYaw, sneaking, 64.0, 64.0);
}

fn add_model_box_with_texture(
    mesh: &mut PlayerModelMesh,
    spec: ModelBoxSpec,
    position: [f32; 3],
    bodyYaw: f32,
    sneaking: bool,
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
    let transform = PlayerWorldBoxTransform::new(spec, position, bodyYaw, sneaking);
    let base = mesh.vertices.len() as u32;
    mesh.vertices.reserve(geometry.len());
    for vertex in geometry.iter() {
        mesh.vertices.push(PlayerModelVertex {
            position: transform.apply(vertex.position),
            uv: vertex.uv,
        });
    }
    mesh.indices.reserve(MODEL_BOX_FACE_INDICES.len());
    mesh.indices
        .extend(MODEL_BOX_FACE_INDICES.iter().map(|index| base + index));
}

fn add_model_box_local(mesh: &mut PlayerModelMesh, spec: ModelBoxSpec) {
    add_model_box_local_with_texture(mesh, spec, 64.0, 64.0);
}

fn add_model_box_local_with_texture(
    mesh: &mut PlayerModelMesh,
    spec: ModelBoxSpec,
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
    let transform = PlayerLocalBoxTransform::new(spec);
    let base = mesh.vertices.len() as u32;
    mesh.vertices.reserve(geometry.len());
    for vertex in geometry.iter() {
        mesh.vertices.push(PlayerModelVertex {
            position: transform.apply(vertex.position),
            uv: vertex.uv,
        });
    }
    mesh.indices.reserve(MODEL_BOX_FACE_INDICES.len());
    mesh.indices
        .extend(MODEL_BOX_FACE_INDICES.iter().map(|index| base + index));
}

fn model_to_local(mut point: [f32; 3], pivot: [f32; 3], rotation: [f32; 3]) -> [f32; 3] {
    point = rotate_x(point, rotation[0]);
    point = rotate_y(point, rotation[1]);
    point = rotate_z(point, rotation[2]);
    let scale = 0.0625;
    [
        (point[0] + pivot[0]) * scale,
        (point[1] + pivot[1]) * scale,
        (point[2] + pivot[2]) * scale,
    ]
}

fn model_to_world(
    mut point: [f32; 3],
    pivot: [f32; 3],
    rotation: [f32; 3],
    position: [f32; 3],
    bodyYaw: f32,
    sneaking: bool,
) -> [f32; 3] {
    point = rotate_x(point, rotation[0]);
    point = rotate_y(point, rotation[1]);
    point = rotate_z(point, rotation[2]);
    point[0] += pivot[0];
    point[1] += pivot[1];
    point[2] += pivot[2];
    // `RenderLivingBase.prepareScale` + `RenderPlayer.preRenderCallback`.
    let scale = 0.0625 * 0.9375;
    let mut local = [
        -point[0] * scale,
        1.501 * 0.9375 - point[1] * scale,
        point[2] * scale,
    ];
    if sneaking {
        // `RenderPlayer.doRender` lowers the entity origin by 0.125 blocks,
        // then both ModelBiped and the wear-layer pass translate by 0.2 in
        // model space under the inverted 0.9375 living-model scale.
        local[1] -= 0.125 + 0.2 * 0.9375;
    }
    let yaw = (180.0 - bodyYaw).to_radians();
    let x = local[0] * yaw.cos() + local[2] * yaw.sin();
    let z = -local[0] * yaw.sin() + local[2] * yaw.cos();
    [position[0] + x, position[1] + local[1], position[2] + z]
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
fn wrap_degrees(mut value: f32) -> f32 {
    value %= 360.0;
    if value >= 180.0 {
        value -= 360.0;
    }
    if value < -180.0 {
        value += 360.0;
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    fn neutral_input() -> PlayerRenderInput {
        PlayerRenderInput {
            position: [0.0; 3],
            bodyYaw: 0.0,
            headYaw: 0.0,
            headPitch: 0.0,
            limbSwing: 0.0,
            limbSwingAmount: 0.0,
            ageInTicks: 0.0,
            swingProgress: 0.0,
            sneaking: false,
            riding: false,
            slim: false,
            skinParts: 0,
            swingingArmIsLeft: false,
            leftArmPose: ArmPose::Empty,
            rightArmPose: ArmPose::Empty,
            ticksElytraFlying: 0,
            motion: [0.0; 3],
        }
    }

    #[test]
    fn player_model_has_six_faces_per_base_box() {
        let mesh = RenderPlayer::buildMesh(neutral_input());
        assert_eq!(mesh.indices.len(), 6 * 6 * 6);
    }

    #[test]
    fn yaw_zero_faces_positive_z_like_entity_get_look() {
        let front = model_to_world(
            [0.0, 6.0, -2.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            0.0,
            false,
        );
        assert!(front[2] > 0.0);
        let east_turn = model_to_world(
            [0.0, 6.0, -2.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            90.0,
            false,
        );
        assert!(east_turn[0] < 0.0);
    }

    #[test]
    fn sneaking_includes_render_player_origin_offset() {
        let standing = model_to_world(
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            0.0,
            false,
        );
        let sneaking = model_to_world(
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            0.0,
            true,
        );
        assert!(((standing[1] - sneaking[1]) - 0.3125).abs() < 1.0e-6);
    }

    #[test]
    fn precomputed_player_local_transform_matches_legacy_path() {
        let spec = ModelBoxSpec {
            texture: [8, 16],
            origin: [-2.0, -4.0, -1.0],
            size: [4, 8, 2],
            delta: 0.2,
            mirror: true,
            pose: crate::net::minecraft::client::model::ModelBiped::PartPose {
                pivot: [1.25, 2.5, -0.75],
                rotation: [0.39, -0.28, 0.51],
            },
        };
        let point = [1.1, -2.2, 0.8];
        let expected = model_to_local(point, spec.pose.pivot, spec.pose.rotation);
        let actual = PlayerLocalBoxTransform::new(spec).apply(point);
        for axis in 0..3 {
            assert!((actual[axis] - expected[axis]).abs() < 1.0e-6);
        }
    }

    #[test]
    fn precomputed_player_world_transform_matches_legacy_path() {
        let spec = ModelBoxSpec {
            texture: [0, 0],
            origin: [-4.0, -8.0, -4.0],
            size: [8, 8, 8],
            delta: 0.0,
            mirror: false,
            pose: crate::net::minecraft::client::model::ModelBiped::PartPose {
                pivot: [0.5, 1.75, -0.25],
                rotation: [-0.21, 0.33, -0.47],
            },
        };
        let point = [-1.4, 2.1, 0.6];
        let position = [21.0, 70.0, -13.5];
        let expected = model_to_world(
            point,
            spec.pose.pivot,
            spec.pose.rotation,
            position,
            73.0,
            true,
        );
        let actual = PlayerWorldBoxTransform::new(spec, position, 73.0, true).apply(point);
        for axis in 0..3 {
            assert!((actual[axis] - expected[axis]).abs() < 1.0e-5);
        }
    }

    #[test]
    fn cape_mesh_uses_vanilla_ten_by_sixteen_box_and_64x32_uvs() {
        let cape = RenderPlayer::buildCapeMesh();
        assert_eq!(cape.vertices.len(), 24);
        assert_eq!(cape.indices.len(), 36);
        assert!(cape.vertices.iter().all(|vertex| {
            (0.0..=1.0).contains(&vertex.uv[0]) && (0.0..=1.0).contains(&vertex.uv[1])
        }));
        assert!(cape
            .vertices
            .iter()
            .any(|vertex| (vertex.uv[1] - 0.5625).abs() < 1.0e-6));
    }

    #[test]
    fn first_person_arm_mesh_contains_base_and_enabled_sleeve() {
        let base = RenderPlayer::buildFirstPersonArmMesh(false, EnumHandSide::Right, 0);
        let withSleeve = RenderPlayer::buildFirstPersonArmMesh(false, EnumHandSide::Right, 0x08);
        assert_eq!(base.indices.len(), 36);
        assert_eq!(withSleeve.indices.len(), 72);
        assert!(withSleeve.vertices.iter().all(|vertex| {
            vertex.uv[0] >= 0.0 && vertex.uv[0] <= 1.0 && vertex.uv[1] >= 0.0 && vertex.uv[1] <= 1.0
        }));
    }

    #[test]
    fn elytra_corpse_pitch_reaches_vanilla_ninety_degree_pose() {
        let rotation =
            RenderPlayer::elytraCorpseRotation(10, 0.0, 0.0, [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]);
        assert!((rotation.pitchDegrees + 90.0).abs() < 1.0e-6);
        assert!(rotation.yawDegrees.abs() < 1.0e-6);
    }

    #[test]
    fn elytra_corpse_yaw_uses_vanilla_cross_product_sign() {
        let rotation =
            RenderPlayer::elytraCorpseRotation(10, 0.0, 0.0, [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        assert!((rotation.yawDegrees - 90.0).abs() < 1.0e-5);
    }
}
