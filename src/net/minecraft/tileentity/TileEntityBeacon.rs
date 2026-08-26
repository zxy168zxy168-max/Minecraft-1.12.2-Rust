use std::sync::atomic::{AtomicI64, AtomicU32, Ordering};

use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::math::BlockPos::BlockPos;

/// MCP 1.12.2 `TileEntityBeacon.BeamSegment`.
#[derive(Debug, Clone, PartialEq)]
pub struct BeamSegment {
    pub colors: [f32; 3],
    pub height: i32,
}

impl BeamSegment {
    pub const fn new(colors: [f32; 3]) -> Self {
        Self { colors, height: 1 }
    }
    pub fn incrementHeight(&mut self) {
        self.height += 1;
    }
}

/// Client-visible state from MCP 1.12.2 `TileEntityBeacon`.
///
/// Effect application remains server-authoritative; the client owns the
/// pyramid/beam scan and the renderer fade exactly because those values are
/// not transmitted in a dedicated packet.
#[derive(Debug)]
pub struct TileEntityBeacon {
    pub pos: BlockPos,
    beamSegments: Vec<BeamSegment>,
    beamRenderCounter: AtomicI64,
    beamRenderScaleBits: AtomicU32,
    isComplete: bool,
    levels: i32,
    primaryEffect: i32,
    secondaryEffect: i32,
}

impl Clone for TileEntityBeacon {
    fn clone(&self) -> Self {
        Self {
            pos: self.pos,
            beamSegments: self.beamSegments.clone(),
            beamRenderCounter: AtomicI64::new(self.beamRenderCounter.load(Ordering::Relaxed)),
            beamRenderScaleBits: AtomicU32::new(self.beamRenderScaleBits.load(Ordering::Relaxed)),
            isComplete: self.isComplete,
            levels: self.levels,
            primaryEffect: self.primaryEffect,
            secondaryEffect: self.secondaryEffect,
        }
    }
}

impl TileEntityBeacon {
    pub const BLOCK_ID: i32 = 138;

    pub const fn new(pos: BlockPos) -> Self {
        Self {
            pos,
            beamSegments: Vec::new(),
            beamRenderCounter: AtomicI64::new(0),
            beamRenderScaleBits: AtomicU32::new(0),
            isComplete: false,
            levels: -1,
            primaryEffect: 0,
            secondaryEffect: 0,
        }
    }

    pub fn fromNbt(tag: &NBTTagCompound) -> Option<Self> {
        let id = tag.getString("id");
        if !id.is_empty() && id != "minecraft:beacon" && id != "Beacon" {
            return None;
        }
        let mut beacon = Self::new(BlockPos::new(
            tag.getInteger("x"),
            tag.getInteger("y"),
            tag.getInteger("z"),
        ));
        beacon.levels = tag.getInteger("Levels");
        beacon.primaryEffect = tag.getInteger("Primary");
        beacon.secondaryEffect = tag.getInteger("Secondary");
        Some(beacon)
    }

    pub fn update<F>(&mut self, totalWorldTime: i64, mut getState: F)
    where
        F: FnMut(BlockPos) -> IBlockState,
    {
        if totalWorldTime.rem_euclid(80) == 0 || self.beamSegments.is_empty() {
            self.updateSegmentColors(&mut getState);
        }
    }

    /// Port of MCP `TileEntityBeacon#shouldBeamRender`.
    ///
    /// The Java renderer mutates this interpolation state once per rendered
    /// frame. Rust capture currently owns an immutable world snapshot, so the
    /// caller advances it immediately before capture through this method.
    pub fn shouldBeamRender(&self, totalWorldTime: i64) -> f32 {
        if !self.isComplete {
            return 0.0;
        }

        let previousCounter = self
            .beamRenderCounter
            .swap(totalWorldTime, Ordering::Relaxed);
        let elapsed = totalWorldTime.saturating_sub(previousCounter) as i32;
        let mut scale = f32::from_bits(self.beamRenderScaleBits.load(Ordering::Relaxed));
        if elapsed > 1 {
            scale -= elapsed as f32 / 40.0;
            if scale < 0.0 {
                scale = 0.0;
            }
        }
        scale = (scale + 0.025).min(1.0);
        self.beamRenderScaleBits
            .store(scale.to_bits(), Ordering::Relaxed);
        scale
    }

    pub fn beamRenderScale(&self) -> f32 {
        if self.isComplete {
            f32::from_bits(self.beamRenderScaleBits.load(Ordering::Relaxed))
        } else {
            0.0
        }
    }

    pub fn getBeamSegments(&self) -> &[BeamSegment] {
        &self.beamSegments
    }
    pub const fn getLevels(&self) -> i32 {
        self.levels
    }
    pub const fn isComplete(&self) -> bool {
        self.isComplete
    }

