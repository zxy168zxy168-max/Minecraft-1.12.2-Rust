use crate::net::minecraft::client::entity::EntityOtherClient::EntityOtherClient;
use crate::net::minecraft::client::model::ModelBoxGeometry::{
    model_box_geometry, MODEL_BOX_FACE_INDICES,
};
use crate::net::minecraft::client::renderer::entity::RenderLivingBase::{
    LivingModelMesh, LivingModelVertex,
};

const MODEL_SCALE: f32 = 0.0625;
type Mat4 = [[f32; 4]; 4];

pub struct ModelDragon;

impl ModelDragon {
    /// Direct geometry/animation port of MCP 1.12.2 `ModelDragon#render`.
    /// `landingScale` is the exact `max(distanceToEndPodium / 4, 1)` term
    /// used only by Landing/Takeoff; ordinary phases ignore it.
    pub fn mesh(
        entity: &EntityOtherClient,
        partialTicks: f32,
        landingScale: f32,
    ) -> LivingModelMesh {
        let partial = partialTicks.clamp(0.0, 1.0);
        let position = [
            (entity.entity.prevPosX
                + (entity.entity.posX - entity.entity.prevPosX) * partial as f64)
                as f32,
            (entity.entity.prevPosY
                + (entity.entity.posY - entity.entity.prevPosY) * partial as f64)
                as f32,
            (entity.entity.prevPosZ
                + (entity.entity.posZ - entity.entity.prevPosZ) * partial as f64)
                as f32,
        ];

        // RenderDragon#rotateCorpse, followed by RenderLivingBase#prepareScale.
        let corpseYaw = entity.dragonMovementOffsets(7, partial)[0] as f32;
        let corpsePitch = (entity.dragonMovementOffsets(5, partial)[1]
            - entity.dragonMovementOffsets(10, partial)[1]) as f32
            * 10.0;
        let mut root = translation(position);
        root = mul(root, rotation_y_degrees(-corpseYaw));
        root = mul(root, rotation_x_degrees(corpsePitch));
        root = mul(root, translation([0.0, 0.0, 1.0]));
        root = mul(root, scale([-1.0, -1.0, 1.0]));
        root = mul(root, translation([0.0, -1.501, 0.0]));

        let f = entity.dragonPrevAnimTime
            + (entity.dragonAnimTime - entity.dragonPrevAnimTime) * partial;
        let jawAngle = ((f * std::f32::consts::TAU).sin() + 1.0) * 0.2;
        let mut bob = (f * std::f32::consts::TAU - 1.0).sin() + 1.0;
        bob = (bob * bob + bob * 2.0) * 0.05;
        root = mul(root, translation([0.0, bob - 2.0, -3.0]));
        root = mul(root, rotation_x_degrees(bob * 2.0));

        let mut mesh = LivingModelMesh::default();
        let baseOffsets = entity.dragonMovementOffsets(6, partial);
        let f6 = update_rotations(
            entity.dragonMovementOffsets(5, partial)[0]
                - entity.dragonMovementOffsets(10, partial)[0],
        );
        let f7 = update_rotations(entity.dragonMovementOffsets(5, partial)[0] + f6 as f64 / 2.0);
        let phase = entity.dragonPhaseId();
        let mut y = 20.0_f32;
        let mut z = -12.0_f32;
        let mut x = 0.0_f32;
        let cycle = f * std::f32::consts::TAU;

        for i in 0..5 {
            let sample = entity.dragonMovementOffsets(5 - i, partial);
            let wave = ((i as f32) * 0.45 + cycle).cos() * 0.15;
            let rotY = update_rotations(sample[0] - baseOffsets[0]).to_radians() * 1.5;
            let rotX = wave
                + head_part_y_offset(phase, i, baseOffsets, sample, landingScale).to_radians()
                    * 1.5
                    * 5.0;
            let rotZ = -update_rotations(sample[0] - f7 as f64).to_radians() * 1.5;
            let part = part_matrix(root, [x, y, z], [rotX, rotY, rotZ]);
            append_spine(&mut mesh, part, false);
            y += rotX.sin() * 10.0;
            z -= rotY.cos() * rotX.cos() * 10.0;
            x -= rotY.sin() * rotX.cos() * 10.0;
        }

        let headSample = entity.dragonMovementOffsets(0, partial);
        let headRotY = update_rotations(headSample[0] - baseOffsets[0]).to_radians();
        let headRotX =
            update_rotations(
                head_part_y_offset(phase, 6, baseOffsets, headSample, landingScale) as f64,
            )
            .to_radians()
                * 1.5
                * 5.0;
        let headRotZ = -update_rotations(headSample[0] - f7 as f64).to_radians();
        let head = part_matrix(root, [x, y, z], [headRotX, headRotY, headRotZ]);
        append_head(&mut mesh, head, jawAngle, false);

        // ModelDragon's body/wing/leg pushMatrix group.
        let mut bodyRoot = mul(root, translation([0.0, 1.0, 0.0]));
        bodyRoot = mul(bodyRoot, rotation_z_degrees(-f6 * 1.5));
        bodyRoot = mul(bodyRoot, translation([0.0, -1.0, 0.0]));
        append_body(
            &mut mesh,
            part_matrix(bodyRoot, [0.0, 4.0, 8.0], [0.0; 3]),
            false,
        );

        let wingX = 0.125 - cycle.cos() * 0.2;
        let wingY = 0.25;
        let wingZ = (cycle.sin() + 0.125) * 0.8;
        let wingTipZ = -((cycle + 2.0).sin() + 0.5) * 0.75;
        for side in 0..2 {
            let mirrored = side == 1;
            let sideRoot = if mirrored {
                mul(bodyRoot, scale([-1.0, 1.0, 1.0]))
            } else {
                bodyRoot
            };
            let wing = part_matrix(sideRoot, [-12.0, 5.0, 2.0], [wingX, wingY, wingZ]);
            append_wing(&mut mesh, wing, wingTipZ, mirrored);
            let front = part_matrix(sideRoot, [-12.0, 20.0, 2.0], [1.3 + bob * 0.1, 0.0, 0.0]);
            append_front_leg(
                &mut mesh,
                front,
                -0.5 - bob * 0.1,
                0.75 + bob * 0.1,
                mirrored,
            );
            let rear = part_matrix(sideRoot, [-16.0, 16.0, 42.0], [1.0 + bob * 0.1, 0.0, 0.0]);
            append_rear_leg(&mut mesh, rear, 0.5 + bob * 0.1, 0.75 + bob * 0.1, mirrored);
        }

        // Tail is outside the body group and uses the main model root.
        let mut tailWave = 0.0_f32;
        y = 10.0;
        z = 60.0;
        x = 0.0;
        let tailBase = entity.dragonMovementOffsets(11, partial);
        for k in 0..12 {
            let sample = entity.dragonMovementOffsets(12 + k, partial);
            tailWave += ((k as f32) * 0.45 + cycle).sin() * 0.05000000074505806;
            let rotY = (update_rotations(sample[0] - tailBase[0]) * 1.5 + 180.0).to_radians();
            let rotX = tailWave + (sample[1] - tailBase[1]) as f32 * 0.017453292 * 1.5 * 5.0;
            let rotZ = update_rotations(sample[0] - f7 as f64).to_radians() * 1.5;
            let part = part_matrix(root, [x, y, z], [rotX, rotY, rotZ]);
            append_spine(&mut mesh, part, false);
            y += rotX.sin() * 10.0;
            z -= rotY.cos() * rotX.cos() * 10.0;
            x -= rotY.sin() * rotX.cos() * 10.0;
        }
        mesh
    }
}

