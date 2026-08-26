use crate::net::minecraft::client::model::ModelBoxGeometry::{
    model_box_geometry, ModelBoxRotation, MODEL_BOX_FACE_INDICES,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VehicleModelVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct VehicleModelMesh {
    pub vertices: Vec<VehicleModelVertex>,
    pub indices: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VehicleBox {
    pub texture: [i32; 2],
    pub textureSize: [f32; 2],
    pub origin: [f32; 3],
    pub size: [i32; 3],
    pub pivot: [f32; 3],
    /// MCP ModelRenderer order: X, then Y, then Z fields; OpenGL application
    /// is equivalent to Rz * Ry * Rx for a submitted vertex.
    pub rotation: [f32; 3],
}

pub fn append_box(mesh: &mut VehicleModelMesh, spec: VehicleBox) {
    let geometry = model_box_geometry(
        spec.texture,
        spec.origin,
        spec.size,
        0.0,
        false,
        spec.textureSize[0],
        spec.textureSize[1],
    );
    let rotation = ModelBoxRotation::new(spec.rotation);
    let base = mesh.vertices.len() as u32;
    mesh.vertices.reserve(geometry.len());
    for vertex in geometry.iter() {
        let point = rotation.apply(vertex.position);
        mesh.vertices.push(VehicleModelVertex {
            position: [
                (point[0] + spec.pivot[0]) * 0.0625,
                (point[1] + spec.pivot[1]) * 0.0625,
                (point[2] + spec.pivot[2]) * 0.0625,
            ],
            uv: vertex.uv,
        });
    }
    mesh.indices.reserve(MODEL_BOX_FACE_INDICES.len());
    mesh.indices
        .extend(MODEL_BOX_FACE_INDICES.iter().map(|index| base + index));
}
