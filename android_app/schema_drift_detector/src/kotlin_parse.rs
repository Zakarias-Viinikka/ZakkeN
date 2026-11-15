//! Parsers for the Kotlin files the drift detector compares against.
//!
//! Two file shapes:
//!
//!   TableDescriptors.kt — one `object XTableSchema { ... }` per table,
//!   containing `TABLE_NAME`, individual `val ident = ColumnDescriptor(...)`
//!   declarations, and an `allColumns = listOf(idents...)` list that fixes
//!   the on-Kotlin column order.
//!
//!   Version<N>.kt — one `object Version<N> { val tables: List<FrozenTable>
//!   = listOf(FrozenTable(name=..., columns=listOf(ColumnDef...),
//!   foreignKeys=listOf(ForeignKeyDef...)), ...) }`.
//!
//! Both parsers work on the blanked source (see `scan::blank_comments`),
//! so comment lines don't affect line numbers or match counts.

use crate::model::{Column, ForeignKey, Table};
use crate::scan::{
    blank_comments, find_calls, parse_literal, split_named_arg, split_top_level_args, Literal,
    ScanError,
};

#[derive(Debug)]
pub enum ParseError {
    Scan(ScanError),
    Shape(String),
}

impl From<ScanError> for ParseError {
    fn from(e: ScanError) -> Self {
        ParseError::Scan(e)
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Scan(e) => write!(f, "line {}: {}", e.line, e.msg),
            ParseError::Shape(m) => write!(f, "{}", m),
        }
    }
}

// ---------- ColumnDescriptor ----------

/// One `val ident = ColumnDescriptor("name", "TYPE") { (it as Col.Variant).v1 }`.
/// `ident` is the Kotlin identifier on the left. `cast_variant` is the
/// last segment of the `as Col.X` cast (`Integer`, `Text`, `Blob`, `Real`,
/// `Null`), so we can cross-check against `column_type`.
#[derive(Debug, Clone)]
pub struct KotlinDescriptor {
    pub ident: String,
    pub name: String,
    pub column_type: String,
    pub cast_variant: Option<String>,
    pub line: usize,
}

pub struct ParsedDescriptorsFile {
    pub tables: Vec<ParsedDescriptorsTable>,
}

pub struct ParsedDescriptorsTable {
    pub object_name: String,
    pub table_name: String,
    pub descriptors: Vec<KotlinDescriptor>,
    pub all_columns_order: Vec<String>,
    /// First line of the `object XTableSchema {` block, for messages.
    pub line: usize,
}

/// Parse `TableDescriptors.kt`. Object blocks are found by matching
/// `object <Name> {` and brace-counting the body. Any `ColumnDescriptor`
/// call inside is a descriptor; `allColumns = listOf(...)` names the
/// order.
pub fn parse_table_descriptors(src: &str) -> Result<ParsedDescriptorsFile, ParseError> {
    let blanked = blank_comments(src)?;
    let mut tables = Vec::new();
    let mut i = 0usize;
    let bytes = blanked.as_bytes();
    while i + 7 <= bytes.len() {
        // Find `object `.
        if &bytes[i..i + 7] == b"object " {
            let start = i;
            let after = i + 7;
            // Read identifier.
            let ident_end = read_ident_end(&blanked, after);
            if ident_end == after {
                i += 7;
                continue;
            }
            let obj_name = blanked[after..ident_end].to_string();
            // Skip whitespace to `{`.
            let mut j = ident_end;
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t' || bytes[j] == b'\n') {
                j += 1;
            }
            if j >= bytes.len() || bytes[j] != b'{' {
                i = ident_end;
                continue;
            }
            let body_close = match find_matching_brace(&blanked, j) {
                Some(c) => c,
                None => {
                    return Err(ParseError::Shape(format!(
                        "object {} at line {}: unbalanced braces",
                        obj_name,
                        1 + blanked[..start].bytes().filter(|b| *b == b'\n').count()
                    )))
                }
            };
            let body = &blanked[j + 1..body_close];
            let body_line = 1 + blanked[..j].bytes().filter(|b| *b == b'\n').count();

            if let Some(t) = parse_descriptors_object(&obj_name, body, body_line)? {
                tables.push(t);
            }
            i = body_close + 1;
            continue;
        }
        i += 1;
    }
    Ok(ParsedDescriptorsFile { tables })
}

