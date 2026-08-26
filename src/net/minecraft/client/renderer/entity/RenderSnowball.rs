use crate::net::minecraft::client::entity::EntityOtherClient::ObjectSpawnType;
use crate::net::minecraft::item::ItemStack::ItemStack;

pub struct RenderSnowball;

impl RenderSnowball {
    /// Exact item registry objects selected by RenderManager 1.12.2.
    pub fn getStackToRender(
        objectType: ObjectSpawnType,
        metadataStack: Option<&ItemStack>,
    ) -> Option<ItemStack> {
        let itemId = match objectType {
            ObjectSpawnType::Snowball => 332,
            ObjectSpawnType::Egg => 344,
            ObjectSpawnType::EnderPearl => 368,
            ObjectSpawnType::EyeOfEnder => 381,
            ObjectSpawnType::ExperienceBottle => 384,
            ObjectSpawnType::FireworkRocket => 401,
            ObjectSpawnType::Potion => {
                return Some(
                    metadataStack
                        .cloned()
                        .filter(|stack| !stack.isEmpty())
                        .unwrap_or(ItemStack {
                            itemId: 438,
                            count: 1,
                            itemDamage: 0,
                            tagCompound: None,
                        }),
                );
            }
            _ => return None,
        };
        Some(ItemStack {
            itemId,
            count: 1,
            itemDamage: 0,
            tagCompound: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_item_registry_ids_are_retained() {
        assert_eq!(
            RenderSnowball::getStackToRender(ObjectSpawnType::Snowball, None)
                .unwrap()
                .itemId,
            332
        );
        assert_eq!(
            RenderSnowball::getStackToRender(ObjectSpawnType::EnderPearl, None)
                .unwrap()
                .itemId,
            368
        );
        assert_eq!(
            RenderSnowball::getStackToRender(ObjectSpawnType::ExperienceBottle, None)
                .unwrap()
                .itemId,
            384
        );
    }
}
