use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumFacing::EnumFacing;

/// Four-corner result produced by MCP 1.12.2
/// `BlockModelRenderer.AmbientOcclusionFace`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AmbientOcclusionResult {
    pub vertexColorMultiplier: [f32; 4],
    pub vertexBrightness: [u32; 4],
}

impl AmbientOcclusionResult {
    pub const fn flat(multiplier: f32, brightness: u32) -> Self {
        Self {
            vertexColorMultiplier: [multiplier; 4],
            vertexBrightness: [brightness; 4],
        }
    }
}

pub struct BlockModelRenderer;

impl BlockModelRenderer {
    /// Source port of the full-face branch of
    /// `AmbientOcclusionFace.updateVertexBrightness`.
    ///
    /// The same four neighbour directions, diagonal fallbacks, packed-light
    /// averaging and `VertexTranslations` are retained. For inset/non-cubic
    /// quads the four source corner values are bilinearly evaluated at each
    /// baked vertex, which is the Rust equivalent of the original orientation
    /// weight tables without storing the 192 enum constants.
    pub fn updateVertexBrightness(
        _state: IBlockState,
        centerPos: BlockPos,
        direction: EnumFacing,
        positions: [[f32; 3]; 4],
        offsetFace: bool,
        mut getState: impl FnMut(BlockPos) -> IBlockState,
        mut getPackedLight: impl FnMut(BlockPos) -> u32,
    ) -> AmbientOcclusionResult {
        let blockPos = if offsetFace {
            offset(centerPos, direction)
        } else {
            centerPos
        };
        let corners = neighbour_corners(direction);
        let p1 = offset(blockPos, corners[0]);
        let p2 = offset(blockPos, corners[1]);
        let p3 = offset(blockPos, corners[2]);
        let p4 = offset(blockPos, corners[3]);

        let i = getPackedLight(p1);
        let j = getPackedLight(p2);
        let k = getPackedLight(p3);
        let l = getPackedLight(p4);
        let f = ambient_occlusion_light(getState(p1));
        let f1 = ambient_occlusion_light(getState(p2));
        let f2 = ambient_occlusion_light(getState(p3));
        let f3 = ambient_occlusion_light(getState(p4));

        let flag = is_translucent(getState(offset(p1, direction)));
        let flag1 = is_translucent(getState(offset(p2, direction)));
        let flag2 = is_translucent(getState(offset(p3, direction)));
        let flag3 = is_translucent(getState(offset(p4, direction)));

        let (f25, i1) = if !flag2 && !flag {
            (f, i)
        } else {
            let diagonal = offset(p1, corners[2]);
            (
                ambient_occlusion_light(getState(diagonal)),
                getPackedLight(diagonal),
            )
        };
        let (f26, j1) = if !flag3 && !flag {
            (f, i)
        } else {
            let diagonal = offset(p1, corners[3]);
            (
                ambient_occlusion_light(getState(diagonal)),
                getPackedLight(diagonal),
            )
        };
        let (f27, k1) = if !flag2 && !flag1 {
            (f1, j)
        } else {
            let diagonal = offset(p2, corners[2]);
            (
                ambient_occlusion_light(getState(diagonal)),
                getPackedLight(diagonal),
            )
        };
        let (f28, l1) = if !flag3 && !flag1 {
            (f1, j)
        } else {
            let diagonal = offset(p2, corners[3]);
            (
                ambient_occlusion_light(getState(diagonal)),
                getPackedLight(diagonal),
            )
        };

        let adjacent = offset(centerPos, direction);
        let i3 = if offsetFace || !getState(adjacent).getBlock().isOpaqueCube() {
            getPackedLight(adjacent)
        } else {
            getPackedLight(centerPos)
        };
        let f4 = ambient_occlusion_light(getState(if offsetFace { blockPos } else { centerPos }));

        let cornerAo = [
            (f3 + f + f26 + f4) * 0.25,
            (f2 + f + f25 + f4) * 0.25,
            (f2 + f1 + f27 + f4) * 0.25,
            (f3 + f1 + f28 + f4) * 0.25,
        ];
        let cornerBrightness = [
            get_ao_brightness(l, i, j1, i3),
            get_ao_brightness(k, i, i1, i3),
            get_ao_brightness(k, j, k1, i3),
            get_ao_brightness(l, j, l1, i3),
        ];

        let translation = vertex_translation(direction);
        let mut translatedAo = [0.0; 4];
        let mut translatedBrightness = [0; 4];
        for source in 0..4 {
            translatedAo[translation[source]] = cornerAo[source];
            translatedBrightness[translation[source]] = cornerBrightness[source];
        }

        // Full face quads already use the exact source vertex translation.
        // Inset model elements need the source orientation weighting. Bilinear
        // coordinates reproduce those products directly from baked positions.
        if is_full_face(positions, direction) {
            return AmbientOcclusionResult {
                vertexColorMultiplier: translatedAo,
                vertexBrightness: translatedBrightness,
            };
        }

        let mut result = AmbientOcclusionResult::flat(1.0, i3);
        for vertex in 0..4 {
            let (u, v) = face_coordinates(direction, positions[vertex]);
            let weights = [(1.0 - u) * (1.0 - v), u * (1.0 - v), u * v, (1.0 - u) * v];
            result.vertexColorMultiplier[vertex] = translatedAo
                .iter()
                .zip(weights)
                .map(|(value, weight)| value * weight)
                .sum();
            result.vertexBrightness[vertex] = weighted_brightness(translatedBrightness, weights);
        }
        result
    }
}

