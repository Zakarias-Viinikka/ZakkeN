//! End-to-end mutation tests. Each test copies the real Kotlin fixture
//! files into a tempdir, mutates one of them, runs the tool with
//! --android-root pointing at the tempdir, and checks the report names
//! the specific drift.
//!
//! The Rust-side live data comes from `live_tables()` (compiled in), so
//! only Kotlin-side mutations can be tested here. Rust-side mutation
//! testing means editing the crate, which the plan forbids.

use std::path::PathBuf;
use std::process::Command;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn real_android_root() -> PathBuf {
    crate_dir().parent().unwrap().to_path_buf()
}

fn real_blueprints_root() -> PathBuf {
    real_android_root()
        .parent()
        .unwrap()
        .join("client_table_blueprints")
}

const FIXTURE_RELS: &[&str] = &[
    "app/src/main/java/z/zndroid/db/migrations/SchemaVersion.kt",
    "app/src/main/java/z/zndroid/db/migrations/schemas/Version3.kt",
    "app/src/main/java/z/zndroid/db/schema/TableDescriptors.kt",
];

struct Fixture {
    _tmp: tempfile::TempDir,
    root: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let tmp = tempfile::tempdir().expect("create tmpdir");
        for rel in FIXTURE_RELS {
            let src = real_android_root().join(rel);
            let dst = tmp.path().join(rel);
            std::fs::create_dir_all(dst.parent().unwrap()).expect("mkdir fixture");
            std::fs::copy(&src, &dst).expect("copy fixture file");
        }
        Self {
            root: tmp.path().to_path_buf(),
            _tmp: tmp,
        }
    }

    fn path(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }

    fn replace(&self, rel: &str, old: &str, new: &str) {
        let path = self.path(rel);
        let content = std::fs::read_to_string(&path).expect("read fixture");
        assert!(
            content.contains(old),
            "fixture {} does not contain expected text:\n---\n{}\n---",
            rel,
            old
        );
        let new_content = content.replacen(old, new, 1);
        assert_ne!(content, new_content);
        std::fs::write(&path, new_content).expect("write fixture");
    }

    fn run(&self) -> RunResult {
        let output = Command::new(env!("CARGO_BIN_EXE_schema_drift_detector"))
            .arg("check")
            .arg("--no-stale-check")
            .arg("--android-root")
            .arg(&self.root)
            .arg("--blueprints-root")
            .arg(real_blueprints_root())
            .env_remove("SCHEMA_DRIFT_CHECKED")
            .env_remove("SKIP_SCHEMA_DRIFT")
            .output()
            .expect("spawn tool");
        RunResult {
            code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }
}

struct RunResult {
    code: i32,
    stdout: String,
    stderr: String,
}

impl RunResult {
    fn combined(&self) -> String {
        format!("{}\n{}", self.stdout, self.stderr)
    }
    fn assert_clean(&self) {
        assert_eq!(
            self.code, 0,
            "expected clean exit, got {}:\n{}",
            self.code,
            self.combined()
        );
        assert!(
            self.stdout.contains("clean"),
            "expected 'clean' in stdout:\n{}",
            self.stdout
        );
    }
    fn assert_drift(&self, needle: &str) {
        assert_eq!(
            self.code, 1,
            "expected drift exit, got {}:\n{}",
            self.code,
            self.combined()
        );
        assert!(
            self.stdout.contains(needle),
            "expected '{}' in report:\n{}",
            needle,
            self.stdout
        );
    }
    fn assert_tool_error(&self, needle: &str) {
        assert_eq!(
            self.code, 2,
            "expected tool error, got {}:\n{}",
            self.code,
            self.combined()
        );
        assert!(
            self.combined().contains(needle),
            "expected '{}' in error:\n{}",
            needle,
            self.combined()
        );
    }
}

#[test]
fn clean_fixture_passes() {
    Fixture::new().run().assert_clean();
}

#[test]
fn remove_column_from_version_fires_a() {
    let fx = Fixture::new();
    // Remove a middle column (has a trailing comma in the fixture, so
    // the fixture stays syntactically valid). The tool reports it as
    // COLUMN_ADDED_IN_RUST.
    fx.replace(
        "app/src/main/java/z/zndroid/db/migrations/schemas/Version3.kt",
        "                ColumnDef(\"my_id_as_given_by_yrs\", \"TEXT\", primaryKey = false, notNull = true, unique = false, defaultValue = \"\", autoincrement = false),\n",
        "",
    );
    fx.run().assert_drift("COLUMN_ADDED_IN_RUST");
}

#[test]
fn change_type_in_version_fires_a() {
    let fx = Fixture::new();
    fx.replace(
        "app/src/main/java/z/zndroid/db/migrations/schemas/Version3.kt",
        "ColumnDef(\"is_title\", \"TEXT\"",
        "ColumnDef(\"is_title\", \"INTEGER\"",
    );
    let r = fx.run();
    r.assert_drift("TYPE_CHANGED");
    assert!(r.stdout.contains("is_title"), "should name the column:\n{}", r.stdout);
}