fn head_part_y_offset(
    phase: i32,
    index: i32,
    base: [f64; 3],
    sample: [f64; 3],
    landingScale: f32,
) -> f32 {
    if phase != 3 && phase != 4 {
        if matches!(phase, 5 | 6 | 7 | 10) {
            index as f32
        } else if index == 6 {
            0.0
        } else {
            (sample[1] - base[1]) as f32
        }
    } else {
        index as f32 / landingScale.max(1.0)
    }
}

fn update_rotations(mut value: f64) -> f32 {
    while value >= 180.0 {
        value -= 360.0;
    }
    while value < -180.0 {
        value += 360.0;
    }
    value as f32
}

fn append_head(mesh: &mut LivingModelMesh, matrix: Mat4, jawAngle: f32, reverse: bool) {
    append_box(
        mesh,
        [176, 44],
        [-6.0, -1.0, -24.0],
        [12, 5, 16],
        false,
        matrix,
        reverse,
    );
    append_box(
        mesh,
        [112, 30],
        [-8.0, -8.0, -10.0],
        [16, 16, 16],
        false,
        matrix,
        reverse,
    );
    append_box(
        mesh,
        [0, 0],
        [-5.0, -12.0, -4.0],
        [2, 4, 6],
        true,
        matrix,
        reverse,
    );
    append_box(
        mesh,
        [112, 0],
        [-5.0, -3.0, -22.0],
        [2, 2, 4],
        true,
        matrix,
        reverse,
    );
    append_box(
        mesh,
        [0, 0],
        [3.0, -12.0, -4.0],
        [2, 4, 6],
        false,
        matrix,
        reverse,
    );
    append_box(
        mesh,
        [112, 0],
        [3.0, -3.0, -22.0],
        [2, 2, 4],
        false,
        matrix,
        reverse,
    );
    let jaw = part_matrix(matrix, [0.0, 4.0, -8.0], [jawAngle, 0.0, 0.0]);
    append_box(
        mesh,
        [176, 65],
        [-6.0, 0.0, -16.0],
        [12, 4, 16],
        false,
        jaw,
        reverse,
    );
}

