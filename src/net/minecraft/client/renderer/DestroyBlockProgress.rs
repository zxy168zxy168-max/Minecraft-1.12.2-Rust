use crate::net::minecraft::util::math::BlockPos::BlockPos;

/// MCP 1.12.2 `DestroyBlockProgress` state used by `RenderGlobal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DestroyBlockProgress {
    miningPlayerEntId: i32,
    position: BlockPos,
    partialBlockProgress: i32,
    createdAtCloudUpdateTick: i32,
}

impl DestroyBlockProgress {
    pub const fn new(miningPlayerEntIdIn: i32, positionIn: BlockPos) -> Self {
        Self {
            miningPlayerEntId: miningPlayerEntIdIn,
            position: positionIn,
            partialBlockProgress: 0,
            createdAtCloudUpdateTick: 0,
        }
    }

    pub const fn getMiningPlayerEntId(&self) -> i32 {
        self.miningPlayerEntId
    }
    pub const fn getPosition(&self) -> BlockPos {
        self.position
    }
    pub fn setPartialBlockDamage(&mut self, damage: i32) {
        self.partialBlockProgress = damage.min(10);
    }
    pub const fn getPartialBlockDamage(&self) -> i32 {
        self.partialBlockProgress
    }
    pub fn setCloudUpdateTick(&mut self, tick: i32) {
        self.createdAtCloudUpdateTick = tick;
    }
    pub const fn getCreationCloudUpdateTick(&self) -> i32 {
        self.createdAtCloudUpdateTick
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn damage_is_capped_like_vanilla() {
        let mut progress = DestroyBlockProgress::new(7, BlockPos::new(1, 2, 3));
        progress.setPartialBlockDamage(12);
        assert_eq!(progress.getPartialBlockDamage(), 10);
        assert_eq!(progress.getMiningPlayerEntId(), 7);
    }
}
