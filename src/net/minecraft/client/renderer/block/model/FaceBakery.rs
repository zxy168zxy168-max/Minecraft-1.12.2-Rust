use crate::net::minecraft::client::renderer::block::model::BlockFaceUV::BlockFaceUV;
use crate::net::minecraft::client::renderer::block::model::ModelBlock::{
    BlockPart, BlockPartRotation,
};
use crate::net::minecraft::client::renderer::block::model::ModelRotation::ModelRotation;
use crate::net::minecraft::util::EnumFacing::EnumFacing;

const EPSILON: f32 = 1.0e-5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BakedGeometry {
    pub positions: [[f32; 3]; 4],
    pub uvs: [[f32; 2]; 4],
    pub face: EnumFacing,
}

/// MCP 1.12.2 `FaceBakery` geometry/UV port. Texture-atlas interpolation is
/// deferred, but the 0..16 face UV values, ModelRotation vertex permutation,
/// UV-lock table and `applyFacing` canonicalisation are retained.
pub struct FaceBakery;

impl FaceBakery {
    pub fn makeBakedQuad(
        part: &BlockPart,
        facing: EnumFacing,
        mut faceUv: BlockFaceUV,
        modelRotation: ModelRotation,
        partRotation: Option<&BlockPartRotation>,
        uvLocked: bool,
    ) -> BakedGeometry {
        if uvLocked {
            faceUv = apply_uv_lock(faceUv, facing, modelRotation);
        }

        let originalPositions = face_positions(part, facing);
        let mut positions = [[0.0_f32; 3]; 4];
        let mut uvs = [[0.0_f32; 2]; 4];

        for vertexIndex in 0..4 {
            let mut position = originalPositions[vertexIndex];
            if let Some(rotation) = partRotation {
                position = rotate_part(position, rotation);
            }
            position = modelRotation.transformVertex(position);
            snap_vertex(&mut position);

            let storeIndex = modelRotation.rotateVertex(facing, vertexIndex);
            positions[storeIndex] = position;

            // TextureAtlasSprite interpolation in FaceBakery offsets every UV
            // 0.1% toward the opposite corner to suppress atlas bleeding.
            let opposite = (vertexIndex + 2) % 4;
            let u = faceUv.getVertexU(vertexIndex) * 0.999 + faceUv.getVertexU(opposite) * 0.001;
            let v = faceUv.getVertexV(vertexIndex) * 0.999 + faceUv.getVertexV(opposite) * 0.001;
            uvs[storeIndex] = [u / 16.0, v / 16.0];
        }

        let face = facing_from_vertices(positions);
        if partRotation.is_none() {
            apply_facing(&mut positions, &mut uvs, face);
        }

        BakedGeometry {
            positions,
            uvs,
            face,
        }
    }
}

fn face_positions(part: &BlockPart, facing: EnumFacing) -> [[f32; 3]; 4] {
    let west = part.from[0] / 16.0;
    let down = part.from[1] / 16.0;
    let north = part.from[2] / 16.0;
    let east = part.to[0] / 16.0;
    let up = part.to[1] / 16.0;
    let south = part.to[2] / 16.0;
    // Exact EnumFaceDirection.VertexInformation order.
    match facing {
        EnumFacing::Down => [
            [west, down, south],
            [west, down, north],
            [east, down, north],
            [east, down, south],
        ],
        EnumFacing::Up => [
            [west, up, north],
            [west, up, south],
            [east, up, south],
            [east, up, north],
        ],
        EnumFacing::North => [
            [east, up, north],
            [east, down, north],
            [west, down, north],
            [west, up, north],
        ],
        EnumFacing::South => [
            [west, up, south],
            [west, down, south],
            [east, down, south],
            [east, up, south],
        ],
        EnumFacing::West => [
            [west, up, north],
            [west, down, north],
            [west, down, south],
            [west, up, south],
        ],
        EnumFacing::East => [
            [east, up, south],
            [east, down, south],
            [east, down, north],
            [east, up, north],
        ],
    }
}

fn rotate_part(mut position: [f32; 3], rotation: &BlockPartRotation) -> [f32; 3] {
    let origin = [
        rotation.origin[0] / 16.0,
        rotation.origin[1] / 16.0,
        rotation.origin[2] / 16.0,
    ];
    position = subtract(position, origin);
    position = rotate_axis(
        position,
        rotation.axis.as_str(),
        rotation.angle.to_radians(),
    );

    if rotation.rescale {
        let scale = if (rotation.angle.abs() - 22.5).abs() <= EPSILON {
            1.0 / 22.5_f32.to_radians().cos()
        } else {
            1.0 / 45.0_f32.to_radians().cos()
        };
        match rotation.axis.as_str() {
            "x" => {
                position[1] *= scale;
                position[2] *= scale;
            }
            "y" => {
                position[0] *= scale;
                position[2] *= scale;
            }
            "z" => {
                position[0] *= scale;
                position[1] *= scale;
            }
            _ => {}
        }
    }
    add(position, origin)
}

