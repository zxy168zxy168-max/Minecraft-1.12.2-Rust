use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::tileentity::TileEntityPiston::TileEntityPiston;
use crate::net::minecraft::util::EnumFacing::EnumFacing;

/// Model submissions selected by MCP 1.12.2 `TileEntityPistonRenderer`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PistonRenderPart {
    MovingState {
        state: IBlockState,
        offset: [f32; 3],
        checkSides: bool,
    },
    Head {
        facing: EnumFacing,
        sticky: bool,
        short: bool,
        offset: [f32; 3],
    },
    ExtendedBase {
        state: IBlockState,
    },
}

pub struct TileEntityPistonRenderer;

impl TileEntityPistonRenderer {
    pub fn buildPlan(tile: &TileEntityPiston, partialTicks: f32) -> Vec<PistonRenderPart> {
        let progress = tile.getProgress(partialTicks);
        if progress >= 1.0 || tile.pistonState.getBlockId() == 0 {
            return Vec::new();
        }
        let offset = tile.offset(partialTicks);
        let block_id = tile.pistonState.getBlockId();
        if block_id == 34 && progress <= 0.25 {
            return vec![PistonRenderPart::Head {
                facing: tile.pistonFacing,
                sticky: (tile.pistonState.getMetadata() & 8) != 0,
                short: true,
                offset,
            }];
        }
        if tile.shouldHeadBeRendered && !tile.extending && matches!(block_id, 29 | 33) {
            let sticky = block_id == 29;
            let extended_state = IBlockState::fromGlobalStateId(
                (block_id << 4) | (tile.pistonState.getMetadata() & 7) | 8,
            );
            return vec![
                PistonRenderPart::Head {
                    facing: tile.pistonFacing,
                    sticky,
                    short: progress >= 0.5,
                    offset,
                },
                PistonRenderPart::ExtendedBase {
                    state: extended_state,
                },
            ];
        }
        vec![PistonRenderPart::MovingState {
            state: tile.pistonState,
            offset,
            checkSides: false,
        }]
    }
}