fn parse_descriptors_object(
    obj_name: &str,
    body: &str,
    _body_line: usize,
) -> Result<Option<ParsedDescriptorsTable>, ParseError> {
    // TABLE_NAME = "x"
    let mut table_name: Option<String> = None;
    for segment in body.split(';') {
        let _ = segment;
        break;
    }
    // Simpler: find TABLE_NAME identifier followed by `=` and a literal.
    if let Some(idx) = body.find("TABLE_NAME") {
        let after = idx + "TABLE_NAME".len();
        if let Some(eq) = body[after..].find('=') {
            let lit_start = after + eq + 1;
            let val = body[lit_start..].trim_start();
            match parse_literal(val.split_whitespace().next().unwrap_or("")) {
                Literal::Str(s) => table_name = Some(s),
                _ => {}
            }
        }
    }
    let Some(table_name) = table_name else {
        return Ok(None);
    };

    // ColumnDescriptors in the body.
    let calls = find_calls(body, "ColumnDescriptor");
    if calls.is_empty() {
        return Ok(Some(ParsedDescriptorsTable {
            object_name: obj_name.to_string(),
            table_name,
            descriptors: Vec::new(),
            all_columns_order: Vec::new(),
            line: _body_line,
        }));
    }

    // Each ColumnDescriptor call sits on the right side of a `val ident = ...`.
    let body_bytes = body.as_bytes();
    let mut descriptors = Vec::new();
    for call in &calls {
        let line = 1 + body[..call.start].bytes().filter(|b| *b == b'\n').count();
        let args = split_top_level_args(&call.args);
        if args.len() < 2 {
            return Err(ParseError::Shape(format!(
                "{}: ColumnDescriptor at line {} has fewer than 2 positional args",
                obj_name, line
            )));
        }
        let name = match parse_literal(&args[0]) {
            Literal::Str(s) => s,
            other => {
                return Err(ParseError::Shape(format!(
                    "{}: ColumnDescriptor at line {}: first arg is not a string literal ({:?})",
                    obj_name, line, other
                )))
            }
        };
        let column_type = match parse_literal(&args[1]) {
            Literal::Str(s) => s,
            other => {
                return Err(ParseError::Shape(format!(
                    "{}: ColumnDescriptor at line {}: second arg is not a string literal ({:?})",
                    obj_name, line, other
                )))
            }
        };
        // Find `val <ident>` immediately before the call.
        let mut k = call.start;
        while k > 0 {
            let b = body_bytes[k - 1];
            if b == b' ' || b == b'\t' || b == b'\n' {
                k -= 1;
                continue;
            }
            break;
        }
        if k == 0 || body_bytes[k - 1] != b'=' {
            return Err(ParseError::Shape(format!(
                "{}: ColumnDescriptor at line {} is not preceded by `val ident =`",
                obj_name, line
            )));
        }
        k -= 1;
        // Skip whitespace before `=`.
        while k > 0 && (body_bytes[k - 1] == b' ' || body_bytes[k - 1] == b'\t') {
            k -= 1;
        }
        // Read the identifier backwards.
        let ident_end = k;
        let mut ident_start = ident_end;
        while ident_start > 0 && is_ident_byte(body_bytes[ident_start - 1]) {
            ident_start -= 1;
        }
        if ident_start == ident_end {
            return Err(ParseError::Shape(format!(
                "{}: ColumnDescriptor at line {} has no `val ident`",
                obj_name, line
            )));
        }
        let ident = body[ident_start..ident_end].to_string();

        // Extract the cast variant from `(it as Col.Variant)` after the
        // `)` that closes ColumnDescriptor. Look at the text between the
        // call end and the next `\n` (or a `{` lambda if there is one).
        let after_call = call.end;
        let tail_end = body[after_call..]
            .find(|c: char| c == '\n' || c == ';')
            .map(|n| after_call + n)
            .unwrap_or(body.len());
        let tail = &body[after_call..tail_end];
        let cast_variant = parse_cast_variant(tail);

        descriptors.push(KotlinDescriptor {
            ident,
            name,
            column_type,
            cast_variant,
            line,
        });
    }

    // allColumns = listOf(idents...)
    let all_columns_order = parse_all_columns(body, obj_name)?;

    Ok(Some(ParsedDescriptorsTable {
        object_name: obj_name.to_string(),
        table_name,
        descriptors,
        all_columns_order,
        line: _body_line,
    }))
}

