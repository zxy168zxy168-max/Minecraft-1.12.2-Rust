use std::collections::HashMap;

use crate::compat::Java::JavaRandom;
use crate::compat::JavaProperties::parse_java_properties;
use crate::net::minecraft::client::resources::SimpleReloadableResourceManager::{
    ResourceManager, ResourceManagerError,
};
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::vulkan::NativeImage::{NativeImage, NativeImageError};
use thiserror::Error;

use crate::vulkan::GuiDrawList::GuiDrawList;

/// Minecraft 1.12.2's 256-character default-font lookup table.
const DEFAULT_FONT_CHARS: &str = "ÀÁÂÈÊËÍÓÔÕÚßãõğİıŒœŞşŴŵžȇ\0\0\0\0\0\0\0 !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~\0ÇüéâäàåçêëèïîìÄÅÉæÆôöòûùÿÖÜø£Ø×ƒáíóúñÑªº¿®¬½¼¡«»░▒▓│┤╡╢╖╕╣║╗╝╜╛┐└┴┬├─┼╞╟╚╔╩╦╠═╬╧╨╤╥╙╘╒╓╫╪┘┌█▄▌▐▀αβΓπΣσμτΦΘΩδ∞∅∈∩≡±≥≤⌠⌡÷≈°∙·√ⁿ²■\0";
const FORMAT_CODES: &str = "0123456789abcdefklmnor";

