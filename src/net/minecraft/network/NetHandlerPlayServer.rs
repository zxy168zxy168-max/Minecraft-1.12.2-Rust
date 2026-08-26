use crate::net::minecraft::entity::player::EntityPlayerMP::EntityPlayerMP;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::network::play::server::SPacketPlayerPosLook::{
    EnumFlags, SPacketPlayerPosLook,
};
use crate::net::minecraft::network::NetworkManager::NetworkManager;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    read_bool, read_f32_be, read_f64_be, read_i16_be, read_i64_be, read_u8, read_var_i32,
};
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::EnumFacing::EnumFacing;
use crate::net::minecraft::util::EnumHand::EnumHand;
use crate::net::minecraft::world::WorldServer::WorldServer;

/// Source-shaped integrated-server port of MCP `NetHandlerPlayServer`.
/// Movement plus the first server-authoritative inventory/block interaction
/// handlers are owned here; unsupported entity/container/use-item branches are
/// deliberately left distinct instead of being folded into block placement.
#[derive(Debug, Default)]
pub struct NetHandlerPlayServer {
    teleportId: i32,
    targetPos: Option<[f64; 3]>,
    networkTickCount: i32,
    lastPositionUpdate: i32,
}
impl NetHandlerPlayServer {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn setPlayerLocation(
        &mut self,
        network: &mut NetworkManager,
        player: &mut EntityPlayerMP,
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
    ) -> Result<(), String> {
        self.targetPos = Some([x, y, z]);
        self.teleportId = self.teleportId.wrapping_add(1);
        if self.teleportId == i32::MAX {
            self.teleportId = 0;
        }
        self.lastPositionUpdate = self.networkTickCount;
        player.setPlayerLocation(x, y, z, yaw, pitch);
        network
            .sendPacket(
                &SPacketPlayerPosLook::new(
                    x,
                    y,
                    z,
                    yaw,
                    pitch,
                    EnumFlags::empty(),
                    self.teleportId,
                )
                .writePacketData(),
            )
            .map_err(|e| e.to_string())
    }
    pub fn update(&mut self) {
        self.networkTickCount = self.networkTickCount.wrapping_add(1);
    }

