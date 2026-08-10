use crate::net::minecraft::world::WorldProvider::WorldProvider;

/// Values consumed by the Vulkan equivalent of vanilla's 16 x 16 lightmap.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightmapParameters {
    pub sunBrightness: f32,
    pub torchFlickerX: f32,
    pub gammaSetting: f32,
    pub dimension: i32,
}

/// Rendering-light subset of MCP 1.12.2 `EntityRenderer`.
///
/// Both native backends now consume the same CPU-built 16 x 16 lightmap,
/// mirroring vanilla's `DynamicTexture` responsibility. RenderChunk geometry
/// remains immutable when time, torch flicker or gamma changes; only the tiny
/// lightmap texture is refreshed.
pub struct EntityRenderer;

impl EntityRenderer {
    pub fn getSunBrightness(
        provider: &WorldProvider,
        worldTime: i64,
        partialTicks: f32,
    ) -> f32 {
        let angle = provider.calculateCelestialAngle(worldTime, partialTicks);
        let mut brightness = 1.0 - ((angle * std::f32::consts::TAU).cos() * 2.0 + 0.2);
        brightness = brightness.clamp(0.0, 1.0);
        (1.0 - brightness) * 0.8 + 0.2
    }

    pub fn lightmapParameters(
        provider: &WorldProvider,
        worldTime: i64,
        partialTicks: f32,
        torchFlickerX: f32,
        gammaSetting: f32,
    ) -> LightmapParameters {
        LightmapParameters {
            sunBrightness: Self::getSunBrightness(provider, worldTime, partialTicks),
            torchFlickerX,
            gammaSetting: gammaSetting.clamp(0.0, 1.0),
            dimension: provider.getDimension(),
        }
    }

    /// Vanilla `EntityRenderer.updateLightmap` for one discrete sky/block pair.
    /// Weather, lightning, boss tint and night vision are not fabricated here;
    /// they will be connected after their source packets/state are ported.
    pub fn lightmapColor(
        provider: &WorldProvider,
        skyLight: u8,
        blockLight: u8,
        parameters: LightmapParameters,
    ) -> [f32; 3] {
        let table = provider.getLightBrightnessTable();
        let sun = parameters.sunBrightness;
        let sunScale = sun * 0.95 + 0.05;
        let sky = table[skyLight.min(15) as usize] * sunScale;
        let block = table[blockLight.min(15) as usize]
            * (parameters.torchFlickerX * 0.1 + 1.5);

        let skyRedGreen = sky * (sun * 0.65 + 0.35);
        let blockGreen = block * ((block * 0.6 + 0.4) * 0.6 + 0.4);
        let blockBlue = block * (block * block * 0.6 + 0.4);
        let mut red = skyRedGreen + block;
        let mut green = skyRedGreen + blockGreen;
        let mut blue = sky + blockBlue;
        red = red * 0.96 + 0.03;
        green = green * 0.96 + 0.03;
        blue = blue * 0.96 + 0.03;

        if parameters.dimension == 1 {
            red = 0.22 + block * 0.75;
            green = 0.28 + blockGreen * 0.75;
            blue = 0.25 + blockBlue * 0.75;
        }

        red = red.clamp(0.0, 1.0);
        green = green.clamp(0.0, 1.0);
        blue = blue.clamp(0.0, 1.0);

        let gamma = parameters.gammaSetting;
        let gammaRed = 1.0 - (1.0 - red).powi(4);
        let gammaGreen = 1.0 - (1.0 - green).powi(4);
        let gammaBlue = 1.0 - (1.0 - blue).powi(4);
        red = red * (1.0 - gamma) + gammaRed * gamma;
        green = green * (1.0 - gamma) + gammaGreen * gamma;
        blue = blue * (1.0 - gamma) + gammaBlue * gamma;

        [
            (red * 0.96 + 0.03).clamp(0.0, 1.0),
            (green * 0.96 + 0.03).clamp(0.0, 1.0),
            (blue * 0.96 + 0.03).clamp(0.0, 1.0),
        ]
    }

    /// Builds the 16 x 16 RGBA lightmap backing MCP `DynamicTexture`.
    /// Indexing matches `EntityRenderer#updateLightmap`: sky light is the
    /// major coordinate (`i / 16`) and block light is the minor coordinate
    /// (`i % 16`). The currently ported state intentionally excludes weather,
    /// lightning, boss tint and night vision until those authoritative source
    /// states are connected; both native render backends consume this one
    /// implementation so they cannot drift from each other.
    pub fn buildLightmapRgba(parameters: LightmapParameters) -> [u8; 16 * 16 * 4] {
        let provider = WorldProvider::new(parameters.dimension);
        let mut rgba = [0_u8; 16 * 16 * 4];
        for skyLight in 0_u8..16 {
            for blockLight in 0_u8..16 {
                let color = Self::lightmapColor(&provider, skyLight, blockLight, parameters);
                let offset = (skyLight as usize * 16 + blockLight as usize) * 4;
                rgba[offset] = (color[0] * 255.0).floor() as u8;
                rgba[offset + 1] = (color[1] * 255.0).floor() as u8;
                rgba[offset + 2] = (color[2] * 255.0).floor() as u8;
                rgba[offset + 3] = 255;
            }
        }
        rgba
    }

    /// Builds the native-backend texture directly from the compact frame
    /// parameters stored in `WorldPushConstants`.
    pub fn buildLightmapRgbaFromArray(parameters: [f32; 4]) -> [u8; 16 * 16 * 4] {
        Self::buildLightmapRgba(LightmapParameters {
            sunBrightness: parameters[0],
            torchFlickerX: parameters[1],
            gammaSetting: parameters[2].clamp(0.0, 1.0),
            dimension: parameters[3] as i32,
        })
    }

    /// MCP `updateTorchFlicker`, with random samples supplied by the caller so
    /// the state transition remains testable.
    pub fn updateTorchFlicker(
        torchFlickerX: &mut f32,
        torchFlickerDX: &mut f32,
        randomA: f64,
        randomB: f64,
        randomC: f64,
        randomD: f64,
    ) {
        *torchFlickerDX += ((randomA - randomB) * randomC * randomD) as f32;
        *torchFlickerDX *= 0.9;
        *torchFlickerX += *torchFlickerDX - *torchFlickerX;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_sky_and_block_light_reach_white_range() {
        let provider = WorldProvider::new(0);
        let parameters =
            EntityRenderer::lightmapParameters(&provider, 6_000, 0.0, 0.0, 0.0);
        let color = EntityRenderer::lightmapColor(&provider, 15, 15, parameters);
        assert!(color
            .iter()
            .all(|channel| *channel > 0.95 && *channel <= 1.0));
    }

    #[test]
    fn gamma_brightens_dark_lightmap_texels() {
        let provider = WorldProvider::new(0);
        let dark = EntityRenderer::lightmapColor(
            &provider,
            0,
            2,
            EntityRenderer::lightmapParameters(&provider, 18_000, 0.0, 0.0, 0.0),
        );
        let bright = EntityRenderer::lightmapColor(
            &provider,
            0,
            2,
            EntityRenderer::lightmapParameters(&provider, 18_000, 0.0, 0.0, 1.0),
        );
        assert!(bright[0] > dark[0]);
        assert!(bright[1] > dark[1]);
        assert!(bright[2] > dark[2]);
    }
}
