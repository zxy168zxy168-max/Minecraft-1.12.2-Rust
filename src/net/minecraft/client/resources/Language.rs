//! MCP `Language`: one entry of a pack's `language` metadata section
//! (`pack.mcmeta`). Equality and ordering match the Java class — languages
//! are compared by language code only — and `Display` renders the same
//! `"name (region)"` text the language GUI draws for each row.

#[derive(Debug, Clone)]
pub struct Language {
    languageCode: String,
    region: String,
    name: String,
    bidirectional: bool,
}

impl Language {
    pub fn new(
        languageCode: impl Into<String>,
        region: impl Into<String>,
        name: impl Into<String>,
        bidirectional: bool,
    ) -> Self {
        Self {
            languageCode: languageCode.into(),
            region: region.into(),
            name: name.into(),
            bidirectional,
        }
    }

    pub fn getLanguageCode(&self) -> &str {
        &self.languageCode
    }

    pub fn isBidirectional(&self) -> bool {
        self.bidirectional
    }
}

/// MCP `Language#toString`: `String.format("%s (%s)", name, region)`.
impl std::fmt::Display for Language {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{} ({})", self.name, self.region)
    }
}

/// MCP `Language#equals`: language code only.
impl PartialEq for Language {
    fn eq(&self, other: &Self) -> bool {
        self.languageCode == other.languageCode
    }
}

impl Eq for Language {}

/// MCP `Language#hashCode`: language code only, matching `equals`.
impl std::hash::Hash for Language {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.languageCode.hash(state);
    }
}

/// MCP `Language#compareTo`: language code ordering (the TreeSet the
/// LanguageManager exposes sorts by it).
impl PartialOrd for Language {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Language {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.languageCode.cmp(&other.languageCode)
    }
}

#[cfg(test)]
mod tests {
    use super::Language;

    #[test]
    fn display_renders_name_region_like_java() {
        let language = Language::new("zh_cn", "中国", "简体中文", false);
        assert_eq!(language.to_string(), "简体中文 (中国)");
    }

    #[test]
    fn equality_and_ordering_use_language_code_only() {
        let a = Language::new("en_us", "US", "English", false);
        let b = Language::new("en_us", "GB", "English (UK)", false);
        assert_eq!(a, b);
        assert_eq!(a.cmp(&Language::new("af_za", "", "", false)), std::cmp::Ordering::Greater);
        assert!(a < Language::new("zh_cn", "", "", false));
    }
}
