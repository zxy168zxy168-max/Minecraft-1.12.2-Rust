/// MCP 1.12.2 `net.minecraft.client.renderer.block.model.BlockFaceUV`.
///
/// The JSON rectangle remains in the vanilla 0..16 coordinate system. Vertex
/// access applies the face-local 0/90/180/270 degree rotation exactly as the
/// Java implementation does.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockFaceUV {
    pub uvs: [f32; 4],
    pub rotation: i32,
}

impl BlockFaceUV {
    pub fn new(uvs: [f32; 4], rotation: i32) -> Self {
        Self {
            uvs,
            rotation: rotation.rem_euclid(360),
        }
    }

    pub fn getVertexU(self, vertexIndex: usize) -> f32 {
        let index = self.getVertexRotated(vertexIndex);
        if index == 0 || index == 1 {
            self.uvs[0]
        } else {
            self.uvs[2]
        }
    }

    pub fn getVertexV(self, vertexIndex: usize) -> f32 {
        let index = self.getVertexRotated(vertexIndex);
        if index == 0 || index == 3 {
            self.uvs[1]
        } else {
            self.uvs[3]
        }
    }

    fn getVertexRotated(self, vertexIndex: usize) -> usize {
        (vertexIndex + (self.rotation / 90) as usize) % 4
    }

    pub fn getVertexRotatedRev(self, vertexIndex: usize) -> usize {
        (vertexIndex + (4 - (self.rotation / 90) as usize)) % 4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertex_rotation_matches_mcp_block_face_uv() {
        let face = BlockFaceUV::new([2.0, 4.0, 10.0, 12.0], 90);
        assert_eq!(face.getVertexU(0), 2.0);
        assert_eq!(face.getVertexV(0), 12.0);
        assert_eq!(face.getVertexRotatedRev(0), 3);
    }
}