fn append_spine(mesh: &mut LivingModelMesh, matrix: Mat4, reverse: bool) {
    append_box(
        mesh,
        [192, 104],
        [-5.0, -5.0, -5.0],
        [10, 10, 10],
        false,
        matrix,
        reverse,
    );
    append_box(
        mesh,
        [48, 0],
        [-1.0, -9.0, -3.0],
        [2, 4, 6],
        false,
        matrix,
        reverse,
    );
}

fn append_body(mesh: &mut LivingModelMesh, matrix: Mat4, reverse: bool) {
    append_box(
        mesh,
        [0, 0],
        [-12.0, 0.0, -16.0],
        [24, 24, 64],
        false,
        matrix,
        reverse,
    );
    for z in [-10.0, 10.0, 30.0] {
        append_box(
            mesh,
            [220, 53],
            [-1.0, -6.0, z],
            [2, 6, 12],
            false,
            matrix,
            reverse,
        );
    }
}

fn append_wing(mesh: &mut LivingModelMesh, matrix: Mat4, tipZ: f32, reverse: bool) {
    append_box(
        mesh,
        [112, 88],
        [-56.0, -4.0, -4.0],
        [56, 8, 8],
        false,
        matrix,
        reverse,
    );
    append_box(
        mesh,
        [-56, 88],
        [-56.0, 0.0, 2.0],
        [56, 0, 56],
        false,
        matrix,
        reverse,
    );
    let tip = part_matrix(matrix, [-56.0, 0.0, 0.0], [0.0, 0.0, tipZ]);
    append_box(
        mesh,
        [112, 136],
        [-56.0, -2.0, -2.0],
        [56, 4, 4],
        false,
        tip,
        reverse,
    );
    append_box(
        mesh,
        [-56, 144],
        [-56.0, 0.0, 2.0],
        [56, 0, 56],
        false,
        tip,
        reverse,
    );
}

fn append_front_leg(
    mesh: &mut LivingModelMesh,
    matrix: Mat4,
    tipX: f32,
    footX: f32,
    reverse: bool,
) {
    append_box(
        mesh,
        [112, 104],
        [-4.0, -4.0, -4.0],
        [8, 24, 8],
        false,
        matrix,
        reverse,
    );
    let tip = part_matrix(matrix, [0.0, 20.0, -1.0], [tipX, 0.0, 0.0]);
    append_box(
        mesh,
        [226, 138],
        [-3.0, -1.0, -3.0],
        [6, 24, 6],
        false,
        tip,
        reverse,
    );
    let foot = part_matrix(tip, [0.0, 23.0, 0.0], [footX, 0.0, 0.0]);
    append_box(
        mesh,
        [144, 104],
        [-4.0, 0.0, -12.0],
        [8, 4, 16],
        false,
        foot,
        reverse,
    );
}

fn append_rear_leg(mesh: &mut LivingModelMesh, matrix: Mat4, tipX: f32, footX: f32, reverse: bool) {
    append_box(
        mesh,
        [0, 0],
        [-8.0, -4.0, -8.0],
        [16, 32, 16],
        false,
        matrix,
        reverse,
    );
    let tip = part_matrix(matrix, [0.0, 32.0, -4.0], [tipX, 0.0, 0.0]);
    append_box(
        mesh,
        [196, 0],
        [-6.0, -2.0, 0.0],
        [12, 32, 12],
        false,
        tip,
        reverse,
    );
    let foot = part_matrix(tip, [0.0, 31.0, 4.0], [footX, 0.0, 0.0]);
    append_box(
        mesh,
        [112, 0],
        [-9.0, 0.0, -20.0],
        [18, 6, 24],
        false,
        foot,
        reverse,
    );
}

