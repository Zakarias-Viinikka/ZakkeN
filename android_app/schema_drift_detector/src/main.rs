use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use schema_drift_detector::diff;
use schema_drift_detector::kotlin_parse;
use schema_drift_detector::model::Table;
use schema_drift_detector::registry::{live_tables, REGISTRY};
use schema_drift_detector::version_parse;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if std::env::var_os("SKIP_SCHEMA_DRIFT").is_some() {
        eprintln!("schema_drift_detector: SKIP_SCHEMA_DRIFT is set — skipping (this hides real drift)");
        return ExitCode::from(0);
    }
    match run(&args) {
        Ok(code) => ExitCode::from(code),
        Err(msg) => {
            eprintln!("schema_drift_detector: TOOL ERROR: {}", msg);
            ExitCode::from(2)
        }
    }
}

struct Opts {
    cmd: Command,
    warn_only: bool,
    stale_check: bool,
    android_root: PathBuf,
    blueprints_root: PathBuf,
}

enum Command {
    Check,
    EmitSnapshot(u32),
}

fn parse_args(args: &[String]) -> Result<Opts, String> {
    let cargo_manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| "CARGO_MANIFEST_DIR not set; run via check.sh".to_string())?;
    let crate_dir = PathBuf::from(cargo_manifest_dir);
    let default_android = crate_dir
        .parent()
        .ok_or_else(|| "crate dir has no parent".to_string())?
        .to_path_buf();
    let default_blueprints = default_android
        .parent()
        .map(|p| p.join("client_table_blueprints"))
        .ok_or_else(|| "android_root has no parent".to_string())?;

    let mut cmd: Option<Command> = None;
    let mut warn_only = false;
    let mut stale_check = true;
    let mut android_root = default_android;
    let mut blueprints_root = default_blueprints;

    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        match a.as_str() {
            "check" => {
                cmd = Some(Command::Check);
                i += 1;
            }
            "emit-snapshot" => {
                let n = args
                    .get(i + 1)
                    .ok_or_else(|| "emit-snapshot needs a version number".to_string())?;
                let v: u32 = n
                    .parse()
                    .map_err(|_| format!("emit-snapshot: `{}` is not a number", n))?;
                cmd = Some(Command::EmitSnapshot(v));
                i += 2;
            }
            "--warn-only" => {
                warn_only = true;
                i += 1;
            }
            "--no-stale-check" => {
                stale_check = false;
                i += 1;
            }
            "--android-root" => {
                android_root = PathBuf::from(
                    args.get(i + 1)
                        .ok_or_else(|| "--android-root needs a value".to_string())?,
                );
                i += 2;
            }
            "--blueprints-root" => {
                blueprints_root = PathBuf::from(
                    args.get(i + 1)
                        .ok_or_else(|| "--blueprints-root needs a value".to_string())?,
                );
                i += 2;
            }
            other => {
                return Err(format!("unknown argument `{}`", other));
            }
        }
    }

    Ok(Opts {
        cmd: cmd.unwrap_or(Command::Check),
        warn_only,
        stale_check,
        android_root,
        blueprints_root,
    })
}

fn run(args: &[String]) -> Result<u8, String> {
    let opts = parse_args(args)?;
    match opts.cmd {
        Command::Check => run_check(&opts),
        Command::EmitSnapshot(n) => run_emit_snapshot(&opts, n),
    }
}

