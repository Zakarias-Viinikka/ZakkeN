//! String-aware Kotlin source scanning. The rules:
//!
//!   - `//` and `/* */` comments are blanked out before anything else
//!     looks at the text. Blanked, not removed: every character that was
//!     in the comment is replaced by a space, and every newline is kept,
//!     so line numbers and byte offsets line up with the original.
//!   - `//` and `/*` inside string literals are not comments.
//!   - `"`, `'`, `\"`, `\\` inside strings/comments are tracked so nothing
//!     trips on a quote.
//!
//! Everything else (finding calls, splitting args) works on the blanked
//! text. Line numbers reported in messages are therefore the original.

#[derive(Debug, Clone)]
pub struct ScanError {
    pub msg: String,
    pub line: usize,
}

pub fn blank_comments(src: &str) -> Result<String, ScanError> {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    let mut line = 1usize;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\n' {
            out.push('\n');
            line += 1;
            i += 1;
            continue;
        }
        // Line comment.
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' {
                out.push(' ');
                i += 1;
            }
            continue;
        }
        // Block comment. Kotlin block comments nest; handle one level of nesting
        // which covers everything this repo has.
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            let start_line = line;
            out.push(' ');
            out.push(' ');
            i += 2;
            let mut depth = 1;
            while i < bytes.len() && depth > 0 {
                let c = bytes[i];
                if c == b'\n' {
                    out.push('\n');
                    line += 1;
                    i += 1;
                    continue;
                }
                if c == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                    depth += 1;
                    out.push(' ');
                    out.push(' ');
                    i += 2;
                    continue;
                }
                if c == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    depth -= 1;
                    out.push(' ');
                    out.push(' ');
                    i += 2;
                    continue;
                }
                out.push(' ');
                i += 1;
            }
            if depth != 0 {
                return Err(ScanError {
                    msg: "unterminated block comment".to_string(),
                    line: start_line,
                });
            }
            continue;
        }
        // String literal (regular and triple-quoted).
        if b == b'"' {
            if i + 2 < bytes.len() && bytes[i + 1] == b'"' && bytes[i + 2] == b'"' {
                // Triple-quoted: copy until next """.
                out.push_str("\"\"\"");
                i += 3;
                let mut closed = false;
                while i < bytes.len() {
                    let c = bytes[i];
                    if c == b'"'
                        && i + 2 < bytes.len()
                        && bytes[i + 1] == b'"'
                        && bytes[i + 2] == b'"'
                    {
                        out.push_str("\"\"\"");
                        i += 3;
                        closed = true;
                        break;
                    }
                    if c == b'\n' {
                        out.push('\n');
                        line += 1;
                    } else {
                        out.push(c as char);
                    }
                    i += 1;
                }
                if !closed {
                    return Err(ScanError {
                        msg: "unterminated triple-quoted string".to_string(),
                        line,
                    });
                }
                continue;
            }
            // Regular string.
            out.push('"');
            i += 1;
            while i < bytes.len() {
                let c = bytes[i];
                if c == b'\\' && i + 1 < bytes.len() {
                    out.push('\\');
                    out.push(bytes[i + 1] as char);
                    i += 2;
                    continue;
                }
                if c == b'"' {
                    out.push('"');
                    i += 1;
                    break;
                }
                if c == b'\n' {
                    return Err(ScanError {
                        msg: "unterminated string literal".to_string(),
                        line,
                    });
                }
                out.push(c as char);
                i += 1;
            }
            continue;
        }
        out.push(b as char);
        i += 1;
    }
    Ok(out)
}

/// A call site: `Name(args)`. `args` is the text between the outer
/// parens. `line` is 1-based, from the blanked source.
#[derive(Debug, Clone)]
pub struct CallSite {
    pub name: String,
    pub args: String,
    pub line: usize,
    pub start: usize,
    pub end: usize,
}

