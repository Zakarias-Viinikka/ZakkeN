//! Parse SchemaVersion.kt to learn which snapshot LATEST points at, and
//! verify the version plumbing (check C from the plan).

use crate::scan::{blank_comments, ScanError};

#[derive(Debug)]
pub enum VersionParseError {
    Scan(ScanError),
    Shape(String),
}

impl From<ScanError> for VersionParseError {
    fn from(e: ScanError) -> Self {
        VersionParseError::Scan(e)
    }
}

impl std::fmt::Display for VersionParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionParseError::Scan(e) => write!(f, "line {}: {}", e.line, e.msg),
            VersionParseError::Shape(m) => write!(f, "{}", m),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LatestInfo {
    /// The V<N> enum entry, e.g. `V3`.
    pub variant: String,
    /// The integer in the parens, e.g. 3.
    pub version: u32,
    /// Whether the body of the `V<N> { ... }` block contains the
    /// literal text `Version<N>.tables`. Required to prove bootstrap
    /// uses the frozen snapshot rather than its own inline column lists.
    pub body_uses_snapshot_tables: bool,
}

pub fn parse_latest(src: &str) -> Result<LatestInfo, VersionParseError> {
    let blanked = blank_comments(src)?;
    // Find the declaration `val LATEST =` (or `var LATEST =`). Narrower
    // than just "LATEST" so uses like `LATEST.version` don't match first.
    let (idx, after) = if let Some(i) = blanked.find("val LATEST") {
        (i, i + "val LATEST".len())
    } else if let Some(i) = blanked.find("var LATEST") {
        (i, i + "var LATEST".len())
    } else {
        return Err(VersionParseError::Shape(
            "SchemaVersion.kt: no `val LATEST = ...` declaration".to_string(),
        ));
    };
    let _ = idx;
    let rest = &blanked[after..];
    let eq = rest.find('=').ok_or_else(|| {
        VersionParseError::Shape("SchemaVersion.kt: `LATEST` not followed by `=`".to_string())
    })?;
    let rhs_start = after + eq + 1;
    let rhs = blanked[rhs_start..].trim_start();
    // Read `V` then digits.
    let bytes = rhs.as_bytes();
    if bytes.first() != Some(&b'V') {
        return Err(VersionParseError::Shape(format!(
            "SchemaVersion.kt: LATEST points at `{}`, expected `V<N>`",
            rhs.split_whitespace().next().unwrap_or("")
        )));
    }
    let mut i = 1usize;
    let digits_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == digits_start {
        return Err(VersionParseError::Shape(
            "SchemaVersion.kt: LATEST = V followed by no digits".to_string(),
        ));
    }
    let version: u32 = rhs[digits_start..i]
        .parse()
        .map_err(|_| VersionParseError::Shape("LATEST version number overflow".to_string()))?;
    let variant = format!("V{}", version);

    // Find the enum entry `V<N>(<N>) {` (or `V<N>(<N>)` followed by
    // whitespace then `{`). Brace-match the body and check for
    // `Version<N>.tables`.
    let needle = format!("{}({})", variant, version);
    let body_uses_snapshot_tables = if let Some(pos) = blanked.find(&needle) {
        // Skip whitespace to `{`.
        let mut k = pos + needle.len();
        let b = blanked.as_bytes();
        while k < b.len() && b[k].is_ascii_whitespace() {
            k += 1;
        }
        if k < b.len() && b[k] == b'{' {
            let close = crate::scan::find_matching_paren(&blanked, k);
            // NB: find_matching_paren is paren-based, not brace-based.
            // Write a local brace-match.
            let _ = close;
            if let Some(end) = find_matching_brace(&blanked, k) {
                let body = &blanked[k + 1..end];
                let expected = format!("Version{}.tables", version);
                body.contains(&expected)
            } else {
                return Err(VersionParseError::Shape(format!(
                    "SchemaVersion.kt: `{}` entry has unbalanced braces",
                    variant
                )));
            }
        } else {
            return Err(VersionParseError::Shape(format!(
                "SchemaVersion.kt: `{}` entry has no `{{`",
                variant
            )));
        }
    } else {
        return Err(VersionParseError::Shape(format!(
            "SchemaVersion.kt: LATEST = {} but no `{}({}) {{` entry",
            variant, variant, version
        )));
    };

    Ok(LatestInfo {
        variant,
        version,
        body_uses_snapshot_tables,
    })
}

fn find_matching_brace(src: &str, open: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    if bytes[open] != b'{' {
        return None;
    }
    let mut depth = 0i32;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
enum class SchemaVersion(val version: Int) {
    V2(2) { override fun bootstrap(db: LiveForever) { V1.bootstrap(db) }; override fun updateOnce(db: LiveForever): SchemaVersion = V3 },
    V3(3) {
        override fun bootstrap(db: LiveForever) { createSnapshotTables(db, Version3.tables) }
        override fun updateOnce(db: LiveForever): SchemaVersion = this
    };
    companion object { val LATEST = V3 }
}
"#;

    #[test]
    fn parse_latest_finds_v3() {
        let info = parse_latest(SAMPLE).unwrap();
        assert_eq!(info.version, 3);
        assert_eq!(info.variant, "V3");
        assert!(info.body_uses_snapshot_tables);
    }

    #[test]
    fn parse_latest_missing_errors() {
        assert!(parse_latest("enum class SchemaVersion {}").is_err());
    }

    #[test]
    fn parse_latest_body_not_using_snapshot() {
        let bad = r#"
enum class SchemaVersion(val version: Int) {
    V3(3) {
        override fun bootstrap(db: LiveForever) { /* inline columns */ }
        override fun updateOnce(db: LiveForever): SchemaVersion = this
    };
    companion object { val LATEST = V3 }
}
"#;
        let info = parse_latest(bad).unwrap();
        assert!(!info.body_uses_snapshot_tables);
    }
}
