use crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType;
use crate::net::minecraft::client::renderer::entity::RenderCaveSpider::RenderCaveSpider;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiderVariant {
    Spider,
    CaveSpider,
}

pub struct RenderSpider;

impl RenderSpider {
    pub fn variant(entityType: MobEntityType) -> Option<SpiderVariant> {
        match entityType.registryName {
            "spider" => Some(SpiderVariant::Spider),
            "cave_spider" => Some(SpiderVariant::CaveSpider),
            _ => None,
        }
    }
    pub fn texture(variant: SpiderVariant) -> ResourceLocation {
        match variant {
            SpiderVariant::Spider => {
                ResourceLocation::new("minecraft", "textures/entity/spider/spider.png")
            }
            SpiderVariant::CaveSpider => RenderCaveSpider::texture(),
        }
    }
    pub fn preScale(variant: SpiderVariant) -> f32 {
        match variant {
            SpiderVariant::Spider => 1.0,
            SpiderVariant::CaveSpider => RenderCaveSpider::preScale(),
        }
    }
}
