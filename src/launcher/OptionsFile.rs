use std::{fs, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OptionsFileError {
    #[error("failed to read options file: {0}")]
    Read(#[from] std::io::Error),
}

/// Order-preserving parser for Minecraft's `key:value` options format.
/// Unknown keys and duplicate keys are retained unless the caller explicitly
/// replaces the last occurrence with `set`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct OptionsFile {
    entries: Vec<(String, String)>,
}

impl OptionsFile {
    pub fn parse(contents: &str) -> Self {
        let entries = contents
            .lines()
            .filter_map(|line| line.split_once(':'))
            .map(|(key, value)| (key.to_owned(), value.to_owned()))
            .collect();
        Self { entries }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, OptionsFileError> {
        Ok(Self::parse(&fs::read_to_string(path)?))
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .rev()
            .find(|(candidate, _)| candidate == key)
            .map(|(_, value)| value.as_str())
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        let value = value.into();
        if let Some((_, current)) = self
            .entries
            .iter_mut()
            .rev()
            .find(|(candidate, _)| candidate == &key)
        {
            *current = value;
        } else {
            self.entries.push((key, value));
        }
    }

    pub fn remove(&mut self, key: &str) {
        self.entries.retain(|(candidate, _)| candidate != key);
    }

    pub fn entries(&self) -> &[(String, String)] {
        &self.entries
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        for (key, value) in &self.entries {
            output.push_str(key);
            output.push(':');
            output.push_str(value);
            output.push('\n');
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_may_contain_colons() {
        let options = OptionsFile::parse("lastServer:127.0.0.1:25565\n");
        assert_eq!(options.get("lastServer"), Some("127.0.0.1:25565"));
    }

    #[test]
    fn preserves_file_order() {
        let source = "fov:0.0\ngamma:1.0\n";
        assert_eq!(OptionsFile::parse(source).render(), source);
    }

    #[test]
    fn remove_deletes_all_legacy_occurrences() {
        let mut options = OptionsFile::parse("clouds:2\nfov:0.0\nclouds:1\n");
        options.remove("clouds");
        assert_eq!(options.render(), "fov:0.0\n");
    }
}
