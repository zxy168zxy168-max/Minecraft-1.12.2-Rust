use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServerStatusResponse {
    description: Option<String>,
    players: Option<Players>,
    version: Option<Version>,
    favicon: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Players {
    maxPlayers: i32,
    onlinePlayerCount: i32,
    players: Vec<GameProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameProfile {
    pub id: Option<Uuid>,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    name: String,
    protocol: i32,
}

impl ServerStatusResponse {
    pub fn fromJson(json: &str) -> Result<Self, serde_json::Error> {
        let value: Value = serde_json::from_str(json)?;
        let description = value.get("description").map(flattenTextComponent);
        let players = value
            .get("players")
            .and_then(Value::as_object)
            .map(|object| {
                let maxPlayers = object.get("max").and_then(Value::as_i64).unwrap_or(0) as i32;
                let onlinePlayerCount =
                    object.get("online").and_then(Value::as_i64).unwrap_or(0) as i32;
                let players = object
                    .get("sample")
                    .and_then(Value::as_array)
                    .map(|sample| {
                        sample
                            .iter()
                            .filter_map(|entry| {
                                let object = entry.as_object()?;
                                let name = object
                                    .get("name")
                                    .and_then(Value::as_str)
                                    .unwrap_or_default()
                                    .to_owned();
                                let id = object
                                    .get("id")
                                    .and_then(Value::as_str)
                                    .and_then(|value| Uuid::parse_str(value).ok());
                                Some(GameProfile { id, name })
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                Players {
                    maxPlayers,
                    onlinePlayerCount,
                    players,
                }
            });
        let version = value
            .get("version")
            .and_then(Value::as_object)
            .map(|object| Version {
                name: object
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
                protocol: object.get("protocol").and_then(Value::as_i64).unwrap_or(0) as i32,
            });
        let favicon = value
            .get("favicon")
            .and_then(Value::as_str)
            .map(str::to_owned);
        Ok(Self {
            description,
            players,
            version,
            favicon,
        })
    }

    pub fn getServerDescription(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn getPlayers(&self) -> Option<&Players> {
        self.players.as_ref()
    }
    pub fn getVersion(&self) -> Option<&Version> {
        self.version.as_ref()
    }
    pub fn getFavicon(&self) -> Option<&str> {
        self.favicon.as_deref()
    }
}

impl Players {
    pub const fn getMaxPlayers(&self) -> i32 {
        self.maxPlayers
    }
    pub const fn getOnlinePlayerCount(&self) -> i32 {
        self.onlinePlayerCount
    }
    pub fn getPlayers(&self) -> &[GameProfile] {
        &self.players
    }
}

impl Version {
    pub fn getName(&self) -> &str {
        &self.name
    }
    pub const fn getProtocol(&self) -> i32 {
        self.protocol
    }
}

#[derive(Debug, Clone, Default)]
struct EffectiveStyle {
    color: Option<&'static str>,
    bold: bool,
    italic: bool,
    underlined: bool,
    strikethrough: bool,
    obfuscated: bool,
}

fn flattenTextComponent(value: &Value) -> String {
    let mut output = String::new();
    appendTextComponent(value, &EffectiveStyle::default(), &mut output);
    output
}

fn appendTextComponent(value: &Value, inherited: &EffectiveStyle, output: &mut String) {
    match value {
        Value::String(text) => appendStyledText(text, inherited, output),
        Value::Array(values) => {
            for value in values {
                appendTextComponent(value, inherited, output);
            }
        }
        Value::Object(object) => {
            let style = resolveStyle(object, inherited);
            let text = object
                .get("text")
                .and_then(Value::as_str)
                .or_else(|| object.get("translate").and_then(Value::as_str))
                .unwrap_or_default();
            appendStyledText(text, &style, output);
            if let Some(extra) = object.get("extra").and_then(Value::as_array) {
                for part in extra {
                    appendTextComponent(part, &style, output);
                }
            }
        }
        _ => {}
    }
}

fn appendStyledText(text: &str, style: &EffectiveStyle, output: &mut String) {
    if text.is_empty() {
        return;
    }
    if let Some(color) = style.color {
        output.push_str(color);
    }
    if style.bold {
        output.push_str("§l");
    }
    if style.italic {
        output.push_str("§o");
    }
    if style.underlined {
        output.push_str("§n");
    }
    if style.obfuscated {
        output.push_str("§k");
    }
    if style.strikethrough {
        output.push_str("§m");
    }
    output.push_str(text);
    output.push_str("§r");
}

fn resolveStyle(
    object: &serde_json::Map<String, Value>,
    inherited: &EffectiveStyle,
) -> EffectiveStyle {
    let mut style = inherited.clone();
    if let Some(color) = object
        .get("color")
        .and_then(Value::as_str)
        .and_then(colorCode)
    {
        style.color = Some(color);
    }
    if let Some(value) = object.get("bold").and_then(Value::as_bool) {
        style.bold = value;
    }
    if let Some(value) = object.get("italic").and_then(Value::as_bool) {
        style.italic = value;
    }
    if let Some(value) = object.get("underlined").and_then(Value::as_bool) {
        style.underlined = value;
    }
    if let Some(value) = object.get("strikethrough").and_then(Value::as_bool) {
        style.strikethrough = value;
    }
    if let Some(value) = object.get("obfuscated").and_then(Value::as_bool) {
        style.obfuscated = value;
    }
    style
}

fn colorCode(value: &str) -> Option<&'static str> {
    Some(match value {
        "black" => "§0",
        "dark_blue" => "§1",
        "dark_green" => "§2",
        "dark_aqua" => "§3",
        "dark_red" => "§4",
        "dark_purple" => "§5",
        "gold" => "§6",
        "gray" => "§7",
        "dark_gray" => "§8",
        "blue" => "§9",
        "green" => "§a",
        "aqua" => "§b",
        "red" => "§c",
        "light_purple" => "§d",
        "yellow" => "§e",
        "white" => "§f",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn status_json_preserves_motd_version_players_and_favicon() {
        let status = ServerStatusResponse::fromJson(r#"{"version":{"name":"1.12.2","protocol":340},"players":{"max":20,"online":2,"sample":[{"name":"Alex","id":"00000000-0000-0000-0000-000000000001"}]},"description":{"text":"Hello ","extra":[{"text":"world"}]},"favicon":"data:image/png;base64,AAA="}"#).unwrap();
        assert_eq!(status.getServerDescription(), Some("Hello §rworld§r"));
        assert_eq!(status.getVersion().unwrap().getProtocol(), 340);
        assert_eq!(status.getPlayers().unwrap().getPlayers()[0].name, "Alex");
        assert!(status
            .getFavicon()
            .unwrap()
            .starts_with("data:image/png;base64,"));
    }

    #[test]
    fn status_description_keeps_minecraft_formatting_codes() {
        let status = ServerStatusResponse::fromJson(r#"{"description":{"text":"Red","color":"red","bold":true,"extra":[{"text":" child"}]}}"#).unwrap();
        assert_eq!(status.getServerDescription(), Some("§c§lRed§r§c§l child§r"));
    }
}
