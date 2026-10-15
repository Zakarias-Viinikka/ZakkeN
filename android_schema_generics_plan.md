# android_schema_generics_plan

This is a plan, not a pre-plan. It will get re-parsed into a pre-plan
when the work is ready to start. Loose on purpose.

## Why this exists

uniffi doesn't support generics. The schema crate
(client_table_blueprints) uses `SchemaColumn<T>` to encode each
column's type, where T is String, Vec<u8>, or similar. On the Rust
side that's fine. Across the uniffi boundary it isn't, so Android
can't use the generic helpers.

Today, Android has hand-written methods that unwrap a `Col` to a
specific type. If the schema drifts — a column changes from Text to
Blob, say — the schema const changes but Android's hand-written
methods silently keep expecting the old type. The mismatch shows up
at runtime, not compile time.

## The idea

A program that reads the schema files and checks them against a
declaration of what Android expects. If the schema has drifted, the
program reports it. Android's build depends on the program, so drift
is caught at build time.

The tool is a script that runs a program and takes some kind of JSON
argument. The android build just relies on it.

## What the tool does

- Walks a folder. Finds every file whose name starts with `tbl_`.
- Reads each file. Finds the `const` declarations of type
  `SchemaColumn<T>`.
- Extracts, per column: the column name (the `name:` field) and the
  type (either the `T` in `SchemaColumn<T>`, or the `TypeOfCol`
  variant, or both — see open questions).
- Takes an input — a JSON or similar — that says what Android
  expects. Something like "page_id is Text", "blobbed_page is Blob".
- Compares. Reports whether the schema is what was expected. "The
  schema is what you expected the schema to be", or it isn't and
  here's what changed.

## What the tool does NOT do

It does not generate the unwrap / destructure boilerplate. Android
writes those methods by hand. The tool only detects drift.

It does not enforce anything at runtime. It's a build-time check.

It does not replace the schema crate's Rust-side helpers. Those stay
as they are.

## How Android uses it

Android writes hand-written methods that unwrap a `Col` to an
expected type. Those methods expect a specific type. If the schema
drifts, the expected type is now wrong.

The tool is part of the android build script. When the schema drifts,
the build reports it. Android then updates its methods to match. That
should also break any methods that consume the unwrapped type,
because the type they were consuming has changed — which is exactly
what you want, because it tells you everything that needs updating.

Android can also write methods that check the unwrap expected type
against the schema expectations. This is the same idea expressed in
code: instead of the tool doing the comparison, Android's code does
it.

The user also described it as: "it can write code that doesn't
compile unless the schema matches up with what it expects the schema
to be." This may mean the tool generates a compile-time assertion
file. Whether the tool generates that file or Android writes it by
hand is an open question (see below).

## What is being accepted

Android becomes a second-class citizen. It can't use the generic
helpers. There's a lot of boilerplate for destructuring db columns.

The user doesn't touch Android code much and doesn't mind the
boilerplate. It's easy to verify because the idea is so simple: parse
the schema, compare against expectation, report drift.

## Files the tool reads

~~~
/home/zakke/ProgStuff/ZakkeN/client_table_blueprints/src/tbl_backlinks.rs
/home/zakke/ProgStuff/ZakkeN/client_table_blueprints/src/tbl_every_block_in_existence.rs
/home/zakke/ProgStuff/ZakkeN/client_table_blueprints/src/tbl_incoming_love_letters.rs
/home/zakke/ProgStuff/ZakkeN/client_table_blueprints/src/tbl_key_value_storage.rs
/home/zakke/ProgStuff/ZakkeN/client_table_blueprints/src/tbl_logs.rs
/home/zakke/ProgStuff/ZakkeN/client_table_blueprints/src/tbl_pages.rs
/home/zakke/ProgStuff/ZakkeN/client_table_blueprints/src/tbl_uncommitted_diffs.rs
/home/zakke/ProgStuff/ZakkeN/client_table_blueprints/src/lib.rs
/home/zakke/ProgStuff/ZakkeN/client_table_blueprints/src/new_row_helper.rs
/home/zakke/ProgStuff/ZakkeN/client_table_blueprints/Cargo.toml
/home/zakke/ProgStuff/ZakkeN/client_table_blueprints/uniffi.toml
/home/zakke/ProgStuff/ZakkeN/client_table_blueprints/BUILD_INSTRUCTS.txt
/home/zakke/ProgStuff/ZakkeN/client_table_blueprints/2026_09_25_temp_todo.md
/home/zakke/ProgStuff/ZakkeN/client_table_blueprints/BREAKING_CHANGES.md
/home/zakke/ProgStuff/ZakkeN/client_table_blueprints/android_output
~~~

## Related but separate

- /home/zakke/ProgStuff/z_db/add_new_method_to_wrapper — a different
  tool with a similar shape (a script that touches code that would
  otherwise be written by hand). Not the same problem, worth knowing
  about.
- /home/zakke/ProgStuff/z_db/db_wrapper_macro_explained — explains the
  macro core / per-target mascot system, which is the Rust-side
  version of this same "one definition, many targets" problem.
- /home/zakke/ProgStuff/z_db/db_wrapper — has per-target mascots
  (android_mascot, web_mascot, testing_mascot). The same fundamental
  problem — one definition, per-target implementations — is solved
  there with macros. Android can't use macros because uniffi is
  involved, which is why the schema-drift tool is needed at all.

## The schema shape the tool parses

From client_table_blueprints/src/tbl_pages.rs, the consts look like:

~~~
pub const PAGE_ID: SchemaColumn<String> = SchemaColumn::<String> {
    name: "page_id",
    type_of_col: &TypeOfCol::Text,
    can_be_null: false,
    _marker: PhantomData,
};
~~~

So the name is in `name:`, the Rust type is in `SchemaColumn<T>`,
and the db-facing type is in `type_of_col: &TypeOfCol::...`. Both
encodings are available. Which one the tool reads, and whether it
cross-checks them against each other, is an open question.

Every tbl_*.rs file also exports a `new_table_X()` fn that returns
`Vec<ColumnDef>`. That's the *actual* schema as the db sees it, and
it's a second source of truth the tool could cross-check against the
consts.

## Open questions

- Where does the JSON live? In the Android project? In the schema
  crate? Somewhere else?
- What's the exact JSON shape? Flat ("page_id: Text" per column), or
  something more structured?
- Does the tool read `type_of_col` (the db-facing type), the `T` in
  `SchemaColumn<T>` (the Rust type), or both? Should it check that
  they agree with each other?
- Should the tool also cross-check the consts against the
  `new_table_X()` fn in the same file? That fn is the real schema and
  the consts are a parallel declaration; if they disagree, that's
  also drift.
- Does the tool live in its own crate, or as a binary inside
  client_table_blueprints, or somewhere else?
- When drift is detected, does the build fail, or warn?
- Does the tool generate a compile-time assertion file ("code that
  doesn't compile unless the schema matches"), or does Android write
  that by hand?

## Not in scope

- No changes to the schema crate's Rust-side helpers.
- No changes to how the schema is written.
- No generation of unwrap / destructure methods.
- No runtime validation.
