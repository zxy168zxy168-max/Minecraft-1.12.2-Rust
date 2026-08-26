use crate::net::minecraft::block::state::IBlockState::IBlockState;

pub const fn isBlockRedstoneRepeater(state: IBlockState) -> bool {
    matches!(state.getBlockId(), 93 | 94)
}

/// Exact remote-world result of `BlockRedstoneRepeater#onBlockActivated`.
/// DELAY is encoded as `1 + (meta >> 2)` and cycles 1,2,3,4,1 while FACING is
/// preserved. Player edit permission is evaluated by the controller.
pub fn onBlockActivatedState(state: IBlockState) -> Option<IBlockState> {
    if !isBlockRedstoneRepeater(state) {
        return None;
    }
    let delay = 1 + ((state.getMetadata() >> 2) & 3);
    let nextDelay = if delay == 4 { 1 } else { delay + 1 };
    let metadata = (state.getMetadata() & 3) | ((nextDelay - 1) << 2);
    Some(IBlockState::fromGlobalStateId(
        (state.getBlockId() << 4) | metadata,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activation_cycles_delay_and_preserves_facing() {
        let delayFourNorth = IBlockState::fromGlobalStateId((93 << 4) | 14);
        let delayOneNorth = onBlockActivatedState(delayFourNorth).unwrap();
        assert_eq!(delayOneNorth.getMetadata(), 2);
        assert_eq!(
            delayOneNorth.getMetadata() & 3,
            delayFourNorth.getMetadata() & 3
        );
    }
}
