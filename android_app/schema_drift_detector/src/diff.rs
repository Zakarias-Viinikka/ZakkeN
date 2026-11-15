//! Diff two `Vec<Table>`s (or their column/FK lists) into human-readable
//! drift lines. Every function returns lines that name the table and
//! column, plus what changed. Nothing here panics; callers print the
//! lines and count them.

use crate::model::{Column, ForeignKey, Table};

/// Compare `frozen` against `live`. Every difference is one line.
/// `side_frozen` and `side_live` name the sides in every line, e.g.
/// "frozen V3" and "live Rust".
pub fn diff_tables(
    frozen: &[Table],
    live: &[Table],
    side_frozen: &str,
    side_live: &str,
) -> Vec<String> {
    let mut lines = Vec::new();
    let frozen_by_name: std::collections::HashMap<&str, &Table> =
        frozen.iter().map(|t| (t.name.as_str(), t)).collect();
    let live_by_name: std::collections::HashMap<&str, &Table> =
        live.iter().map(|t| (t.name.as_str(), t)).collect();

    for t in frozen {
        if !live_by_name.contains_key(t.name.as_str()) {
            lines.push(format!(
                "[TABLE_MISSING_IN_RUST] table '{}': present in {} but absent in {}",
                t.name, side_frozen, side_live
            ));
        }
    }
    for t in live {
        if !frozen_by_name.contains_key(t.name.as_str()) {
            lines.push(format!(
                "[TABLE_ONLY_IN_RUST] table '{}': present in {} but absent in {}",
                t.name, side_live, side_frozen
            ));
        }
    }

    for ft in frozen {
        let Some(lt) = live_by_name.get(ft.name.as_str()) else {
            continue;
        };
        diff_columns(&ft.name, &ft.columns, &lt.columns, side_frozen, side_live, &mut lines);
        diff_fks(
            &ft.name,
            &ft.foreign_keys,
            &lt.foreign_keys,
            side_frozen,
            side_live,
            &mut lines,
        );
    }

    lines
}

