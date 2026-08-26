use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// Geometry parameters produced by MCP 1.12.2
/// `TileEntityBeaconRenderer#renderBeamSegment`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BeaconBeamSegmentGeometry {
    pub innerCorners: [[f32; 2]; 4],
    pub outerCorners: [[f32; 2]; 4],
    pub innerV: [f32; 2],
    pub outerV: [f32; 2],
}

pub struct TileEntityBeaconRenderer;

impl TileEntityBeaconRenderer {
    pub fn texture() -> ResourceLocation {
        ResourceLocation::new("minecraft", "textures/entity/beacon_beam.png")
    }

    pub fn segmentGeometry(
        partialTicks: f32,
        textureScale: f32,
        totalWorldTime: i64,
        height: i32,
        beamRadius: f32,
        glowRadius: f32,
    ) -> BeaconBeamSegmentGeometry {
        let time = totalWorldTime as f64 + partialTicks as f64;
        let directionTime = if height < 0 { time } else { -time };
        let scroll = (directionTime * 0.2 - (directionTime * 0.1).floor()).rem_euclid(1.0) as f32;
        let rotation = time as f32 * 0.025 * -1.5;
        let angles = [
            rotation + 2.356_194_5,
            rotation + std::f32::consts::FRAC_PI_4,
            rotation + 3.926_990_7,
            rotation + 5.497_787_0,
        ];
        let mut inner = [[0.0; 2]; 4];
        for (corner, angle) in inner.iter_mut().zip(angles) {
            *corner = [
                0.5 + angle.cos() * beamRadius,
                0.5 + angle.sin() * beamRadius,
            ];
        }
        BeaconBeamSegmentGeometry {
            innerCorners: inner,
            outerCorners: [
                [0.5 - glowRadius, 0.5 - glowRadius],
                [0.5 + glowRadius, 0.5 - glowRadius],
                [0.5 - glowRadius, 0.5 + glowRadius],
                [0.5 + glowRadius, 0.5 + glowRadius],
            ],
            innerV: [
                -1.0 + scroll,
                height as f32 * textureScale * (0.5 / beamRadius) - 1.0 + scroll,
            ],
            outerV: [-1.0 + scroll, height as f32 * textureScale - 1.0 + scroll],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_direction_time_uses_java_fraction_semantics() {
        let geometry = TileEntityBeaconRenderer::segmentGeometry(0.0, 1.0, 6, 10, 0.2, 0.25);
        assert!((geometry.outerV[0] - -0.2).abs() < 1.0e-6);
        assert!((geometry.outerV[1] - 9.8).abs() < 1.0e-6);
    }

    #[test]
    fn source_radii_are_preserved() {
        let geometry = TileEntityBeaconRenderer::segmentGeometry(0.0, 1.0, 0, 10, 0.2, 0.25);
        let dx = geometry.innerCorners[0][0] - 0.5;
        let dz = geometry.innerCorners[0][1] - 0.5;
        assert!(((dx * dx + dz * dz).sqrt() - 0.2).abs() < 1.0e-5);
        assert_eq!(geometry.outerCorners[0], [0.25, 0.25]);
        assert_eq!(geometry.outerCorners[3], [0.75, 0.75]);
    }
}
