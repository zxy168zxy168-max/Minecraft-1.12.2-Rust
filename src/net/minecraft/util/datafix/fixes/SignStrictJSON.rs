use serde_json::{Map, Value};

use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IFixableData::IFixableData;

/// MCP 1.12.2 `SignStrictJSON` (DataVersion 101).
///
/// The source first tries a legacy primitive/array-only Gson adapter, then the
/// normal component serializer, then its lenient parser, finally treating the
/// original value as literal text. `normalizeTextComponentJson` preserves that
/// fallback order while serializing the result as the same component-object
/// shape used by `ITextComponent.Serializer#componentToJson`.
pub struct SignStrictJSON;

impl SignStrictJSON {
    pub(crate) fn normalizeTextComponentJson(source: &str) -> String {
        if source == "null" || source.is_empty() {
            return Self::textComponent("");
        }

        let bytes = source.as_bytes();
        let quoted_or_object = bytes.len() >= 2
            && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
                || (bytes[0] == b'{' && bytes[bytes.len() - 1] == b'}'));

        if !quoted_or_object {
            return Self::textComponent(source);
        }

        // SignStrictJSON.GSON_INSTANCE accepts primitives and arrays only.
        if let Ok(value) = serde_json::from_str::<Value>(source) {
            if let Some(component) = Self::legacyGsonComponent(&value) {
                return component.to_string();
            }

            // ITextComponent.Serializer.jsonToComponent handles valid normal
            // component objects (and its recursive primitive/array children).
            if let Some(component) = Self::vanillaComponent(&value) {
                return component.to_string();
            }
        }

