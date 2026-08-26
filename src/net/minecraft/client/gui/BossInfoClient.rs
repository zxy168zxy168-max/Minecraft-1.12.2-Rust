use crate::net::minecraft::client::resources::Locale::Locale;
use crate::net::minecraft::network::play::server::SPacketUpdateBossInfo::{
    Operation, SPacketUpdateBossInfo,
};
use crate::net::minecraft::world::BossInfo::BossInfo;

/// Client-side boss state with MCP 1.12.2's 100 ms percent interpolation.
#[derive(Debug, Clone, PartialEq)]
pub struct BossInfoClient {
    info: BossInfo,
    rawPercent: f32,
    percentSetTime: u64,
}

impl BossInfoClient {
    pub fn new(
        packetIn: &SPacketUpdateBossInfo,
        systemTimeMillis: u64,
        locale: &Locale,
    ) -> Option<Self> {
        let name = packetIn.getName()?.resolveWithLocale(locale);
        let color = packetIn.getColor()?;
        let overlay = packetIn.getOverlay()?;
        let mut info = BossInfo::new(packetIn.getUniqueId(), name, color, overlay);
        info.setPercent(packetIn.getPercent());
        info.setDarkenSky(packetIn.shouldDarkenSky());
        info.setPlayEndBossMusic(packetIn.shouldPlayEndBossMusic());
        info.setCreateFog(packetIn.shouldCreateFog());
        Some(Self {
            rawPercent: packetIn.getPercent(),
            percentSetTime: systemTimeMillis,
            info,
        })
    }

    pub fn setPercent(&mut self, percentIn: f32, systemTimeMillis: u64) {
        let interpolated = self.getPercent(systemTimeMillis);
        self.info.setPercent(interpolated);
        self.rawPercent = percentIn;
        self.percentSetTime = systemTimeMillis;
    }

    pub fn getPercent(&self, systemTimeMillis: u64) -> f32 {
        let elapsed = systemTimeMillis.saturating_sub(self.percentSetTime);
        let interpolation = (elapsed as f32 / 100.0).clamp(0.0, 1.0);
        self.info.getPercent() + (self.rawPercent - self.info.getPercent()) * interpolation
    }

    pub fn updateFromPacket(
        &mut self,
        packetIn: &SPacketUpdateBossInfo,
        systemTimeMillis: u64,
        locale: &Locale,
    ) {
        match packetIn.getOperation() {
            Operation::UpdateName => {
                if let Some(name) = packetIn.getName() {
                    self.info.setName(name.resolveWithLocale(locale));
                }
            }
            Operation::UpdatePct => self.setPercent(packetIn.getPercent(), systemTimeMillis),
            Operation::UpdateStyle => {
                if let Some(color) = packetIn.getColor() {
                    self.info.setColor(color);
                }
                if let Some(overlay) = packetIn.getOverlay() {
                    self.info.setOverlay(overlay);
                }
            }
            Operation::UpdateProperties => {
                self.info.setDarkenSky(packetIn.shouldDarkenSky());
                self.info
                    .setPlayEndBossMusic(packetIn.shouldPlayEndBossMusic());
                self.info.setCreateFog(packetIn.shouldCreateFog());
            }
            Operation::Add | Operation::Remove => {}
        }
    }

    pub fn info(&self) -> &BossInfo {
        &self.info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::network::Packet::RawPacket;
    use crate::net::minecraft::network::PacketBuffer::{
        write_f32_be, write_string, write_uuid, write_var_i32,
    };
    use uuid::Uuid;

    fn add_packet(percent: f32) -> SPacketUpdateBossInfo {
        let mut payload = Vec::new();
        write_uuid(Uuid::nil(), &mut payload);
        write_var_i32(0, &mut payload);
        write_string(r#"{"text":"Boss"}"#, 32_767, &mut payload).unwrap();
        write_f32_be(percent, &mut payload);
        write_var_i32(2, &mut payload);
        write_var_i32(0, &mut payload);
        payload.push(0);
        SPacketUpdateBossInfo::readPacketData(&RawPacket::new(0x0C, payload)).unwrap()
    }

    #[test]
    fn percentage_interpolates_over_one_hundred_milliseconds() {
        let add = add_packet(1.0);
        let mut client = BossInfoClient::new(&add, 1_000, &Locale::default()).unwrap();
        let mut payload = Vec::new();
        write_uuid(Uuid::nil(), &mut payload);
        write_var_i32(2, &mut payload);
        write_f32_be(0.0, &mut payload);
        let update = SPacketUpdateBossInfo::readPacketData(&RawPacket::new(0x0C, payload)).unwrap();
        client.updateFromPacket(&update, 1_000, &Locale::default());
        assert!((client.getPercent(1_050) - 0.5).abs() < 0.001);
        assert_eq!(client.getPercent(1_100), 0.0);
    }
}
