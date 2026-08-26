use crate::vulkan::NativeImage::{NativeImage, NativeImageError};

/// Pixel-for-pixel port of MCP 1.12.2 `ImageBufferDownload`.
pub struct ImageBufferDownload;

impl ImageBufferDownload {
    pub fn parseUserSkin(image: NativeImage) -> Result<NativeImage, NativeImageError> {
        let source_width = image.width();
        let source_height = image.height();
        let mut scale = 1_u32;
        let mut image_width = 64_u32;
        let mut image_height = 64_u32;
        while image_width < source_width || image_height < source_height {
            image_width *= 2;
            image_height *= 2;
            scale *= 2;
        }

        let mut output = NativeImage::from_rgba(
            image_width,
            image_height,
            vec![0; image_width as usize * image_height as usize * 4],
        )?;
        copy_rect(
            &image,
            &mut output,
            0,
            0,
            source_width,
            source_height,
            0,
            0,
            false,
            false,
        );
        let legacy = source_height == 32 * scale;
        if legacy {
            clear_rect(&mut output, 0, 32 * scale, 64 * scale, 64 * scale);
            // Java Graphics.drawImage calls use reversed destination X bounds
            // to mirror the old right limbs into the 1.8 left-limb regions.
            copy_rect(
                &output.clone(),
                &mut output,
                4 * scale,
                16 * scale,
                8 * scale,
                20 * scale,
                20 * scale,
                48 * scale,
                true,
                false,
            );
            copy_rect(
                &output.clone(),
                &mut output,
                8 * scale,
                16 * scale,
                12 * scale,
                20 * scale,
                24 * scale,
                48 * scale,
                true,
                false,
            );
            copy_rect(
                &output.clone(),
                &mut output,
                8 * scale,
                20 * scale,
                12 * scale,
                32 * scale,
                16 * scale,
                52 * scale,
                true,
                false,
            );
            copy_rect(
                &output.clone(),
                &mut output,
                4 * scale,
                20 * scale,
                8 * scale,
                32 * scale,
                20 * scale,
                52 * scale,
                true,
                false,
            );
            copy_rect(
                &output.clone(),
                &mut output,
                0 * scale,
                20 * scale,
                4 * scale,
                32 * scale,
                24 * scale,
                52 * scale,
                true,
                false,
            );
            copy_rect(
                &output.clone(),
                &mut output,
                12 * scale,
                20 * scale,
                16 * scale,
                32 * scale,
                28 * scale,
                52 * scale,
                true,
                false,
            );
            copy_rect(
                &output.clone(),
                &mut output,
                44 * scale,
                16 * scale,
                48 * scale,
                20 * scale,
                36 * scale,
                48 * scale,
                true,
                false,
            );
            copy_rect(
                &output.clone(),
                &mut output,
                48 * scale,
                16 * scale,
                52 * scale,
                20 * scale,
                40 * scale,
                48 * scale,
                true,
                false,
            );
            copy_rect(
                &output.clone(),
                &mut output,
                48 * scale,
                20 * scale,
                52 * scale,
                32 * scale,
                32 * scale,
                52 * scale,
                true,
                false,
            );
            copy_rect(
                &output.clone(),
                &mut output,
                44 * scale,
                20 * scale,
                48 * scale,
                32 * scale,
                36 * scale,
                52 * scale,
                true,
                false,
            );
            copy_rect(
                &output.clone(),
                &mut output,
                40 * scale,
                20 * scale,
                44 * scale,
                32 * scale,
                40 * scale,
                52 * scale,
                true,
                false,
            );
            copy_rect(
                &output.clone(),
                &mut output,
                52 * scale,
                20 * scale,
                56 * scale,
                32 * scale,
                44 * scale,
                52 * scale,
                true,
                false,
            );
        }

        set_area_opaque(&mut output, 0, 0, 32 * scale, 16 * scale);
        if legacy {
            do_transparency_hack(&mut output, 32 * scale, 0, 64 * scale, 32 * scale);
        }
        set_area_opaque(&mut output, 0, 16 * scale, 64 * scale, 32 * scale);
        set_area_opaque(&mut output, 16 * scale, 48 * scale, 48 * scale, 64 * scale);
        Ok(output)
    }
}

fn clear_rect(image: &mut NativeImage, x0: u32, y0: u32, x1: u32, y1: u32) {
    let width = image.width();
    for y in y0.min(image.height())..y1.min(image.height()) {
        for x in x0.min(width)..x1.min(width) {
            let offset = ((y * width + x) * 4) as usize;
            image.rgba_mut()[offset..offset + 4].fill(0);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn copy_rect(
    source: &NativeImage,
    target: &mut NativeImage,
    sx0: u32,
    sy0: u32,
    sx1: u32,
    sy1: u32,
    dx0: u32,
    dy0: u32,
    mirror_x: bool,
    mirror_y: bool,
) {
    let width = sx1.saturating_sub(sx0);
    let height = sy1.saturating_sub(sy0);
    for y in 0..height {
        for x in 0..width {
            let source_x = if mirror_x { sx1 - 1 - x } else { sx0 + x };
            let source_y = if mirror_y { sy1 - 1 - y } else { sy0 + y };
            let target_x = dx0 + x;
            let target_y = dy0 + y;
            if source_x >= source.width()
                || source_y >= source.height()
                || target_x >= target.width()
                || target_y >= target.height()
            {
                continue;
            }
            let pixel = source.pixel_rgba(source_x, source_y);
            let offset = ((target_y * target.width() + target_x) * 4) as usize;
            target.rgba_mut()[offset..offset + 4].copy_from_slice(&pixel);
        }
    }
}

fn set_area_opaque(image: &mut NativeImage, x0: u32, y0: u32, x1: u32, y1: u32) {
    let width = image.width();
    for y in y0.min(image.height())..y1.min(image.height()) {
        for x in x0.min(width)..x1.min(width) {
            image.rgba_mut()[((y * width + x) * 4 + 3) as usize] = 255;
        }
    }
}

fn do_transparency_hack(image: &mut NativeImage, x0: u32, y0: u32, x1: u32, y1: u32) {
    let width = image.width();
    for y in y0.min(image.height())..y1.min(image.height()) {
        for x in x0.min(width)..x1.min(width) {
            if image.rgba()[((y * width + x) * 4 + 3) as usize] < 128 {
                return;
            }
        }
    }
    for y in y0.min(image.height())..y1.min(image.height()) {
        for x in x0.min(width)..x1.min(width) {
            image.rgba_mut()[((y * width + x) * 4 + 3) as usize] = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_skin_is_promoted_to_64_by_64_and_base_regions_are_opaque() {
        let input = NativeImage::from_rgba(64, 32, vec![255; 64 * 32 * 4]).unwrap();
        let output = ImageBufferDownload::parseUserSkin(input).unwrap();
        assert_eq!((output.width(), output.height()), (64, 64));
        assert_eq!(output.alpha(0, 0), 255);
        assert_eq!(output.alpha(20, 48), 255);
    }

    #[test]
    fn modern_skin_preserves_native_resolution() {
        let input = NativeImage::from_rgba(64, 64, vec![127; 64 * 64 * 4]).unwrap();
        let output = ImageBufferDownload::parseUserSkin(input).unwrap();
        assert_eq!((output.width(), output.height()), (64, 64));
    }
}
