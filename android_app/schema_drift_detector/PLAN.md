# Schema drift detector — plan

Written 2026-11-15. Handed to Claude for implementation. Not yet built.

The user's pain: schema expectations break in the Android app and it's obnoxious to figure out that the failure was a schema issue, or where. This plan is the design agreed with the user. Two hard constraints: (a) the tool runs on the host during the Android build, not on the device; (b) it must report the *specific* drift — which table, which column, what changed — not just "schema mismatch".

The tool lives at `android_app/schema_drift_detector/` once built. Below is the plan exactly as it was handed to Claude.

---

> # Task: build a schema drift detector for the Android app
>
> Build a host-side CLI at /home/zakke/ProgStuff/ZakkeN/android_app/schema_drift_detector/ and wire it into the Android build so that drift between the Rust schema (client_table_blueprints) and the Kotlin side fails the build with a message naming the specific drift. "Schema mismatch" alone is never acceptable; every report line must say which table, which column, and what kind of drift.
>
> The user's recurring pain: schema expectations break and it's obnoxious to figure out that it was a schema issue, or where. Optimize for that: precise, located, actionable messages.
>
> Design freedom was given, and the design below is already decided. Follow it, and use your judgment on details and bugs. If something here contradicts reality in the code, adapt and say so in your report.
>
> ## Hard rules
> - Runs on the host, not the device. The app's native libs are arm64-only and cannot load on the host JVM. That is why this tool links the Rust crate instead of loading the .so.
> - Do NOT fix any real drift the tool finds on day one. Report it to the human verbatim. Specifically: build the new Version3.kt snapshot by copying what the current V3 bootstrap effectively creates (see below), never by generating it from live Rust, otherwise drift gets masked.
> - Do not edit the Rust crates (client_table_blueprints, protocol, yrs).
> - Fail closed: any check that could silently find nothing (zero tables parsed, zero Rust blueprint fns found, regex matched nothing) must fail with a message that blames the tool's parsing, exit code 2.
>
> ## First: read before designing details
> - /home/zakke/ProgStuff/ZakkeN/client_table_blueprints/src/ (all of it): find the blueprint functions (new_table_pages, new_table_every_block_in_existence, new_table_client_love_letter, new_table_key_value_storage, new_table_backlinks, new_table_incoming_love_letters, new_table_logs, get_foreign_def_backlinks, get_foreign_def_every_block_in_existence, or whatever the real names are), their return types, and whether the crate defines table-name constants (if so, use them instead of string literals).
> - The Rust definition of ColumnDef and ForeignKeyDef in /home/zakke/ProgStuff/z_db/protocol (real field names, types, whether default_value is String or Option).
> - The generated Kotlin protocol.kt (app/src/main/java/uniffi/protocol/protocol.kt): ColumnDef/ForeignKeyDef constructor parameter order and defaults, and the variants of the sealed class Col (Integer/Text/Blob/Real/...).
> - The Kotlin files under app/src/main/java/z/zndroid/: db/schema/TableDescriptors.kt, db/migrations/SchemaVersion.kt, db/migrations/MigrationManager.kt, db/migrations/schemas/Version0-2.kt, Tests/{SchemaCanaryTest,SchemaSyncTest,ExpectedSchema,RegisterTests,DevCheckRegistry}.kt, test/.../BuildVerificationTest.kt.
> - android_app/build_and_install.sh, android_app/update.sh, ZakkeN/build_all_android.sh, android_app/app/build.gradle.kts.
> If you cannot read client_table_blueprints directly, ask the human to run `tree /home/zakke/ProgStuff/ZakkeN/client_table_blueprints` and paste the key files.
>
> ## Architecture
> Single Rust crate (lib + bin, so tests can live in tests/) in schema_drift_detector/. It must declare its own empty [workspace] in Cargo.toml so a parent workspace (if any) doesn't capture it. Add a .gitignore for target/. Dependencies: client_table_blueprints by relative path (../../client_table_blueprints), protocol by the SAME absolute path string client_table_blueprints uses (/home/zakke/ProgStuff/z_db/protocol) so the types unify, plus regex (and anything small you like).
>
> Internal normalized model, used for BOTH sides: Table { name, columns: ordered list, foreign_keys: ordered list }, Column { name, type string, primary_key, not_null, unique, default_value, autoincrement, plus source location (file:line) when known }, ForeignKey { column, ref_table, ref_column, location }. The Rust side converts protocol types into this model by field access (do not rely on Debug formatting). All diffing happens on this model.
>
> ### Rust side (the crux)
> A hand-maintained registry in the detector: list of (table name, Rust fn name as a string, fn pointer for columns, optional fn pointer for foreign keys). It calls the real blueprint functions directly, so there is no Rust source parsing. Keep registry order = table creation order (pages before FK tables).
> Check E, coverage: regex-scan client_table_blueprints/src/**/*.rs for `pub fn new_table_*` and `pub fn get_foreign_def_*` (adapt to the real naming convention). Any such fn missing from the registry fails with: "Rust defines <fn> (file:line) but the drift detector doesn't know it. Add it to the registry in src/<file>, and add the table to Version<N> snapshot + TableDescriptors.kt." Zero matches found = tool error (the pattern is stale). Also record each fn's file:line so reports can point at the Rust side.
> Known blind spot to document in the README: the table NAME is not part of blueprint output, so a Rust-side rename of a table is undetectable unless a name constant exists in the crate.
> If the crate cannot be built/linked for the host after reasonable effort, STOP and report the exact compiler error to the human instead of writing a Rust-source parser.
>
> ### Kotlin side: a small text parser (no Kotlin compiler)
> Write a string-aware scanner, not a pile of multi-line regexes: (1) blank out // and /* */ comments while preserving line numbers and not being fooled by comment markers inside strings; (2) find_calls(src, "Name"): locate identifier `Name` followed by `(`, return balanced-paren argument text, start line, and end index (so a trailing `{ lambda }` can be read); (3) split top-level args on commas respecting strings/parens; (4) parse literals (string, boolean, int).
> - ColumnDef(...): first two positional args are name and type; everything else by name (primaryKey, notNull, unique, defaultValue, autoincrement). Any unknown/missing/positional-extra arg = tool error with file:line. Mirror the constructor's real defaults from protocol.kt.
> - ForeignKeyDef("col","table","refcol"): positional, order as in protocol.kt.
> - FrozenTable(...) blocks (new, see Kotlin refactor): name, columns = listOf(ColumnDef...), foreignKeys = listOf(ForeignKeyDef...). Find ColumnDef/ForeignKeyDef calls inside each block's slice.
> - TableDescriptors.kt: each `object XTableSchema { ... }` containing TABLE_NAME; each `val ident = ColumnDescriptor("name", "TYPE") { (it as Col.Variant).v1 }` (capture ident, name, type, cast variant, line); `val allColumns = listOf(idents...)`. Resolve idents to descriptors, in allColumns order.
> - SchemaVersion.kt: `val LATEST = V<N>`; enum entry `V<N>(<N>) {` body (brace-matched).
>
> ## Checks (all failures are reported together, grouped by check, not just the first)
> A. Frozen latest snapshot vs live Rust. Which snapshot: LATEST -> N -> schemas/Version<N>.kt. Full fidelity. Drift kinds, each with its own wording:
>    - TABLE_MISSING_IN_KOTLIN / TABLE_ONLY_IN_KOTLIN
>    - COLUMN_ADDED_IN_RUST / COLUMN_REMOVED_FROM_RUST (include the position; if one removed + one added share type and index, append "(looks like a rename old -> new)")
>    - TYPE_CHANGED (TEXT in Rust, REAL in Kotlin)
>    - FLAG_CHANGED, one line per flag: primary key / not null / unique / default value / autoincrement
>    - ORDER_CHANGED (same column set, different order): show both orders and the first index where they diverge
>    - FK_ADDED / FK_REMOVED / FK_CHANGED (same column, different target) / FK_ORDER_CHANGED
>    - DUPLICATE_COLUMN inside a Kotlin table
>    Host strictness must be at least the device check's strictness (device compares lists with ==), hence FK order counts.
> B. TableDescriptors.kt vs live Rust: table coverage both ways (live table with no descriptor object; descriptor object for no live table); per table: names, types, order (names/types/order only; descriptors don't carry flags). Plus internal consistency: descriptor `val` not listed in allColumns (orphan), duplicate names, and cast-vs-type mismatch (declared INTEGER but unwrap casts Col.Text; build the type->Col variant map from protocol.kt; unknown type = loud failure). For COLUMN_ADDED here, print a ready-to-paste descriptor line (camelCase ident, correct cast) and the index to insert in allColumns.
> C. Version plumbing: LATEST parses; Version<N>.kt exists and yields >=1 table; the V<N> enum entry exists and its body contains the text `Version<N>.tables` (catches "bumped LATEST, forgot to repoint bootstrap"); optional: FK-referenced tables appear earlier in the snapshot's table order.
> D. Native lib freshness (skippable with --no-stale-check):
>    - android_output .so (client_table_blueprints/android_output/jniLibs/arm64-v8a/libclient_table_blueprints.so) older than the newest mtime of client_table_blueprints/src/** and Cargo.toml -> "STALE NATIVE LIB: <file> modified <time> is newer than the built .so (<time>). Run ZakkeN/build_all_android.sh."
>    - app/src/main/jniLibs/arm64-v8a copy missing, or sha256 differs from the android_output copy -> "update.sh hasn't been run since the last native build."
>    - Do NOT compare the app copy's mtime against sources (update.sh's cp resets it and would hide staleness). android_output missing entirely = WARN-level skip of the hash compare only if the app copy exists; app copy missing = fail.
> E. Rust coverage (above).
>
> ## Report format
> Header names the tool and counts problems. Grouped by check with a one-line description of what that check compares. Each drift line: `table.column` + kind + both values + `Kotlin: <relative path>:<line>` + `Rust: <fn name> (<file>:<line>)`. After each group, one "Fix:" hint. For group A: "Rust blueprints changed since Version<N> was frozen. Freeze a new version: run `schema_drift_detector/check.sh emit-snapshot <N+1>`, save as schemas/Version<N+1>.kt, add V<N+1> to SchemaVersion with bootstrap from Version<N+1>.tables, write V<N>.updateOnce migration returning V<N+1>, set LATEST = V<N+1>, update TableDescriptors.kt (group B lists exactly what), then run build_all_android.sh." Color only when stdout is a TTY and NO_COLOR is unset. Exit codes: 0 clean, 1 drift, 2 tool error (also must fail the build; message must make clear it's the tool's problem, not the schema's).
>
> ## CLI
> `check` (default). `emit-snapshot <N>`: prints a complete Version<N>.kt (package, imports, `object Version<N>` with `val tables: List<FrozenTable>`, in registry order) from the LIVE Rust schema, same format check A parses. Flags: --warn-only (print report, exit 0), --no-stale-check, --android-root / --blueprints-root overrides (defaults derived from CARGO_MANIFEST_DIR). Env: SKIP_SCHEMA_DRIFT=1 bypasses everything with a loud one-line warning; SCHEMA_DRIFT_CHECKED=1 is set by build scripts so the Gradle hook doesn't re-run.
> `check.sh` wrapper in the tool dir: finds cargo (source $HOME/.cargo/env if cargo not on PATH, so Android Studio launched from a desktop icon works), runs `cargo run --quiet --manifest-path <this dir>/Cargo.toml -- "$@"`, propagates the exit code. Cargo output must not drown the report. Expect a slow first run (host build of protocol/yrs/uniffi) and sub-second-ish warm runs; report both timings.
>
> ## Kotlin refactor (so there's exactly one definition of "latest")
> 1. New schemas/FrozenTable.kt: data class FrozenTable(name, columns: List<ColumnDef>, foreignKeys: List<ForeignKeyDef> = emptyList()).
> 2. New schemas/Version3.kt: `object Version3 { val tables: List<FrozenTable> = listOf(...) }`, a FULL snapshot, copied verbatim from what V3.bootstrap (-> V2.bootstrap) creates today, in the same order: key_value_storage (Version1), pages (V1), client_love_letter (V1), logs (V1), incoming_love_letters (V1), backlinks (V1) + its FK defs (V1), every_block_in_existence (Version2) + its FK def (V2). Header comment: frozen; to change the schema add Version<N+1>.kt via emit-snapshot. Leave Version0/1/2 untouched (migration history).
> 3. SchemaVersion.kt: add a helper that creates all tables of a snapshot (createTable when no FKs, createForeignTable otherwise, in list order). V3.bootstrap uses `Version3.tables` through it (the literal text `Version3.tables` must appear in V3's entry; check C depends on that). Add an overridable per-entry snapshot property so the device-side check can fetch LATEST's tables. Leave V0/V1/V2 bootstraps alone.
> 4. Rewrite assertLatestMatchesBlueprints to loop over LATEST's snapshot, fetch the live blueprint for each table by name via one `when` (unknown name -> loud IllegalStateException, all 7 tables + 2 FK defs covered), keep the skip-on-UnsatisfiedLinkError/NoClassDefFoundError behavior, and make SchemaMismatchException name the table and the first differing column/field instead of dumping two lists. Keep it as the device-side backstop (it validates the actually shipped .so).
> 5. Delete SchemaCanaryTest.kt and ExpectedSchema.kt (grep for every reference first, including MigrationTest and other tests; keep anything still used). Remove the canary from RegisterTests and DevCheckRegistry. In SchemaSyncTest drop "Check A" (code drift, superseded by the host tool), keep only the live-DB-vs-blueprint check, extend it to all 7 tables, and remove it from RegisterTests (it always skips on host; it stays in DevCheckRegistry).
> 6. TableDescriptors.kt stays as is; the tool parses it.
>
> ## Wiring
> - build_and_install.sh: right before the gradle assemble step, run check.sh; on nonzero, `fail` with "schema drift detected, see report above" (the report itself is the detailed message). Export SCHEMA_DRIFT_CHECKED=1 afterwards. Respect set -euo pipefail.
> - app/build.gradle.kts: an Exec task running <rootDir>/schema_drift_detector/check.sh, skipped when SCHEMA_DRIFT_CHECKED is set, and preBuild dependsOn it. This covers Android Studio and `./gradlew test`.
> - update.sh (also runs at the end of build_all_android.sh): after copying, run `check.sh --warn-only || true`, so the dev sees Kotlin work still needed right after rebuilding Rust libs, without blocking native rebuilds (building the libs is a legitimate step mid-schema-change). Do not hook the START of build_all_android.sh.
> - Short README in the tool dir: what each check compares, how to read a report, the schema-change workflow (from the Fix hint), how to add a table (registry + snapshot + descriptor), flags, and the blind spots: Rust-side table renames, stringly-typed column names in app code (recommend using descriptor `.name` over literals), migration-chain results (MigrationTest on device covers that).
>
> ## Verification (do all, don't just claim)
> 1. cargo test: unit tests for the scanner (comments, strings containing // and parens, nested listOf, trailing lambdas), the Kotlin parsers on fixture strings, and the differ (every drift kind above, including the rename hint and order-only change).
> 2. Run the real tool on the repo. Record the exact output. Clean or real drift are both acceptable outcomes; real drift goes to the human, unfixed.
> 3. Mutation tests via in-memory fixtures or a temp copy using --android-root: remove a column, add a column, change a type, swap two columns, flip notNull, change an FK target, remove a descriptor from allColumns, change a descriptor's type, change a descriptor's cast, bump LATEST without a snapshot file, repoint nothing in the V<N> entry. Each must exit 1 (or 2 where appropriate) with a message naming the specific drift.
> 4. One Rust-side mutation by hand: temporarily change a column in a blueprint, confirm the named report (and stale-lib check D firing), then revert and confirm the Rust tree is byte-identical to before.
> 5. Compile the Kotlin side (`./gradlew :app:compileDebugKotlin`, then `:app:testDebugUnitTest` if the environment allows) and `bash -n` every edited script. Check that the Gradle hook runs and that it doesn't double-run under the scripts.
>
> ## Report back to the human
> Day-one tool output verbatim; real Rust ColumnDef/ForeignKeyDef field names you found; any deviations from this plan and why; cold/warm run times; what was deleted; anything you couldn't verify (e.g. no SDK/device).