fn diff_columns(
    table: &str,
    frozen: &[Column],
    live: &[Column],
    side_frozen: &str,
    side_live: &str,
    out: &mut Vec<String>,
) {
    let frozen_by_name: std::collections::HashMap<&str, &Column> =
        frozen.iter().map(|c| (c.name.as_str(), c)).collect();
    let live_by_name: std::collections::HashMap<&str, &Column> =
        live.iter().map(|c| (c.name.as_str(), c)).collect();

    let mut removed: Vec<(usize, &Column)> = Vec::new();
    for (i, c) in frozen.iter().enumerate() {
        if !live_by_name.contains_key(c.name.as_str()) {
            removed.push((i, c));
        }
    }
    let mut added: Vec<(usize, &Column)> = Vec::new();
    for (i, c) in live.iter().enumerate() {
        if !frozen_by_name.contains_key(c.name.as_str()) {
            added.push((i, c));
        }
    }

    // Rename heuristic: a removal and an addition at the same index with
    // the same type. Only pair each removed with one added.
    let mut used_added = vec![false; added.len()];
    for (rem_idx, rem_col) in &removed {
        let mut hint_added: Option<usize> = None;
        for (ai, (add_idx, add_col)) in added.iter().enumerate() {
            if used_added[ai] {
                continue;
            }
            if *add_idx == *rem_idx && add_col.column_type == rem_col.column_type {
                hint_added = Some(ai);
                break;
            }
        }
        match hint_added {
            Some(ai) => {
                used_added[ai] = true;
                let (add_idx, add_col) = added[ai];
                out.push(format!(
                    "[COLUMN_RENAMED?] {}.{}: removed at index {} in {}; \
                     {} has {} at index {} with same type '{}' — looks like a rename {} -> {}",
                    table, rem_col.name, rem_idx, side_live,
                    side_frozen, add_col.name, add_idx, add_col.column_type,
                    rem_col.name, add_col.name
                ));
            }
            None => {
                out.push(format!(
                    "[COLUMN_REMOVED_FROM_RUST] {}.{}: at index {} in {}, missing in {}",
                    table, rem_col.name, rem_idx, side_frozen, side_live
                ));
            }
        }
    }
    for (ai, (add_idx, add_col)) in added.iter().enumerate() {
        if used_added[ai] {
            continue;
        }
        out.push(format!(
            "[COLUMN_ADDED_IN_RUST] {}.{}: added at index {} in {}, not in {}",
            table, add_col.name, add_idx, side_live, side_frozen
        ));
    }

    // Per-column field drift for columns present on both sides.
    for fc in frozen {
        let Some(lc) = live_by_name.get(fc.name.as_str()) else {
            continue;
        };
        if fc.column_type != lc.column_type {
            out.push(format!(
                "[TYPE_CHANGED] {}.{}: {}='{}' {}='{}'",
                table, fc.name, side_frozen, fc.column_type, side_live, lc.column_type
            ));
        }
        if fc.primary_key != lc.primary_key {
            out.push(format!(
                "[FLAG_CHANGED primary_key] {}.{}: {}={} {}={}",
                table, fc.name, side_frozen, fc.primary_key, side_live, lc.primary_key
            ));
        }
        if fc.not_null != lc.not_null {
            out.push(format!(
                "[FLAG_CHANGED not_null] {}.{}: {}={} {}={}",
                table, fc.name, side_frozen, fc.not_null, side_live, lc.not_null
            ));
        }
        if fc.unique != lc.unique {
            out.push(format!(
                "[FLAG_CHANGED unique] {}.{}: {}={} {}={}",
                table, fc.name, side_frozen, fc.unique, side_live, lc.unique
            ));
        }
        if fc.default_value != lc.default_value {
            out.push(format!(
                "[FLAG_CHANGED default_value] {}.{}: {}='{}' {}='{}'",
                table, fc.name, side_frozen, fc.default_value, side_live, lc.default_value
            ));
        }
        if fc.autoincrement != lc.autoincrement {
            out.push(format!(
                "[FLAG_CHANGED autoincrement] {}.{}: {}={} {}={}",
                table, fc.name, side_frozen, fc.autoincrement, side_live, lc.autoincrement
            ));
        }
    }

    // Order check: only when sets are equal and no per-column type
    // change already fired. Otherwise the order lines would be noise on
    // top of "this column is added".
    let frozen_names: Vec<&str> = frozen.iter().map(|c| c.name.as_str()).collect();
    let live_names: Vec<&str> = live.iter().map(|c| c.name.as_str()).collect();
    if frozen_names == live_names {
        return;
    }
    let same_set = {
        let mut f = frozen_names.clone();
        let mut l = live_names.clone();
        f.sort();
        l.sort();
        f == l
    };
    if !same_set {
        return;
    }
    let first_diff = frozen_names
        .iter()
        .zip(live_names.iter())
        .position(|(a, b)| a != b);
    let idx = first_diff.unwrap_or(0);
    out.push(format!(
        "[ORDER_CHANGED] {}.{}: first divergence at index {}. {}={:?} {}={:?}",
        table, frozen_names[idx], idx, side_frozen, frozen_names, side_live, live_names
    ));
}

fn diff_fks(
    table: &str,
    frozen: &[ForeignKey],
    live: &[ForeignKey],
    side_frozen: &str,
    side_live: &str,
    out: &mut Vec<String>,
) {
    if frozen.len() == live.len() {
        let mut order_changed = false;
        for i in 0..frozen.len() {
            let f = &frozen[i];
            let l = &live[i];
            if f.column == l.column && f.ref_table == l.ref_table && f.ref_column == l.ref_column {
                continue;
            }
            // If the pair (column) matches another slot in live but the
            // target differs, it's a target change; if the column exists
            // in live elsewhere, it's an order change.
            let same_column_live_idx = live.iter().position(|x| x.column == f.column);
            match same_column_live_idx {
                Some(j) if j != i => {
                    order_changed = true;
                }
                Some(_) => {
                    out.push(format!(
                        "[FK_CHANGED] {}.{}: {}->{}.{} {}->{}.{}",
                        table, f.column,
                        side_frozen, f.ref_table, f.ref_column,
                        side_live, l.ref_table, l.ref_column
                    ));
                }
                None => {
                    out.push(format!(
                        "[FK_REMOVED] {}.{}: present in {} not in {}",
                        table, f.column, side_frozen, side_live
                    ));
                }
            }
        }
        if order_changed {
            let f_cols: Vec<&str> = frozen.iter().map(|f| f.column.as_str()).collect();
            let l_cols: Vec<&str> = live.iter().map(|f| f.column.as_str()).collect();
            out.push(format!(
                "[FK_ORDER_CHANGED] {}: {}={:?} {}={:?}",
                table, side_frozen, f_cols, side_live, l_cols
            ));
        }
        return;
    }

    // Different counts: report per-column.
    let frozen_cols: std::collections::HashSet<&str> =
        frozen.iter().map(|f| f.column.as_str()).collect();
    let live_cols: std::collections::HashSet<&str> =
        live.iter().map(|f| f.column.as_str()).collect();
    for f in frozen {
        if !live_cols.contains(f.column.as_str()) {
            out.push(format!(
                "[FK_REMOVED] {}.{}: present in {} not in {}",
                table, f.column, side_frozen, side_live
            ));
        } else {
            let l = live.iter().find(|x| x.column == f.column).unwrap();
            if f.ref_table != l.ref_table || f.ref_column != l.ref_column {
                out.push(format!(
                    "[FK_CHANGED] {}.{}: {}->{}.{} {}->{}.{}",
                    table, f.column,
                    side_frozen, f.ref_table, f.ref_column,
                    side_live, l.ref_table, l.ref_column
                ));
            }
        }
    }
    for l in live {
        if !frozen_cols.contains(l.column.as_str()) {
            out.push(format!(
                "[FK_ADDED] {}.{}: present in {} not in {}",
                table, l.column, side_live, side_frozen
            ));
        }
    }
}