#[derive(Debug, Error)]
pub enum FontRendererError {
    #[error(transparent)]
    Resource(#[from] ResourceManagerError),
    #[error(transparent)]
    Image(#[from] NativeImageError),
    #[error("font/glyph_sizes.bin must contain at least 65536 bytes, found {0}")]
    InvalidGlyphSizes(usize),
    #[error("font texture dimensions must be divisible by 16, found {0}x{1}")]
    InvalidFontDimensions(u32, u32),
}

#[derive(Debug, Clone)]
pub struct FontRenderer {
    char_width: [i32; 256],
    char_width_float: [f32; 256],
    glyph_width: Vec<u8>,
    color_code: [u32; 32],
    location_font_texture_base: ResourceLocation,
    location_font_texture: ResourceLocation,
    unicode_flag: bool,
    bidi_flag: bool,
    pub font_height: i32,
    pub offset_bold: f32,
    pub blend: bool,
    font_random: JavaRandom,
    custom_fonts: bool,
    unicode_page_locations: Vec<ResourceLocation>,
}

#[derive(Debug, Clone, Copy, Default)]
struct Styles {
    random: bool,
    bold: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
}

impl FontRenderer {
    pub fn load(
        resource_manager: &ResourceManager,
        location: ResourceLocation,
        unicode: bool,
        anaglyph: bool,
        custom_fonts: bool,
    ) -> Result<Self, FontRendererError> {
        let location_font_texture = hd_font_location(resource_manager, &location, custom_fonts);
        let mut renderer = Self {
            char_width: [0; 256],
            char_width_float: [0.0; 256],
            glyph_width: vec![0; 65_536],
            color_code: build_color_codes(anaglyph),
            location_font_texture_base: location,
            location_font_texture,
            unicode_flag: unicode,
            bidi_flag: false,
            font_height: 9,
            offset_bold: 1.0,
            blend: false,
            font_random: JavaRandom::new(0),
            custom_fonts,
            unicode_page_locations: build_unicode_page_locations(resource_manager, custom_fonts),
        };
        renderer.read_font_texture(resource_manager)?;
        renderer.read_glyph_sizes(resource_manager)?;
        Ok(renderer)
    }

    pub fn reload(&mut self, resource_manager: &ResourceManager) -> Result<(), FontRendererError> {
        self.location_font_texture = hd_font_location(
            resource_manager,
            &self.location_font_texture_base,
            self.custom_fonts,
        );
        self.unicode_page_locations =
            build_unicode_page_locations(resource_manager, self.custom_fonts);
        self.read_font_texture(resource_manager)?;
        self.read_glyph_sizes(resource_manager)
    }

    pub fn test_metric_renderer() -> Self {
        let mut renderer = Self {
            char_width: [6; 256],
            char_width_float: [6.0; 256],
            glyph_width: vec![0; 65_536],
            color_code: build_color_codes(false),
            location_font_texture_base: ResourceLocation::parse("textures/font/ascii.png"),
            location_font_texture: ResourceLocation::parse("textures/font/ascii.png"),
            unicode_flag: false,
            bidi_flag: false,
            font_height: 9,
            offset_bold: 1.0,
            blend: false,
            font_random: JavaRandom::new(0),
            custom_fonts: false,
            unicode_page_locations: (0_u16..=255)
                .map(|page| {
                    ResourceLocation::new(
                        "minecraft",
                        format!("textures/font/unicode_page_{page:02x}.png"),
                    )
                })
                .collect(),
        };
        renderer.char_width[32] = 4;
        renderer.char_width_float[32] = 4.0;
        renderer
    }

    pub const fn unicode_flag(&self) -> bool {
        self.unicode_flag
    }
    pub fn set_unicode_flag(&mut self, unicode: bool) {
        self.unicode_flag = unicode;
    }
    pub const fn bidi_flag(&self) -> bool {
        self.bidi_flag
    }
    pub fn set_bidi_flag(&mut self, bidi: bool) {
        self.bidi_flag = bidi;
    }
    pub fn font_texture(&self) -> &ResourceLocation {
        &self.location_font_texture
    }

    pub fn get_string_width(&self, text: &str) -> i32 {
        let units: Vec<u16> = text.encode_utf16().collect();
        let mut width = 0.0_f32;
        let mut bold = false;
        let mut index = 0_usize;
        while index < units.len() {
            let mut unit = units[index];
            let mut char_width = self.get_char_width_float(unit);
            if char_width < 0.0 && index + 1 < units.len() {
                index += 1;
                unit = units[index];
                match ascii_lower(unit) {
                    b'l' => bold = true,
                    b'r' => bold = false,
                    _ => {}
                }
                char_width = 0.0;
            }
            width += char_width;
            if bold && char_width > 0.0 {
                width += if self.unicode_flag {
                    1.0
                } else {
                    self.offset_bold
                };
            }
            index += 1;
        }
        java_round_f32(width)
    }

    pub fn get_char_width(&self, unit: u16) -> i32 {
        java_round_f32(self.get_char_width_float(unit))
    }

    pub fn trim_string_to_width(&self, text: &str, max_width: i32, reverse: bool) -> String {
        let units: Vec<u16> = text.encode_utf16().collect();
        let mut output = Vec::new();
        let mut width = 0.0_f32;
        let mut formatting = false;
        let mut bold = false;
        let mut index = if reverse { units.len() as isize - 1 } else { 0 };
        let step = if reverse { -1 } else { 1 };
        while index >= 0 && (index as usize) < units.len() && width < max_width as f32 {
            let unit = units[index as usize];
            let char_width = self.get_char_width_float(unit);
            if formatting {
                formatting = false;
                match ascii_lower(unit) {
                    b'l' => bold = true,
                    b'r' => bold = false,
                    _ => {}
                }
            } else if char_width < 0.0 {
                formatting = true;
            } else {
                width += char_width;
                if bold {
                    width += 1.0;
                }
            }
            if width > max_width as f32 {
                break;
            }
            if reverse {
                output.insert(0, unit);
            } else {
                output.push(unit);
            }
            index += step;
        }
        String::from_utf16_lossy(&output)
    }

    pub fn list_formatted_string_to_width(&self, text: &str, wrap_width: i32) -> Vec<String> {
        self.wrap_formatted_string_to_width(text, wrap_width)
            .split('\n')
            .map(str::to_owned)
            .collect()
    }

    fn wrap_formatted_string_to_width(&self, text: &str, wrap_width: i32) -> String {
        let units: Vec<u16> = text.encode_utf16().collect();
        if units.len() <= 1 {
            return text.to_owned();
        }
        let split = self.size_string_to_width(&units, wrap_width);
        if units.len() <= split {
            return text.to_owned();
        }
        let head = String::from_utf16_lossy(&units[..split]);
        let split_unit = units[split];
        let skip = usize::from(split_unit == b' ' as u16 || split_unit == b'\n' as u16);
        let tail = String::from_utf16_lossy(&units[(split + skip)..]);
        let continuation = format!("{}{}", get_format_from_string(&head), tail);
        format!(
            "{head}\n{}",
            self.wrap_formatted_string_to_width(&continuation, wrap_width)
        )
    }

    fn size_string_to_width(&self, units: &[u16], wrap_width: i32) -> usize {
        let mut width = 0.0_f32;
        let mut index = 0_usize;
        let mut last_space = None;
        let mut bold = false;
        while index < units.len() {
            let unit = units[index];
            match unit {
                10 => {
                    last_space = Some(index + 1);
                    break;
                }
                32 => {
                    last_space = Some(index);
                    width += self.get_char_width_float(unit);
                    if bold {
                        width += 1.0;
                    }
                }
                167 if index + 1 < units.len() => {
                    index += 1;
                    let code = ascii_lower(units[index]);
                    if code == b'l' {
                        bold = true;
                    } else if code == b'r'
                        || (b'0'..=b'9').contains(&code)
                        || (b'a'..=b'f').contains(&code)
                    {
                        bold = false;
                    }
                }
                _ => {
                    width += self.get_char_width_float(unit);
                    if bold {
                        width += 1.0;
                    }
                }
            }
            if unit == 10 {
                break;
            }
            index += 1;
            if java_round_f32(width) > wrap_width {
                break;
            }
        }
        if index != units.len() {
            if let Some(space) = last_space {
                if space < index {
                    return space;
                }
            }
        }
        index
    }

    pub fn draw_string_with_shadow(
        &mut self,
        draw_list: &mut GuiDrawList,
        text: &str,
        x: f32,
        y: f32,
        color: i32,
    ) -> i32 {
        self.draw_string(draw_list, text, x, y, color, true)
    }

    pub fn draw_string(
        &mut self,
        draw_list: &mut GuiDrawList,
        text: &str,
        x: f32,
        y: f32,
        color: i32,
        drop_shadow: bool,
    ) -> i32 {
        if drop_shadow {
            let shadow_end = self.render_string(draw_list, text, x + 1.0, y + 1.0, color, true);
            shadow_end.max(self.render_string(draw_list, text, x, y, color, false))
        } else {
            self.render_string(draw_list, text, x, y, color, false)
        }
    }

    pub fn draw_centered_string_with_shadow(
        &mut self,
        draw_list: &mut GuiDrawList,
        text: &str,
        center_x: i32,
        y: i32,
        color: i32,
    ) -> i32 {
        let x = center_x - self.get_string_width(text) / 2;
        self.draw_string_with_shadow(draw_list, text, x as f32, y as f32, color)
    }

    fn render_string(
        &mut self,
        draw_list: &mut GuiDrawList,
        text: &str,
        mut x: f32,
        mut y: f32,
        mut color: i32,
        shadow: bool,
    ) -> i32 {
        if (color & -67_108_864) == 0 {
            color |= -16_777_216;
        }
        if shadow {
            color = (color & 16_579_836) >> 2 | color & -16_777_216;
        }
        let base_color = color as u32;
        let alpha = (base_color >> 24) & 255;
        let mut current_color = base_color;
        let mut styles = Styles::default();
        let units: Vec<u16> = text.encode_utf16().collect();
        let default_chars: Vec<u16> = DEFAULT_FONT_CHARS.encode_utf16().collect();
        let mut index = 0_usize;

        while index < units.len() {
            let mut unit = units[index];
            if unit == 167 && index + 1 < units.len() {
                index += 1;
                let code = ascii_lower(units[index]);
                let format_index = FORMAT_CODES
                    .as_bytes()
                    .iter()
                    .position(|&candidate| candidate == code)
                    .map(|v| v as i32)
                    .unwrap_or(-1);
                if format_index < 16 {
                    styles = Styles::default();
                    let mut color_index = if !(0..=15).contains(&format_index) {
                        15
                    } else {
                        format_index as usize
                    };
                    if shadow {
                        color_index += 16;
                    }
                    current_color = (alpha << 24) | self.color_code[color_index];
                } else {
                    match format_index {
                        16 => styles.random = true,
                        17 => styles.bold = true,
                        18 => styles.strikethrough = true,
                        19 => styles.underline = true,
                        20 => styles.italic = true,
                        21 => {
                            styles = Styles::default();
                            current_color = base_color;
                        }
                        _ => {}
                    }
                }
                index += 1;
                continue;
            }

            let mut default_index = default_chars
                .iter()
                .position(|&candidate| candidate == unit)
                .map(|v| v as i32)
                .unwrap_or(-1);
            if styles.random && default_index != -1 {
                let target_width = self.get_char_width(unit);
                loop {
                    default_index = self.font_random.next_i32_bound(default_chars.len() as i32);
                    let candidate = default_chars[default_index as usize];
                    if target_width == self.get_char_width(candidate) {
                        unit = candidate;
                        break;
                    }
                }
            }

            let bold_offset = if default_index != -1 && !self.unicode_flag {
                self.offset_bold
            } else {
                0.5
            };
            let shadow_unicode_offset =
                (unit == 0 || default_index == -1 || self.unicode_flag) && shadow;
            if shadow_unicode_offset {
                x -= bold_offset;
                y -= bold_offset;
            }
            let mut advance = self.render_char(
                draw_list,
                unit,
                default_index,
                x,
                y,
                current_color,
                styles.italic,
            );
            if shadow_unicode_offset {
                x += bold_offset;
                y += bold_offset;
            }

            if styles.bold {
                x += bold_offset;
                if shadow_unicode_offset {
                    x -= bold_offset;
                    y -= bold_offset;
                }
                self.render_char(
                    draw_list,
                    unit,
                    default_index,
                    x,
                    y,
                    current_color,
                    styles.italic,
                );
                x -= bold_offset;
                if shadow_unicode_offset {
                    x += bold_offset;
                    y += bold_offset;
                }
                advance += bold_offset;
            }

            if styles.strikethrough {
                let middle = y + (self.font_height / 2) as f32;
                draw_list.push_solid_quad([
                    (x, middle, current_color),
                    (x + advance, middle, current_color),
                    (x + advance, middle - 1.0, current_color),
                    (x, middle - 1.0, current_color),
                ]);
            }
            if styles.underline {
                let underline = y + self.font_height as f32;
                draw_list.push_solid_quad([
                    (x - 1.0, underline, current_color),
                    (x + advance, underline, current_color),
                    (x + advance, underline - 1.0, current_color),
                    (x - 1.0, underline - 1.0, current_color),
                ]);
            }
            x += advance;
            index += 1;
        }
        x as i32
    }

    fn render_char(
        &self,
        draw_list: &mut GuiDrawList,
        unit: u16,
        default_index: i32,
        x: f32,
        y: f32,
        color: u32,
        italic: bool,
    ) -> f32 {
        if unit == b' ' as u16 || unit == 160 {
            return if self.unicode_flag {
                4.0
            } else {
                self.char_width_float[unit as usize]
            };
        }
        if default_index != -1 && !self.unicode_flag {
            self.render_default_char(draw_list, default_index as usize, x, y, color, italic)
        } else {
            self.render_unicode_char(draw_list, unit, x, y, color, italic)
        }
    }

    fn render_default_char(
        &self,
        draw_list: &mut GuiDrawList,
        index: usize,
        x: f32,
        y: f32,
        color: u32,
        italic: bool,
    ) -> f32 {
        let texture_x = (index % 16 * 8) as f32;
        let texture_y = (index / 16 * 8) as f32;
        let italic_offset = if italic { 1.0 } else { 0.0 };
        let width = self.char_width_float[index];
        let render_size = 7.99_f32;
        draw_list.push_triangle_strip(
            self.location_font_texture.clone(),
            [
                (
                    x + italic_offset,
                    y,
                    texture_x / 128.0,
                    texture_y / 128.0,
                    color,
                ),
                (
                    x - italic_offset,
                    y + 7.99,
                    texture_x / 128.0,
                    (texture_y + 7.99) / 128.0,
                    color,
                ),
                (
                    x + render_size - 1.0 + italic_offset,
                    y,
                    (texture_x + render_size - 1.0) / 128.0,
                    texture_y / 128.0,
                    color,
                ),
                (
                    x + render_size - 1.0 - italic_offset,
                    y + 7.99,
                    (texture_x + render_size - 1.0) / 128.0,
                    (texture_y + 7.99) / 128.0,
                    color,
                ),
            ],
        );
        width
    }

    fn render_unicode_char(
        &self,
        draw_list: &mut GuiDrawList,
        unit: u16,
        x: f32,
        y: f32,
        color: u32,
        italic: bool,
    ) -> f32 {
        let packed = self.glyph_width[unit as usize];
        if packed == 0 {
            return 0.0;
        }
        let page = (unit / 256) as u8;
        let start = (packed >> 4) as f32;
        let end = ((packed & 15) + 1) as f32;
        let texture_x = (unit as usize % 16 * 16) as f32 + start;
        let texture_y = (((unit & 255) as usize / 16) * 16) as f32;
        let render_width = end - start - 0.02;
        let italic_offset = if italic { 1.0 } else { 0.0 };
        let texture = self.unicode_page_location(page).clone();
        draw_list.push_triangle_strip(
            texture,
            [
                (
                    x + italic_offset,
                    y,
                    texture_x / 256.0,
                    texture_y / 256.0,
                    color,
                ),
                (
                    x - italic_offset,
                    y + 7.99,
                    texture_x / 256.0,
                    (texture_y + 15.98) / 256.0,
                    color,
                ),
                (
                    x + render_width / 2.0 + italic_offset,
                    y,
                    (texture_x + render_width) / 256.0,
                    texture_y / 256.0,
                    color,
                ),
                (
                    x + render_width / 2.0 - italic_offset,
                    y + 7.99,
                    (texture_x + render_width) / 256.0,
                    (texture_y + 15.98) / 256.0,
                    color,
                ),
            ],
        );
        (end - start) / 2.0 + 1.0
    }

    fn get_char_width_float(&self, unit: u16) -> f32 {
        if unit == 167 {
            return -1.0;
        }
        if unit == b' ' as u16 || unit == 160 {
            return self.char_width_float[32];
        }
        let default_index = DEFAULT_FONT_CHARS
            .encode_utf16()
            .position(|candidate| candidate == unit);
        if unit > 0 && default_index.is_some() && !self.unicode_flag {
            return self.char_width_float[default_index.unwrap()];
        }
        let packed = self.glyph_width[unit as usize];
        if packed == 0 {
            return 0.0;
        }
        let start = packed >> 4;
        let end = (packed & 15) + 1;
        (((end as i32 - start as i32) / 2) + 1) as f32
    }

    pub fn unicode_page_location(&self, page: u8) -> &ResourceLocation {
        &self.unicode_page_locations[page as usize]
    }

    pub fn unicode_pages_with_glyphs(&self) -> Vec<u8> {
        (0_u16..=255)
            .filter(|page| {
                let start = *page as usize * 256;
                self.glyph_width[start..start + 256]
                    .iter()
                    .any(|&width| width != 0)
            })
            .map(|page| page as u8)
            .collect()
    }

    fn read_font_texture(
        &mut self,
        resource_manager: &ResourceManager,
    ) -> Result<(), FontRendererError> {
        let resource = resource_manager.get_resource(&self.location_font_texture)?;
        let image = NativeImage::decode_png(&resource.bytes)?;
        if image.width() % 16 != 0 || image.height() % 16 != 0 {
            return Err(FontRendererError::InvalidFontDimensions(
                image.width(),
                image.height(),
            ));
        }
        let properties_location = ResourceLocation::new(
            self.location_font_texture.getNamespace(),
            self.location_font_texture
                .getPath()
                .strip_suffix(".png")
                .map(|path| format!("{path}.properties"))
                .unwrap_or_default(),
        );
        let properties = resource_manager
            .get_resource(&properties_location)
            .ok()
            .map(|resource| parse_properties(&resource.bytes))
            .unwrap_or_default();
        self.blend = read_bool(&properties, "blend", false);
        let image_width = image.width() as i32;
        let image_height = image.height() as i32;
        let char_width = image_width / 16;
        let char_height = image_height / 16;
        let scale = image_width as f32 / 128.0;
        let bold_scale_factor = scale.clamp(1.0, 2.0);
        self.offset_bold = 1.0 / bold_scale_factor;
        if let Some(value) = read_float(&properties, "offsetBold") {
            if value >= 0.0 {
                self.offset_bold = value;
            }
        }

        for glyph in 0..256_usize {
            let column = glyph as i32 % 16;
            let row = glyph as i32 / 16;
            let mut right = char_width - 1;
            while right >= 0 {
                let pixel_x = column * char_width + right;
                let mut empty = true;
                for pixel_y in 0..char_height {
                    if image.alpha(pixel_x as u32, (row * char_height + pixel_y) as u32) > 16 {
                        empty = false;
                        break;
                    }
                }
                if !empty {
                    break;
                }
                right -= 1;
            }
            if glyph == 32 {
                right = if char_width <= 8 {
                    (2.0 * scale) as i32
                } else {
                    (1.5 * scale) as i32
                };
            }
            self.char_width_float[glyph] = (right + 1) as f32 / scale + 1.0;
        }

        for (key, value) in &properties {
            let Some(index_text) = key.strip_prefix("width.") else {
                continue;
            };
            let Ok(index) = index_text.parse::<usize>() else {
                continue;
            };
            let Ok(width) = value.parse::<f32>() else {
                continue;
            };
            if index < 256 && width >= 0.0 {
                self.char_width_float[index] = width;
            }
        }
        for index in 0..256 {
            self.char_width[index] = java_round_f32(self.char_width_float[index]);
        }
        Ok(())
    }

    fn read_glyph_sizes(
        &mut self,
        resource_manager: &ResourceManager,
    ) -> Result<(), FontRendererError> {
        let resource = resource_manager
            .get_resource(&ResourceLocation::new("minecraft", "font/glyph_sizes.bin"))?;
        if resource.bytes.len() < 65_536 {
            return Err(FontRendererError::InvalidGlyphSizes(resource.bytes.len()));
        }
        self.glyph_width.copy_from_slice(&resource.bytes[..65_536]);
        Ok(())
    }
}

fn build_unicode_page_locations(
    resource_manager: &ResourceManager,
    custom_fonts: bool,
) -> Vec<ResourceLocation> {
    (0_u16..=255)
        .map(|page| {
            let base = ResourceLocation::new(
                "minecraft",
                format!("textures/font/unicode_page_{page:02x}.png"),
            );
            if !custom_fonts {
                return base;
            }
            let candidate = ResourceLocation::new(
                base.getNamespace(),
                format!("mcpatcher/font/unicode_page_{page:02x}.png"),
            );
            if resource_manager.resource_exists(&candidate) {
                candidate
            } else {
                base
            }
        })
        .collect()
}

fn hd_font_location(
    resource_manager: &ResourceManager,
    base: &ResourceLocation,
    custom_fonts: bool,
) -> ResourceLocation {
    if !custom_fonts {
        return base.clone();
    }
    let Some(path) = base.getPath().strip_prefix("textures/") else {
        return base.clone();
    };
    let candidate = ResourceLocation::new(base.getNamespace(), format!("mcpatcher/{path}"));
    if resource_manager.resource_exists(&candidate) {
        candidate
    } else {
        base.clone()
    }
}

fn get_format_from_string(text: &str) -> String {
    let units: Vec<u16> = text.encode_utf16().collect();
    let mut result = String::new();
    let mut index = 0_usize;
    while index + 1 < units.len() {
        if units[index] == 167 {
            let code = ascii_lower(units[index + 1]);
            if (b'0'..=b'9').contains(&code) || (b'a'..=b'f').contains(&code) {
                result.clear();
                result.push('§');
                result.push(code as char);
            } else if (b'k'..=b'o').contains(&code) || code == b'r' {
                result.push('§');
                result.push(code as char);
            }
            index += 1;
        }
        index += 1;
    }
    result
}

fn build_color_codes(anaglyph: bool) -> [u32; 32] {
    let mut colors = [0_u32; 32];
    for index in 0..32_i32 {
        let modifier = (index >> 3 & 1) * 85;
        let mut red = (index >> 2 & 1) * 170 + modifier;
        let mut green = (index >> 1 & 1) * 170 + modifier;
        let mut blue = (index & 1) * 170 + modifier;
        if index == 6 {
            red += 85;
        }
        if anaglyph {
            let gray_red = (red * 30 + green * 59 + blue * 11) / 100;
            let gray_green = (red * 30 + green * 70) / 100;
            let gray_blue = (red * 30 + blue * 70) / 100;
            red = gray_red;
            green = gray_green;
            blue = gray_blue;
        }
        if index >= 16 {
            red /= 4;
            green /= 4;
            blue /= 4;
        }
        colors[index as usize] = ((red & 255) << 16 | (green & 255) << 8 | blue & 255) as u32;
    }
    colors
}

fn parse_properties(bytes: &[u8]) -> HashMap<String, String> {
    parse_java_properties(bytes)
}

fn read_bool(values: &HashMap<String, String>, key: &str, default: bool) -> bool {
    match values
        .get(key)
        .map(|value| value.trim().to_ascii_lowercase())
    {
        Some(value) if value == "true" || value == "on" => true,
        Some(value) if value == "false" || value == "off" => false,
        _ => default,
    }
}

fn read_float(values: &HashMap<String, String>, key: &str) -> Option<f32> {
    values.get(key)?.trim().parse().ok()
}

fn ascii_lower(unit: u16) -> u8 {
    let byte = unit as u8;
    if byte.is_ascii_uppercase() {
        byte.to_ascii_lowercase()
    } else {
        byte
    }
}

fn java_round_f32(value: f32) -> i32 {
    (value + 0.5).floor() as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vulkan::GuiDrawList::GuiDrawCommand;

    fn metric_renderer() -> FontRenderer {
        let mut renderer = FontRenderer {
            char_width: [0; 256],
            char_width_float: [6.0; 256],
            glyph_width: vec![0; 65_536],
            color_code: build_color_codes(false),
            location_font_texture_base: ResourceLocation::parse("textures/font/ascii.png"),
            location_font_texture: ResourceLocation::parse("textures/font/ascii.png"),
            unicode_flag: false,
            bidi_flag: false,
            font_height: 9,
            offset_bold: 1.0,
            blend: false,
            font_random: JavaRandom::new(0),
            custom_fonts: false,
            unicode_page_locations: (0_u16..=255)
                .map(|page| {
                    ResourceLocation::new(
                        "minecraft",
                        format!("textures/font/unicode_page_{page:02x}.png"),
                    )
                })
                .collect(),
        };
        renderer.char_width_float[32] = 4.0;
        renderer
    }

    #[test]
    fn formatting_codes_have_no_width_and_bold_adds_offset() {
        let renderer = metric_renderer();
        assert_eq!(renderer.get_string_width("AB"), 12);
        assert_eq!(renderer.get_string_width("A§lB"), 13);
        assert_eq!(renderer.get_string_width("A§lB§rC"), 19);
    }

    #[test]
    fn minecraft_color_codes_match_known_values() {
        let colors = build_color_codes(false);
        assert_eq!(colors[0], 0x000000);
        assert_eq!(colors[6], 0xFFAA00);
        assert_eq!(colors[15], 0xFFFFFF);
        assert_eq!(colors[16], 0x000000);
    }

    #[test]
    fn trims_using_mcp_format_state() {
        let renderer = metric_renderer();
        assert_eq!(renderer.trim_string_to_width("ABC", 12, false), "AB");
        assert_eq!(renderer.trim_string_to_width("ABC", 12, true), "BC");
    }

    #[test]
    fn unicode_text_uses_the_matching_glyph_page_instead_of_question_mark_fallback() {
        let mut renderer = metric_renderer();
        let glyph = '箱' as usize;
        renderer.glyph_width[glyph] = 0x0F;
        let mut drawList = GuiDrawList::new();
        renderer.draw_string(&mut drawList, "箱", 8.0, 6.0, 4_210_752, false);
        let GuiDrawCommand::Quad {
            texture: Some(texture),
            ..
        } = &drawList.commands()[0]
        else {
            panic!("Unicode glyph did not emit a textured font quad");
        };
        assert_eq!(
            texture,
            &ResourceLocation::new("minecraft", "textures/font/unicode_page_7b.png")
        );
    }
}