#[test]
fn swap_two_columns_in_version_fires_a() {
    let fx = Fixture::new();
    let before = "                ColumnDef(\"is_title\", \"TEXT\", primaryKey = false, notNull = true, unique = false, defaultValue = \"\", autoincrement = false),\n                ColumnDef(\"is_part_of_main_menu_page\", \"TEXT\", primaryKey = false, notNull = true, unique = false, defaultValue = \"\", autoincrement = false),\n";
    let after = "                ColumnDef(\"is_part_of_main_menu_page\", \"TEXT\", primaryKey = false, notNull = true, unique = false, defaultValue = \"\", autoincrement = false),\n                ColumnDef(\"is_title\", \"TEXT\", primaryKey = false, notNull = true, unique = false, defaultValue = \"\", autoincrement = false),\n";
    fx.replace(
        "app/src/main/java/z/zndroid/db/migrations/schemas/Version3.kt",
        before,
        after,
    );
    fx.run().assert_drift("ORDER_CHANGED");
}

#[test]
fn remove_column_from_descriptors_fires_b() {
    let fx = Fixture::new();
    fx.replace(
        "app/src/main/java/z/zndroid/db/schema/TableDescriptors.kt",
        "    val isTitle = ColumnDescriptor(\"is_title\", \"TEXT\") { (it as Col.Text).v1 }\n",
        "",
    );
    fx.replace(
        "app/src/main/java/z/zndroid/db/schema/TableDescriptors.kt",
        "val allColumns = listOf(id, isTitle, isPartOfMainMenuPage, content, myIdAsGivenByYrs, idOfPageIBelongTo, fracPosition, pageIsDisabled)",
        "val allColumns = listOf(id, isPartOfMainMenuPage, content, myIdAsGivenByYrs, idOfPageIBelongTo, fracPosition, pageIsDisabled)",
    );
    fx.run().assert_drift("DESCRIPTOR_MISSING");
}

#[test]
fn change_descriptor_type_fires_b() {
    let fx = Fixture::new();
    fx.replace(
        "app/src/main/java/z/zndroid/db/schema/TableDescriptors.kt",
        "val pageId = ColumnDescriptor(\"page_id\", \"TEXT\") { (it as Col.Text).v1 }",
        "val pageId = ColumnDescriptor(\"page_id\", \"INTEGER\") { (it as Col.Integer).v1 }",
    );
    fx.run().assert_drift("DESCRIPTOR_TYPE_CHANGED");
}

#[test]
fn remove_latest_fires_tool_error() {
    let fx = Fixture::new();
    fx.replace(
        "app/src/main/java/z/zndroid/db/migrations/SchemaVersion.kt",
        "val LATEST = V3",
        "// removed",
    );
    fx.run().assert_tool_error("no `val LATEST =");
}

#[test]
fn point_latest_at_missing_version_fires_tool_error() {
    let fx = Fixture::new();
    // Point LATEST at a version with no enum entry. The tool fires on
    // the missing enum entry (the more useful diagnostic).
    fx.replace(
        "app/src/main/java/z/zndroid/db/migrations/SchemaVersion.kt",
        "val LATEST = V3",
        "val LATEST = V99",
    );
    fx.run().assert_tool_error("no `V99(99) {`");
}

#[test]
fn emit_snapshot_roundtrips_through_the_parser() {
    let output = Command::new(env!("CARGO_BIN_EXE_schema_drift_detector"))
        .arg("emit-snapshot")
        .arg("99")
        .env_remove("SCHEMA_DRIFT_CHECKED")
        .env_remove("SKIP_SCHEMA_DRIFT")
        .output()
        .expect("spawn tool");
    assert_eq!(output.status.code(), Some(0));
    let emitted = String::from_utf8_lossy(&output.stdout).into_owned();

    let parsed = schema_drift_detector::kotlin_parse::parse_version_file(&emitted)
        .expect("emitted snapshot must parse");
    let live = schema_drift_detector::registry::live_tables();
    assert_eq!(parsed.len(), live.len(), "table count");
    for (p, l) in parsed.iter().zip(live.iter()) {
        assert_eq!(p.name, l.name, "table order");
        assert_eq!(p.columns.len(), l.columns.len(), "{} columns", p.name);
        for (pc, lc) in p.columns.iter().zip(l.columns.iter()) {
            assert_eq!(pc.name, lc.name, "{} column order", p.name);
            assert_eq!(pc.column_type, lc.column_type, "{} column type", p.name);
            assert_eq!(pc.primary_key, lc.primary_key, "{} primary_key", p.name);
            assert_eq!(pc.not_null, lc.not_null, "{} not_null", p.name);
            assert_eq!(pc.unique, lc.unique, "{} unique", p.name);
            assert_eq!(pc.autoincrement, lc.autoincrement, "{} autoincrement", p.name);
        }
        assert_eq!(p.foreign_keys.len(), l.foreign_keys.len(), "{} FKs", p.name);
    }
}
