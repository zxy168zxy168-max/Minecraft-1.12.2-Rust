use serde_json::{json, Value};

use crate::net::minecraft::util::text::ITextComponent::ITextComponent;

pub fn TextComponentTranslation(key: &str, arguments: &[&str]) -> ITextComponent {
    let with = arguments
        .iter()
        .map(|argument| Value::String((*argument).to_owned()))
        .collect::<Vec<_>>();
    ITextComponent::fromValue(&json!({ "translate": key, "with": with }))
        .unwrap_or_else(|_| ITextComponent::fromPlainText(key))
}