/// Find every occurrence of `Name(...)` in `blanked`. Skips `Name` when
/// the character before it is an identifier char (so `newTablePages` does
/// not match `TablePages`).
pub fn find_calls(blanked: &str, name: &str) -> Vec<CallSite> {
    let bytes = blanked.as_bytes();
    let mut result = Vec::new();
    let name_bytes = name.as_bytes();
    let mut i = 0usize;
    while i + name_bytes.len() <= bytes.len() {
        if &bytes[i..i + name_bytes.len()] != name_bytes {
            i += 1;
            continue;
        }
        // Reject if preceded by an identifier char.
        if i > 0 && is_ident_char(bytes[i - 1]) {
            i += name_bytes.len();
            continue;
        }
        // Reject if followed by an identifier char (so `Foo` doesn't
        // match `FooBar`).
        let after = i + name_bytes.len();
        if after < bytes.len() && is_ident_char(bytes[after]) {
            i += name_bytes.len();
            continue;
        }
        // Skip whitespace.
        let mut j = after;
        while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
            j += 1;
        }
        if j >= bytes.len() || bytes[j] != b'(' {
            i += name_bytes.len();
            continue;
        }
        // Find the matching close paren.
        let args_start = j + 1;
        if let Some(args_end) = find_matching_paren(blanked, j) {
            let line = 1 + blanked[..i].bytes().filter(|b| *b == b'\n').count();
            result.push(CallSite {
                name: name.to_string(),
                args: blanked[args_start..args_end].to_string(),
                line,
                start: i,
                end: args_end + 1,
            });
            i = args_end + 1;
        } else {
            i += name_bytes.len();
        }
    }
    result
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// `open` is the index of the opening `(`. Returns the index of the
/// matching `)`, or None on unbalanced input.
pub fn find_matching_paren(blanked: &str, open: usize) -> Option<usize> {
    let bytes = blanked.as_bytes();
    if bytes[open] != b'(' {
        return None;
    }
    let mut depth = 0i32;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => {
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

/// Split `args` at top-level commas. String literals and nested parens
/// protect their commas. Empty arg lists produce an empty Vec, not `[""]`.
pub fn split_top_level_args(args: &str) -> Vec<String> {
    let mut result = Vec::new();
    if args.trim().is_empty() {
        return result;
    }
    let bytes = args.as_bytes();
    let mut depth = 0i32;
    let mut start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        match b {
            b'"' => {
                // Skip to the closing quote, honoring escapes.
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == b'\\' && i + 1 < bytes.len() {
                        i += 2;
                        continue;
                    }
                    if bytes[i] == b'"' {
                        break;
                    }
                    i += 1;
                }
            }
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b',' if depth == 0 => {
                result.push(args[start..i].to_string());
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    let last = args[start..].trim();
    if !last.is_empty() {
        result.push(args[start..].to_string());
    }
    result
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Str(String),
    Bool(bool),
    Int(i64),
    Unrecognized(String),
}

pub fn parse_literal(raw: &str) -> Literal {
    let t = raw.trim();
    if t.len() >= 2 && t.starts_with('"') && t.ends_with('"') {
        let inner = &t[1..t.len() - 1];
        let mut out = String::with_capacity(inner.len());
        let mut chars = inner.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some('r') => out.push('\r'),
                    Some('"') => out.push('"'),
                    Some('\\') => out.push('\\'),
                    Some(other) => out.push(other),
                    None => out.push('\\'),
                }
            } else {
                out.push(c);
            }
        }
        return Literal::Str(out);
    }
    if t == "true" {
        return Literal::Bool(true);
    }
    if t == "false" {
        return Literal::Bool(false);
    }
    if let Ok(n) = t.parse::<i64>() {
        return Literal::Int(n);
    }
    Literal::Unrecognized(t.to_string())
}

/// Split `key = value` at the top-level `=`. Returns (key, value) where
/// value is the right-hand side trimmed. Returns None for a bare
/// positional argument (no top-level `=`).
pub fn split_named_arg(arg: &str) -> Option<(String, String)> {
    let bytes = arg.as_bytes();
    let mut depth = 0i32;
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        match b {
            b'"' => {
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == b'\\' && i + 1 < bytes.len() {
                        i += 2;
                        continue;
                    }
                    if bytes[i] == b'"' {
                        break;
                    }
                    i += 1;
                }
            }
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b'=' if depth == 0 => {
                // Skip `==`, `<=`, `>=`, `!=` — check both neighbours,
                // since either side of a two-char operator can be the
                // first `=` we see.
                let prev = if i > 0 { bytes[i - 1] } else { 0 };
                let next = if i + 1 < bytes.len() { bytes[i + 1] } else { 0 };
                if prev == b'=' || prev == b'<' || prev == b'>' || prev == b'!' || next == b'=' {
                    i += 1;
                    continue;
                }
                let key = arg[..i].trim().to_string();
                let value = arg[i + 1..].trim().to_string();
                return Some((key, value));
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

    #[test]
    fn comments_blanked_strings_preserved() {
        let src = "// a line comment\nval a = \"not // a comment\"\n/* block */ val b = 1\n";
        let b = blank_comments(src).unwrap();
        assert_eq!(b.lines().count(), src.lines().count());
        assert!(b.contains("\"not // a comment\""));
        assert!(!b.contains("line comment"));
        assert!(!b.contains("block"));
        assert!(b.contains("val b = 1"));
    }

    #[test]
    fn triple_quoted_string_preserved() {
        let src = "val s = \"\"\"multi\nline\"\"\"\nval x = 1";
        let b = blank_comments(src).unwrap();
        assert!(b.contains("multi"));
        assert!(b.contains("line"));
        assert!(b.contains("val x = 1"));
    }

    #[test]
    fn unterminated_block_comment_errors() {
        assert!(blank_comments("/* nope").is_err());
    }

    #[test]
    fn find_calls_respects_identifier_boundary() {
        let src = "val a = ColumnDef(\"x\")\nval b = MyColumnDef(\"y\")\n";
        let b = blank_comments(src).unwrap();
        let hits = find_calls(&b, "ColumnDef");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].line, 1);
    }

    #[test]
    fn find_calls_nested_parens() {
        let src = "val a = ColumnDef(\"x\", \"TEXT\", f(1, 2), true)";
        let b = blank_comments(src).unwrap();
        let hits = find_calls(&b, "ColumnDef");
        assert_eq!(hits.len(), 1);
        let args = split_top_level_args(&hits[0].args);
        assert_eq!(args.len(), 4);
        assert_eq!(args[2].trim(), "f(1, 2)");
    }

    #[test]
    fn split_args_honors_commas_in_strings() {
        let args = split_top_level_args("\"a, b\", true, 3");
        assert_eq!(args.len(), 3);
        assert_eq!(parse_literal(&args[0]), Literal::Str("a, b".to_string()));
        assert_eq!(parse_literal(&args[1]), Literal::Bool(true));
        assert_eq!(parse_literal(&args[2]), Literal::Int(3));
    }

    #[test]
    fn split_named_arg_finds_top_level_equals() {
        let n = split_named_arg("primaryKey = true");
        assert_eq!(n, Some(("primaryKey".to_string(), "true".to_string())));
        let positional = split_named_arg("\"x\"");
        assert_eq!(positional, None);
        let cmp = split_named_arg("a == b");
        assert_eq!(cmp, None);
    }
}
