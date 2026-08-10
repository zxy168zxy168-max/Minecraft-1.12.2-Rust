use crate::net::minecraft::client::entity::EntityOtherClient::{
    ClientEntityKind, ObjectSpawnType,
};

/// Concrete renderer selection table from Minecraft 1.12.2 `RenderManager`.
/// Unsupported entries remain explicit rather than receiving a generic cube.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityRendererKind {
    Boat,
    Minecart,
    EntityItem,
    FallingBlock,
    ExperienceOrb,
    Snowball,
    Arrow,
    TntPrimed,
    EnderCrystal,
    Zombie,
    Skeleton,
    ArmorStand,
    Pig,
    Cow,
    Sheep,
    Chicken,
    Mooshroom,
    Creeper,
    Spider,
    Enderman,
    Squid,
    EnderDragon,
    Slime,
    MagmaCube,
    Blaze,
    Ghast,
    Guardian,
    Shulker,
    ShulkerBullet,
    Fireball,
    DragonFireball,
    WitherSkull,
    FishHook,
    AreaEffectCloud,
    Painting,
    ItemFrame,
    LeashKnot,
    Wolf,
    Ocelot,
    Rabbit,
    PolarBear,
    Horse,
    Llama,
    Villager,
    Witch,
    Illager,
    ZombieVillager,
    Unsupported,
}

pub struct RenderManager;