    pub fn processPacket(
        &mut self,
        network: &mut NetworkManager,
        world: &mut WorldServer,
        player: &mut EntityPlayerMP,
        raw: &RawPacket,
    ) -> Result<bool, String> {
        match raw.id {
            0x00 => {
                let mut input = raw.payload.as_slice();
                let id = read_var_i32(&mut input).map_err(|e| e.to_string())?;
                if id == self.teleportId {
                    if let Some([x, y, z]) = self.targetPos.take() {
                        let (yaw, pitch) = (player.entity.rotationYaw, player.entity.rotationPitch);
                        player.setPlayerLocation(x, y, z, yaw, pitch);
                    }
                }
                Ok(false)
            }
            0x0C => {
                let mut input = raw.payload.as_slice();
                player.entity.onGround = read_bool(&mut input).map_err(|e| e.to_string())?;
                Ok(false)
            }
            0x0D => {
                let mut input = raw.payload.as_slice();
                let x = read_f64_be(&mut input).map_err(|e| e.to_string())?;
                let y = read_f64_be(&mut input).map_err(|e| e.to_string())?;
                let z = read_f64_be(&mut input).map_err(|e| e.to_string())?;
                let on = read_bool(&mut input).map_err(|e| e.to_string())?;
                let (yaw, pitch) = (player.entity.rotationYaw, player.entity.rotationPitch);
                player.setPlayerLocation(x, y, z, yaw, pitch);
                player.entity.onGround = on;
                Ok(true)
            }
            0x0E => {
                let mut input = raw.payload.as_slice();
                let x = read_f64_be(&mut input).map_err(|e| e.to_string())?;
                let y = read_f64_be(&mut input).map_err(|e| e.to_string())?;
                let z = read_f64_be(&mut input).map_err(|e| e.to_string())?;
                let yaw = read_f32_be(&mut input).map_err(|e| e.to_string())?;
                let pitch = read_f32_be(&mut input).map_err(|e| e.to_string())?;
                let on = read_bool(&mut input).map_err(|e| e.to_string())?;
                player.setPlayerLocation(x, y, z, yaw, pitch);
                player.entity.onGround = on;
                Ok(true)
            }
            0x0F => {
                let mut input = raw.payload.as_slice();
                let yaw = read_f32_be(&mut input).map_err(|e| e.to_string())?;
                let pitch = read_f32_be(&mut input).map_err(|e| e.to_string())?;
                let on = read_bool(&mut input).map_err(|e| e.to_string())?;
                let (x, y, z) = (player.entity.posX, player.entity.posY, player.entity.posZ);
                player.setPlayerLocation(x, y, z, yaw, pitch);
                player.entity.onGround = on;
                Ok(false)
            }

            // MCP `processEntityAction`: the server must own the player's
            // sneaking state because right-click block activation versus
            // ItemBlock placement depends on it. Other action ordinals retain
            // their separate future handlers rather than being conflated here.
            0x15 => {
                let mut input = raw.payload.as_slice();
                let _entity_id = read_var_i32(&mut input).map_err(|e| e.to_string())?;
                let action = read_var_i32(&mut input).map_err(|e| e.to_string())?;
                let _aux = read_var_i32(&mut input).map_err(|e| e.to_string())?;
                match action {
                    0 => player.entity.sneaking = true,
                    1 => player.entity.sneaking = false,
                    _ => {}
                }
                Ok(false)
            }

            // MCP `processPlayerDigging` (first authoritative branches).
            0x14 => {
                let mut input = raw.payload.as_slice();
                let action = read_var_i32(&mut input).map_err(|e| e.to_string())?;
                let pos = BlockPos::from_long(read_i64_be(&mut input).map_err(|e| e.to_string())?);
                let side =
                    EnumFacing::getFront(read_u8(&mut input).map_err(|e| e.to_string())? as i32);
                match action {
                    6 => {
                        // SWAP_HELD_ITEMS
                        if player.getGameType()
                            != crate::net::minecraft::world::GameType::GameType::Spectator
                        {
                            let off = player.getHeldItem(EnumHand::OffHand).clone();
                            let main = player.getHeldItem(EnumHand::MainHand).clone();
                            player.setHeldItem(EnumHand::OffHand, main);
                            player.setHeldItem(EnumHand::MainHand, off);
                        }
                    }
                    0 => {
                        // START_DESTROY_BLOCK
                        let dx = player.entity.posX - (pos.x as f64 + 0.5);
                        let dy = player.entity.posY - (pos.y as f64 + 0.5) + 1.5;
                        let dz = player.entity.posZ - (pos.z as f64 + 0.5);
                        if dx * dx + dy * dy + dz * dz <= 36.0 && pos.y < 256 {
                            let mut manager = player.interactionManager.clone();
                            let _ = manager.onBlockClicked(network, world, player, pos, side)?;
                            player.interactionManager = manager;
                        }
                    }
                    1 | 2 => {
                        // ABORT/STOP: progressive survival destroy remains pending; server corrects state.
                        crate::net::minecraft::server::management::PlayerInteractionManager::PlayerInteractionManager::sendBlockChange(network,world,pos)?;
                    }
                    _ => {}
                }
                Ok(false)
            }

            // MCP `processHeldItemChange`.
            0x1A => {
                let mut input = raw.payload.as_slice();
                let slot = read_i16_be(&mut input).map_err(|e| e.to_string())? as i32;
                if (0..9).contains(&slot) {
                    player
                        .inventory
                        .setCurrentItem(slot)
                        .map_err(|e| e.to_string())?;
                }
                Ok(false)
            }

            // MCP `processCreativeInventoryAction`, player-inventory slots.
            0x1B => {
                let mut input = raw.payload.as_slice();
                let slot = read_i16_be(&mut input).map_err(|e| e.to_string())? as i32;
                let stack = ItemStack::readFromBuffer(&mut input).map_err(|e| e.to_string())?;
                if player.interactionManager.isCreative() && (1..=45).contains(&slot) {
                    player
                        .inventory
                        .applyContainerPlayerSlot(slot, stack)
                        .map_err(|e| e.to_string())?;
                }
                Ok(false)
            }

            // MCP `processRightClickBlock`.
            0x1F => {
                let mut input = raw.payload.as_slice();
                let pos = BlockPos::from_long(read_i64_be(&mut input).map_err(|e| e.to_string())?);
                let side =
                    EnumFacing::getFront(read_var_i32(&mut input).map_err(|e| e.to_string())?);
                let hand = match read_var_i32(&mut input).map_err(|e| e.to_string())? {
                    0 => EnumHand::MainHand,
                    1 => EnumHand::OffHand,
                    _ => return Ok(false),
                };
                let hit_x = read_f32_be(&mut input).map_err(|e| e.to_string())?;
                let hit_y = read_f32_be(&mut input).map_err(|e| e.to_string())?;
                let hit_z = read_f32_be(&mut input).map_err(|e| e.to_string())?;
                let dx = player.entity.posX - (pos.x as f64 + 0.5);
                let dy = player.entity.posY - (pos.y as f64 + 0.5);
                let dz = player.entity.posZ - (pos.z as f64 + 0.5);
                if dx * dx + dy * dy + dz * dz < 64.0 {
                    let mut manager = player.interactionManager.clone();
                    let _ = manager.processRightClickBlock(
                        network, world, player, hand, pos, side, hit_x, hit_y, hit_z,
                    )?;
                    player.interactionManager = manager;
                }
                Ok(false)
            }

            // client settings / brand are accepted here; their full server-side
            // preference ownership is a later EntityPlayerMP tranche.
            0x04 | 0x09 => Ok(false),
            _ => Ok(false),
        }
    }
}
