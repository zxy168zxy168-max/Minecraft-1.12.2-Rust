use crate::net::minecraft::entity::player::EntityPlayerMP::EntityPlayerMP;
use crate::net::minecraft::network::play::server::SPacketChunkData::SPacketChunkData;
use crate::net::minecraft::network::NetworkManager::NetworkManager;
use crate::net::minecraft::world::WorldServer::WorldServer;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Debug, Default)]
struct PlayerChunks {
    center: (i32, i32),
    pending: Vec<(i32, i32)>,
    sent: HashSet<(i32, i32)>,
}

/// MCP 1.12.2 `PlayerChunkMap` initial streaming responsibilities.
/// Missing-chunk provision keeps the source 50 ms budget and its post-decrement
/// boundary (up to 50 successful missing entries); the separate pending-send
/// pass likewise preserves the source's effective 82-entry boundary.
#[derive(Debug)]
pub struct PlayerChunkMap {
    playerViewRadius: i32,
    players: HashMap<Uuid, PlayerChunks>,
}
impl PlayerChunkMap {
    pub fn new(viewDistance: i32) -> Self {
        Self {
            playerViewRadius: viewDistance.clamp(3, 32),
            players: HashMap::new(),
        }
    }
    fn desired(center: (i32, i32), radius: i32) -> Vec<(i32, i32)> {
        let mut v = Vec::new();
        for x in center.0 - radius..=center.0 + radius {
            for z in center.1 - radius..=center.1 + radius {
                v.push((x, z));
            }
        }
        v.sort_by_key(|&(x, z)| {
            let dx = x - center.0;
            let dz = z - center.1;
            dx * dx + dz * dz
        });
        v
    }
    pub fn addPlayer(&mut self, player: &mut EntityPlayerMP) {
        let Some(id) = player.getGameProfile().getId() else {
            return;
        };
        let center = (
            (player.entity.posX as i32) >> 4,
            (player.entity.posZ as i32) >> 4,
        );
        player.managedPosX = player.entity.posX;
        player.managedPosZ = player.entity.posZ;
        self.players.insert(
            id,
            PlayerChunks {
                center,
                pending: Self::desired(center, self.playerViewRadius),
                sent: HashSet::new(),
            },
        );
    }
    /// Source-shaped movement window update. Coordinates leaving the view
    /// square are returned to PlayerList so it can send SPacketUnloadChunk.
    pub fn updateMovingPlayer(&mut self, player: &mut EntityPlayerMP) -> Vec<(i32, i32)> {
        let Some(id) = player.getGameProfile().getId() else {
            return Vec::new();
        };
        let dx = player.managedPosX - player.entity.posX;
        let dz = player.managedPosZ - player.entity.posZ;
        if dx * dx + dz * dz < 64.0 {
            return Vec::new();
        }
        let center = (
            (player.entity.posX as i32) >> 4,
            (player.entity.posZ as i32) >> 4,
        );
        let mut unloads = Vec::new();
        if let Some(state) = self.players.get_mut(&id) {
            if state.center != center {
                let desired = Self::desired(center, self.playerViewRadius);
                let desiredSet: HashSet<_> = desired.iter().copied().collect();
                unloads.extend(
                    state
                        .sent
                        .iter()
                        .copied()
                        .filter(|pos| !desiredSet.contains(pos)),
                );
                state.sent.retain(|pos| desiredSet.contains(pos));
                state.center = center;
                state.pending = desired
                    .into_iter()
                    .filter(|p| !state.sent.contains(p))
                    .collect();
            }
        }
        player.managedPosX = player.entity.posX;
        player.managedPosZ = player.entity.posZ;
        unloads
    }
    pub fn tickPlayer(
        &mut self,
        world: &mut WorldServer,
        network: &mut NetworkManager,
        player: &EntityPlayerMP,
    ) -> Result<usize, String> {
        let Some(id) = player.getGameProfile().getId() else {
            return Ok(0);
        };
        let Some(state) = self.players.get_mut(&id) else {
            return Ok(0);
        };
        let deadline = Instant::now() + Duration::from_millis(50);
        let mut generated = 0usize;
        let mut sent = 0usize;
        let hasSky = world.provider.hasSkyLight();
        while !state.pending.is_empty() && sent < 82 {
            let (x, z) = state.pending[0];
            if state.sent.contains(&(x, z)) {
                state.pending.remove(0);
                continue;
            }
            if !world.isChunkLoaded(x, z) {
                if generated >= 50 || Instant::now() > deadline {
                    break;
                }
                let _ = world.provideChunkSnapshot(x, z)?;
                generated += 1;
            }
            let chunk = world.provideChunkSnapshot(x, z)?;
            let packet = SPacketChunkData::new(&chunk, 65535, hasSky)
                .writePacketData()
                .map_err(|e| e.to_string())?;
            network.sendPacket(&packet).map_err(|e| e.to_string())?;
            state.pending.remove(0);
            state.sent.insert((x, z));
            sent += 1;
        }
        Ok(sent)
    }
    pub fn pendingFor(&self, id: Uuid) -> usize {
        self.players.get(&id).map_or(0, |s| s.pending.len())
    }
}
