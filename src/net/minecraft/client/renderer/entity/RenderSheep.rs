use crate::net::minecraft::client::entity::EntityOtherClient::MobEntityType;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

pub struct RenderSheep;

impl RenderSheep {
    pub fn supports(entityType: MobEntityType) -> bool {
        entityType.registryName == "sheep"
    }
    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/sheep/sheep.png")
    }
    pub fn woolColor(metadata: u8) -> [f32; 4] {
        const COLORS: [u32; 16] = [
            0xF9FFFE, 0xF9801D, 0xC74EBD, 0x3AB3DA, 0xFED83D, 0x80C71F, 0xF38BAA, 0x474F52,
            0x9D9D97, 0x169C9C, 0x8932B8, 0x3C44AA, 0x835432, 0x5E7C16, 0xB02E26, 0x1D1D21,
        ];
        let rgb = COLORS.get(metadata as usize).copied().unwrap_or(COLORS[0]);
        if metadata == 0 {
            return [0.9019608, 0.9019608, 0.9019608, 1.0];
        }
        [
            ((rgb >> 16) & 255) as f32 / 255.0 * 0.75,
            ((rgb >> 8) & 255) as f32 / 255.0 * 0.75,
            (rgb & 255) as f32 / 255.0 * 0.75,
            1.0,
        ]
    }

    pub fn jebColor(entityId: i32, ticksExisted: i32, partialTicks: f32) -> [f32; 4] {
        let cycle = ticksExisted.div_euclid(25) + entityId;
        let first = cycle.rem_euclid(16) as u8;
        let second = (cycle + 1).rem_euclid(16) as u8;
        let factor = (ticksExisted.rem_euclid(25) as f32 + partialTicks) / 25.0;
        let a = Self::woolColor(first);
        let b = Self::woolColor(second);
        [
            a[0] * (1.0 - factor) + b[0] * factor,
            a[1] * (1.0 - factor) + b[1] * factor,
            a[2] * (1.0 - factor) + b[2] * factor,
            1.0,
        ]
    }
}
