use crate::compat::Java::JavaRandom;
use crate::net::minecraft::block::state::IBlockState::IBlockState;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::math::MathHelper::{cos, floor_f64, sin, PI};
use crate::net::minecraft::world::biome::BiomeProviderKind::BiomeProviderKind;
use crate::net::minecraft::world::chunk::ChunkPrimer::ChunkPrimer;
use crate::net::minecraft::world::gen::MapGenBase::MapGenBase;

/// Exact MCP 1.12.2 `MapGenRavine` carving algorithm.
#[derive(Debug, Clone)]
pub struct MapGenRavine {
    base: MapGenBase,
    rs: [f32; 1024],
}
impl Default for MapGenRavine {
    fn default() -> Self {
        Self {
            base: MapGenBase::new(),
            rs: [0.0; 1024],
        }
    }
}
impl MapGenRavine {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn generate(
        &mut self,
        world_seed: i64,
        provider: &BiomeProviderKind,
        current_top: &[IBlockState; 256],
        target_x: i32,
        target_z: i32,
        primer: &mut ChunkPrimer,
    ) {
        let range = self.base.range;
        self.base.rand.set_seed(world_seed);
        let j = self.base.rand.next_i64();
        let k = self.base.rand.next_i64();
        for sx in target_x - range..=target_x + range {
            for sz in target_z - range..=target_z + range {
                self.base.rand.set_seed(
                    (sx as i64).wrapping_mul(j) ^ (sz as i64).wrapping_mul(k) ^ world_seed,
                );
                self.recursiveGenerate(provider, current_top, sx, sz, target_x, target_z, primer);
            }
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn addTunnel(
        &mut self,
        provider: &BiomeProviderKind,
        current_top: &[IBlockState; 256],
        seed: i64,
        target_x: i32,
        target_z: i32,
        primer: &mut ChunkPrimer,
        mut x: f64,
        mut y: f64,
        mut z: f64,
        size: f32,
        mut yaw: f32,
        mut pitch: f32,
        mut step: i32,
        mut max_steps: i32,
        vertical_scale: f64,
    ) {
        let mut random = JavaRandom::new(seed);
        let center_x = (target_x * 16 + 8) as f64;
        let center_z = (target_z * 16 + 8) as f64;
        let mut yaw_vel = 0.0_f32;
        let mut pitch_vel = 0.0_f32;
        if max_steps <= 0 {
            let i = self.base.range * 16 - 16;
            max_steps = i - random.next_i32_bound(i / 4);
        }
        let mut single = false;
        if step == -1 {
            step = max_steps / 2;
            single = true;
        }
        let mut f2 = 1.0_f32;
        for j in 0..256usize {
            if j == 0 || random.next_i32_bound(3) == 0 {
                f2 = 1.0 + random.next_f32() * random.next_f32();
            }
            self.rs[j] = f2 * f2;
        }
        while step < max_steps {
            let mut radius = 1.5 + sin(step as f32 * PI / max_steps as f32) as f64 * size as f64;
            let mut vradius = radius * vertical_scale;
            radius *= random.next_f32() as f64 * 0.25 + 0.75;
            vradius *= random.next_f32() as f64 * 0.25 + 0.75;
            let cp = cos(pitch);
            let sp = sin(pitch);
            x += (cos(yaw) * cp) as f64;
            y += sp as f64;
            z += (sin(yaw) * cp) as f64;
            pitch *= 0.7;
            pitch += pitch_vel * 0.05;
            yaw += yaw_vel * 0.05;
            pitch_vel *= 0.8;
            yaw_vel *= 0.5;
            pitch_vel += (random.next_f32() - random.next_f32()) * random.next_f32() * 2.0;
            yaw_vel += (random.next_f32() - random.next_f32()) * random.next_f32() * 4.0;
            if single || random.next_i32_bound(4) != 0 {
                let dx = x - center_x;
                let dz = z - center_z;
                let remain = (max_steps - step) as f64;
                let bound = (size + 2.0 + 16.0) as f64;
                if dx * dx + dz * dz - remain * remain > bound * bound {
                    return;
                }
                if x >= center_x - 16.0 - radius * 2.0
                    && z >= center_z - 16.0 - radius * 2.0
                    && x <= center_x + 16.0 + radius * 2.0
                    && z <= center_z + 16.0 + radius * 2.0
                {
                    let mut min_x = floor_f64(x - radius) - target_x * 16 - 1;
                    let mut max_x = floor_f64(x + radius) - target_x * 16 + 1;
                    let mut min_y = floor_f64(y - vradius) - 1;
                    let mut max_y = floor_f64(y + vradius) + 1;
                    let mut min_z = floor_f64(z - radius) - target_z * 16 - 1;
                    let mut max_z = floor_f64(z + radius) - target_z * 16 + 1;
                    min_x = min_x.max(0);
                    max_x = max_x.min(16);
                    min_y = min_y.max(1);
                    max_y = max_y.min(248);
                    min_z = min_z.max(0);
                    max_z = max_z.min(16);
                    let mut water = false;
                    let mut sx = min_x;
                    while !water && sx < max_x {
                        let mut sz = min_z;
                        while !water && sz < max_z {
                            let mut sy = max_y + 1;
                            while !water && sy >= min_y - 1 {
                                if (0..256).contains(&sy) {
                                    let id = primer
                                        .getBlockState(sx as usize, sy as usize, sz as usize)
                                        .getBlockId();
                                    if id == 8 || id == 9 {
                                        water = true;
                                    }
                                    if sy != min_y - 1
                                        && sx != min_x
                                        && sx != max_x - 1
                                        && sz != min_z
                                        && sz != max_z - 1
                                    {
                                        sy = min_y;
                                    }
                                }
                                sy -= 1;
                            }
                            sz += 1;
                        }
                        sx += 1;
                    }
                    if !water {
                        for cx in min_x..max_x {
                            let nx = ((cx + target_x * 16) as f64 + 0.5 - x) / radius;
                            for cz in min_z..max_z {
                                let nz = ((cz + target_z * 16) as f64 + 0.5 - z) / radius;
                                let mut surface = false;
                                if nx * nx + nz * nz < 1.0 {
                                    for cy in ((min_y + 1)..=max_y).rev() {
                                        let ny = ((cy - 1) as f64 + 0.5 - y) / vradius;
                                        if (nx * nx + nz * nz) * self.rs[(cy - 1) as usize] as f64
                                            + ny * ny / 6.0
                                            < 1.0
                                        {
                                            let state = primer.getBlockState(
                                                cx as usize,
                                                cy as usize,
                                                cz as usize,
                                            );
                                            if state.getBlockId() == 2 {
                                                surface = true;
                                            }
                                            if matches!(state.getBlockId(), 1 | 2 | 3) {
                                                if cy - 1 < 10 {
                                                    primer.setBlockState(
                                                        cx as usize,
                                                        cy as usize,
                                                        cz as usize,
                                                        IBlockState::fromGlobalStateId(10 << 4),
                                                    );
                                                } else {
                                                    primer.setBlockState(
                                                        cx as usize,
                                                        cy as usize,
                                                        cz as usize,
                                                        IBlockState::fromGlobalStateId(0),
                                                    );
                                                    if surface
                                                        && primer
                                                            .getBlockState(
                                                                cx as usize,
                                                                (cy - 1) as usize,
                                                                cz as usize,
                                                            )
                                                            .getBlockId()
                                                            == 3
                                                    {
                                                        let biome =
                                                            provider.getBiome(BlockPos::new(
                                                                cx + target_x * 16,
                                                                0,
                                                                cz + target_z * 16,
                                                            ));
                                                        primer.setBlockState(
                                                            cx as usize,
                                                            (cy - 1) as usize,
                                                            cz as usize,
                                                            current_top[biome.getId() as usize],
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if single {
                            break;
                        }
                    }
                }
            }
            step += 1;
        }
    }
    fn recursiveGenerate(
        &mut self,
        provider: &BiomeProviderKind,
        current_top: &[IBlockState; 256],
        source_x: i32,
        source_z: i32,
        target_x: i32,
        target_z: i32,
        primer: &mut ChunkPrimer,
    ) {
        if self.base.rand.next_i32_bound(50) != 0 {
            return;
        }
        let x = (source_x * 16 + self.base.rand.next_i32_bound(16)) as f64;
        let inner = self.base.rand.next_i32_bound(40) + 8;
        let y = (self.base.rand.next_i32_bound(inner) + 20) as f64;
        let z = (source_z * 16 + self.base.rand.next_i32_bound(16)) as f64;
        let yaw = self.base.rand.next_f32() * (PI * 2.0);
        let pitch = (self.base.rand.next_f32() - 0.5) * 2.0 / 8.0;
        let size = (self.base.rand.next_f32() * 2.0 + self.base.rand.next_f32()) * 2.0;
        let seed = self.base.rand.next_i64();
        self.addTunnel(
            provider,
            current_top,
            seed,
            target_x,
            target_z,
            primer,
            x,
            y,
            z,
            size,
            yaw,
            pitch,
            0,
            0,
            3.0,
        );
    }
}
