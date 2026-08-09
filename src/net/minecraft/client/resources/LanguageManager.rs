//! MCP `LanguageManager`: the language map parsed from each pack's
//! `pack.mcmeta` "language" section (the `LanguageMetadataSection`), the
//! current language with `en_us` fallback, and the code-sorted language list
//! the language GUI renders.

use std::collections::HashMap;

use super::Language::Language;

pub struct LanguageManager {
    currentLanguage: String,
    languageMap: HashMap<String, Language>,
}

impl LanguageManager {
    pub fn new(currentLanguage: impl Into<String>) -> Self {
        Self {
            currentLanguage: currentLanguage.into(),
            languageMap: HashMap::new(),
        }
    }

    /// MCP `parseLanguageMetadata`: merge each pack's metadata section in
    /// pack order; the first pack that declares a code wins (vanilla keeps
    /// the existing entry when the map already contains the code).
    ///
    /// The metadata layout follows `LanguageMetadataSectionSerializer`:
    /// `{"language": {"<code>": {"region": ..., "name": ..., "bidirectional": ...}}}`
    /// where `region` and `name` are required strings and `bidirectional`
    /// defaults to false. Vanilla treats a malformed section as an IOException
    /// and skips that pack; this port warns and skips the offending entry.
    pub fn parseLanguageMetadata(&mut self, metadataSections: &[Vec<u8>]) {
        self.languageMap.clear();
        for section in metadataSections {
            let Ok(value) = serde_json::from_slice::<serde_json::Value>(section) else {
                log::warn!("unable to parse language metadata section");
                continue;
            };
            let Some(languages) = value.get("language").and_then(serde_json::Value::as_object) else {
                continue;
            };
            for (code, entry) in languages {
                if self.languageMap.contains_key(code) {
                    continue;
                }
                let Some(entry) = entry.as_object() else { continue; };
                let Some(region) = entry.get("region").and_then(serde_json::Value::as_str) else {
                    log::warn!("language {code} metadata is missing region; skipping");
                    continue;
                };
                let Some(name) = entry.get("name").and_then(serde_json::Value::as_str) else {
                    log::warn!("language {code} metadata is missing name; skipping");
                    continue;
                };
                let bidirectional = entry
                    .get("bidirectional")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false);
                self.languageMap.insert(
                    code.clone(),
                    Language::new(code.clone(), region, name, bidirectional),
                );
            }
        }
    }

    /// MCP `setCurrentLanguage`.
    pub fn setCurrentLanguage(&mut self, languageCode: &str) {
        self.currentLanguage = languageCode.to_owned();
    }

    /// MCP `getCurrentLanguage`: falls back to `en_us` when the map does not
    /// contain the current code.
    pub fn getCurrentLanguage(&self) -> &Language {
        self.languageMap
            .get(&self.currentLanguage)
            .or_else(|| self.languageMap.get("en_us"))
            .expect("language metadata must declare en_us")
    }

    /// MCP `getLanguages` (a `TreeSet`): all languages sorted by code.
    pub fn getLanguages(&self) -> Vec<&Language> {
        let mut languages: Vec<&Language> = self.languageMap.values().collect();
        languages.sort();
        languages
    }

    /// MCP `isCurrentLanguageBidirectional`.
    pub fn isCurrentLanguageBidirectional(&self) -> bool {
        self.getCurrentLanguage().isBidirectional()
    }
}

#[cfg(test)]
mod tests {
    use super::LanguageManager;

    fn metadata(entries: &str) -> Vec<Vec<u8>> {
        vec![format!(r#"{{"language":{{{entries}}}}}"#).into_bytes()]
    }

    #[test]
    fn parses_metadata_section_and_falls_back_to_en_us() {
        let mut manager = LanguageManager::new("zh_cn");
        manager.parseLanguageMetadata(&metadata(r#""en_us":{"region":"US","name":"English"},"zh_cn":{"region":"中国","name":"简体中文","bidirectional":false}"#));
        assert_eq!(manager.getCurrentLanguage().getLanguageCode(), "zh_cn");
        assert!(!manager.isCurrentLanguageBidirectional());
        assert_eq!(manager.getLanguages().len(), 2);
        // TreeSet ordering is by language code.
        assert_eq!(manager.getLanguages()[0].getLanguageCode(), "en_us");
        assert_eq!(manager.getLanguages()[1].getLanguageCode(), "zh_cn");
        assert_eq!(manager.getLanguages()[1].to_string(), "简体中文 (中国)");
    }

    #[test]
    fn unknown_current_language_falls_back_to_en_us() {
        let mut manager = LanguageManager::new("missing_lang");
        manager.parseLanguageMetadata(&metadata(r#""en_us":{"region":"US","name":"English"}"#));
        assert_eq!(manager.getCurrentLanguage().getLanguageCode(), "en_us");
    }

    #[test]
    fn first_pack_wins_for_a_code() {
        let mut manager = LanguageManager::new("zh_cn");
        manager.parseLanguageMetadata(&[
            r#"{"language":{"zh_cn":{"region":"中国","name":"简体中文"}}}"#.as_bytes().to_vec(),
            r#"{"language":{"zh_cn":{"region":"TW","name":"繁體中文"}}}"#.as_bytes().to_vec(),
        ]);
        assert_eq!(manager.getLanguages()[0].to_string(), "简体中文 (中国)");
    }

    #[test]
    fn bidirectional_flag_is_optional_and_defaults_false() {
        let mut manager = LanguageManager::new("ar_sa");
        manager.parseLanguageMetadata(&metadata(r#""ar_sa":{"region":"المملكة العربية السعودية","name":"العربية","bidirectional":true},"he_il":{"region":"ישראל","name":"עברית"}"#));
        assert!(manager.isCurrentLanguageBidirectional());
        manager.setCurrentLanguage("he_il");
        assert!(!manager.isCurrentLanguageBidirectional());
    }
}
