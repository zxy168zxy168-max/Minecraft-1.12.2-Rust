use crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// MCP 1.12.2 `RenderCaveSpider` specialization. The shared body and gait remain
/// owned by `ModelSpider`; this class owns the cave-spider texture and 0.7 scale.
pub struct RenderCaveSpider;

impl RenderCaveSpider {
    pub fn supports(entityType: MobEntityType) -> bool {
        entityType.registryName == "cave_spider"
    }
    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/spider/cave_spider.png")
    }
    pub const fn preScale() -> f32 {
        0.7
    }
}
