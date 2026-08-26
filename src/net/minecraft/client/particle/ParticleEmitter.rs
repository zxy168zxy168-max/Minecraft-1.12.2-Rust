use std::time::{SystemTime, UNIX_EPOCH};

use crate::compat::Java::JavaRandom;
use crate::net::minecraft::client::particle::ParticleSpawnRequest::ParticleSpawnRequest;
use crate::net::minecraft::entity::Entity::Entity;
use crate::net::minecraft::util::EnumParticleTypes::EnumParticleTypes;

/// MCP 1.12.2 `ParticleEmitter` state. The renderer owns the concrete
/// particles; this object follows one entity for a finite number of ticks and
/// emits the source's 16 sphere samples on every update.
#[derive(Debug, Clone, PartialEq)]
pub struct ParticleEmitter {
    attachedEntityId: i32,
    age: i32,
    lifetime: i32,
    particleType: EnumParticleTypes,
    random: JavaRandom,
}

impl ParticleEmitter {
    pub fn new(attached_entity_id: i32, particle_type: EnumParticleTypes, lifetime: i32) -> Self {
        Self {
            attachedEntityId: attached_entity_id,
            age: 0,
            lifetime: lifetime.max(1),
            particleType: particle_type,
            random: JavaRandom::new(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as i64,
            ),
        }
    }

    pub const fn attachedEntityId(&self) -> i32 {
        self.attachedEntityId
    }
    pub const fn isExpired(&self) -> bool {
        self.age >= self.lifetime
    }

    pub fn onUpdate(&mut self, entity: &Entity) -> Vec<ParticleSpawnRequest> {
        let mut requests = Vec::with_capacity(16);
        for _ in 0..16 {
            let d0 = f64::from(self.random.next_f32() * 2.0 - 1.0);
            let d1 = f64::from(self.random.next_f32() * 2.0 - 1.0);
            let d2 = f64::from(self.random.next_f32() * 2.0 - 1.0);
            if d0 * d0 + d1 * d1 + d2 * d2 <= 1.0 {
                requests.push(ParticleSpawnRequest::new(
                    self.particleType,
                    [
                        entity.posX + d0 * f64::from(entity.width) / 4.0,
                        entity.boundingBox.min_y
                            + f64::from(entity.height / 2.0)
                            + d1 * f64::from(entity.height) / 4.0,
                        entity.posZ + d2 * f64::from(entity.width) / 4.0,
                    ],
                    [d0, d1 + 0.2, d2],
                    [0, 0],
                ));
            }
        }
        self.age += 1;
        requests
    }
}

#[cfg(test)]
mod tests {
    use super::ParticleEmitter;
    use crate::net::minecraft::entity::Entity::Entity;
    use crate::net::minecraft::util::EnumParticleTypes::EnumParticleTypes;

    #[test]
    fn vanilla_lifetime_expires_after_requested_updates() {
        let entity = Entity::default();
        let mut emitter = ParticleEmitter::new(7, EnumParticleTypes::Totem, 3);
        assert!(!emitter.isExpired());
        emitter.onUpdate(&entity);
        emitter.onUpdate(&entity);
        assert!(!emitter.isExpired());
        emitter.onUpdate(&entity);
        assert!(emitter.isExpired());
    }
}