fn apply_facing(positions: &mut [[f32; 3]; 4], uvs: &mut [[f32; 2]; 4], facing: EnumFacing) {
    let oldPositions = *positions;
    let oldUvs = *uvs;
    let mut minimum = [f32::INFINITY; 3];
    let mut maximum = [f32::NEG_INFINITY; 3];
    for position in oldPositions {
        for axis in 0..3 {
            minimum[axis] = minimum[axis].min(position[axis]);
            maximum[axis] = maximum[axis].max(position[axis]);
        }
    }
    let canonical = face_positions_from_bounds(minimum, maximum, facing);
    for (index, target) in canonical.into_iter().enumerate() {
        positions[index] = target;
        if let Some(sourceIndex) = oldPositions
            .iter()
            .position(|candidate| approximately_position(*candidate, target))
        {
            uvs[index] = oldUvs[sourceIndex];
        }
    }
}

fn face_positions_from_bounds(
    minimum: [f32; 3],
    maximum: [f32; 3],
    facing: EnumFacing,
) -> [[f32; 3]; 4] {
    let part = BlockPart {
        from: [minimum[0] * 16.0, minimum[1] * 16.0, minimum[2] * 16.0],
        to: [maximum[0] * 16.0, maximum[1] * 16.0, maximum[2] * 16.0],
        rotation: None,
        shade: true,
        faces: Default::default(),
    };
    face_positions(&part, facing)
}

#[derive(Debug, Clone, Copy)]
enum UvLockRotation {
    Zero,
    Ninety,
    TwoSeventy,
    Inverse,
}

fn apply_uv_lock(face: BlockFaceUV, facing: EnumFacing, rotation: ModelRotation) -> BlockFaceUV {
    let reverse0 = face.getVertexRotatedRev(0);
    let reverse2 = face.getVertexRotatedRev(2);
    let u0 = face.getVertexU(reverse0);
    let v0 = face.getVertexV(reverse0);
    let u2 = face.getVertexU(reverse2);
    let v2 = face.getVertexV(reverse2);

    match uv_lock_rotation(rotation.x(), rotation.y(), facing) {
        UvLockRotation::Zero => BlockFaceUV::new([u0, v0, u2, v2], 0),
        UvLockRotation::TwoSeventy => BlockFaceUV::new([v2, 16.0 - u0, v0, 16.0 - u2], 270),
        UvLockRotation::Inverse => {
            BlockFaceUV::new([16.0 - u0, 16.0 - v0, 16.0 - u2, 16.0 - v2], 0)
        }
        UvLockRotation::Ninety => BlockFaceUV::new([16.0 - v0, u2, 16.0 - v2, u0], 90),
    }
}

/// Exact 96-entry `FaceBakery.UV_ROTATIONS` table, compacted by X/Y row.
fn uv_lock_rotation(x: i32, y: i32, facing: EnumFacing) -> UvLockRotation {
    use UvLockRotation::*;
    let row = match (x.rem_euclid(360), y.rem_euclid(360)) {
        (0, 0) => [Zero, Zero, Zero, Zero, Zero, Zero],
        (0, 90) => [TwoSeventy, Ninety, Zero, Zero, Zero, Zero],
        (0, 180) => [Inverse, Inverse, Zero, Zero, Zero, Zero],
        (0, 270) => [Ninety, TwoSeventy, Zero, Zero, Zero, Zero],
        (90, 0) => [Zero, Inverse, Inverse, Zero, TwoSeventy, Ninety],
        (90, 90) => [Zero, Inverse, Ninety, Ninety, TwoSeventy, Ninety],
        (90, 180) => [Zero, Inverse, Zero, Inverse, TwoSeventy, Ninety],
        (90, 270) => [Zero, Inverse, TwoSeventy, TwoSeventy, TwoSeventy, Ninety],
        (180, 0) => [Zero, Zero, Inverse, Inverse, Inverse, Inverse],
        (180, 90) => [Ninety, TwoSeventy, Inverse, Inverse, Inverse, Inverse],
        (180, 180) => [Inverse, Inverse, Inverse, Inverse, Inverse, Inverse],
        (180, 270) => [TwoSeventy, Ninety, Inverse, Inverse, Inverse, Inverse],
        (270, 0) => [Inverse, Zero, Inverse, Zero, Ninety, TwoSeventy],
        (270, 90) => [Inverse, Zero, TwoSeventy, TwoSeventy, Ninety, TwoSeventy],
        (270, 180) => [Inverse, Zero, Zero, Inverse, Ninety, TwoSeventy],
        (270, 270) => [Inverse, Zero, Ninety, Ninety, Ninety, TwoSeventy],
        _ => [Zero, Zero, Zero, Zero, Zero, Zero],
    };
    row[facing_index(facing)]
}

