use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::block::Block::Block;

/// MCP 1.12.2 `FlatLayerInfo`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatLayerInfo {
    version: i32,
    layerMaterial: IBlockState,
    layerCount: i32,
    layerMinimumY: i32,
}

impl FlatLayerInfo {
    pub fn new(layerCount: i32, block: Block) -> Self {
        Self::newVersioned(3, layerCount, block, 0)
    }

    pub fn newVersioned(version: i32, height: i32, block: Block, metadata: i32) -> Self {
        let metadata = metadata.clamp(0, 15);
        Self {
            version,
            layerMaterial: IBlockState::fromGlobalStateId(
                (Block::getIdFromBlock(block) << 4) | metadata,
            ),
            layerCount: height,
            layerMinimumY: 0,
        }
    }

    pub const fn getLayerCount(&self) -> i32 {
        self.layerCount
    }
    pub const fn getLayerMaterial(&self) -> IBlockState {
        self.layerMaterial
    }
    pub const fn getMinY(&self) -> i32 {
        self.layerMinimumY
    }
    pub fn setMinY(&mut self, minY: i32) {
        self.layerMinimumY = minY;
    }

    pub fn toGeneratorString(&self) -> String {
        let block = self.layerMaterial.getBlock();
        let mut text = if self.version >= 3 {
            let name = block.getRegistryName().to_string();
            if self.layerCount > 1 {
                format!("{}*{}", self.layerCount, name)
            } else {
                name
            }
        } else {
            let id = Block::getIdFromBlock(block).to_string();
            if self.layerCount > 1 {
                format!("{}x{}", self.layerCount, id)
            } else {
                id
            }
        };
        let meta = self.layerMaterial.getMetadata();
        if meta > 0 {
            text.push_str(&format!(":{meta}"));
        }
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_three_uses_registry_name_and_star_count() {
        let mut layer = FlatLayerInfo::new(2, Block::getBlockById(3));
        layer.setMinY(1);
        assert_eq!(layer.getMinY(), 1);
        assert_eq!(layer.toGeneratorString(), "2*minecraft:dirt");
    }
}
