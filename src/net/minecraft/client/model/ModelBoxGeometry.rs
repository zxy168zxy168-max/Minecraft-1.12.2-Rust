use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LocalModelBoxVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ModelBoxGeometryKey {
    texture: [i32; 2],
    originBits: [u32; 3],
    size: [i32; 3],
    deltaBits: u32,
    mirror: bool,
    textureWidthBits: u32,
    textureHeightBits: u32,
}

pub const MODEL_BOX_FACE_INDICES: [u32; 36] = [
    0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 8, 9, 10, 8, 10, 11, 12, 13, 14, 12, 14, 15, 16, 17, 18,
    16, 18, 19, 20, 21, 22, 20, 22, 23,
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelBoxRotation {
    sinX: f32,
    cosX: f32,
    sinY: f32,
    cosY: f32,
    sinZ: f32,
    cosZ: f32,
}

impl ModelBoxRotation {
    pub fn new(rotation: [f32; 3]) -> Self {
        let (sinX, cosX) = rotation[0].sin_cos();
        let (sinY, cosY) = rotation[1].sin_cos();
        let (sinZ, cosZ) = rotation[2].sin_cos();
        Self {
            sinX,
            cosX,
            sinY,
            cosY,
            sinZ,
            cosZ,
        }
    }

    /// Exact ModelRenderer order: rotate X, then Y, then Z.
    pub fn apply(self, point: [f32; 3]) -> [f32; 3] {
        let afterX = [
            point[0],
            point[1] * self.cosX - point[2] * self.sinX,
            point[1] * self.sinX + point[2] * self.cosX,
        ];
        let afterY = [
            afterX[0] * self.cosY + afterX[2] * self.sinY,
            afterX[1],
            -afterX[0] * self.sinY + afterX[2] * self.cosY,
        ];
        [
            afterY[0] * self.cosZ - afterY[1] * self.sinZ,
            afterY[0] * self.sinZ + afterY[1] * self.cosZ,
            afterY[2],
        ]
    }
}

thread_local! {
    /// Retain immutable cuboid geometry separately from the current pose while
    /// preserving the exact MCP 1.12.2 ModelBox face and UV construction.
    /// Each render worker keeps a
    /// lock-free local cache; pose, parent pivots, child transforms and world
    /// placement remain per-frame responsibilities of RenderPlayer and
    /// RenderLivingBase.
    static MODEL_BOX_GEOMETRY_CACHE: RefCell<HashMap<ModelBoxGeometryKey, Arc<[LocalModelBoxVertex; 24]>>> =
        RefCell::new(HashMap::new());
}

pub fn model_box_geometry(
    texture: [i32; 2],
    origin: [f32; 3],
    size: [i32; 3],
    delta: f32,
    mirror: bool,
    textureWidth: f32,
    textureHeight: f32,
) -> Arc<[LocalModelBoxVertex; 24]> {
    let key = ModelBoxGeometryKey {
        texture,
        originBits: origin.map(f32::to_bits),
        size,
        deltaBits: delta.to_bits(),
        mirror,
        textureWidthBits: textureWidth.to_bits(),
        textureHeightBits: textureHeight.to_bits(),
    };
    MODEL_BOX_GEOMETRY_CACHE.with(|cache| {
        if let Some(geometry) = cache.borrow().get(&key).cloned() {
            return geometry;
        }
        let geometry = Arc::new(build_model_box_geometry(
            texture,
            origin,
            size,
            delta,
            mirror,
            textureWidth,
            textureHeight,
        ));
        cache.borrow_mut().insert(key, Arc::clone(&geometry));
        geometry
    })
}

fn build_model_box_geometry(
    texture: [i32; 2],
    origin: [f32; 3],
    size: [i32; 3],
    delta: f32,
    mirror: bool,
    textureWidth: f32,
    textureHeight: f32,
) -> [LocalModelBoxVertex; 24] {
    let [dx, dy, dz] = size;
    let [x, y, z] = origin;
    let mut x1 = x - delta;
    let y1 = y - delta;
    let z1 = z - delta;
    let mut x2 = x + dx as f32 + delta;
    let y2 = y + dy as f32 + delta;
    let z2 = z + dz as f32 + delta;
    if mirror {
        std::mem::swap(&mut x1, &mut x2);
    }
    let points = [
        [x1, y1, z1],
        [x2, y1, z1],
        [x2, y2, z1],
        [x1, y2, z1],
        [x1, y1, z2],
        [x2, y1, z2],
        [x2, y2, z2],
        [x1, y2, z2],
    ];
    let [u, v] = texture;
    let faces = [
        (
            [5usize, 1, 2, 6],
            [u + dz + dx, v + dz, u + dz + dx + dz, v + dz + dy],
        ),
        ([0usize, 4, 7, 3], [u, v + dz, u + dz, v + dz + dy]),
        ([5usize, 4, 0, 1], [u + dz, v, u + dz + dx, v + dz]),
        (
            [2usize, 3, 7, 6],
            [u + dz + dx, v + dz, u + dz + dx + dx, v],
        ),
        (
            [1usize, 0, 3, 2],
            [u + dz, v + dz, u + dz + dx, v + dz + dy],
        ),
        (
            [4usize, 5, 6, 7],
            [u + dz + dx + dz, v + dz, u + dz + dx + dz + dx, v + dz + dy],
        ),
    ];
    let mut output = [LocalModelBoxVertex {
        position: [0.0; 3],
        uv: [0.0; 2],
    }; 24];
    let mut cursor = 0usize;
    for (order, uvRect) in faces {
        let u1 = uvRect[0] as f32 / textureWidth;
        let v1 = uvRect[1] as f32 / textureHeight;
        let u2 = uvRect[2] as f32 / textureWidth;
        let v2 = uvRect[3] as f32 / textureHeight;
        let uvs = [[u2, v1], [u1, v1], [u1, v2], [u2, v2]];
        let mut textured = [
            (order[0], uvs[0]),
            (order[1], uvs[1]),
            (order[2], uvs[2]),
            (order[3], uvs[3]),
        ];
        // MCP ModelBox flips the fully textured quad after swapping the X
        // endpoints. Reversing position/UV pairs preserves that exact order.
        if mirror {
            textured.reverse();
        }
        for (pointIndex, uv) in textured {
            output[cursor] = LocalModelBoxVertex {
                position: points[pointIndex],
                uv,
            };
            cursor += 1;
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_model_boxes_reuse_local_geometry() {
        let first = model_box_geometry(
            [0, 0],
            [-4.0, -8.0, -4.0],
            [8, 8, 8],
            0.0,
            false,
            64.0,
            64.0,
        );
        let second = model_box_geometry(
            [0, 0],
            [-4.0, -8.0, -4.0],
            [8, 8, 8],
            0.0,
            false,
            64.0,
            64.0,
        );
        assert!(Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn mirrored_geometry_preserves_modelbox_flip_face_order() {
        let normal = model_box_geometry([0, 0], [0.0, 0.0, 0.0], [2, 3, 4], 0.0, false, 64.0, 32.0);
        let mirrored =
            model_box_geometry([0, 0], [0.0, 0.0, 0.0], [2, 3, 4], 0.0, true, 64.0, 32.0);
        assert_eq!(normal[0].uv, mirrored[3].uv);
        assert_eq!(normal[3].uv, mirrored[0].uv);
        assert_eq!(normal.len(), mirrored.len());
    }

    #[test]
    fn precomputed_rotation_matches_sequential_modelrenderer_order() {
        let point = [1.25, -2.5, 0.75];
        let angles = [0.37, -0.61, 1.13];
        let rotate_x = |p: [f32; 3], a: f32| {
            let (sin, cos) = a.sin_cos();
            [p[0], p[1] * cos - p[2] * sin, p[1] * sin + p[2] * cos]
        };
        let rotate_y = |p: [f32; 3], a: f32| {
            let (sin, cos) = a.sin_cos();
            [p[0] * cos + p[2] * sin, p[1], -p[0] * sin + p[2] * cos]
        };
        let rotate_z = |p: [f32; 3], a: f32| {
            let (sin, cos) = a.sin_cos();
            [p[0] * cos - p[1] * sin, p[0] * sin + p[1] * cos, p[2]]
        };
        let expected = rotate_z(rotate_y(rotate_x(point, angles[0]), angles[1]), angles[2]);
        let actual = ModelBoxRotation::new(angles).apply(point);
        for axis in 0..3 {
            assert!((actual[axis] - expected[axis]).abs() < 1.0e-6);
        }
    }

    #[test]
    fn texture_dimensions_are_part_of_geometry_identity() {
        let skin = model_box_geometry(
            [0, 0],
            [-4.0, -8.0, -4.0],
            [8, 8, 8],
            0.0,
            false,
            64.0,
            64.0,
        );
        let armor = model_box_geometry(
            [0, 0],
            [-4.0, -8.0, -4.0],
            [8, 8, 8],
            0.0,
            false,
            64.0,
            32.0,
        );
        assert_ne!(skin[0].uv, armor[0].uv);
    }
}