fn append_box(
    mesh: &mut LivingModelMesh,
    texture: [i32; 2],
    origin: [f32; 3],
    size: [i32; 3],
    mirror: bool,
    matrix: Mat4,
    reverse: bool,
) {
    let geometry = model_box_geometry(texture, origin, size, 0.0, mirror, 256.0, 256.0);
    let base = mesh.vertices.len() as u32;
    mesh.vertices
        .extend(geometry.iter().map(|v| LivingModelVertex {
            position: transform_point(
                matrix,
                [
                    v.position[0] * MODEL_SCALE,
                    v.position[1] * MODEL_SCALE,
                    v.position[2] * MODEL_SCALE,
                ],
            ),
            uv: v.uv,
        }));
    if reverse {
        for tri in MODEL_BOX_FACE_INDICES.chunks_exact(3) {
            mesh.indices
                .extend_from_slice(&[base + tri[0], base + tri[2], base + tri[1]]);
        }
    } else {
        mesh.indices
            .extend(MODEL_BOX_FACE_INDICES.iter().map(|i| base + *i));
    }
}

fn part_matrix(parent: Mat4, pivot: [f32; 3], rotation: [f32; 3]) -> Mat4 {
    let mut out = mul(
        parent,
        translation([
            pivot[0] * MODEL_SCALE,
            pivot[1] * MODEL_SCALE,
            pivot[2] * MODEL_SCALE,
        ]),
    );
    if rotation[2] != 0.0 {
        out = mul(out, rotation_z(rotation[2]));
    }
    if rotation[1] != 0.0 {
        out = mul(out, rotation_y(rotation[1]));
    }
    if rotation[0] != 0.0 {
        out = mul(out, rotation_x(rotation[0]));
    }
    out
}
fn identity() -> Mat4 {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}
fn mul(a: Mat4, b: Mat4) -> Mat4 {
    let mut o = [[0.0; 4]; 4];
    for r in 0..4 {
        for c in 0..4 {
            o[r][c] = (0..4).map(|k| a[r][k] * b[k][c]).sum();
        }
    }
    o
}
fn translation(v: [f32; 3]) -> Mat4 {
    let mut m = identity();
    m[0][3] = v[0];
    m[1][3] = v[1];
    m[2][3] = v[2];
    m
}
fn scale(v: [f32; 3]) -> Mat4 {
    let mut m = identity();
    m[0][0] = v[0];
    m[1][1] = v[1];
    m[2][2] = v[2];
    m
}
fn rotation_x(a: f32) -> Mat4 {
    let (s, c) = a.sin_cos();
    [
        [1., 0., 0., 0.],
        [0., c, -s, 0.],
        [0., s, c, 0.],
        [0., 0., 0., 1.],
    ]
}
fn rotation_y(a: f32) -> Mat4 {
    let (s, c) = a.sin_cos();
    [
        [c, 0., s, 0.],
        [0., 1., 0., 0.],
        [-s, 0., c, 0.],
        [0., 0., 0., 1.],
    ]
}
fn rotation_z(a: f32) -> Mat4 {
    let (s, c) = a.sin_cos();
    [
        [c, -s, 0., 0.],
        [s, c, 0., 0.],
        [0., 0., 1., 0.],
        [0., 0., 0., 1.],
    ]
}
fn rotation_x_degrees(a: f32) -> Mat4 {
    rotation_x(a.to_radians())
}
fn rotation_y_degrees(a: f32) -> Mat4 {
    rotation_y(a.to_radians())
}
fn rotation_z_degrees(a: f32) -> Mat4 {
    rotation_z(a.to_radians())
}
fn transform_point(m: Mat4, p: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * p[0] + m[0][1] * p[1] + m[0][2] * p[2] + m[0][3],
        m[1][0] * p[0] + m[1][1] * p[1] + m[1][2] * p[2] + m[1][3],
        m[2][0] * p[0] + m[2][1] * p[1] + m[2][2] * p[2] + m[2][3],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rotation_wrapping_matches_mcp_range() {
        assert_eq!(update_rotations(181.0), -179.0);
        assert_eq!(update_rotations(-181.0), 179.0);
    }
    #[test]
    fn sitting_phase_uses_segment_index_for_head_y_offset() {
        assert_eq!(head_part_y_offset(5, 4, [0.0; 3], [0.0; 3], 1.0), 4.0);
    }
}
