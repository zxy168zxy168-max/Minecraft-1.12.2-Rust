use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::client::entity::EntityPlayerSP::EntityPlayerSP;
use crate::net::minecraft::util::math::AxisAlignedBB::AxisAlignedBB;
use crate::net::minecraft::util::EnumFacing::EnumFacing;

/// Source port of the state, placement and bounds owned by MCP 1.12.2
/// `BlockEndPortalFrame`. Portal-pattern matching and comparator updates remain
/// server authoritative; the client uses the exact metadata for rendering,
/// selection and collision.
pub struct BlockEndPortalFrame;

impl BlockEndPortalFrame {
    pub const BLOCK_ID: i32 = 120;
    pub const AABB_BLOCK: AxisAlignedBB = AxisAlignedBB {
        min_x: 0.0,
        min_y: 0.0,
        min_z: 0.0,
        max_x: 1.0,
        max_y: 0.8125,
        max_z: 1.0,
    };
    pub const AABB_EYE: AxisAlignedBB = AxisAlignedBB {
        min_x: 0.3125,
        min_y: 0.8125,
        min_z: 0.3125,
        max_x: 0.6875,
        max_y: 1.0,
        max_z: 0.6875,
    };

    pub const fn isBlockEndPortalFrame(state: IBlockState) -> bool {
        state.getBlockId() == Self::BLOCK_ID
    }

    /// `BlockEndPortalFrame#getStateFromMeta`.
    pub const fn getFacing(state: IBlockState) -> EnumFacing {
        match state.getMetadata() & 3 {
            0 => EnumFacing::South,
            1 => EnumFacing::West,
            2 => EnumFacing::North,
            _ => EnumFacing::East,
        }
    }

    pub const fn hasEye(state: IBlockState) -> bool {
        state.getMetadata() & 4 != 0
    }

    pub const fn getBoundingBox() -> AxisAlignedBB {
        Self::AABB_BLOCK
    }

    /// `BlockEndPortalFrame#addCollisionBoxToList`: the eye contributes a
    /// second box while the selected outline remains the base frame box.
    pub fn getCollisionBoxes(state: IBlockState) -> Vec<AxisAlignedBB> {
        let mut boxes = vec![Self::AABB_BLOCK];
        if Self::hasEye(state) {
            boxes.push(Self::AABB_EYE);
        }
        boxes
    }

    /// `BlockEndPortalFrame#onBlockPlaced`: horizontal facing is the placer's
    /// opposite and a placed frame never starts with an eye.
    pub fn onBlockPlacedState(player: &EntityPlayerSP) -> IBlockState {
        let facing = EnumFacing::fromAngle(player.entity.rotationYaw as f64).opposite();
        let meta = facing.horizontalIndex().unwrap_or(2) as i32;
        IBlockState::fromGlobalStateId((Self::BLOCK_ID << 4) | meta)
    }

    pub fn modelVariant(state: IBlockState) -> String {
        let facing = match Self::getFacing(state) {
            EnumFacing::South => "south",
            EnumFacing::West => "west",
            EnumFacing::North => "north",
            EnumFacing::East => "east",
            _ => "north",
        };
        format!("eye={},facing={facing}", Self::hasEye(state))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_matches_horizontal_facing_and_eye_property() {
        let state = IBlockState::fromGlobalStateId((120 << 4) | 5);
        assert_eq!(BlockEndPortalFrame::getFacing(state), EnumFacing::West);
        assert!(BlockEndPortalFrame::hasEye(state));
        assert_eq!(
            BlockEndPortalFrame::modelVariant(state),
            "eye=true,facing=west"
        );
    }

    #[test]
    fn collision_adds_eye_but_selection_remains_frame_height() {
        let empty = IBlockState::fromGlobalStateId(120 << 4);
        let eye = IBlockState::fromGlobalStateId((120 << 4) | 4);
        assert_eq!(
            BlockEndPortalFrame::getCollisionBoxes(empty),
            vec![BlockEndPortalFrame::AABB_BLOCK]
        );
        assert_eq!(BlockEndPortalFrame::getCollisionBoxes(eye).len(), 2);
        assert_eq!(BlockEndPortalFrame::getBoundingBox().max_y, 0.8125);
    }

    #[test]
    fn placement_faces_opposite_the_player() {
        let mut player = EntityPlayerSP::new(1);
        player.entity.rotationYaw = 0.0; // SOUTH, frame faces NORTH
        assert_eq!(
            BlockEndPortalFrame::onBlockPlacedState(&player).getMetadata(),
            2
        );
    }
}