/// TableDescriptors.kt against live Rust, by name / type / order.
/// Flags aren't carried by descriptors so they're not checked here.
pub fn diff_descriptors_against_live(
    descriptors: &[crate::kotlin_parse::ParsedDescriptorsTable],
    live: &[Table],
) -> Vec<String> {
    let mut lines = Vec::new();
    let live_by_name: std::collections::HashMap<&str, &Table> =
        live.iter().map(|t| (t.name.as_str(), t)).collect();
    let desc_by_name: std::collections::HashMap<&str, &crate::kotlin_parse::ParsedDescriptorsTable> =
        descriptors.iter().map(|d| (d.table_name.as_str(), d)).collect();

    for t in live {
        if !desc_by_name.contains_key(t.name.as_str()) {
            lines.push(format!(
                "[TABLE_NO_DESCRIPTOR] table '{}': live Rust has it, TableDescriptors.kt has no object for it",
                t.name
            ));
        }
    }
    for d in descriptors {
        if !live_by_name.contains_key(d.table_name.as_str()) {
            lines.push(format!(
                "[TABLE_NO_LIVE] table '{}': TableDescriptors.kt has {} but Rust has no blueprint for it",
                d.table_name, d.object_name
            ));
        }
    }

    for d in descriptors {
        let Some(lt) = live_by_name.get(d.table_name.as_str()) else {
            continue;
        };
        if d.all_columns_order.is_empty() {
            lines.push(format!(
                "[DESCRIPTOR_NO_ALLCOLS] {}.allColumns is empty — the descriptor order is undefined",
                d.object_name
            ));
            continue;
        }
        // Resolve allColumns idents to descriptors.
        let by_ident: std::collections::HashMap<&str, &crate::kotlin_parse::KotlinDescriptor> =
            d.descriptors.iter().map(|x| (x.ident.as_str(), x)).collect();
        let mut resolved = Vec::new();
        for ident in &d.all_columns_order {
            match by_ident.get(ident.as_str()) {
                Some(desc) => resolved.push(*desc),
                None => lines.push(format!(
                    "[DESCRIPTOR_ORPHAN] {}.allColumns references `{}` but no `val {} = ColumnDescriptor` exists",
                    d.object_name, ident, ident
                )),
            }
        }
        // Duplicate descriptor names inside one object.
        let mut seen = std::collections::HashSet::new();
        for desc in &d.descriptors {
            if !seen.insert(desc.name.as_str()) {
                lines.push(format!(
                    "[DUPLICATE_COLUMN] {}.{}: two descriptors declare the same column name",
                    d.object_name, desc.name
                ));
            }
        }

        let live_cols: std::collections::HashMap<&str, &Column> =
            lt.columns.iter().map(|c| (c.name.as_str(), c)).collect();
        let resolved_names: Vec<&str> = resolved.iter().map(|r| r.name.as_str()).collect();

        for r in &resolved {
            match live_cols.get(r.name.as_str()) {
                None => lines.push(format!(
                    "[DESCRIPTOR_EXTRA] {}.{}: TableDescriptors.kt has it, Rust does not",
                    d.object_name, r.name
                )),
                Some(lc) => {
                    if lc.column_type != r.column_type {
                        lines.push(format!(
                            "[DESCRIPTOR_TYPE_CHANGED] {}.{}: descriptor says '{}', Rust says '{}'",
                            d.object_name, r.name, r.column_type, lc.column_type
                        ));
                    }
                    if let Some(cast) = &r.cast_variant {
                        let expected_cast = match r.column_type.as_str() {
                            "INTEGER" => Some("Integer"),
                            "TEXT" => Some("Text"),
                            "BLOB" => Some("Blob"),
                            "REAL" => Some("Real"),
                            _ => None,
                        };
                        if let Some(exp) = expected_cast {
                            if cast != exp {
                                lines.push(format!(
                                    "[DESCRIPTOR_CAST_MISMATCH] {}.{}: declared '{}' but casts to Col.{} (expected Col.{})",
                                    d.object_name, r.name, r.column_type, cast, exp
                                ));
                            }
                        }
                    }
                }
            }
        }

        let live_names: Vec<&str> = lt.columns.iter().map(|c| c.name.as_str()).collect();
        for ln in &live_names {
            if !resolved_names.contains(ln) {
                // Produce a pasteable descriptor line.
                let ident = camel_case(ln);
                let cast = match live_cols.get(*ln).map(|c| c.column_type.as_str()) {
                    Some("INTEGER") => "Integer",
                    Some("TEXT") => "Text",
                    Some("BLOB") => "Blob",
                    Some("REAL") => "Real",
                    _ => "Null",
                };
                let idx = live_names.iter().position(|x| x == ln).unwrap();
                lines.push(format!(
                    "[DESCRIPTOR_MISSING] {}.{}: Rust has it, TableDescriptors.kt does not.\n  \
                     Add: `val {} = ColumnDescriptor(\"{}\", \"{}\") {{ (it as Col.{}).v1 }}`\n  \
                     Insert `{}` at index {} of allColumns",
                    d.object_name, ln, ident, ln,
                    live_cols.get(*ln).map(|c| c.column_type.clone()).unwrap_or_default(),
                    cast, ident, idx
                ));
            }
        }

        // Order check: only when sets are equal.
        if live_names.len() == resolved_names.len() {
            let mut ls = live_names.clone();
            let mut rs = resolved_names.clone();
            ls.sort();
            rs.sort();
            if ls == rs && live_names != resolved_names {
                let first = live_names
                    .iter()
                    .zip(resolved_names.iter())
                    .position(|(a, b)| a != b)
                    .unwrap_or(0);
                lines.push(format!(
                    "[DESCRIPTOR_ORDER_CHANGED] {}.{}: first divergence at index {}. Rust={:?} Kotlin={:?}",
                    d.object_name, live_names[first], first, live_names, resolved_names
                ));
            }
        }
    }

    lines
}