fn parse_cast_variant(tail: &str) -> Option<String> {
    let idx = tail.find("as Col.")?;
    let after = &tail[idx + "as Col.".len()..];
    let end = after
        .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .unwrap_or(after.len());
    if end == 0 {
        None
    } else {
        Some(after[..end].to_string())
    }
}

fn parse_all_columns(body: &str, obj_name: &str) -> Result<Vec<String>, ParseError> {
    let idx = match body.find("allColumns") {
        Some(i) => i,
        None => return Ok(Vec::new()),
    };
    let after_ident = idx + "allColumns".len();
    // Skip whitespace to `=`.
    let mut k = after_ident;
    let bytes = body.as_bytes();
    while k < bytes.len() && (bytes[k] == b' ' || bytes[k] == b'\t') {
        k += 1;
    }
    if k >= bytes.len() || bytes[k] != b'=' {
        return Err(ParseError::Shape(format!(
            "{}: allColumns is not followed by `=`",
            obj_name
        )));
    }
    k += 1;
    // Find `listOf(` and take its args.
    let rest = &body[k..];
    let listof_idx = match rest.find("listOf") {
        Some(i) => i,
        None => {
            return Err(ParseError::Shape(format!(
                "{}: allColumns is not a listOf(...)",
                obj_name
            )))
        }
    };
    let open = k + listof_idx + "listOf".len();
    // Find `(`.
    let mut p = open;
    while p < bytes.len() && (bytes[p] == b' ' || bytes[p] == b'\t' || bytes[p] == b'\n') {
        p += 1;
    }
    if p >= bytes.len() || bytes[p] != b'(' {
        return Err(ParseError::Shape(format!(
            "{}: allColumns listOf has no `(`",
            obj_name
        )));
    }
    let close = crate::scan::find_matching_paren(body, p).ok_or_else(|| {
        ParseError::Shape(format!("{}: allColumns listOf has unbalanced parens", obj_name))
    })?;
    let args = &body[p + 1..close];
    let idents: Vec<String> = split_top_level_args(args)
        .into_iter()
        .map(|a| a.trim().to_string())
        .filter(|a| !a.is_empty())
        .collect();
    Ok(idents)
}

// ---------- Version<N>.kt (FrozenTable blocks) ----------

/// Parse `Version<N>.kt`. Returns tables in the order they appear in the
/// `listOf(...)`. Each FrozenTable has a name, columns, and foreign keys.
pub fn parse_version_file(src: &str) -> Result<Vec<Table>, ParseError> {
    let blanked = blank_comments(src)?;
    let frozen_calls = find_calls(&blanked, "FrozenTable");
    let mut tables = Vec::new();
    for call in frozen_calls {
        let line = call.line;
        let args = split_top_level_args(&call.args);
        let mut name: Option<String> = None;
        let mut columns_text: Option<String> = None;
        let mut fks_text: Option<String> = None;
        for a in &args {
            match split_named_arg(a) {
                Some((k, v)) => match k.as_str() {
                    "name" => {
                        name = match parse_literal(&v) {
                            Literal::Str(s) => Some(s),
                            other => {
                                return Err(ParseError::Shape(format!(
                                    "FrozenTable at line {}: name is not a string literal ({:?})",
                                    line, other
                                )))
                            }
                        };
                    }
                    "columns" => columns_text = Some(v),
                    "foreignKeys" => fks_text = Some(v),
                    other => {
                        return Err(ParseError::Shape(format!(
                            "FrozenTable at line {}: unknown key `{}`",
                            line, other
                        )))
                    }
                },
                None => {
                    return Err(ParseError::Shape(format!(
                        "FrozenTable at line {}: positional argument (all should be named)",
                        line
                    )))
                }
            }
        }
        let name = name.ok_or_else(|| {
            ParseError::Shape(format!("FrozenTable at line {}: missing name", line))
        })?;
        let columns_text = columns_text.ok_or_else(|| {
            ParseError::Shape(format!("FrozenTable at line {}: missing columns", line))
        })?;
        let columns = parse_column_def_list(&columns_text, line)?;
        let foreign_keys = match fks_text {
            Some(t) => parse_foreign_key_def_list(&t, line)?,
            None => Vec::new(),
        };
        tables.push(Table {
            name,
            columns,
            foreign_keys,
        });
    }
    Ok(tables)
}

