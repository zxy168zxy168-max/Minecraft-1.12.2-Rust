use std::io::Cursor;

use png::{BitDepth, ColorType, Decoder, Transformations};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeImage {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum NativeImageError {
    #[error("failed to decode PNG: {0}")]
    Png(#[from] png::DecodingError),
    #[error("unsupported PNG output format {color_type:?}/{bit_depth:?}")]
    Unsupported {
        color_type: ColorType,
        bit_depth: BitDepth,
    },
    #[error("decoded PNG buffer has an invalid length")]
    InvalidLength,
}

impl NativeImage {
    pub fn from_rgba(width: u32, height: u32, rgba: Vec<u8>) -> Result<Self, NativeImageError> {
        let expected = width as usize * height as usize * 4;
        if rgba.len() != expected {
            return Err(NativeImageError::InvalidLength);
        }
        Ok(Self {
            width,
            height,
            rgba,
        })
    }

    pub fn decode_png(bytes: &[u8]) -> Result<Self, NativeImageError> {
        let mut decoder = Decoder::new(Cursor::new(bytes));
        decoder.set_transformations(Transformations::EXPAND | Transformations::STRIP_16);
        let mut reader = decoder.read_info()?;
        let mut buffer = vec![0_u8; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buffer)?;
        let input = &buffer[..info.buffer_size()];
        let pixel_count = info.width as usize * info.height as usize;
        let mut rgba = Vec::with_capacity(pixel_count * 4);

        match (info.color_type, info.bit_depth) {
            (ColorType::Rgba, BitDepth::Eight) => rgba.extend_from_slice(input),
            (ColorType::Rgb, BitDepth::Eight) => {
                for pixel in input.chunks_exact(3) {
                    rgba.extend_from_slice(&[pixel[0], pixel[1], pixel[2], 255]);
                }
            }
            (ColorType::Grayscale, BitDepth::Eight) => {
                for &value in input {
                    rgba.extend_from_slice(&[value, value, value, 255]);
                }
            }
            (ColorType::GrayscaleAlpha, BitDepth::Eight) => {
                for pixel in input.chunks_exact(2) {
                    rgba.extend_from_slice(&[pixel[0], pixel[0], pixel[0], pixel[1]]);
                }
            }
            (color_type, bit_depth) => {
                return Err(NativeImageError::Unsupported {
                    color_type,
                    bit_depth,
                })
            }
        }

        if rgba.len() != pixel_count * 4 {
            return Err(NativeImageError::InvalidLength);
        }

        Ok(Self {
            width: info.width,
            height: info.height,
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
    pub fn into_rgba(self) -> Vec<u8> {
        self.rgba
    }

    pub fn pixel_rgba(&self, x: u32, y: u32) -> [u8; 4] {
        let x = x.min(self.width.saturating_sub(1));
        let y = y.min(self.height.saturating_sub(1));
        let offset = ((y * self.width + x) * 4) as usize;
        [
            self.rgba[offset],
            self.rgba[offset + 1],
            self.rgba[offset + 2],
            self.rgba[offset + 3],
        ]
    }

    pub fn alpha(&self, x: u32, y: u32) -> u8 {
        assert!(x < self.width && y < self.height, "pixel out of range");
        self.rgba[((y * self.width + x) * 4 + 3) as usize]
    }
}