    /// Port of `TileEntityBeacon#updateSegmentColors`.
    fn updateSegmentColors<F>(&mut self, getState: &mut F)
    where
        F: FnMut(BlockPos) -> IBlockState,
    {
        let i = self.pos.x;
        let j = self.pos.y;
        let k = self.pos.z;
        self.levels = 0;
        self.beamSegments.clear();
        self.isComplete = true;
        self.beamSegments.push(BeamSegment::new(dye_color(0)));
        let mut firstColor = true;

        for y in (j + 1)..256 {
            let state = getState(BlockPos::new(i, y, k));
            let color = match state.getBlockId() {
                95 | 160 => Some(dye_color(state.getMetadata().clamp(0, 15) as usize)),
                _ => None,
            };
            if let Some(mut color) = color {
                let currentColors = self
                    .beamSegments
                    .last()
                    .map(|segment| segment.colors)
                    .unwrap_or_else(|| dye_color(0));
                if !firstColor {
                    color = [
                        (currentColors[0] + color[0]) * 0.5,
                        (currentColors[1] + color[1]) * 0.5,
                        (currentColors[2] + color[2]) * 0.5,
                    ];
                }
                if colors_equal(color, currentColors) {
                    if let Some(last) = self.beamSegments.last_mut() {
                        last.incrementHeight();
                    }
                } else {
                    self.beamSegments.push(BeamSegment::new(color));
                }
                firstColor = false;
            } else if light_opacity(state) >= 15 && state.getBlockId() != 7 {
                self.isComplete = false;
                self.beamSegments.clear();
                break;
            } else {
                if let Some(last) = self.beamSegments.last_mut() {
                    last.incrementHeight();
                }
            }
        }

        if self.isComplete {
            for level in 1..=4 {
                let y = j - level;
                if y < 0 {
                    break;
                }
                let mut valid = true;
                'outer: for x in (i - level)..=(i + level) {
                    for z in (k - level)..=(k + level) {
                        if !matches!(
                            getState(BlockPos::new(x, y, z)).getBlockId(),
                            42 | 41 | 57 | 133
                        ) {
                            valid = false;
                            break 'outer;
                        }
                    }
                }
                if !valid {
                    break;
                }
                self.levels = level;
            }
            if self.levels == 0 {
                self.isComplete = false;
            }
        }
    }
}

fn colors_equal(a: [f32; 3], b: [f32; 3]) -> bool {
    (a[0] - b[0]).abs() < f32::EPSILON
        && (a[1] - b[1]).abs() < f32::EPSILON
        && (a[2] - b[2]).abs() < f32::EPSILON
}

fn light_opacity(state: IBlockState) -> i32 {
    // `IBlockState#getLightOpacity` delegates to the block's configured
    // opacity. For beacon scanning, the relevant 1.12.2 distinction is
    // opaque full blocks (15) versus transparent/cutout blocks (0).
    if state.getBlock().isOpaqueCube() {
        15
    } else {
        0
    }
}

fn dye_color(meta: usize) -> [f32; 3] {
    // `EnumDyeColor.field_193351_w`, metadata order.
    const COLORS: [u32; 16] = [
        16_383_998, 16_351_261, 13_061_821, 3_847_130, 16_701_501, 8_439_583, 15_961_002,
        4_673_362, 10_329_495, 1_481_884, 8_991_416, 3_949_738, 8_606_770, 6_192_150, 11_546_150,
        1_908_001,
    ];
    let color = COLORS[meta.min(15)];
    [
        ((color >> 16) & 255) as f32 / 255.0,
        ((color >> 8) & 255) as f32 / 255.0,
        (color & 255) as f32 / 255.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn beacon_state_is_send_and_sync() {
        assert_send_sync::<TileEntityBeacon>();
    }

    #[test]
    fn four_layer_pyramid_completes_and_beam_reaches_build_height() {
        let mut beacon = TileEntityBeacon::new(BlockPos::new(0, 10, 0));
        beacon.update(0, |pos| {
            if (6..=9).contains(&pos.y) && pos.x.abs() <= 10 - pos.y && pos.z.abs() <= 10 - pos.y {
                IBlockState::fromGlobalStateId(42 << 4)
            } else {
                IBlockState::fromGlobalStateId(0)
            }
        });
        assert!(beacon.isComplete());
        assert_eq!(beacon.getLevels(), 4);
        // Segment starts at height 1 for the beacon block itself and grows
        // once for each block from y+1 through build height 255.
        assert_eq!(beacon.getBeamSegments()[0].height, 246);
    }
}