fn parse_column_def_list(text: &str, ctx_line: usize) -> Result<Vec<Column>, ParseError> {
    // Expect `listOf(...)`.
    let inner = extract_list_of(text).ok_or_else(|| {
        ParseError::Shape(format!(
            "FrozenTable at line {}: columns is not `listOf(...)`",
            ctx_line
        ))
    })?;
    let calls = find_calls(&inner, "ColumnDef");
    let mut out = Vec::new();
    for call in &calls {
        let args = split_top_level_args(&call.args);
        if args.len() < 2 {
            return Err(ParseError::Shape(format!(
                "ColumnDef at line {}: fewer than 2 positional args",
                ctx_line + call.line - 1
            )));
        }
        let name = match parse_literal(&args[0]) {
            Literal::Str(s) => s,
            _ => {
                return Err(ParseError::Shape(format!(
                    "ColumnDef at line {}: first arg is not a string",
                    ctx_line + call.line - 1
                )))
            }
        };
        let column_type = match parse_literal(&args[1]) {
            Literal::Str(s) => s,
            _ => {
                return Err(ParseError::Shape(format!(
                    "ColumnDef at line {}: second arg is not a string",
                    ctx_line + call.line - 1
                )))
            }
        };
        let mut primary_key = false;
        let mut not_null = false;
        let mut unique = false;
        let mut default_value = String::new();
        let mut autoincrement = false;
        for a in args.iter().skip(2) {
            let (k, v) = split_named_arg(a).ok_or_else(|| {
                ParseError::Shape(format!(
                    "ColumnDef at line {}: positional argument after type",
                    ctx_line + call.line - 1
                ))
            })?;
            match k.as_str() {
                "primaryKey" => primary_key = expect_bool(&v, ctx_line + call.line - 1, "primaryKey")?,
                "notNull" => not_null = expect_bool(&v, ctx_line + call.line - 1, "notNull")?,
                "unique" => unique = expect_bool(&v, ctx_line + call.line - 1, "unique")?,
                "defaultValue" => match parse_literal(&v) {
                    Literal::Str(s) => default_value = s,
                    _ => {
                        return Err(ParseError::Shape(format!(
                            "ColumnDef at line {}: defaultValue is not a string",
                            ctx_line + call.line - 1
                        )))
                    }
                },
                "autoincrement" => autoincrement = expect_bool(&v, ctx_line + call.line - 1, "autoincrement")?,
                other => {
                    return Err(ParseError::Shape(format!(
                        "ColumnDef at line {}: unknown key `{}`",
                        ctx_line + call.line - 1,
                        other
                    )))
                }
            }
        }
        out.push(Column {
            name,
            column_type,
            primary_key,
            not_null,
            unique,
            default_value,
            autoincrement,
        });
    }
    Ok(out)
}

fn parse_foreign_key_def_list(text: &str, ctx_line: usize) -> Result<Vec<ForeignKey>, ParseError> {
    let inner = extract_list_of(text).ok_or_else(|| {
        ParseError::Shape(format!(
            "FrozenTable at line {}: foreignKeys is not `listOf(...)`",
            ctx_line
        ))
    })?;
    let calls = find_calls(&inner, "ForeignKeyDef");
    let mut out = Vec::new();
    for call in &calls {
        let args = split_top_level_args(&call.args);
        if args.len() != 3 {
            return Err(ParseError::Shape(format!(
                "ForeignKeyDef at line {}: expected 3 positional args, got {}",
                ctx_line + call.line - 1,
                args.len()
            )));
        }
        let column = match parse_literal(&args[0]) {
            Literal::Str(s) => s,
            _ => {
                return Err(ParseError::Shape(format!(
                    "ForeignKeyDef at line {}: arg 1 is not a string",
                    ctx_line + call.line - 1
                )))
            }
        };
        let ref_table = match parse_literal(&args[1]) {
            Literal::Str(s) => s,
            _ => {
                return Err(ParseError::Shape(format!(
                    "ForeignKeyDef at line {}: arg 2 is not a string",
                    ctx_line + call.line - 1
                )))
            }
        };
        let ref_column = match parse_literal(&args[2]) {
            Literal::Str(s) => s,
            _ => {
                return Err(ParseError::Shape(format!(
                    "ForeignKeyDef at line {}: arg 3 is not a string",
                    ctx_line + call.line - 1
                )))
            }
        };
        out.push(ForeignKey {
            column,
            ref_table,
            ref_column,
        });
    }
    Ok(out)
}

