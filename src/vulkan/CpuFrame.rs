use rayon::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuFrame {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl CpuFrame {
    pub fn new(width: u32, height: u32) -> Self {
        let length = width as usize * height as usize * 4;
        Self {
            width,
            height,
            rgba: vec![0; length],
        }
    }

    pub fn from_rgba(width: u32, height: u32, rgba: Vec<u8>) -> anyhow::Result<Self> {
        anyhow::ensure!(
            rgba.len() == width as usize * height as usize * 4,
            "CPU frame buffer length does not match {width}x{height} RGBA8"
        );
        Ok(Self {
            width,
            height,
            rgba,
        })
    }

    pub const fn width(&self) -> u32 {
        self.width
    }
    pub const fn height(&self) -> u32 {
        self.height
    }
    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }
    pub fn rgba_mut(&mut self) -> &mut [u8] {
        &mut self.rgba
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.rgba.resize(width as usize * height as usize * 4, 0);
    }

    pub fn clear(&mut self, color: [u8; 4]) {
        for pixel in self.rgba.chunks_exact_mut(4) {
            pixel.copy_from_slice(&color);
        }
    }

    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        debug_assert!(x < self.width && y < self.height);
        let offset = ((y * self.width + x) * 4) as usize;
        [
            self.rgba[offset],
            self.rgba[offset + 1],
            self.rgba[offset + 2],
            self.rgba[offset + 3],
        ]
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
        if x >= self.width || y >= self.height {
            return;
        }
        let offset = ((y * self.width + x) * 4) as usize;
        self.rgba[offset..offset + 4].copy_from_slice(&color);
    }

    pub fn blend_pixel(&mut self, x: i32, y: i32, source: [f32; 4]) {
        if x < 0 || y < 0 || x as u32 >= self.width || y as u32 >= self.height {
            return;
        }
        let offset = (((y as u32 * self.width) + x as u32) * 4) as usize;
        let source_alpha = source[3].clamp(0.0, 1.0);
        if source_alpha <= 0.0 {
            return;
        }
        let destination_alpha = self.rgba[offset + 3] as f32 / 255.0;
        let inverse = 1.0 - source_alpha;
        for channel in 0..3 {
            let destination = self.rgba[offset + channel] as f32 / 255.0;
            let value = source[channel].clamp(0.0, 1.0) * source_alpha + destination * inverse;
            self.rgba[offset + channel] = (value * 255.0 + 0.5).floor() as u8;
        }
        let alpha = source_alpha + destination_alpha * inverse;
        self.rgba[offset + 3] = (alpha * 255.0 + 0.5).floor() as u8;
    }

    pub fn write_for_vulkan_format(
        &self,
        format: ash::vk::Format,
        destination: &mut [u8],
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            destination.len() >= self.rgba.len(),
            "mapped Vulkan upload buffer is too small"
        );
        match format {
            ash::vk::Format::R8G8B8A8_UNORM | ash::vk::Format::R8G8B8A8_SRGB => {
                destination[..self.rgba.len()].copy_from_slice(&self.rgba);
            }
            ash::vk::Format::B8G8R8A8_UNORM | ash::vk::Format::B8G8R8A8_SRGB => {
                let pixelCount = self.rgba.len() / 4;
                let workerCount = rayon::current_num_threads()
                    .clamp(1, 8)
                    .min(pixelCount.max(1));
                let pixelsPerWorker = pixelCount.div_ceil(workerCount);
                let bytesPerWorker = pixelsPerWorker * 4;
                destination[..self.rgba.len()]
                    .par_chunks_mut(bytesPerWorker)
                    .enumerate()
                    .for_each(|(chunkIndex, targetChunk)| {
                        let start = chunkIndex * bytesPerWorker;
                        let sourceChunk = &self.rgba[start..start + targetChunk.len()];
                        for (source, target) in sourceChunk
                            .chunks_exact(4)
                            .zip(targetChunk.chunks_exact_mut(4))
                        {
                            target[0] = source[2];
                            target[1] = source[1];
                            target[2] = source[0];
                            target[3] = source[3];
                        }
                    });
            }
            unsupported => anyhow::bail!(
                "software GUI upload does not support swapchain format {unsupported:?}"
            ),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn straight_alpha_blend_matches_minecraft_gui_blending() {
        let mut frame = CpuFrame::new(1, 1);
        frame.set_pixel(0, 0, [0, 0, 255, 255]);
        frame.blend_pixel(0, 0, [1.0, 0.0, 0.0, 0.5]);
        assert_eq!(frame.pixel(0, 0), [128, 0, 128, 255]);
    }

    #[test]
    fn bgra_upload_swaps_red_and_blue_only() {
        let frame = CpuFrame::from_rgba(1, 1, vec![1, 2, 3, 4]).unwrap();
        let mut destination = vec![0; 4];
        frame
            .write_for_vulkan_format(ash::vk::Format::B8G8R8A8_UNORM, &mut destination)
            .unwrap();
        assert_eq!(destination, [3, 2, 1, 4]);
    }
}
