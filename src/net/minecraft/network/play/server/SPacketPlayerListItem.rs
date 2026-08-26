use uuid::Uuid;

use crate::com::mojang::authlib::properties::Property::Property;
use crate::com::mojang::authlib::GameProfile::GameProfile;
use crate::net::minecraft::network::Packet::RawPacket;
use crate::net::minecraft::network::PacketBuffer::{
    read_bool, read_string, read_text_component, read_uuid, read_var_i32, CodecError,
};
use crate::net::minecraft::util::text::ITextComponent::ITextComponent;
use crate::net::minecraft::world::GameType::GameType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    AddPlayer,
    UpdateGameMode,
    UpdateLatency,
    UpdateDisplayName,
    RemovePlayer,
}
impl Action {
    fn fromId(id: i32) -> Result<Self, CodecError> {
        Ok(match id {
            0 => Self::AddPlayer,
            1 => Self::UpdateGameMode,
            2 => Self::UpdateLatency,
            3 => Self::UpdateDisplayName,
            4 => Self::RemovePlayer,
            _ => {
                return Err(CodecError::InvalidData(format!(
                    "invalid player-list action {id}"
                )))
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddPlayerData {
    pub profile: GameProfile,
    pub ping: i32,
    pub gameMode: GameType,
    pub displayName: Option<ITextComponent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SPacketPlayerListItem {
    action: Action,
    players: Vec<AddPlayerData>,
}
impl SPacketPlayerListItem {
    pub fn readPacketData(packet: &RawPacket) -> Result<Self, CodecError> {
        let mut input = packet.payload.as_slice();
        let action = Action::fromId(read_var_i32(&mut input)?)?;
        let count = read_var_i32(&mut input)?;
        if count < 0 || count > 100_000 {
            return Err(CodecError::InvalidData(format!(
                "invalid player-list count {count}"
            )));
        }
        let mut players = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let id: Uuid = read_uuid(&mut input)?;
            let mut profile = GameProfile::new(Some(id), "");
            let mut ping = 0;
            let mut gameMode = GameType::NotSet;
            let mut displayName = None;
            match action {
                Action::AddPlayer => {
                    profile = GameProfile::new(Some(id), read_string(&mut input, 16)?);
                    let propertyCount = read_var_i32(&mut input)?;
                    if propertyCount < 0 || propertyCount > 16_384 {
                        return Err(CodecError::InvalidData(format!(
                            "invalid profile-property count {propertyCount}"
                        )));
                    }
                    for _ in 0..propertyCount {
                        let name = read_string(&mut input, 32767)?;
                        let value = read_string(&mut input, 32767)?;
                        let signature = if read_bool(&mut input)? {
                            Some(read_string(&mut input, 32767)?)
                        } else {
                            None
                        };
                        profile.addProperty(Property::new(name, value, signature));
                    }
                    gameMode = GameType::getByID(read_var_i32(&mut input)?);
                    ping = read_var_i32(&mut input)?;
                    if read_bool(&mut input)? {
                        displayName = Some(read_text_component(&mut input)?);
                    }
                }
                Action::UpdateGameMode => {
                    gameMode = GameType::getByID(read_var_i32(&mut input)?);
                }
                Action::UpdateLatency => {
                    ping = read_var_i32(&mut input)?;
                }
                Action::UpdateDisplayName => {
                    if read_bool(&mut input)? {
                        displayName = Some(read_text_component(&mut input)?);
                    }
                }
                Action::RemovePlayer => {}
            }
            players.push(AddPlayerData {
                profile,
                ping,
                gameMode,
                displayName,
            });
        }
        if !input.is_empty() {
            return Err(CodecError::InvalidData(format!(
                "{} unread player-list bytes",
                input.len()
            )));
        }
        Ok(Self { action, players })
    }
    pub const fn getAction(&self) -> Action {
        self.action
    }
    pub fn getEntries(&self) -> &[AddPlayerData] {
        &self.players
    }
}
