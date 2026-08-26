use std::collections::HashMap;

/// Parses the byte-oriented format consumed by Java
/// `java.util.Properties.load(InputStream)`.
///
/// Minecraft 1.12.2 and OptiFine C6 use this API for font and panorama
/// properties. Input bytes are therefore ISO-8859-1, not UTF-8. Logical-line
/// continuation, escaped separators, whitespace rules, and `\\uXXXX` escapes
/// are preserved here so resource-pack behavior does not drift.
pub fn parse_java_properties(bytes: &[u8]) -> HashMap<String, String> {
    let physical = latin1(bytes);
    let logical_lines = join_logical_lines(&physical);
    let mut result = HashMap::new();

    for line in logical_lines {
        let chars: Vec<char> = line.chars().collect();
        let mut cursor = 0usize;
        while cursor < chars.len() && is_properties_whitespace(chars[cursor]) {
            cursor += 1;
        }
        if cursor >= chars.len() || matches!(chars[cursor], '#' | '!') {
            continue;
        }

        let key_start = cursor;
        let mut escaped = false;
        let mut separator = chars.len();
        while cursor < chars.len() {
            let ch = chars[cursor];
            if !escaped && (ch == '=' || ch == ':' || is_properties_whitespace(ch)) {
                separator = cursor;
                break;
            }
            if ch == '\\' {
                escaped = !escaped;
            } else {
                escaped = false;
            }
            cursor += 1;
        }

        let key_raw: String = chars[key_start..separator].iter().collect();
        cursor = separator;
        while cursor < chars.len() && is_properties_whitespace(chars[cursor]) {
            cursor += 1;
        }
        if cursor < chars.len() && matches!(chars[cursor], '=' | ':') {
            cursor += 1;
        }
        while cursor < chars.len() && is_properties_whitespace(chars[cursor]) {
            cursor += 1;
        }
        let value_raw: String = chars[cursor..].iter().collect();
        result.insert(unescape_property(&key_raw), unescape_property(&value_raw));
    }

    result
}

fn latin1(bytes: &[u8]) -> String {
    bytes.iter().map(|&byte| char::from(byte)).collect()
}

fn join_logical_lines(input: &str) -> Vec<String> {
    let normalized = input.replace("\r\n", "\n").replace('\r', "\n");
    let mut result = Vec::new();
    let mut current = String::new();
    let mut continuing = false;

    for physical in normalized.split('\n') {
        let segment = if continuing {
            physical.trim_start_matches(is_properties_whitespace)
        } else {
            physical
        };
        current.push_str(segment);

        if has_odd_trailing_backslashes(&current) {
            current.pop();
            continuing = true;
        } else {
            result.push(std::mem::take(&mut current));
            continuing = false;
        }
    }

    if continuing || !current.is_empty() {
        result.push(current);
    }
    result
}

fn has_odd_trailing_backslashes(value: &str) -> bool {
    value.chars().rev().take_while(|&ch| ch == '\\').count() % 2 == 1
}

fn unescape_property(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    let mut utf16 = Vec::<u16>::with_capacity(value.len());
    let mut index = 0usize;
    while index < chars.len() {
        let ch = chars[index];
        if ch != '\\' {
            let mut buffer = [0u16; 2];
            utf16.extend_from_slice(ch.encode_utf16(&mut buffer));
            index += 1;
            continue;
        }

        index += 1;
        if index >= chars.len() {
            utf16.push('\\' as u16);
            break;
        }
        match chars[index] {
            't' => utf16.push('\t' as u16),
            'n' => utf16.push('\n' as u16),
            'r' => utf16.push('\r' as u16),
            'f' => utf16.push(0x000C),
            'u' => {
                if index + 4 < chars.len() {
                    let digits: String = chars[index + 1..index + 5].iter().collect();
                    if let Ok(code) = u16::from_str_radix(&digits, 16) {
                        // Java appends a UTF-16 code unit. Deferring conversion
                        // until the end correctly combines surrogate pairs.
                        utf16.push(code);
                        index += 4;
                    } else {
                        utf16.push('u' as u16);
                    }
                } else {
                    utf16.push('u' as u16);
                }
            }
            other => {
                let mut buffer = [0u16; 2];
                utf16.extend_from_slice(other.encode_utf16(&mut buffer));
            }
        }
        index += 1;
    }
    String::from_utf16_lossy(&utf16)
}

const fn is_properties_whitespace(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\u{000C}')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_latin1_and_unicode_escapes() {
        let values = parse_java_properties(b"name=caf\xE9\nwide=\\u5BBD\\u5EA6\n");
        assert_eq!(values.get("name").map(String::as_str), Some("caf\u{00E9}"));
        assert_eq!(
            values.get("wide").map(String::as_str),
            Some("\u{5BBD}\u{5EA6}")
        );
        let emoji = parse_java_properties(b"emoji=\\uD83D\\uDE00\n");
        assert_eq!(emoji.get("emoji").map(String::as_str), Some("\u{1F600}"));
    }

    #[test]
    fn preserves_java_separators_and_escaped_spaces() {
        let values = parse_java_properties(b"key\\ with\\ spaces : value\\:part\nplain value\n");
        assert_eq!(
            values.get("key with spaces").map(String::as_str),
            Some("value:part")
        );
        assert_eq!(values.get("plain").map(String::as_str), Some("value"));
    }

    #[test]
    fn joins_continued_lines_and_skips_leading_whitespace() {
        let values = parse_java_properties(b"message=first\\\n   second\\\n\tthird\n");
        assert_eq!(
            values.get("message").map(String::as_str),
            Some("firstsecondthird")
        );
    }

    #[test]
    fn escaped_backslash_does_not_continue_line() {
        let values = parse_java_properties(b"path=C:\\\\\nnext=value\n");
        assert_eq!(values.get("path").map(String::as_str), Some("C:\\"));
        assert_eq!(values.get("next").map(String::as_str), Some("value"));
    }
}
