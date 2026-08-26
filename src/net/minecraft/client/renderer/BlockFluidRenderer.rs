use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::block::BlockLiquid::LiquidMaterial;
use crate::net::minecraft::block::{BlockLiquid, BlockSlab};
use crate::net::minecraft::client::renderer::color::BlockColors::BlockColors;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::world::biome::BiomeColorHelper::BiomeAccess;
use crate::net::minecraft::world::IBlockAccess::IBlockAccess;

/// Atlas rectangles owned by MCP `BlockFluidRenderer`. Coordinates are exact
/// `TextureAtlasSprite` min/max values; interpolation below uses the original
/// 0..16 sprite coordinate convention.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FluidSprites {
    pub still: [f32; 4],
    pub flow: [f32; 4],
    pub overlay: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FluidVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
    pub packedLight: u32,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct FluidMesh {
    pub vertices: Vec<FluidVertex>,
    pub indices: Vec<u32>,
}

impl FluidMesh {
    fn quad(&mut self, vertices: [FluidVertex; 4]) {
        let base = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&vertices);
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

fn interpolate(rect: [f32; 4], u: f32, v: f32) -> [f32; 2] {
    [
        rect[0] + (rect[2] - rect[0]) * (u / 16.0),
        rect[1] + (rect[3] - rect[1]) * (v / 16.0),
    ]
}

fn vertex(
    position: [f32; 3],
    uv: [f32; 2],
    tint: [f32; 3],
    shade: f32,
    packedLight: u32,
) -> FluidVertex {
    FluidVertex {
        position,
        uv,
        color: [tint[0] * shade, tint[1] * shade, tint[2] * shade, 1.0],
        packedLight,
    }
}

fn should_side_be_rendered<A: IBlockAccess>(
    access: &A,
    _state: IBlockState,
    pos: BlockPos,
    side: EnumFacing,
    material: LiquidMaterial,
) -> bool {
    let neighbour = access.getBlockState(pos.offset(side, 1));
    if material.contains(neighbour) {
        false
    } else if side == EnumFacing::Up {
        true
    } else {
        !neighbour.getBlock().isOpaqueCube()
    }
}

fn should_render_top_backface<A: IBlockAccess>(
    access: &A,
    pos: BlockPos,
    material: LiquidMaterial,
) -> bool {
    for dx in -1..=1 {
        for dz in -1..=1 {
            let state = access.getBlockState(BlockPos::new(pos.x + dx, pos.y, pos.z + dz));
            if !material.contains(state) && !state.getBlock().isFullyOpaque(state) {
                return true;
            }
        }
    }
    false
}

/// Exact weighted corner-height algorithm from MCP
/// `BlockFluidRenderer#getFluidHeight`.
pub fn getFluidHeight<A: IBlockAccess>(access: &A, pos: BlockPos, material: LiquidMaterial) -> f32 {
    let mut count = 0_i32;
    let mut total = 0.0_f32;
    for sample in 0..4 {
        let sample_pos = BlockPos::new(pos.x - (sample & 1), pos.y, pos.z - ((sample >> 1) & 1));
        if material.contains(access.getBlockState(sample_pos.up(1))) {
            return 1.0;
        }
        let state = access.getBlockState(sample_pos);
        if !material.contains(state) {
            if !BlockLiquid::materialBlocksMovement(state) {
                total += 1.0;
                count += 1;
            }
        } else {
            let level = BlockLiquid::getLevel(state);
            if level >= 8 || level == 0 {
                total += BlockLiquid::getLiquidHeightPercent(level) * 10.0;
                count += 10;
            }
            total += BlockLiquid::getLiquidHeightPercent(level);
            count += 1;
        }
    }
    if count == 0 {
        0.0
    } else {
        1.0 - total / count as f32
    }
}

/// One-to-one geometry port of MCP `BlockFluidRenderer#renderFluid`.
pub fn renderFluid<A, F>(
    access: &A,
    state: IBlockState,
    pos: BlockPos,
    blockColors: &BlockColors,
    sprites: FluidSprites,
    mut packed_light: F,
) -> FluidMesh
where
    A: IBlockAccess + BiomeAccess,
    F: FnMut(BlockPos, IBlockState) -> u32,
{
    let Some(material) = LiquidMaterial::fromState(state) else {
        return FluidMesh::default();
    };
    let top = should_side_be_rendered(access, state, pos, EnumFacing::Up, material);
    let bottom = should_side_be_rendered(access, state, pos, EnumFacing::Down, material);
    let sides = [
        should_side_be_rendered(access, state, pos, EnumFacing::North, material),
        should_side_be_rendered(access, state, pos, EnumFacing::South, material),
        should_side_be_rendered(access, state, pos, EnumFacing::West, material),
        should_side_be_rendered(access, state, pos, EnumFacing::East, material),
    ];
    if !top && !bottom && !sides.iter().any(|shown| *shown) {
        return FluidMesh::default();
    }

    let color = blockColors.colorMultiplier(state, access, pos, 0);
    let tint = [
        ((color >> 16) & 255) as f32 / 255.0,
        ((color >> 8) & 255) as f32 / 255.0,
        (color & 255) as f32 / 255.0,
    ];
    let mut heights = [
        getFluidHeight(access, pos, material),
        getFluidHeight(access, pos.south(1), material),
        getFluidHeight(access, pos.east(1).south(1), material),
        getFluidHeight(access, pos.east(1), material),
    ];
    let x = pos.x as f32;
    let y = pos.y as f32;
    let z = pos.z as f32;
    let epsilon = 0.001_f32;
    let mut mesh = FluidMesh::default();

    if top {
        let slope = BlockLiquid::getSlopeAngle(access, pos, state);
        let sprite = if slope > -999.0 {
            sprites.flow
        } else {
            sprites.still
        };
        for height in &mut heights {
            *height -= epsilon;
        }
        let uvs = if slope < -999.0 {
            [
                interpolate(sprite, 0.0, 0.0),
                interpolate(sprite, 0.0, 16.0),
                interpolate(sprite, 16.0, 16.0),
                interpolate(sprite, 16.0, 0.0),
            ]
        } else {
            let sin = slope.sin() * 0.25;
            let cos = slope.cos() * 0.25;
            [
                interpolate(sprite, 8.0 + (-cos - sin) * 16.0, 8.0 + (-cos + sin) * 16.0),
                interpolate(sprite, 8.0 + (-cos + sin) * 16.0, 8.0 + (cos + sin) * 16.0),
                interpolate(sprite, 8.0 + (cos + sin) * 16.0, 8.0 + (cos - sin) * 16.0),
                interpolate(sprite, 8.0 + (cos - sin) * 16.0, 8.0 + (-cos - sin) * 16.0),
            ]
        };
        let light = packed_light(pos, state);
        let top_vertices = [
            vertex([x, y + heights[0], z], uvs[0], tint, 1.0, light),
            vertex([x, y + heights[1], z + 1.0], uvs[1], tint, 1.0, light),
            vertex([x + 1.0, y + heights[2], z + 1.0], uvs[2], tint, 1.0, light),
            vertex([x + 1.0, y + heights[3], z], uvs[3], tint, 1.0, light),
        ];
        mesh.quad(top_vertices);
        if should_render_top_backface(access, pos.up(1), material) {
            mesh.quad([
                top_vertices[0],
                top_vertices[3],
                top_vertices[2],
                top_vertices[1],
            ]);
        }
    }

    if bottom {
        let light = packed_light(pos.down(1), state);
        mesh.quad([
            vertex(
                [x, y, z + 1.0],
                interpolate(sprites.still, 0.0, 16.0),
                tint,
                0.5,
                light,
            ),
            vertex(
                [x, y, z],
                interpolate(sprites.still, 0.0, 0.0),
                tint,
                0.5,
                light,
            ),
            vertex(
                [x + 1.0, y, z],
                interpolate(sprites.still, 16.0, 0.0),
                tint,
                0.5,
                light,
            ),
            vertex(
                [x + 1.0, y, z + 1.0],
                interpolate(sprites.still, 16.0, 16.0),
                tint,
                0.5,
                light,
            ),
        ]);
    }

    for side_index in 0..4 {
        if !sides[side_index] {
            continue;
        }
        let (dx, dz) = match side_index {
            0 => (0, -1),
            1 => (0, 1),
            2 => (-1, 0),
            _ => (1, 0),
        };
        let neighbour_pos = BlockPos::new(pos.x + dx, pos.y, pos.z + dz);
        let neighbour = access.getBlockState(neighbour_pos);
        let overlay = material == LiquidMaterial::Water
            && matches!(neighbour.getBlockId(), 20 | 95 | 138 | 165);
        let sprite = if overlay {
            sprites.overlay
        } else {
            sprites.flow
        };
        let mut lower_a = 0.0_f32;
        let mut lower_b = 0.0_f32;
        if material == LiquidMaterial::Water {
            if matches!(neighbour.getBlockId(), 60 | 208) {
                lower_a = 0.9375;
                lower_b = 0.9375;
            } else if BlockSlab::isBlockSlab(neighbour)
                && !BlockSlab::isDouble(neighbour)
                && !BlockSlab::isTop(neighbour)
            {
                lower_a = 0.5;
                lower_b = 0.5;
            }
        }
        let (high_a, high_b, p0, p1) = match side_index {
            0 => (
                heights[0],
                heights[3],
                [x, z + epsilon],
                [x + 1.0, z + epsilon],
            ),
            1 => (
                heights[2],
                heights[1],
                [x + 1.0, z + 1.0 - epsilon],
                [x, z + 1.0 - epsilon],
            ),
            2 => (
                heights[1],
                heights[0],
                [x + epsilon, z + 1.0],
                [x + epsilon, z],
            ),
            _ => (
                heights[3],
                heights[2],
                [x + 1.0 - epsilon, z],
                [x + 1.0 - epsilon, z + 1.0],
            ),
        };
        if high_a <= lower_a && high_b <= lower_b {
            continue;
        }
        lower_a = lower_a.min(high_a);
        lower_b = lower_b.min(high_b);
        if lower_a > epsilon {
            lower_a -= epsilon;
        }
        if lower_b > epsilon {
            lower_b -= epsilon;
        }
        let uv0 = interpolate(sprite, 0.0, (1.0 - high_a) * 8.0);
        let uv1 = interpolate(sprite, 8.0, (1.0 - high_b) * 8.0);
        let uv2 = interpolate(sprite, 8.0, (1.0 - lower_b) * 8.0);
        let uv3 = interpolate(sprite, 0.0, (1.0 - lower_a) * 8.0);
        let light = packed_light(neighbour_pos, state);
        let shade = if side_index < 2 { 0.8 } else { 0.6 };
        let front = [
            vertex([p0[0], y + high_a, p0[1]], uv0, tint, shade, light),
            vertex([p1[0], y + high_b, p1[1]], uv1, tint, shade, light),
            vertex([p1[0], y + lower_b, p1[1]], uv2, tint, shade, light),
            vertex([p0[0], y + lower_a, p0[1]], uv3, tint, shade, light),
        ];
        mesh.quad(front);
        if !overlay {
            mesh.quad([front[3], front[2], front[1], front[0]]);
        }
    }
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct Access(HashMap<BlockPos, IBlockState>);
    impl IBlockAccess for Access {
        fn getBlockState(&self, pos: BlockPos) -> IBlockState {
            self.0.get(&pos).copied().unwrap_or_default()
        }
    }
    impl BiomeAccess for Access {
        fn getBiomeId(&self, _pos: BlockPos) -> u8 {
            1
        }
        fn getBlockStateForColor(&self, pos: BlockPos) -> IBlockState {
            self.getBlockState(pos)
        }
    }

    #[test]
    fn source_block_with_source_neighbours_has_level_top() {
        let origin = BlockPos::new(0, 64, 0);
        let source = IBlockState::fromGlobalStateId(9 << 4);
        let mut states = HashMap::new();
        for dx in -1..=1 {
            for dz in -1..=1 {
                states.insert(BlockPos::new(dx, 64, dz), source);
            }
        }
        assert!(
            (getFluidHeight(&Access(states), origin, LiquidMaterial::Water) - 8.0 / 9.0).abs()
                < 1.0e-5
        );
    }

    #[test]
    fn atlas_interpolation_uses_sixteen_texel_space() {
        assert_eq!(interpolate([0.25, 0.5, 0.75, 1.0], 8.0, 8.0), [0.5, 0.75]);
    }
}