fn ambient_occlusion_light(state: IBlockState) -> f32 {
    if state.getBlock().isOpaqueCube() {
        0.2
    } else {
        1.0
    }
}

fn is_translucent(state: IBlockState) -> bool {
    !state.getBlock().isOpaqueCube()
}

fn get_ao_brightness(mut br1: u32, mut br2: u32, mut br3: u32, br4: u32) -> u32 {
    if br1 == 0 {
        br1 = br4;
    }
    if br2 == 0 {
        br2 = br4;
    }
    if br3 == 0 {
        br3 = br4;
    }
    br1.wrapping_add(br2).wrapping_add(br3).wrapping_add(br4) >> 2 & 0x00FF00FF
}

fn weighted_brightness(values: [u32; 4], weights: [f32; 4]) -> u32 {
    let sky = values
        .iter()
        .zip(weights)
        .map(|(value, weight)| ((value >> 16) & 255) as f32 * weight)
        .sum::<f32>() as u32
        & 255;
    let block = values
        .iter()
        .zip(weights)
        .map(|(value, weight)| (value & 255) as f32 * weight)
        .sum::<f32>() as u32
        & 255;
    sky << 16 | block
}

fn offset(pos: BlockPos, facing: EnumFacing) -> BlockPos {
    let (x, y, z) = facing.offsets();
    BlockPos::new(pos.x + x, pos.y + y, pos.z + z)
}

fn neighbour_corners(direction: EnumFacing) -> [EnumFacing; 4] {
    match direction {
        EnumFacing::Down => [
            EnumFacing::West,
            EnumFacing::East,
            EnumFacing::North,
            EnumFacing::South,
        ],
        EnumFacing::Up => [
            EnumFacing::East,
            EnumFacing::West,
            EnumFacing::North,
            EnumFacing::South,
        ],
        EnumFacing::North => [
            EnumFacing::Up,
            EnumFacing::Down,
            EnumFacing::East,
            EnumFacing::West,
        ],
        EnumFacing::South => [
            EnumFacing::West,
            EnumFacing::East,
            EnumFacing::Down,
            EnumFacing::Up,
        ],
        EnumFacing::West => [
            EnumFacing::Up,
            EnumFacing::Down,
            EnumFacing::North,
            EnumFacing::South,
        ],
        EnumFacing::East => [
            EnumFacing::Down,
            EnumFacing::Up,
            EnumFacing::North,
            EnumFacing::South,
        ],
    }
}

fn vertex_translation(direction: EnumFacing) -> [usize; 4] {
    match direction {
        EnumFacing::Down | EnumFacing::South => [0, 1, 2, 3],
        EnumFacing::Up => [2, 3, 0, 1],
        EnumFacing::North | EnumFacing::West => [3, 0, 1, 2],
        EnumFacing::East => [1, 2, 3, 0],
    }
}

fn face_coordinates(direction: EnumFacing, position: [f32; 3]) -> (f32, f32) {
    match direction {
        EnumFacing::Down => (position[0], position[2]),
        EnumFacing::Up => (1.0 - position[0], position[2]),
        EnumFacing::North => (1.0 - position[0], 1.0 - position[1]),
        EnumFacing::South => (position[0], 1.0 - position[1]),
        EnumFacing::West => (position[2], 1.0 - position[1]),
        EnumFacing::East => (1.0 - position[2], 1.0 - position[1]),
    }
}

fn is_full_face(positions: [[f32; 3]; 4], direction: EnumFacing) -> bool {
    let mut minimum = [f32::INFINITY; 3];
    let mut maximum = [f32::NEG_INFINITY; 3];
    for position in positions {
        for axis in 0..3 {
            minimum[axis] = minimum[axis].min(position[axis]);
            maximum[axis] = maximum[axis].max(position[axis]);
        }
    }
    let eps = 1.0e-4;
    match direction {
        EnumFacing::Down | EnumFacing::Up => {
            minimum[0] <= eps
                && maximum[0] >= 1.0 - eps
                && minimum[2] <= eps
                && maximum[2] >= 1.0 - eps
        }
        EnumFacing::North | EnumFacing::South => {
            minimum[0] <= eps
                && maximum[0] >= 1.0 - eps
                && minimum[1] <= eps
                && maximum[1] >= 1.0 - eps
        }
        EnumFacing::West | EnumFacing::East => {
            minimum[2] <= eps
                && maximum[2] >= 1.0 - eps
                && minimum[1] <= eps
                && maximum[1] >= 1.0 - eps
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertex_translations_match_mcp_enum() {
        assert_eq!(vertex_translation(EnumFacing::Down), [0, 1, 2, 3]);
        assert_eq!(vertex_translation(EnumFacing::Up), [2, 3, 0, 1]);
        assert_eq!(vertex_translation(EnumFacing::East), [1, 2, 3, 0]);
    }

    #[test]
    fn packed_ao_brightness_averages_channels_independently() {
        assert_eq!(
            get_ao_brightness(0x000F000F, 0x000D000D, 0x000B000B, 0x00090009),
            0x000C000C
        );
    }
}
