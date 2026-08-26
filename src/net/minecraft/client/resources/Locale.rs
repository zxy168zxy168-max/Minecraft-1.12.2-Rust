use std::collections::HashMap;

use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

use crate::net::minecraft::client::resources::SimpleReloadableResourceManager::{
    ResourceManager, ResourceManagerError,
};

#[derive(Debug, Clone, Default)]
pub struct Locale {
    properties: HashMap<String, String>,
    unicode: bool,
}

impl Locale {
    pub fn load(
        resource_manager: &ResourceManager,
        language_codes: &[&str],
        domains: &[&str],
    ) -> Self {
        let mut locale = Self::default();
        for language in language_codes {
            for domain in domains {
                let location = ResourceLocation::new(*domain, format!("lang/{language}.lang"));
                let Ok(resources) = resource_manager.get_all_resources(&location) else {
                    continue;
                };
                for resource in resources {
                    locale.load_bytes(&resource.bytes);
                }
            }
        }
        locale.check_unicode();
        locale
    }

    pub fn load_resource(
        &mut self,
        manager: &ResourceManager,
        location: &ResourceLocation,
    ) -> Result<(), ResourceManagerError> {
        for resource in manager.get_all_resources(location)? {
            self.load_bytes(&resource.bytes);
        }
        self.check_unicode();
        Ok(())
    }

    pub fn load_bytes(&mut self, bytes: &[u8]) {
        let text = String::from_utf8_lossy(bytes);
        for line in text.lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            self.properties
                .insert(key.to_owned(), normalize_numeric_formats(value));
        }
    }

    pub fn translate_key<'a>(&'a self, key: &'a str) -> &'a str {
        self.properties.get(key).map(String::as_str).unwrap_or(key)
    }

    pub fn has_key(&self, key: &str) -> bool {
        self.properties.contains_key(key)
    }
    pub const fn is_unicode(&self) -> bool {
        self.unicode
    }
    pub fn len(&self) -> usize {
        self.properties.len()
    }
    pub fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }

    fn check_unicode(&mut self) {
        let mut unicode_count = 0_usize;
        let mut total = 0_usize;
        for value in self.properties.values() {
            for ch in value.chars() {
                total += 1;
                if ch as u32 >= 256 {
                    unicode_count += 1;
                }
            }
        }
        self.unicode = total != 0 && unicode_count as f32 / total as f32 > 0.1;
    }
}

fn normalize_numeric_formats(input: &str) -> String {
    // Port of Locale.PATTERN: %(\d+\$)?[\d\.]*[df] -> %$1s.
    let chars: Vec<char> = input.chars().collect();
    let mut output = String::with_capacity(input.len());
    let mut index = 0;
    while index < chars.len() {
        if chars[index] != '%' {
            output.push(chars[index]);
            index += 1;
            continue;
        }
        let start = index;
        index += 1;
        let digits_start = index;
        while index < chars.len() && chars[index].is_ascii_digit() {
            index += 1;
        }
        let positional = if index < chars.len() && chars[index] == '$' && index > digits_start {
            let value: String = chars[digits_start..=index].iter().collect();
            index += 1;
            Some(value)
        } else {
            index = digits_start;
            None
        };
        while index < chars.len() && (chars[index].is_ascii_digit() || chars[index] == '.') {
            index += 1;
        }
        if index < chars.len() && matches!(chars[index], 'd' | 'f') {
            output.push('%');
            if let Some(positional) = positional {
                output.push_str(&positional);
            }
            output.push('s');
            index += 1;
        } else {
            output.extend(chars[start..index].iter());
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_overrides_and_normalizes_numeric_placeholders() {
        let mut locale = Locale::default();
        locale.load_bytes(b"menu.quit=Quit\ncount=%1$02d blocks\n");
        locale.load_bytes(b"menu.quit=Exit\n");
        assert_eq!(locale.translate_key("menu.quit"), "Exit");
        assert_eq!(locale.translate_key("count"), "%1$s blocks");
        assert_eq!(locale.translate_key("missing"), "missing");
        let dynamic_missing = String::from("dynamic.missing");
        assert_eq!(locale.translate_key(&dynamic_missing), dynamic_missing);
    }
}