fn run_check(opts: &Opts) -> Result<u8, String> {
    if std::env::var_os("SCHEMA_DRIFT_CHECKED").is_some() {
        // Called from a build script that already ran us.
        return Ok(0);
    }

    let mut problems: Vec<(String, Vec<String>, String)> = Vec::new();

    // --- Parse on-disk Kotlin files ---
    let schema_version_path = opts
        .android_root
        .join("app/src/main/java/z/zndroid/db/migrations/SchemaVersion.kt");
    let table_descriptors_path = opts
        .android_root
        .join("app/src/main/java/z/zndroid/db/schema/TableDescriptors.kt");

    let schema_version_src = std::fs::read_to_string(&schema_version_path).map_err(|e| {
        format!(
            "cannot read {}: {}",
            schema_version_path.display(),
            e
        )
    })?;
    let latest = version_parse::parse_latest(&schema_version_src)
        .map_err(|e| format!("parse {}: {}", schema_version_path.display(), e))?;

    let version_file_path = opts.android_root.join(format!(
        "app/src/main/java/z/zndroid/db/migrations/schemas/Version{}.kt",
        latest.version
    ));
    let version_src = std::fs::read_to_string(&version_file_path)
        .map_err(|e| format!("cannot read {}: {}", version_file_path.display(), e))?;
    let frozen_tables = kotlin_parse::parse_version_file(&version_src)
        .map_err(|e| format!("parse {}: {}", version_file_path.display(), e))?;
    if frozen_tables.is_empty() {
        return Err(format!(
            "{} parsed but yielded 0 FrozenTable blocks — the tool's parser is stale or the file shape changed",
            version_file_path.display()
        ));
    }

    let descriptors_src = std::fs::read_to_string(&table_descriptors_path).map_err(|e| {
        format!(
            "cannot read {}: {}",
            table_descriptors_path.display(),
            e
        )
    })?;
    let descriptors = kotlin_parse::parse_table_descriptors(&descriptors_src)
        .map_err(|e| format!("parse {}: {}", table_descriptors_path.display(), e))?;
    if descriptors.tables.is_empty() {
        return Err(format!(
            "{} parsed but yielded 0 `object XTableSchema` blocks — the tool's parser is stale or the file shape changed",
            table_descriptors_path.display()
        ));
    }

    // --- Live Rust ---
    let live = live_tables();
    if live.is_empty() {
        return Err("registry yielded 0 tables — the registry itself is broken".to_string());
    }

    // Tables that exist in live Rust because web or a future feature
    // uses them, but Android doesn't have on the Kotlin side yet. Keep
    // this list small and named — it's the one place the tool allows
    // forward-looking Rust to exist without failing Android's build.
    // Remove an entry here when Android freezes a version that includes
    // the table.
    // Currently empty. Add an entry here only for a Rust table that
    // genuinely shouldn't be on Android yet, and remove it once Android
    // freezes a version that includes the table. `seen_server_letters`
    // graduated on 2026-11-15: it's now in V4.
    const NOT_YET_ON_ANDROID_TABLES: &[&str] = &[];

    let live_for_a: Vec<Table> = live
        .iter()
        .filter(|t| !NOT_YET_ON_ANDROID_TABLES.contains(&t.name.as_str()))
        .cloned()
        .collect();

    // --- Check A: frozen snapshot vs live Rust ---
    let a_lines = diff::diff_tables(
        &frozen_tables,
        &live_for_a,
        &format!("frozen V{}", latest.version),
        "live Rust",
    );
    if !a_lines.is_empty() {
        problems.push((
            format!(
                "A. Frozen V{} snapshot vs live Rust",
                latest.version
            ),
            a_lines,
            format!(
                "Rust blueprints changed since V{} was frozen. Freeze a new version: \
                 run `schema_drift_detector/check.sh emit-snapshot {}`, save as \
                 schemas/Version{}.kt, add V{} to SchemaVersion with bootstrap from \
                 Version{}.tables, write V{}.updateOnce returning V{}, set LATEST = V{}, \
                 update TableDescriptors.kt (check B lists exactly what), then run \
                 build_all_android.sh.",
                latest.version,
                latest.version + 1,
                latest.version + 1,
                latest.version + 1,
                latest.version + 1,
                latest.version,
                latest.version + 1,
                latest.version + 1
            ),
        ));
    }

    // --- Check B: TableDescriptors.kt vs live Rust ---
    let b_lines = diff::diff_descriptors_against_live(&descriptors.tables, &live_for_a);
    if !b_lines.is_empty() {
        problems.push((
            "B. TableDescriptors.kt vs live Rust".to_string(),
            b_lines,
            "Add / remove / fix the descriptor objects in app/src/main/java/z/zndroid/db/schema/TableDescriptors.kt so they match live Rust.".to_string(),
        ));
    }

    // --- Check C: version plumbing ---
    let mut c_lines = Vec::new();
    if !latest.body_uses_snapshot_tables {
        c_lines.push(format!(
            "[VERSION_BODY_NOT_REPOINTED] SchemaVersion.{}'s bootstrap body does not contain `Version{}.tables` — \
             did you bump LATEST without repointing bootstrap to the new snapshot?",
            latest.variant, latest.version
        ));
    }
    if !c_lines.is_empty() {
        problems.push((
            "C. Version plumbing".to_string(),
            c_lines,
            "Repoint LATEST's bootstrap in SchemaVersion.kt to create tables from the frozen snapshot.".to_string(),
        ));
    }

    // --- Check D: native lib freshness ---
    if opts.stale_check {
        let d_lines = check_native_lib_freshness(opts)?;
        if !d_lines.is_empty() {
            problems.push((
                "D. Native lib freshness".to_string(),
                d_lines,
                "Run ZakkeN/build_all_android.sh (which rebuilds native libs and then runs update.sh to copy them into the app).".to_string(),
            ));
        }
    }

    // --- Check E: Rust coverage ---
    let registry_fn_names: HashSet<&str> = REGISTRY
        .iter()
        .flat_map(|e| e.all_fn_names())
        .collect();
    // (Coverage still looks at every fn in the source, including the
    // allowlisted tables' fns — registry entries for those fns prove the
    // tool knows the table exists, and the allowlist in check A is what
    // suppresses the missing-on-Kotlin message.)
    let src_dir = opts.blueprints_root.join("src");
    let e_lines = diff::scan_rust_coverage(&src_dir, &registry_fn_names)
        .map_err(|e| format!("scan {}: {}", src_dir.display(), e))?;
    if !e_lines.is_empty() {
        problems.push((
            "E. Rust coverage".to_string(),
            e_lines,
            "Add the missing blueprint fns to schema_drift_detector/src/registry.rs and to the Kotlin side (V<N> snapshot + TableDescriptors.kt).".to_string(),
        ));
    }

    // --- Report ---
    let total: usize = problems.iter().map(|(_, lines, _)| lines.len()).sum();
    if total == 0 {
        println!(
            "schema_drift_detector: clean. Frozen V{} vs live Rust, TableDescriptors.kt vs live Rust, \
             version plumbing{}, Rust coverage all agree.",
            latest.version,
            if opts.stale_check {
                ", native lib freshness"
            } else {
                ""
            }
        );
        return Ok(0);
    }

    println!("schema_drift_detector: {} problem(s) found.", total);
    for (title, lines, fix) in &problems {
        println!();
        println!("=== {} ===", title);
        for line in lines {
            println!("{}", line);
        }
        println!();
        println!("Fix: {}", fix);
    }
    if opts.warn_only {
        println!();
        println!("(--warn-only: not failing the build)");
        Ok(0)
    } else {
        Ok(1)
    }
}

