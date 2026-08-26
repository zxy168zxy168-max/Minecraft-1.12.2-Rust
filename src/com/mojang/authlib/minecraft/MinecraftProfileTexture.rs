use std::collections::BTreeMap;

/// Rust equivalent of Authlib 1.5.25 `MinecraftProfileTexture`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinecraftProfileTexture {
    url: String,
    metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextureType {
    Skin,
    Cape,
    Elytra,
}

impl MinecraftProfileTexture {
    pub fn new(url: impl Into<String>, metadata: BTreeMap<String, String>) -> Self {
        Self {
            url: url.into(),
            metadata,
        }
    }

    pub fn getUrl(&self) -> &str {
        &self.url
    }

    pub fn getMetadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(String::as_str)
    }

    /// Authlib delegates to `FilenameUtils.getBaseName(url)`: strip the path,
    /// query/fragment, and final extension while retaining the texture hash.
    pub fn getHash(&self) -> String {
        let without_fragment = self.url.split('#').next().unwrap_or(&self.url);
        let without_query = without_fragment
            .split('?')
            .next()
            .unwrap_or(without_fragment);
        let name = without_query.rsplit('/').next().unwrap_or(without_query);
        match name.rfind('.') {
            Some(index) if index > 0 => name[..index].to_owned(),
            _ => name.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_texture_hash_matches_filename_utils_basename() {
        let texture = MinecraftProfileTexture::new(
            "https://textures.minecraft.net/texture/012345abcdef.png?ignored=1",
            BTreeMap::new(),
        );
        assert_eq!(texture.getHash(), "012345abcdef");
    }
}
