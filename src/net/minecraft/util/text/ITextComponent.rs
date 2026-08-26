use serde_json::Value;
use thiserror::Error;

use crate::net::minecraft::client::resources::Locale::Locale;

#[derive(Debug, Error)]
pub enum TextComponentError {
    #[error("invalid text-component JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("text-component root must be a string, object, or array")]
    InvalidRoot,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ITextComponent {
    formattedText: String,
    unformattedText: String,
    rawJson: Option<String>,
}

impl ITextComponent {
    pub fn fromPlainText(text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            formattedText: text.clone(),
            unformattedText: text,
            rawJson: None,
        }
    }

    pub fn fromJsonLenient(json: &str) -> Result<Self, TextComponentError> {
        match serde_json::from_str::<Value>(json) {
            Ok(value) => Self::fromValue(&value),
            Err(_) => Ok(Self::fromPlainText(json)),
        }
    }

    pub fn fromValue(value: &Value) -> Result<Self, TextComponentError> {
        let mut formatted = String::new();
        let mut unformatted = String::new();
        append_component(
            value,
            &mut formatted,
            &mut unformatted,
            TextStyle::default(),
            None,
        )?;
        Ok(Self {
            formattedText: formatted,
            unformattedText: unformatted,
            rawJson: Some(value.to_string()),
        })
    }

    pub fn getFormattedText(&self) -> &str {
        &self.formattedText
    }
    pub fn getUnformattedText(&self) -> &str {
        &self.unformattedText
    }