/// Coverage check: scan `client_table_blueprints/src/**/*.rs` for
/// `pub fn new_table_*` / `pub fn get_foreign_def_*`, and confirm every
/// hit has a registry entry. Returns error lines when the registry is
/// incomplete, and Ok(vec![]) on zero hits only when the crate really
/// has no blueprints (shouldn't happen, so the caller treats zero hits
/// as a parse failure).
pub fn scan_rust_coverage(
    src_root: &std::path::Path,
    registry_fn_names: &std::collections::HashSet<&str>,
) -> Result<Vec<String>, String> {
    let mut hits: Vec<(String, String, usize)> = Vec::new();
    let walker = walk(src_root).map_err(|e| format!("walk {}: {}", src_root.display(), e))?;
    for path in walker {
        let src = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        for (idx, line) in src.lines().enumerate() {
            let t = line.trim_start();
            if let Some(rest) = t.strip_prefix("pub fn ") {
                let name_end = rest
                    .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                    .unwrap_or(rest.len());
                let name = &rest[..name_end];
                if name.starts_with("new_table_") || name.starts_with("get_foreign_def_") {
                    hits.push((name.to_string(), path.display().to_string(), idx + 1));
                }
            }
        }
    }
    if hits.is_empty() {
        return Err(format!(
            "no `pub fn new_table_*` or `pub fn get_foreign_def_*` found under {} — the pattern is stale or the source moved",
            src_root.display()
        ));
    }
    let mut lines = Vec::new();
    for (name, file, line) in &hits {
        if !registry_fn_names.contains(name.as_str()) {
            lines.push(format!(
                "[REGISTRY_MISSING] Rust defines `{}` ({}:{}) but the drift detector doesn't know it. \
                 Add it to the registry in src/registry.rs, and add the table to Version<N> snapshot + TableDescriptors.kt.",
                name, file, line
            ));
        }
    }
    Ok(lines)
}

fn walk(root: &std::path::Path) -> std::io::Result<Vec<std::path::PathBuf>> {
    // (No dotdir skipping — client_table_blueprints/src has no dot dirs.)
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(p) = stack.pop() {
        for entry in std::fs::read_dir(&p)? {
            let entry = entry?;
            let ft = entry.file_type()?;
            if ft.is_dir() {
                stack.push(entry.path());
            } else if ft.is_file() && entry.path().extension().and_then(|e| e.to_str()) == Some("rs") {
                out.push(entry.path());
            }
        }
    }
    out.sort();
    Ok(out)
}