impl RenderManager {
    pub fn getEntityRenderObject(kind: &ClientEntityKind) -> EntityRendererKind {
        match kind {
            ClientEntityKind::ExperienceOrb { .. } => EntityRendererKind::ExperienceOrb,
            ClientEntityKind::Object { objectType, .. } => match objectType {
                ObjectSpawnType::Boat => EntityRendererKind::Boat,
                ObjectSpawnType::Minecart => EntityRendererKind::Minecart,
                ObjectSpawnType::Item => EntityRendererKind::EntityItem,
                ObjectSpawnType::FallingBlock => EntityRendererKind::FallingBlock,
                ObjectSpawnType::Snowball
                | ObjectSpawnType::Egg
                | ObjectSpawnType::EnderPearl
                | ObjectSpawnType::EyeOfEnder
                | ObjectSpawnType::Potion
                | ObjectSpawnType::ExperienceBottle
                | ObjectSpawnType::FireworkRocket => EntityRendererKind::Snowball,
                ObjectSpawnType::TippedArrow | ObjectSpawnType::SpectralArrow => {
                    EntityRendererKind::Arrow
                }
                ObjectSpawnType::PrimedTnt => EntityRendererKind::TntPrimed,
                ObjectSpawnType::EnderCrystal => EntityRendererKind::EnderCrystal,
                ObjectSpawnType::ArmorStand => EntityRendererKind::ArmorStand,
                ObjectSpawnType::ShulkerBullet => EntityRendererKind::ShulkerBullet,
                ObjectSpawnType::LargeFireball | ObjectSpawnType::SmallFireball => {
                    EntityRendererKind::Fireball
                }
                ObjectSpawnType::DragonFireball => EntityRendererKind::DragonFireball,
                ObjectSpawnType::WitherSkull => EntityRendererKind::WitherSkull,
                ObjectSpawnType::FishHook => EntityRendererKind::FishHook,
                ObjectSpawnType::AreaEffectCloud => EntityRendererKind::AreaEffectCloud,
                ObjectSpawnType::ItemFrame => EntityRendererKind::ItemFrame,
                ObjectSpawnType::LeashKnot => EntityRendererKind::LeashKnot,
                _ => EntityRendererKind::Unsupported,
            },
            ClientEntityKind::Mob { entityType } => {
                if crate::net::minecraft::client::renderer::entity::RenderZombie::RenderZombie::variant(*entityType).is_some() {
                    EntityRendererKind::Zombie
                } else if crate::net::minecraft::client::renderer::entity::RenderSkeleton::RenderSkeleton::variant(*entityType).is_some() {
                    EntityRendererKind::Skeleton
                } else if crate::net::minecraft::client::renderer::entity::RenderPig::RenderPig::supports(*entityType) {
                    EntityRendererKind::Pig
                } else if crate::net::minecraft::client::renderer::entity::RenderCow::RenderCow::supports(*entityType) {
                    EntityRendererKind::Cow
                } else if crate::net::minecraft::client::renderer::entity::RenderSheep::RenderSheep::supports(*entityType) {
                    EntityRendererKind::Sheep
                } else if crate::net::minecraft::client::renderer::entity::RenderChicken::RenderChicken::supports(*entityType) {
                    EntityRendererKind::Chicken
                } else if crate::net::minecraft::client::renderer::entity::RenderMooshroom::RenderMooshroom::supports(*entityType) {
                    EntityRendererKind::Mooshroom
                } else if crate::net::minecraft::client::renderer::entity::RenderCreeper::RenderCreeper::supports(*entityType) {
                    EntityRendererKind::Creeper
                } else if crate::net::minecraft::client::renderer::entity::RenderSpider::RenderSpider::variant(*entityType).is_some() {
                    EntityRendererKind::Spider
                } else if crate::net::minecraft::client::renderer::entity::RenderEnderman::RenderEnderman::supports(*entityType) {
                    EntityRendererKind::Enderman
                } else if crate::net::minecraft::client::renderer::entity::RenderSquid::RenderSquid::supports(*entityType) {
                    EntityRendererKind::Squid
                } else if crate::net::minecraft::client::renderer::entity::RenderDragon::RenderDragon::supports(*entityType) {
                    EntityRendererKind::EnderDragon
                } else if crate::net::minecraft::client::renderer::entity::RenderSlime::RenderSlime::supports(*entityType) {
                    EntityRendererKind::Slime
                } else if crate::net::minecraft::client::renderer::entity::RenderMagmaCube::RenderMagmaCube::supports(*entityType) {
                    EntityRendererKind::MagmaCube
                } else if crate::net::minecraft::client::renderer::entity::RenderBlaze::RenderBlaze::supports(*entityType) {
                    EntityRendererKind::Blaze
                } else if crate::net::minecraft::client::renderer::entity::RenderGhast::RenderGhast::supports(*entityType) {
                    EntityRendererKind::Ghast
                } else if crate::net::minecraft::client::renderer::entity::RenderGuardian::RenderGuardian::variant(*entityType).is_some() {
                    EntityRendererKind::Guardian
                } else if crate::net::minecraft::client::renderer::entity::RenderShulker::RenderShulker::supports(*entityType) {
                    EntityRendererKind::Shulker
                } else if crate::net::minecraft::client::renderer::entity::RenderWolf::RenderWolf::supports(*entityType) {
                    EntityRendererKind::Wolf
                } else if crate::net::minecraft::client::renderer::entity::RenderOcelot::RenderOcelot::supports(*entityType) {
                    EntityRendererKind::Ocelot
                } else if crate::net::minecraft::client::renderer::entity::RenderRabbit::RenderRabbit::supports(*entityType) {
                    EntityRendererKind::Rabbit
                } else if crate::net::minecraft::client::renderer::entity::RenderPolarBear::RenderPolarBear::supports(*entityType) {
                    EntityRendererKind::PolarBear
                } else if crate::net::minecraft::client::renderer::entity::RenderHorse::RenderHorse::supports(*entityType)
                    || crate::net::minecraft::client::renderer::entity::RenderAbstractHorse::RenderAbstractHorse::variant(*entityType).is_some() {
                    EntityRendererKind::Horse
                } else if crate::net::minecraft::client::renderer::entity::RenderLlama::RenderLlama::supports(*entityType) {
                    EntityRendererKind::Llama
                } else if crate::net::minecraft::client::renderer::entity::RenderVillager::RenderVillager::supports(*entityType) {
                    EntityRendererKind::Villager
                } else if crate::net::minecraft::client::renderer::entity::RenderWitch::RenderWitch::supports(*entityType) {
                    EntityRendererKind::Witch
                } else if crate::net::minecraft::client::renderer::entity::RenderVindicator::RenderVindicator::supports(*entityType)
                    || crate::net::minecraft::client::renderer::entity::RenderEvoker::RenderEvoker::supports(*entityType)
                    || crate::net::minecraft::client::renderer::entity::RenderIllusionIllager::RenderIllusionIllager::supports(*entityType) {
                    EntityRendererKind::Illager
                } else if crate::net::minecraft::client::renderer::entity::RenderZombieVillager::RenderZombieVillager::supports(*entityType) {
                    EntityRendererKind::ZombieVillager
                } else {
                    EntityRendererKind::Unsupported
                }
            }
            ClientEntityKind::Painting { .. } => EntityRendererKind::Painting,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_registry_does_not_fallback_to_a_generic_renderer() {
        let item = ClientEntityKind::Object {
            objectType: ObjectSpawnType::Item,
            data: 0,
            spawnVelocity: [0.0; 3],
        };
        assert_eq!(RenderManager::getEntityRenderObject(&item), EntityRendererKind::EntityItem);
        let boat = ClientEntityKind::Object {
            objectType: ObjectSpawnType::Boat,
            data: 0,
            spawnVelocity: [0.0; 3],
        };
        assert_eq!(RenderManager::getEntityRenderObject(&boat), EntityRendererKind::Boat);
        let minecart = ClientEntityKind::Object {
            objectType: ObjectSpawnType::Minecart,
            data: 1,
            spawnVelocity: [0.0; 3],
        };
        assert_eq!(RenderManager::getEntityRenderObject(&minecart), EntityRendererKind::Minecart);
        let enderCrystal = ClientEntityKind::Object {
            objectType: ObjectSpawnType::EnderCrystal,
            data: 0,
            spawnVelocity: [0.0; 3],
        };
        assert_eq!(RenderManager::getEntityRenderObject(&enderCrystal), EntityRendererKind::EnderCrystal);
        let armorStand = ClientEntityKind::Object {
            objectType: ObjectSpawnType::ArmorStand,
            data: 0,
            spawnVelocity: [0.0; 3],
        };
        assert_eq!(RenderManager::getEntityRenderObject(&armorStand), EntityRendererKind::ArmorStand);
        let pig = ClientEntityKind::Mob { entityType: crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType::fromId(90).unwrap() };
        assert_eq!(RenderManager::getEntityRenderObject(&pig), EntityRendererKind::Pig);
        let chicken = ClientEntityKind::Mob { entityType: crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType::fromId(93).unwrap() };
        assert_eq!(RenderManager::getEntityRenderObject(&chicken), EntityRendererKind::Chicken);
        let mooshroom = ClientEntityKind::Mob { entityType: crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType::fromId(96).unwrap() };
        assert_eq!(RenderManager::getEntityRenderObject(&mooshroom), EntityRendererKind::Mooshroom);
        let creeper = ClientEntityKind::Mob { entityType: crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType::fromId(50).unwrap() };
        assert_eq!(RenderManager::getEntityRenderObject(&creeper), EntityRendererKind::Creeper);
        let caveSpider = ClientEntityKind::Mob { entityType: crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType::fromId(59).unwrap() };
        assert_eq!(RenderManager::getEntityRenderObject(&caveSpider), EntityRendererKind::Spider);
        let slime = ClientEntityKind::Mob { entityType: crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType::fromId(55).unwrap() };
        assert_eq!(RenderManager::getEntityRenderObject(&slime), EntityRendererKind::Slime);
        let magma = ClientEntityKind::Mob { entityType: crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType::fromId(62).unwrap() };
        assert_eq!(RenderManager::getEntityRenderObject(&magma), EntityRendererKind::MagmaCube);
        let blaze = ClientEntityKind::Mob { entityType: crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType::fromId(61).unwrap() };
        assert_eq!(RenderManager::getEntityRenderObject(&blaze), EntityRendererKind::Blaze);
        let ghast = ClientEntityKind::Mob { entityType: crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType::fromId(56).unwrap() };
        assert_eq!(RenderManager::getEntityRenderObject(&ghast), EntityRendererKind::Ghast);
        let guardian = ClientEntityKind::Mob { entityType: crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType::fromId(68).unwrap() };
        assert_eq!(RenderManager::getEntityRenderObject(&guardian), EntityRendererKind::Guardian);
        let elderGuardian = ClientEntityKind::Mob { entityType: crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType::fromId(4).unwrap() };
        assert_eq!(RenderManager::getEntityRenderObject(&elderGuardian), EntityRendererKind::Guardian);
        let shulker = ClientEntityKind::Mob { entityType: crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType::fromId(69).unwrap() };
        assert_eq!(RenderManager::getEntityRenderObject(&shulker), EntityRendererKind::Shulker);
        let shulkerBullet = ClientEntityKind::Object {
            objectType: ObjectSpawnType::ShulkerBullet,
            data: 0,
            spawnVelocity: [0.0; 3],
        };
        assert_eq!(RenderManager::getEntityRenderObject(&shulkerBullet), EntityRendererKind::ShulkerBullet);
        for (objectType, expected) in [
            (ObjectSpawnType::LargeFireball, EntityRendererKind::Fireball),
            (ObjectSpawnType::SmallFireball, EntityRendererKind::Fireball),
            (ObjectSpawnType::DragonFireball, EntityRendererKind::DragonFireball),
            (ObjectSpawnType::WitherSkull, EntityRendererKind::WitherSkull),
        ] {
            let projectile = ClientEntityKind::Object { objectType, data: 0, spawnVelocity: [0.0; 3] };
            assert_eq!(RenderManager::getEntityRenderObject(&projectile), expected);
        }
        let wolf = ClientEntityKind::Mob { entityType: crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType::fromId(95).unwrap() };
        assert_eq!(RenderManager::getEntityRenderObject(&wolf), EntityRendererKind::Wolf);
        let ocelot = ClientEntityKind::Mob { entityType: crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType::fromId(98).unwrap() };
        assert_eq!(RenderManager::getEntityRenderObject(&ocelot), EntityRendererKind::Ocelot);
        let rabbit = ClientEntityKind::Mob { entityType: crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType::fromId(101).unwrap() };
        assert_eq!(RenderManager::getEntityRenderObject(&rabbit), EntityRendererKind::Rabbit);
        let polarBear = ClientEntityKind::Mob { entityType: crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType::fromId(102).unwrap() };
        assert_eq!(RenderManager::getEntityRenderObject(&polarBear), EntityRendererKind::PolarBear);
        let horse = ClientEntityKind::Mob { entityType: crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType::fromId(100).unwrap() };
        assert_eq!(RenderManager::getEntityRenderObject(&horse), EntityRendererKind::Horse);
        let donkey = ClientEntityKind::Mob { entityType: crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType::fromId(31).unwrap() };
        assert_eq!(RenderManager::getEntityRenderObject(&donkey), EntityRendererKind::Horse);
        let llama = ClientEntityKind::Mob { entityType: crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType::fromId(103).unwrap() };
        assert_eq!(RenderManager::getEntityRenderObject(&llama), EntityRendererKind::Llama);
        let villager = ClientEntityKind::Mob { entityType: crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType::fromId(120).unwrap() };
        assert_eq!(RenderManager::getEntityRenderObject(&villager), EntityRendererKind::Villager);
        let witch = ClientEntityKind::Mob { entityType: crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType::fromId(66).unwrap() };
        assert_eq!(RenderManager::getEntityRenderObject(&witch), EntityRendererKind::Witch);
        let evoker = ClientEntityKind::Mob { entityType: crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType::fromId(34).unwrap() };
        assert_eq!(RenderManager::getEntityRenderObject(&evoker), EntityRendererKind::Illager);
        let zombieVillager = ClientEntityKind::Mob { entityType: crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType::fromId(27).unwrap() };
        assert_eq!(RenderManager::getEntityRenderObject(&zombieVillager), EntityRendererKind::ZombieVillager);
    }
    #[test]
    fn hanging_entities_use_their_concrete_renderers() {
        let painting = ClientEntityKind::Painting {
            title: "Kebab".to_owned(),
            hangingPosition: crate::net::minecraft::util::math::BlockPos::BlockPos::new(0, 0, 0),
            facing: crate::net::minecraft::util::EnumFacing::EnumFacing::North,
        };
        assert_eq!(RenderManager::getEntityRenderObject(&painting), EntityRendererKind::Painting);
        for (objectType, expected) in [
            (ObjectSpawnType::ItemFrame, EntityRendererKind::ItemFrame),
            (ObjectSpawnType::LeashKnot, EntityRendererKind::LeashKnot),
        ] {
            let kind = ClientEntityKind::Object { objectType, data: 0, spawnVelocity: [0.0; 3] };
            assert_eq!(RenderManager::getEntityRenderObject(&kind), expected);
        }
    }

    #[test]
    fn enderman_squid_and_dragon_use_vanilla_renderers() {
        for (id, expected) in [
            (58, EntityRendererKind::Enderman),
            (94, EntityRendererKind::Squid),
            (63, EntityRendererKind::EnderDragon),
        ] {
            let kind = ClientEntityKind::Mob {
                entityType: crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType::fromId(id).unwrap(),
            };
            assert_eq!(RenderManager::getEntityRenderObject(&kind), expected);
        }
    }

}