        // Vanilla's final lenient parser is still JSON-shaped, but accepts a
        // few Gson-lenient forms. serde_json intentionally does not guess those
        // Java parser extensions; as the source does after all parser failures,
        // retain the complete original string as literal text rather than lose
        // data or fabricate a component.
        Self::textComponent(source)
    }

    fn textComponent(text: &str) -> String {
        let mut object = Map::new();
        object.insert("text".to_owned(), Value::String(text.to_owned()));
        Value::Object(object).to_string()
    }

    fn primitiveText(value: &Value) -> Option<String> {
        match value {
            Value::String(value) => Some(value.clone()),
            Value::Number(value) => Some(value.to_string()),
            Value::Bool(value) => Some(value.to_string()),
            _ => None,
        }
    }

    /// Exact shape of the custom deserializer registered in SignStrictJSON:
    /// primitives become TextComponentString and arrays become the first
    /// component with later elements appended as siblings (`extra`). Objects
    /// deliberately fail this first pass.
    fn legacyGsonComponent(value: &Value) -> Option<Value> {
        if let Some(text) = Self::primitiveText(value) {
            return serde_json::from_str(&Self::textComponent(&text)).ok();
        }
        let Value::Array(values) = value else { return None; };
        let mut values = values.iter();
        let first = values.next()?;
        let mut root = Self::legacyGsonComponent(first)?;
        for child in values {
            let child = Self::legacyGsonComponent(child)?;
            Self::appendSibling(&mut root, child)?;
        }
        Some(root)
    }

    /// Source-shaped subset of ITextComponent.Serializer sufficient for the
    /// strict-data migration. Unknown/invalid component objects reject rather
    /// than being silently rewritten.
    fn vanillaComponent(value: &Value) -> Option<Value> {
        if let Some(text) = Self::primitiveText(value) {
            return serde_json::from_str(&Self::textComponent(&text)).ok();
        }
        if let Value::Array(values) = value {
            let mut values = values.iter();
            let mut root = Self::vanillaComponent(values.next()?)?;
            for child in values {
                Self::appendSibling(&mut root, Self::vanillaComponent(child)?)?;
            }
            return Some(root);
        }
        let Value::Object(input) = value else { return None; };

        let mut output = Map::new();
        // Style.Serializer fields are copied onto the component object by the
        // source serializer. Preserve only source-valid JSON field types.
        for key in ["bold", "italic", "underlined", "strikethrough", "obfuscated"] {
            if let Some(Value::Bool(flag)) = input.get(key) { output.insert(key.to_owned(), Value::Bool(*flag)); }
        }
        for key in ["color", "insertion"] {
            if let Some(Value::String(text)) = input.get(key) { output.insert(key.to_owned(), Value::String(text.clone())); }
        }
        // clickEvent / hoverEvent are style payloads; if syntactically objects,
        // retaining them is equivalent to Serializer round-tripping them.
        for key in ["clickEvent", "hoverEvent"] {
            if let Some(Value::Object(object)) = input.get(key) { output.insert(key.to_owned(), Value::Object(object.clone())); }
        }

        if let Some(text) = input.get("text").and_then(Self::primitiveText) {
            output.insert("text".to_owned(), Value::String(text));
        } else if let Some(key) = input.get("translate").and_then(Self::primitiveText) {
            output.insert("translate".to_owned(), Value::String(key));
            if let Some(Value::Array(with)) = input.get("with") {
                let mut args = Vec::with_capacity(with.len());
                for arg in with {
                    let component = Self::vanillaComponent(arg)?;
                    // Serializer collapses a plain, style-free TextComponentString
                    // used as a translation argument back to a primitive string.
                    if let Value::Object(object) = &component {
                        if object.len() == 1 {
                            if let Some(Value::String(text)) = object.get("text") {
                                args.push(Value::String(text.clone()));
                                continue;
                            }
                        }
                    }
                    args.push(component);
                }
                if !args.is_empty() { output.insert("with".to_owned(), Value::Array(args)); }
            }
        } else if let Some(Value::Object(score)) = input.get("score") {
            let name = score.get("name").and_then(Self::primitiveText)?;
            let objective = score.get("objective").and_then(Self::primitiveText)?;
            let mut out_score = Map::new();
            out_score.insert("name".to_owned(), Value::String(name));
            out_score.insert("objective".to_owned(), Value::String(objective));
            if let Some(value) = score.get("value").and_then(Self::primitiveText) {
                out_score.insert("value".to_owned(), Value::String(value));
            }
            output.insert("score".to_owned(), Value::Object(out_score));
        } else if let Some(selector) = input.get("selector").and_then(Self::primitiveText) {
            output.insert("selector".to_owned(), Value::String(selector));
        } else if let Some(keybind) = input.get("keybind").and_then(Self::primitiveText) {
            output.insert("keybind".to_owned(), Value::String(keybind));
        } else {
            return None;
        }

        if let Some(Value::Array(extra)) = input.get("extra") {
            if extra.is_empty() { return None; }
            let mut siblings = Vec::with_capacity(extra.len());
            for child in extra { siblings.push(Self::vanillaComponent(child)?); }
            output.insert("extra".to_owned(), Value::Array(siblings));
        }
        Some(Value::Object(output))
    }

    fn appendSibling(root: &mut Value, child: Value) -> Option<()> {
        let Value::Object(root) = root else { return None; };
        match root.get_mut("extra") {
            Some(Value::Array(extra)) => extra.push(child),
            Some(_) => return None,
            None => { root.insert("extra".to_owned(), Value::Array(vec![child])); }
        }
        Some(())
    }

    fn updateLine(compound: &mut NBTTagCompound, key: &str) {
        let fixed = Self::normalizeTextComponentJson(&compound.getString(key));
        compound.setString(key, fixed);
    }
}

impl IFixableData for SignStrictJSON {
    fn getFixVersion(&self) -> i32 { 101 }
    fn fixTagCompound(&self, mut compound: NBTTagCompound) -> NBTTagCompound {
        if compound.getString("id") == "Sign" {
            Self::updateLine(&mut compound, "Text1");
            Self::updateLine(&mut compound, "Text2");
            Self::updateLine(&mut compound, "Text3");
            Self::updateLine(&mut compound, "Text4");
        }
        compound
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_plain_and_quoted_sign_lines_become_component_objects() {
        let mut sign = NBTTagCompound::new(); sign.setString("id", "Sign");
        sign.setString("Text1", "hello"); sign.setString("Text2", "\"world\"");
        let fixed = SignStrictJSON.fixTagCompound(sign);
        assert_eq!(fixed.getString("Text1"), r#"{"text":"hello"}"#);
        assert_eq!(fixed.getString("Text2"), r#"{"text":"world"}"#);
        assert_eq!(fixed.getString("Text3"), r#"{"text":""}"#);
    }

    #[test]
    fn top_level_array_is_literal_because_source_guard_only_accepts_quotes_or_braces() {
        let out = SignStrictJSON::normalizeTextComponentJson(r#"["a","b"]"#);
        let value: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(value["text"], r#"["a","b"]"#);
    }
}