fn facing_index(facing: EnumFacing) -> usize {
    match facing {
        EnumFacing::Down => 0,
        EnumFacing::Up => 1,
        EnumFacing::North => 2,
        EnumFacing::South => 3,
        EnumFacing::West => 4,
        EnumFacing::East => 5,
    }
}

fn facing_from_vertices(positions: [[f32; 3]; 4]) -> EnumFacing {
    let first = subtract(positions[0], positions[1]);
    let second = subtract(positions[2], positions[1]);
    let normal = normalize(cross(second, first));
    let mut best = EnumFacing::Up;
    let mut bestDot = 0.0_f32;
    for facing in EnumFacing::VALUES {
        let (x, y, z) = facing.offsets();
        let dot = normal[0] * x as f32 + normal[1] * y as f32 + normal[2] * z as f32;
        if dot >= 0.0 && dot > bestDot {
            bestDot = dot;
            best = facing;
        }
    }
    best
}

fn rotate_axis(value: [f32; 3], axis: &str, radians: f32) -> [f32; 3] {
    let (sin, cos) = radians.sin_cos();
    match axis {
        "x" => [
            value[0],
            value[1] * cos - value[2] * sin,
            value[1] * sin + value[2] * cos,
        ],
        "y" => [
            value[0] * cos + value[2] * sin,
            value[1],
            -value[0] * sin + value[2] * cos,
        ],
        "z" => [
            value[0] * cos - value[1] * sin,
            value[0] * sin + value[1] * cos,
            value[2],
        ],
        _ => value,
    }
}

fn snap_vertex(position: &mut [f32; 3]) {
    for component in position {
        if component.abs() < 1.0e-6 {
            *component = 0.0;
        } else if (*component - 1.0).abs() < 1.0e-6 {
            *component = 1.0;
        }
    }
}

fn approximately_position(left: [f32; 3], right: [f32; 3]) -> bool {
    (left[0] - right[0]).abs() <= EPSILON
        && (left[1] - right[1]).abs() <= EPSILON
        && (left[2] - right[2]).abs() <= EPSILON
}

fn add(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] + right[0], left[1] + right[1], left[2] + right[2]]
}

fn subtract(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn cross(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

fn normalize(value: [f32; 3]) -> [f32; 3] {
    let length = (value[0] * value[0] + value[1] * value[1] + value[2] * value[2]).sqrt();
    if length <= EPSILON {
        [0.0, 1.0, 0.0]
    } else {
        [value[0] / length, value[1] / length, value[2] / length]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn cube() -> BlockPart {
        BlockPart {
            from: [0.0, 0.0, 0.0],
            to: [16.0, 16.0, 16.0],
            rotation: None,
            shade: true,
            faces: HashMap::new(),
        }
    }

    #[test]
    fn canonical_faces_point_outward() {
        for facing in EnumFacing::VALUES {
            let baked = FaceBakery::makeBakedQuad(
                &cube(),
                facing,
                BlockFaceUV::new([0.0, 0.0, 16.0, 16.0], 0),
                ModelRotation::new(0, 0),
                None,
                false,
            );
            assert_eq!(baked.face, facing);
        }
    }

    #[test]
    fn y_rotation_keeps_texture_attached_without_uvlock() {
        let baked = FaceBakery::makeBakedQuad(
            &cube(),
            EnumFacing::North,
            BlockFaceUV::new([0.0, 0.0, 16.0, 16.0], 0),
            ModelRotation::new(0, 90),
            None,
            false,
        );
        assert_eq!(baked.face, EnumFacing::East);
        assert!(baked
            .uvs
            .iter()
            .all(|uv| uv[0] >= 0.0 && uv[0] <= 1.0 && uv[1] >= 0.0 && uv[1] <= 1.0));
    }

    #[test]
    fn uvlock_table_matches_mcp_known_entries() {
        assert!(matches!(
            uv_lock_rotation(0, 90, EnumFacing::Up),
            UvLockRotation::Ninety
        ));
        assert!(matches!(
            uv_lock_rotation(180, 180, EnumFacing::West),
            UvLockRotation::Inverse
        ));
        assert!(matches!(
            uv_lock_rotation(270, 0, EnumFacing::East),
            UvLockRotation::TwoSeventy
        ));
    }
}