fn extract_list_of(text: &str) -> Option<String> {
    let idx = text.find("listOf")?;
    let after = idx + "listOf".len();
    let bytes = text.as_bytes();
    let mut p = after;
    while p < bytes.len() && (bytes[p] == b' ' || bytes[p] == b'\t' || bytes[p] == b'\n') {
        p += 1;
    }
    if p >= bytes.len() || bytes[p] != b'(' {
        return None;
    }
    let close = crate::scan::find_matching_paren(text, p)?;
    Some(text[p + 1..close].to_string())
}

fn expect_bool(raw: &str, line: usize, key: &str) -> Result<bool, ParseError> {
    match parse_literal(raw) {
        Literal::Bool(b) => Ok(b),
        other => Err(ParseError::Shape(format!(
            "line {}: {} is not a boolean literal ({:?})",
            line, key, other
        ))),
    }
}

fn read_ident_end(src: &str, start: usize) -> usize {
    let bytes = src.as_bytes();
    let mut i = start;
    while i < bytes.len() && is_ident_byte(bytes[i]) {
        i += 1;
    }
    i
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
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

    const DESCRIPTORS_SAMPLE: &str = r#"
package z.zndroid.db.schema

import uniffi.protocol.Col

object PagesTableSchema {
    const val TABLE_NAME = "pages"
    val id = ColumnDescriptor("id", "INTEGER") { (it as Col.Integer).v1 }
    val pageId = ColumnDescriptor("page_id", "TEXT") { (it as Col.Text).v1 }
    val allColumns = listOf(id, pageId)
}
"#;

    #[test]
    fn parse_descriptors_simple() {
        let f = parse_table_descriptors(DESCRIPTORS_SAMPLE).unwrap();
        assert_eq!(f.tables.len(), 1);
        let t = &f.tables[0];
        assert_eq!(t.table_name, "pages");
        assert_eq!(t.descriptors.len(), 2);
        assert_eq!(t.descriptors[0].name, "id");
        assert_eq!(t.descriptors[0].column_type, "INTEGER");
        assert_eq!(t.descriptors[0].cast_variant.as_deref(), Some("Integer"));
        assert_eq!(t.all_columns_order, vec!["id", "pageId"]);
    }

    const VERSION_SAMPLE: &str = r#"
package z.zndroid.db.migrations.schemas

import uniffi.protocol.ColumnDef
import uniffi.protocol.ForeignKeyDef

object Version9 {
    val tables: List<FrozenTable> = listOf(
        FrozenTable(
            name = "pages",
            columns = listOf(
                ColumnDef("id", "INTEGER", primaryKey = true, notNull = true, unique = false, defaultValue = "", autoincrement = true),
                ColumnDef("page_id", "TEXT", primaryKey = false, notNull = true, unique = true, defaultValue = "", autoincrement = false)
            )
        ),
        FrozenTable(
            name = "backlinks",
            columns = listOf(
                ColumnDef("id", "INTEGER", primaryKey = true, notNull = true, unique = false, defaultValue = "", autoincrement = true),
                ColumnDef("page_that_holds_link_id", "TEXT", primaryKey = false, notNull = true, unique = false, defaultValue = "", autoincrement = false)
            ),
            foreignKeys = listOf(
                ForeignKeyDef("page_that_holds_link_id", "pages", "page_id")
            )
        )
    )
}
"#;

    #[test]
    fn parse_version_file_simple() {
        let tables = parse_version_file(VERSION_SAMPLE).unwrap();
        assert_eq!(tables.len(), 2);
        assert_eq!(tables[0].name, "pages");
        assert_eq!(tables[0].columns.len(), 2);
        assert_eq!(tables[0].columns[0].name, "id");
        assert!(tables[0].columns[0].primary_key);
        assert!(tables[0].columns[0].autoincrement);
        assert_eq!(tables[1].name, "backlinks");
        assert_eq!(tables[1].foreign_keys.len(), 1);
        assert_eq!(tables[1].foreign_keys[0].column, "page_that_holds_link_id");
    }
}