fn run_emit_snapshot(opts: &Opts, version: u32) -> Result<u8, String> {
    let live = live_tables();
    if live.is_empty() {
        return Err("registry yielded 0 tables".to_string());
    }
    let _ = &opts.blueprints_root;
    println!("package z.zndroid.db.migrations.schemas");
    println!();
    println!("import uniffi.protocol.ColumnDef");
    println!("import uniffi.protocol.ForeignKeyDef");
    println!();
    println!("/**");
    println!(" * Frozen snapshot of the schema at V{}. Do not edit by hand.", version);
    println!(" * Generated by schema_drift_detector/check.sh emit-snapshot.");
    println!(" */");
    println!("object Version{} {{", version);
    println!("    val tables: List<FrozenTable> = listOf(");
    for (i, t) in live.iter().enumerate() {
        emit_frozen_table(t);
        if i + 1 != live.len() {
            println!("        ),");
        } else {
            println!("        )");
        }
    }
    println!("    )");
    println!("}}");
    Ok(0)
}

fn emit_frozen_table(t: &Table) {
    println!("        FrozenTable(");
    println!("            name = \"{}\",", t.name);
    println!("            columns = listOf(");
    for (i, c) in t.columns.iter().enumerate() {
        let suffix = if i + 1 != t.columns.len() { "," } else { "" };
        println!(
            "                ColumnDef(\"{}\", \"{}\", primaryKey = {}, notNull = {}, unique = {}, defaultValue = \"{}\", autoincrement = {}){}",
            c.name, c.column_type, c.primary_key, c.not_null, c.unique, c.default_value, c.autoincrement, suffix
        );
    }
    println!("            ){}", if t.foreign_keys.is_empty() { "" } else { "," });
    if !t.foreign_keys.is_empty() {
        println!("            foreignKeys = listOf(");
        for (i, fk) in t.foreign_keys.iter().enumerate() {
            let suffix = if i + 1 != t.foreign_keys.len() { "," } else { "" };
            println!(
                "                ForeignKeyDef(\"{}\", \"{}\", \"{}\"){}",
                fk.column, fk.ref_table, fk.ref_column, suffix
            );
        }
        println!("            )");
    }
}