fn camel_case(snake: &str) -> String {
    let mut out = String::new();
    let mut upper = false;
    for (i, c) in snake.chars().enumerate() {
        if c == '_' {
            upper = true;
            continue;
        }
        if i == 0 {
            out.push(c.to_ascii_lowercase());
        } else if upper {
            out.push(c.to_ascii_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}


#[cfg(test)]
mod tests {
    use super::*;

    fn col(name: &str, ty: &str) -> Column {
        Column {
            name: name.into(),
            column_type: ty.into(),
            primary_key: false,
            not_null: true,
            unique: false,
            default_value: String::new(),
            autoincrement: false,
        }
    }
    fn table(name: &str, columns: Vec<Column>, foreign_keys: Vec<ForeignKey>) -> Table {
        Table {
            name: name.into(),
            columns,
            foreign_keys,
        }
    }

    #[test]
    fn detects_added_column() {
        let f = vec![table("t", vec![col("a", "TEXT")], vec![])];
        let l = vec![table("t", vec![col("a", "TEXT"), col("b", "INTEGER")], vec![])];
        let out = diff_tables(&f, &l, "frozen", "live");
        assert!(out.iter().any(|s| s.contains("COLUMN_ADDED_IN_RUST") && s.contains("t.b")));
    }

    #[test]
    fn detects_removed_column() {
        let f = vec![table("t", vec![col("a", "TEXT"), col("b", "INTEGER")], vec![])];
        let l = vec![table("t", vec![col("a", "TEXT")], vec![])];
        let out = diff_tables(&f, &l, "frozen", "live");
        assert!(out.iter().any(|s| s.contains("COLUMN_REMOVED_FROM_RUST") && s.contains("t.b")));
    }

    #[test]
    fn detects_type_change() {
        let f = vec![table("t", vec![col("a", "TEXT")], vec![])];
        let l = vec![table("t", vec![col("a", "INTEGER")], vec![])];
        let out = diff_tables(&f, &l, "frozen", "live");
        assert!(out.iter().any(|s| s.contains("TYPE_CHANGED") && s.contains("t.a")));
    }

    #[test]
    fn detects_order_change() {
        let f = vec![table("t", vec![col("a", "TEXT"), col("b", "TEXT")], vec![])];
        let l = vec![table("t", vec![col("b", "TEXT"), col("a", "TEXT")], vec![])];
        let out = diff_tables(&f, &l, "frozen", "live");
        assert!(out.iter().any(|s| s.contains("ORDER_CHANGED")));
    }

    #[test]
    fn detects_rename_hint() {
        let f = vec![table("t", vec![col("old", "TEXT")], vec![])];
        let l = vec![table("t", vec![col("new", "TEXT")], vec![])];
        let out = diff_tables(&f, &l, "frozen", "live");
        assert!(out.iter().any(|s| s.contains("COLUMN_RENAMED?")));
    }

    #[test]
    fn detects_fk_change() {
        let f = vec![table(
            "t",
            vec![col("a", "TEXT")],
            vec![ForeignKey { column: "a".into(), ref_table: "x".into(), ref_column: "id".into() }],
        )];
        let l = vec![table(
            "t",
            vec![col("a", "TEXT")],
            vec![ForeignKey { column: "a".into(), ref_table: "y".into(), ref_column: "id".into() }],
        )];
        let out = diff_tables(&f, &l, "frozen", "live");
        assert!(out.iter().any(|s| s.contains("FK_CHANGED")));
    }

    #[test]
    fn detects_flag_change() {
        let mut f_col = col("a", "TEXT");
        f_col.not_null = false;
        let f = vec![table("t", vec![f_col], vec![])];
        let l = vec![table("t", vec![col("a", "TEXT")], vec![])];
        let out = diff_tables(&f, &l, "frozen", "live");
        assert!(out.iter().any(|s| s.contains("FLAG_CHANGED not_null")));
    }

    #[test]
    fn detects_table_missing_and_only() {
        let f = vec![table("a", vec![], vec![])];
        let l = vec![table("b", vec![], vec![])];
        let out = diff_tables(&f, &l, "frozen", "live");
        assert!(out.iter().any(|s| s.contains("TABLE_MISSING_IN_RUST") && s.contains("'a'")));
        assert!(out.iter().any(|s| s.contains("TABLE_ONLY_IN_RUST") && s.contains("'b'")));
    }
}
