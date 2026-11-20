use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedLink {
    pub title: String,
    pub page_id: String,
    pub start_byte: usize,
    pub end_byte: usize,
}

/// Pure Rust wikilink parser matching JS regex `/\[\[([^\]\r\n]+)\|([^\s\]|]+)\]\]/g`.
/// Kept dependency-free so both native tests, backlink callers, and wasm share one implementation.
pub fn parse_links(raw: &str) -> Vec<ParsedLink> {
    let mut out = Vec::new();
    let bytes = raw.as_bytes();
    let mut i = 0;

    while i + 1 < bytes.len() {
        if bytes[i] == b'[' && bytes[i + 1] == b'[' {
            let content_start = i + 2;
            if let Some((close_idx, inner)) = scan_until_double_close(raw, content_start) {
                if let Some((title, page_id)) = split_title_and_id(inner) {
                    out.push(ParsedLink {
                        title: title.to_string(),
                        page_id: page_id.to_string(),
                        start_byte: i,
                        end_byte: close_idx + 2,
                    });
                    i = close_idx + 2;
                    continue;
                }
            }
            i += 2;
        } else {
            // Advance by full UTF-8 char so we never land mid-codepoint.
            let ch_len = raw[i..].chars().next().map_or(1, char::len_utf8);
            i += ch_len;
        }
    }

    out
}

fn scan_until_double_close(raw: &str, start: usize) -> Option<(usize, &str)> {
    let bytes = raw.as_bytes();
    let mut j = start;
    while j < bytes.len() {
        match bytes[j] {
            b'\n' | b'\r' => return None,
            b']' => {
                if j + 1 < bytes.len() && bytes[j + 1] == b']' {
                    return Some((j, &raw[start..j]));
                }
                // A single `]` is forbidden in both title and id.
                return None;
            }
            _ => {
                let ch_len = raw[j..].chars().next().map_or(1, char::len_utf8);
                j += ch_len;
            }
        }
    }
    None
}

fn split_title_and_id(inner: &str) -> Option<(&str, &str)> {
    let bar = inner.rfind('|')?;
    let title = &inner[..bar];
    let page_id = &inner[bar + 1..];
    if title.is_empty() || page_id.is_empty() {
        return None;
    }
    if page_id.chars().any(char::is_whitespace) {
        return None;
    }
    Some((title, page_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct LinkVector {
        name: String,
        input: String,
        expected: Vec<(String, String)>,
    }

    #[test]
    fn shared_link_vectors_pass() {
        let json_str = include_str!("../test_vectors/links.json");
        let vectors: Vec<LinkVector> = serde_json::from_str(json_str).expect("valid vectors json");
        for v in vectors {
            let actual: Vec<(String, String)> = parse_links(&v.input)
                .into_iter()
                .map(|l| (l.title, l.page_id))
                .collect();
            assert_eq!(actual, v.expected, "vector failed: {}", v.name);
        }
    }
}