fn check_native_lib_freshness(opts: &Opts) -> Result<Vec<String>, String> {
    let mut lines = Vec::new();
    let android_output_so = opts
        .blueprints_root
        .join("android_output/jniLibs/arm64-v8a/libclient_table_blueprints.so");
    let app_so = opts
        .android_root
        .join("app/src/main/jniLibs/arm64-v8a/libclient_table_blueprints.so");

    // Only the files that can actually change the built .so. NOT the
    // whole blueprints_root: `target/` and `android_output/` are
    // touched by every cargo build, which would always look newer than
    // the last real .so build and fire the stale check forever.
    let mut newest_dep = newest_mtime_under(&opts.blueprints_root.join("src"))?;
    for f in ["Cargo.toml", "Cargo.lock", "uniffi.toml", "build.rs"] {
        let p = opts.blueprints_root.join(f);
        if let Ok(meta) = std::fs::metadata(&p) {
            if let Ok(m) = meta.modified() {
                if newest_dep.map(|n| m > n).unwrap_or(true) {
                    newest_dep = Some(m);
                }
            }
        }
    }

    let android_output_mtime = std::fs::metadata(&android_output_so)
        .ok()
        .and_then(|m| m.modified().ok());
    match android_output_mtime {
        None => {
            if app_so.exists() {
                // Skip the compare; not a fail. The app has a copy but
                // we can't say it's stale.
            } else {
                lines.push(format!(
                    "[APP_SO_MISSING] app copy missing: {}\n  Run build_all_android.sh to produce it.",
                    app_so.display()
                ));
            }
        }
        Some(ao_mtime) => {
            if let Some(dep) = newest_dep {
                if dep > ao_mtime {
                    lines.push(format!(
                        "[STALE_NATIVE_LIB] {}: sources modified {} is newer than the built .so ({}). Run ZakkeN/build_all_android.sh.",
                        android_output_so.display(),
                        fmt_time(dep),
                        fmt_time(ao_mtime)
                    ));
                }
            }
            match std::fs::metadata(&app_so) {
                Err(_) => lines.push(format!(
                    "[APP_SO_MISSING] app copy missing: {}. update.sh hasn't been run since the last build.",
                    app_so.display()
                )),
                Ok(app_meta) => {
                    let app_mtime = app_meta.modified().ok();
                    match app_mtime {
                        Some(am) if am < ao_mtime => lines.push(format!(
                            "[APP_SO_OUTDATED] app copy older ({}) than android_output copy ({}). update.sh hasn't been run since the last native build.",
                            fmt_time(am),
                            fmt_time(ao_mtime)
                        )),
                        _ => {
                            let ah = sha256_of(&android_output_so)?;
                            let bh = sha256_of(&app_so)?;
                            if ah != bh {
                                lines.push(format!(
                                    "[APP_SO_HASH_MISMATCH] app copy differs from android_output copy. update.sh hasn't been run since the last native build."
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(lines)
}

fn newest_mtime_under(dir: &Path) -> Result<Option<std::time::SystemTime>, String> {
    if !dir.exists() {
        return Ok(None);
    }
    let mut newest: Option<std::time::SystemTime> = None;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(p) = stack.pop() {
        let rd = std::fs::read_dir(&p).map_err(|e| format!("read_dir {}: {}", p.display(), e))?;
        for entry in rd {
            let entry = entry.map_err(|e| format!("read_dir entry: {}", e))?;
            let ft = entry.file_type().map_err(|e| format!("file_type: {}", e))?;
            if ft.is_dir() {
                stack.push(entry.path());
            } else if ft.is_file() {
                if let Ok(meta) = entry.metadata() {
                    if let Ok(m) = meta.modified() {
                        if newest.map(|n| m > n).unwrap_or(true) {
                            newest = Some(m);
                        }
                    }
                }
            }
        }
    }
    Ok(newest)
}

fn fmt_time(t: std::time::SystemTime) -> String {
    use std::time::{Duration, UNIX_EPOCH};
    let secs = t.duration_since(UNIX_EPOCH).unwrap_or(Duration::from_secs(0)).as_secs();
    format!("{}s since epoch", secs)
}

fn sha256_of(path: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {}", path.display(), e))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let digest = hasher.finalize();
    Ok(format!("{:x}", digest))
}