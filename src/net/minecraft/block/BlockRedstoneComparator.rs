use crate::net::minecraft::block::state::IBlockState::IBlockState;

pub const fn isBlockRedstoneComparator(state: IBlockState) -> bool {
    matches!(state.getBlockId(), 149 | 150)
}

/// Exact remote-world state result of
/// `BlockRedstoneComparator#onBlockActivated`. MODE is metadata bit 2;
/// FACING and POWERED remain unchanged. Player edit permission is evaluated by
/// the controller before this method is called.
pub fn onBlockActivatedState(state: IBlockState) -> Option<IBlockState> {
    if !isBlockRedstoneComparator(state) {
        return None;
    }
    Some(IBlockState::fromGlobalStateId(
        (state.getBlockId() << 4) | (state.getMetadata() ^ 4),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activation_cycles_only_mode_bit() {
        let comparePowered = IBlockState::fromGlobalStateId((150 << 4) | 10);
        let subtractPowered = onBlockActivatedState(comparePowered).unwrap();
        assert_eq!(subtractPowered.getBlockId(), 150);
        assert_eq!(subtractPowered.getMetadata(), 14);
        assert_eq!(
            onBlockActivatedState(subtractPowered).unwrap(),
            comparePowered
        );
    }
}