    pub fn resolveWithLocale(&self, locale: &Locale) -> Self {
        let Some(raw) = self.rawJson.as_deref() else {
            return self.clone();
        };
        let Ok(value) = serde_json::from_str::<Value>(raw) else {
            return self.clone();
        };
        let mut formatted = String::new();
        let mut unformatted = String::new();
        if append_component(
            &value,
            &mut formatted,
            &mut unformatted,
            TextStyle::default(),
            Some(locale),
        )
        .is_err()
        {
            return self.clone();
        }
        Self {
            formattedText: formatted,
            unformattedText: unformatted,
            rawJson: self.rawJson.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct TextStyle {
    color: Option<char>,
    bold: bool,
    italic: bool,
    underlined: bool,
    strikethrough: bool,
    obfuscated: bool,
}

impl TextStyle {
    fn resolve(object: &serde_json::Map<String, Value>, inherited: Self) -> Self {
        Self {
            color: object
                .get("color")
                .and_then(Value::as_str)
                .and_then(color_code)
                .or(inherited.color),
            bold: object
                .get("bold")
                .and_then(Value::as_bool)
                .unwrap_or(inherited.bold),
            italic: object
                .get("italic")
                .and_then(Value::as_bool)
                .unwrap_or(inherited.italic),
            underlined: object
                .get("underlined")
                .and_then(Value::as_bool)
                .unwrap_or(inherited.underlined),
            strikethrough: object
                .get("strikethrough")
                .and_then(Value::as_bool)
                .unwrap_or(inherited.strikethrough),
            obfuscated: object
                .get("obfuscated")
                .and_then(Value::as_bool)
                .unwrap_or(inherited.obfuscated),
        }
    }

    fn append_transition(from: Self, to: Self, output: &mut String) {
        if from == to {
            return;
        }
        // A reset followed by the complete effective target style is the only
        // formatting-code representation that can express JSON `false`
        // overriding an inherited true value. Default-to-default emits
        // nothing, preserving empty-component semantics.
        output.push_str("§r");
        if let Some(color) = to.color {
            output.push('§');
            output.push(color);
        }
        if to.obfuscated {
            output.push_str("§k");
        }
        if to.bold {
            output.push_str("§l");
        }
        if to.strikethrough {
            output.push_str("§m");
        }
        if to.underlined {
            output.push_str("§n");
        }
        if to.italic {
            output.push_str("§o");
        }
    }
}

fn append_component(
    value: &Value,
    formatted: &mut String,
    unformatted: &mut String,
    inherited_style: TextStyle,
    locale: Option<&Locale>,
) -> Result<(), TextComponentError> {
    match value {
        Value::String(text) => {
            formatted.push_str(text);
            unformatted.push_str(text);
        }
        Value::Array(values) => {
            for value in values {
                append_component(value, formatted, unformatted, inherited_style, locale)?;
            }
        }
        Value::Object(object) => {
            let style = TextStyle::resolve(object, inherited_style);
            TextStyle::append_transition(inherited_style, style, formatted);

            if let Some(text) = object.get("text").and_then(Value::as_str) {
                formatted.push_str(text);
                unformatted.push_str(text);
            } else if let Some(key) = object.get("translate").and_then(Value::as_str) {
                let template = locale.map_or(key, |locale| locale.translate_key(key));
                let mut formatted_args = Vec::new();
                let mut unformatted_args = Vec::new();
                if let Some(with) = object.get("with").and_then(Value::as_array) {
                    for argument in with {
                        let mut argument_formatted = String::new();
                        let mut argument_unformatted = String::new();
                        append_component(
                            argument,
                            &mut argument_formatted,
                            &mut argument_unformatted,
                            style,
                            locale,
                        )?;
                        formatted_args.push(argument_formatted);
                        unformatted_args.push(argument_unformatted);
                    }
                }
                formatted.push_str(&format_translation(template, &formatted_args));
                unformatted.push_str(&format_translation(template, &unformatted_args));
            } else if let Some(score) = object.get("score") {
                if let Some(value) = score.get("value").and_then(Value::as_str) {
                    formatted.push_str(value);
                    unformatted.push_str(value);
                }
            } else if let Some(selector) = object.get("selector").and_then(Value::as_str) {
                formatted.push_str(selector);
                unformatted.push_str(selector);
            } else if let Some(keybind) = object.get("keybind").and_then(Value::as_str) {
                // Keybind localization is owned by TextComponentKeybind's
                // supplier in vanilla. Until that supplier is ported, retain
                // the exact server key rather than replacing it with `?`.
                formatted.push_str(keybind);
                unformatted.push_str(keybind);
            }

            if let Some(extra) = object.get("extra").and_then(Value::as_array) {
                for child in extra {
                    append_component(child, formatted, unformatted, style, locale)?;
                }
            }
            TextStyle::append_transition(style, inherited_style, formatted);
        }
        Value::Number(number) => {
            let text = number.to_string();
            formatted.push_str(&text);
            unformatted.push_str(&text);
        }
        Value::Bool(boolean) => {
            let text = boolean.to_string();
            formatted.push_str(&text);
            unformatted.push_str(&text);
        }
        Value::Null => {}
    }
    Ok(())
}

fn format_translation(template: &str, arguments: &[String]) -> String {
    let characters = template.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut sequential = 0_usize;
    let mut index = 0_usize;
    while index < characters.len() {
        if characters[index] != '%' {
            output.push(characters[index]);
            index += 1;
            continue;
        }
        if index + 1 < characters.len() && characters[index + 1] == '%' {
            output.push('%');
            index += 2;
            continue;
        }
        let formatStart = index;
        index += 1;
        let digitsStart = index;
        while index < characters.len() && characters[index].is_ascii_digit() {
            index += 1;
        }
        let explicit =
            if index < characters.len() && characters[index] == '$' && index > digitsStart {
                let value = characters[digitsStart..index]
                    .iter()
                    .collect::<String>()
                    .parse::<usize>()
                    .ok();
                index += 1;
                value.and_then(|value| value.checked_sub(1))
            } else {
                index = digitsStart;
                None
            };
        if index < characters.len() && characters[index] == 's' {
            let argumentIndex = explicit.unwrap_or_else(|| {
                let value = sequential;
                sequential += 1;
                value
            });
            if let Some(argument) = arguments.get(argumentIndex) {
                output.push_str(argument);
            }
            index += 1;
        } else {
            output.extend(characters[formatStart..index].iter());
        }
    }
    output
}

fn color_code(color: &str) -> Option<char> {
    Some(match color {
        "black" => '0',
        "dark_blue" => '1',
        "dark_green" => '2',
        "dark_aqua" => '3',
        "dark_red" => '4',
        "dark_purple" => '5',
        "gold" => '6',
        "gray" => '7',
        "dark_gray" => '8',
        "blue" => '9',
        "green" => 'a',
        "aqua" => 'b',
        "red" => 'c',
        "light_purple" => 'd',
        "yellow" => 'e',
        "white" => 'f',
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_extra_and_color() {
        let component =
            ITextComponent::fromJsonLenient(r#"{"text":"A","color":"red","extra":[{"text":"B"}]}"#)
                .unwrap();
        assert_eq!(component.getUnformattedText(), "AB");
        assert!(component.getFormattedText().contains("§cA"));
    }

    #[test]
    fn empty_default_component_stays_empty() {
        let component = ITextComponent::fromJsonLenient(r#"{"text":""}"#).unwrap();
        assert!(component.getFormattedText().is_empty());
        assert!(component.getUnformattedText().is_empty());
    }

    #[test]
    fn child_inherits_and_can_disable_parent_styles() {
        let component = ITextComponent::fromJsonLenient(
            r#"{"text":"A","color":"red","bold":true,"extra":[{"text":"B"},{"text":"C","bold":false}]}"#,
        ).unwrap();
        assert_eq!(component.getUnformattedText(), "ABC");
        assert!(component.getFormattedText().contains("§c§lA"));
        assert!(component.getFormattedText().contains("§c§lB"));
        assert!(component.getFormattedText().contains("§cC"));
    }

    #[test]
    fn translation_supports_sequential_positional_and_literal_percent() {
        let mut locale = Locale::default();
        locale.load_bytes(
            b"test.format=%2$s / %s / %% / %1$s
",
        );
        let component = ITextComponent::fromJsonLenient(
            r#"{"translate":"test.format","with":["first","second"]}"#,
        )
        .unwrap()
        .resolveWithLocale(&locale);
        assert_eq!(component.getUnformattedText(), "second / first / % / first");
    }
}
