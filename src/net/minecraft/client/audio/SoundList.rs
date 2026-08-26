use serde_json::Value;

use crate::net::minecraft::client::audio::Sound::{Sound, Type};

#[derive(Debug, Clone, PartialEq)]
pub struct SoundList {
    sounds: Vec<Sound>,
    replaceExisting: bool,
    subtitle: Option<String>,
}

impl SoundList {
    pub fn fromJson(value: &Value) -> Result<Self, String> {
        let object = value
            .as_object()
            .ok_or_else(|| "sound event entry must be an object".to_owned())?;
        let replaceExisting = object
            .get("replace")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let subtitle = match object.get("subtitle") {
            None | Some(Value::Null) => None,
            Some(Value::String(value)) => Some(value.clone()),
            Some(_) => return Err("sound event subtitle must be a string".to_owned()),
        };
        let mut sounds = Vec::new();
        if let Some(entries) = object.get("sounds") {
            let entries = entries
                .as_array()
                .ok_or_else(|| "sound event sounds must be an array".to_owned())?;
            for entry in entries {
                sounds.push(parse_sound(entry)?);
            }
        }
        Ok(Self {
            sounds,
            replaceExisting,
            subtitle,
        })
    }

    pub fn getSounds(&self) -> &[Sound] {
        &self.sounds
    }
    pub const fn canReplaceExisting(&self) -> bool {
        self.replaceExisting
    }
    pub fn getSubtitle(&self) -> Option<&str> {
        self.subtitle.as_deref()
    }
}

fn parse_sound(value: &Value) -> Result<Sound, String> {
    if let Some(name) = value.as_str() {
        return Ok(Sound::new(name, 1.0, 1.0, 1, Type::File, false));
    }
    let object = value
        .as_object()
        .ok_or_else(|| "sound entry must be a string or object".to_owned())?;
    let name = object
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| "sound object requires a string name".to_owned())?;
    let volume = number(object.get("volume"), 1.0, "volume")?;
    if volume <= 0.0 {
        return Err("Invalid volume".to_owned());
    }
    let pitch = number(object.get("pitch"), 1.0, "pitch")?;
    if pitch <= 0.0 {
        return Err("Invalid pitch".to_owned());
    }
    let weight = match object.get("weight") {
        None => 1,
        Some(value) => value
            .as_i64()
            .and_then(|value| i32::try_from(value).ok())
            .ok_or_else(|| "sound weight must be an integer".to_owned())?,
    };
    if weight <= 0 {
        return Err("Invalid weight".to_owned());
    }
    let streaming = match object.get("stream") {
        None => false,
        Some(value) => value
            .as_bool()
            .ok_or_else(|| "sound stream must be a boolean".to_owned())?,
    };
    let soundType = match object.get("type") {
        None => Type::File,
        Some(value) => Type::getByName(
            value
                .as_str()
                .ok_or_else(|| "sound type must be a string".to_owned())?,
        )
        .ok_or_else(|| "Invalid type".to_owned())?,
    };
    Ok(Sound::new(
        name, volume, pitch, weight, soundType, streaming,
    ))
}

fn number(value: Option<&Value>, default: f32, field: &str) -> Result<f32, String> {
    match value {
        None => Ok(default),
        Some(value) => value
            .as_f64()
            .map(|value| value as f32)
            .ok_or_else(|| format!("sound {field} must be a number")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_string_and_event_entries() {
        let value: Value = serde_json::from_str(r#"{
            "replace": true,
            "subtitle": "subtitles.test",
            "sounds": ["test/a", {"name":"test:nested","type":"event","volume":0.5,"weight":2,"stream":true}]
        }"#).unwrap();
        let list = SoundList::fromJson(&value).unwrap();
        assert!(list.canReplaceExisting());
        assert_eq!(list.getSubtitle(), Some("subtitles.test"));
        assert_eq!(list.getSounds()[1].getType(), Type::SoundEvent);
        assert!(list.getSounds()[1].isStreaming());
    }
}
