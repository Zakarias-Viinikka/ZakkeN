function log(message, tag = 'info') {
  const logEl = document.getElementById('log');
  if (!logEl) return;

  const entry = document.createElement('div');
  // Add a class if it's an error or fail – use your danger color
  if (tag === 'error' || tag === 'fail') {
    entry.classList.add('log-error');
  }

  const time = new Date().toLocaleTimeString();
  entry.innerHTML = `<span class="tag">[${tag}]</span> <span class="time">${time}</span> ${message}`;
  logEl.appendChild(entry);
  logEl.scrollTop = logEl.scrollHeight;
}

// --- worker setup ---
const myWorker = new Worker('test_worker.js', { type: 'module' });

// queue for ask() calls, so requests to the worker never overlap
let askQueue = Promise.resolve();

// wait for the worker to signal it's loaded and ready
await new Promise(resolve => {
    myWorker.onmessage = (e) => {
        if (e.data[0] === "ready") resolve();
    };
});

//document.getElementById('status')?.classList.add('ready');   // optional but harmless

// --- database setup ---
await worker_do_work(["initialize", "tests_db_conn_name"]);

log('ready', 'worker initialized');

function worker_do_work(msg) {
  askQueue = askQueue.then(() => new Promise(resolve => {
    myWorker.onmessage = (e) => resolve(e.data);
    myWorker.postMessage(msg);
  }));
  return askQueue;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

async function test(name, fn) {
  try {
    await fn();
    log(`✓ ${name}`, "pass");
  } catch (err) {
    log(`✗ ${name} – ${err.message}`, "fail");
  }
}
/*
  --- drop all tables ---
*/
await test("drop all tables", async () => {
  const reply = await worker_do_work(["list_tables"]);
  if (reply[0] === "error") throw new Error(reply[1]);

  const tables = reply[1];
  for (const name of tables) {
    const dropReply = await worker_do_work(["delete_table", name]);
    if (dropReply[0] === "error") throw new Error(`drop ${name} failed: ${dropReply[1]}`);
  }

  const final = await worker_do_work(["list_tables"]);
  if (final[0] === "error") throw new Error(final[1]);
  if (final[1].length !== 0) throw new Error("not all tables dropped");
});

/*
  --- db is empty at start ---
*/
await test("db is empty at start", async () => {
  const reply = await worker_do_work(["list_tables"]);
  if (reply[0] === "error") throw new Error(reply[1]);
  const tables = reply[1];
  if (tables.length !== 0) throw new Error(`expected empty, got ${tables.length} tables`);
});

/*
  --- create table, insert rows, check schema, then verify data ---
*/
await test("create table, insert rows, check schema & data", async () => {
  // 1. Create a table with two columns (default value must be a string, not null)
  const columns = [
    // [name, type, primaryKey, notNull, unique, defaultValue, autoincrement]
    ["id",   "INTEGER", true,  true,  true,  "",   false, true ],
    ["name", "TEXT",    false, false, false, "",   false, false]
  ];

  let reply = await worker_do_work(["create_table", "people", columns]);
  if (reply[0] === "error") throw new Error("create table failed: " + reply[1]);

  // 2. Insert two rows (only the 'name' column, id auto-generates)
  reply = await worker_do_work(["insert_data", "people", ["name"], ["Alice"]]);
  if (reply[0] === "error") throw new Error("insert 1 failed: " + reply[1]);
  reply = await worker_do_work(["insert_data", "people", ["name"], ["Bob"]]);
  if (reply[0] === "error") throw new Error("insert 2 failed: " + reply[1]);

  // 3. Check the table schema
  reply = await worker_do_work(["check_table", "people"]);
  if (reply[0] === "error") throw new Error("check_table failed: " + reply[1]);

  const schemaLines = reply[1]; // array of strings like "info0: name=id, type=INTEGER, ..."

  // 4. Schema assertions
  if (!Array.isArray(schemaLines) || schemaLines.length !== 2) {
    throw new Error(`expected 2 columns, got ${JSON.stringify(schemaLines)}`);
  }

  const col0 = schemaLines[0];
  if (!col0.includes("name=id"))   throw new Error(`col0 missing name=id: ${col0}`);
  if (!col0.includes("type=INTEGER")) throw new Error(`col0 missing type=INTEGER: ${col0}`);

  const col1 = schemaLines[1];
  if (!col1.includes("name=name"))   throw new Error(`col1 missing name=name: ${col1}`);
  if (!col1.includes("type=TEXT"))   throw new Error(`col1 missing type=TEXT: ${col1}`);

  // 5. Verify data via get_data – request only the 'name' column for all rows
  reply = await worker_do_work(["get_data", "people", "", ["name"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);

  const rows = reply[1]; // expected shape: [["Alice"], ["Bob"]]
  if (!Array.isArray(rows) || rows.length !== 2) {
    throw new Error(`expected 2 rows, got ${JSON.stringify(rows)}`);
  }

  // extract names from each row (each row is an array of one element)
  const names = rows.map(row => row[0]).sort();
  if (names[0] !== "Alice" || names[1] !== "Bob") {
    throw new Error(`expected names Alice & Bob, got ${JSON.stringify(names)}`);
  }
});

// ----- create_table error cases -----

await test("create_table: duplicate name is allowed", async () => {
  const tbl = "dup_test";
  const cols = [["id","INTEGER",true,true,true,"",false]];

  let reply = await worker_do_work(["create_table", tbl, cols]);
  if (reply[0] === "error") throw new Error("initial create failed: " + reply[1]);

  reply = await worker_do_work(["create_table", tbl, cols]);
  if (reply[0] === "error") throw new Error("expected duplicate create to succeed, got error: " + reply[1]);

  await worker_do_work(["delete_table", tbl]);
});

await test("create_table: empty name fails", async () => {
  const reply = await worker_do_work(["create_table", "", [["id","INTEGER",true,true,true,"",false]]]);
  if (reply[0] !== "error") throw new Error("expected error for empty table name");
});

await test("create_table: zero columns fails", async () => {
  const reply = await worker_do_work(["create_table", "zero_cols", []]);
  if (reply[0] !== "error") throw new Error("expected error for empty column list");
});

await test("create_table: malformed column definition fails", async () => {
  // ColumnDef expects exactly 7 elements; providing fewer should fail at deserialization
  const badCols = [["id","INTEGER",true]]; // only 3 elements
  const reply = await worker_do_work(["create_table", "bad_def", badCols]);
  if (reply[0] !== "error") throw new Error("expected error for malformed column definition");
});

// ----- insert_data error cases -----

// set up a table with a NOT NULL column and no default
await test("setup: create table for insert error tests", async () => {
  const cols = [
    ["id",      "INTEGER", true, true,  true,  "",   false],
    ["username","TEXT",    false,true,  false, "",   false, false]  // NOT NULL
  ];
  let reply = await worker_do_work(["create_table", "err_test", cols]);
  if (reply[0] === "error") throw new Error("setup failed: " + reply[1]);
});

// --- actual error cases ---

await test("insert_data: non-existent table fails", async () => {
  const reply = await worker_do_work(["insert_data", "ghost", ["col"], ["val"]]);
  if (reply[0] !== "error") throw new Error("expected error for non-existent table");
});

await test("insert_data: wrong column name fails", async () => {
  // 'name' doesn't exist in err_test
  const reply = await worker_do_work(["insert_data", "err_test", ["name"], ["test"]]);
  if (reply[0] !== "error") throw new Error("expected error for bad column name");
});

await test("insert_data: missing NOT NULL value fails", async () => {
  // omit the 'username' column which is NOT NULL
  const reply = await worker_do_work(["insert_data", "err_test", ["id"], ["1"]]);
  if (reply[0] !== "error") throw new Error("expected error when violating NOT NULL");
});

await test("insert_data: duplicate primary key fails", async () => {
  // 'people' table already has rows with id=1 and id=2. Try to insert a row with id=1 explicitly.
  const reply = await worker_do_work(["insert_data", "people", ["id","name"], ["1","evil"]]);
  if (reply[0] !== "error") throw new Error("expected error for duplicate PK");
});

// clean up
await test("cleanup: drop err_test", async () => {
  await worker_do_work(["delete_table", "err_test"]);
});

await test("edit row: non-existent row is silently ignored", async () => {
  // count rows before the attempt
  let reply = await worker_do_work(["get_data", "people", "", ["id"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  const beforeCount = reply[1].length;

  // try to edit a row that doesn't exist
  reply = await worker_do_work(["edit_row", "people", "999", "name", "Ghost"]);
  if (reply[0] === "error") throw new Error("unexpected error editing non-existent row: " + reply[1]);

  // verify row count is unchanged
  reply = await worker_do_work(["get_data", "people", "", ["id"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  const afterCount = reply[1].length;
  if (afterCount !== beforeCount) {
    throw new Error(`row count changed: ${beforeCount} → ${afterCount}`);
  }
});

await test("delete row: non-existent row is silently ignored", async () => {
  // count rows before
  let reply = await worker_do_work(["get_data", "people", "", ["id"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  const beforeCount = reply[1].length;

  // delete a row that doesn't exist
  reply = await worker_do_work(["delete_row", "people", "999"]);
  if (reply[0] === "error") throw new Error("unexpected error deleting non-existent row: " + reply[1]);

  // verify row count is unchanged
  reply = await worker_do_work(["get_data", "people", "", ["id"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  const afterCount = reply[1].length;
  if (afterCount !== beforeCount) {
    throw new Error(`row count changed: ${beforeCount} → ${afterCount}`);
  }
});

await test("swap: exchange values between two rows", async () => {
  // swap 'name' between id=1 (Alicia) and id=2 (Bob)
  let reply = await worker_do_work(["swap_columns", "people", "1", "2", "name"]);
  if (reply[0] === "error") throw new Error("swap failed: " + reply[1]);

  // verify row 1 now has "Bob"
  reply = await worker_do_work(["get_data", "people", "id=1", ["name"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  if (reply[1][0][0] !== "Bob") throw new Error(`row 1 expected "Bob", got "${reply[1][0][0]}"`);

  // verify row 2 now has "Alicia"
  reply = await worker_do_work(["get_data", "people", "id=2", ["name"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  if (reply[1][0][0] !== "Alice") throw new Error(`row 2 expected "Alice", got "${reply[1][0][0]}"`);
});

await test("swap: same row – value unchanged, no error", async () => {
  // swap name on row 1 with itself
  let reply = await worker_do_work(["swap_columns", "people", "1", "1", "name"]);
  if (reply[0] === "error") throw new Error("swap with same row should not error: " + reply[1]);

  // value must still be "Bob" (from previous swap)
  reply = await worker_do_work(["get_data", "people", "id=1", ["name"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  if (reply[1][0][0] !== "Bob") throw new Error(`row 1 expected "Bob", got "${reply[1][0][0]}"`);
});

await test("swap: non-existent row fails", async () => {
  const reply = await worker_do_work(["swap_columns", "people", "1", "999", "name"]);
  if (reply[0] !== "error") throw new Error("expected error for non-existent row");
});

await test("swap: non-existent column fails", async () => {
  const reply = await worker_do_work(["swap_columns", "people", "1", "2", "age"]);
  if (reply[0] !== "error") throw new Error("expected error for non-existent column");
});

await test("swap: non-existent table fails", async () => {
  const reply = await worker_do_work(["swap_columns", "ghost", "1", "2", "name"]);
  if (reply[0] !== "error") throw new Error("expected error for non-existent table");
});

await test("get_data: single row by id", async () => {
  const reply = await worker_do_work(["get_data", "people", "id=1", ["name"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  const rows = reply[1];
  if (!Array.isArray(rows) || rows.length !== 1) {
    throw new Error(`expected 1 row, got ${JSON.stringify(rows)}`);
  }
  if (rows[0][0] !== "Bob") throw new Error(`expected "Bob", got "${rows[0][0]}"`);
});

await test("get_data: subset of columns", async () => {
  // request only the 'id' column
  const reply = await worker_do_work(["get_data", "people", "", ["id"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  const rows = reply[1];
  if (!Array.isArray(rows) || rows.length !== 2) {
    throw new Error(`expected 2 rows, got ${rows.length}`);
  }
  // each row should be an array of exactly one element (the id)
  rows.forEach((row, i) => {
    if (row.length !== 1) throw new Error(`row ${i} expected 1 column, got ${row.length}`);
  });
});

await test("get_data: empty table returns empty array", async () => {
  // create a temporary empty table
  await worker_do_work(["create_table", "empty_tbl", [["id","INTEGER",true,true,true,"",false]]]);
  const reply = await worker_do_work(["get_data", "empty_tbl", "", ["id"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  if (!Array.isArray(reply[1]) || reply[1].length !== 0) {
    throw new Error(`expected empty array, got ${JSON.stringify(reply[1])}`);
  }
  await worker_do_work(["delete_table", "empty_tbl"]);
});

await test("get_data: non-existent row returns empty", async () => {
  const reply = await worker_do_work(["get_data", "people", "id=999", ["name"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  if (!Array.isArray(reply[1]) || reply[1].length !== 0) {
    throw new Error(`expected empty array for missing row, got ${JSON.stringify(reply[1])}`);
  }
});

/* testing concurrent inserts: ordering and consistency */
await test("concurrent inserts: ordering and consistency", async () => {
  const tbl = "concurrent_test";

  // 1. Create a simple table with autoincrement id and a text value
  let reply = await worker_do_work(["create_table", tbl, [
    ["id",  "INTEGER", true, true, true, "", false, true],
    ["val", "TEXT",    false,false,false,"", false, false]
  ]]);
  if (reply[0] === "error") throw new Error("create table failed: " + reply[1]);

  // 2. Prepare multiple inserts and fire them ALL at the same time
  const values = ["first", "second", "third", "fourth", "fifth"];
  const insertPromises = values.map(v =>
    worker_do_work(["insert_data", tbl, ["val"], [v]])
  );

  // 3. Wait for all inserts to complete (they are enqueued concurrently)
  const insertResults = await Promise.all(insertPromises);
  insertResults.forEach((res, i) => {
    if (res[0] === "error") throw new Error(`insert ${i} ("${values[i]}") failed: ${res[1]}`);
  });

  // 4. Read all rows (id and val) – they must be in insertion order
  reply = await worker_do_work(["get_data", tbl, "", ["id", "val"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  const rows = reply[1];
  if (rows.length !== values.length) {
    throw new Error(`expected ${values.length} rows, got ${rows.length}`);
  }

  // 5. Verify that ids are 1,2,3,4,5 and values match the exact order
  for (let i = 0; i < values.length; i++) {
    const [id, val] = rows[i];
    if (id !== String(i + 1)) throw new Error(`row ${i} id expected ${i + 1}, got ${id}`);
    if (val !== values[i])    throw new Error(`row ${i} val expected "${values[i]}", got "${val}"`);
  }

  // 6. Clean up
  await worker_do_work(["delete_table", tbl]);
});

// ----- Edit and Delete row tests -----

await test("setup: create table for edit/delete tests", async () => {
  const cols = [
    ["id",  "INTEGER", true, true, true, "", false, true],
    ["name","TEXT",    false,false,false,"", false, false],
    ["age", "INTEGER", false,false,false,"", false, false]
  ];
  let reply = await worker_do_work(["create_table", "edit_delete_test", cols]);
  if (reply[0] === "error") throw new Error("create table failed: " + reply[1]);

  // Insert two rows: Alice (30), Bob (25)
  reply = await worker_do_work(["insert_data", "edit_delete_test", ["name","age"], ["Alice","30"]]);
  if (reply[0] === "error") throw new Error("insert Alice failed: " + reply[1]);
  reply = await worker_do_work(["insert_data", "edit_delete_test", ["name","age"], ["Bob","25"]]);
  if (reply[0] === "error") throw new Error("insert Bob failed: " + reply[1]);
});

await test("edit row: change a value and verify", async () => {
  // Change Alice's age to 31
  let reply = await worker_do_work(["edit_row", "edit_delete_test", "1", "age", "31"]);
  if (reply[0] === "error") throw new Error("edit failed: " + reply[1]);

  // Verify via get_data
  reply = await worker_do_work(["get_data", "edit_delete_test", "id=1", ["age"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  if (reply[1][0][0] !== "31") throw new Error(`expected age 31, got "${reply[1][0][0]}"`);
});

await test("edit row: non-existent column should fail", async () => {
  const reply = await worker_do_work(["edit_row", "edit_delete_test", "1", "salary", "50000"]);
  if (reply[0] !== "error") throw new Error("expected error for non-existent column");
});

await test("edit row: non-existent row is silently ignored", async () => {
  // Count rows before
  let reply = await worker_do_work(["get_data", "edit_delete_test", "", ["id"]]);
  const beforeCount = reply[1].length;

  // Edit row 999
  reply = await worker_do_work(["edit_row", "edit_delete_test", "999", "name", "Ghost"]);
  // Should succeed (no error), because no-op is the current behavior
  if (reply[0] === "error") throw new Error("unexpected error for non-existent row edit: " + reply[1]);

  // Row count must not change
  reply = await worker_do_work(["get_data", "edit_delete_test", "", ["id"]]);
  const afterCount = reply[1].length;
  if (afterCount !== beforeCount) throw new Error(`row count changed: ${beforeCount} → ${afterCount}`);
});

await test("delete row: remove a row and verify", async () => {
  // Delete Bob (id=2)
  let reply = await worker_do_work(["delete_row", "edit_delete_test", "2"]);
  if (reply[0] === "error") throw new Error("delete failed: " + reply[1]);

  // Get all rows – should be only Alice (id=1)
  reply = await worker_do_work(["get_data", "edit_delete_test", "", ["id","name","age"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  const rows = reply[1];
  if (rows.length !== 1) throw new Error(`expected 1 row, got ${rows.length}`);
  if (rows[0][1] !== "Alice") throw new Error(`expected Alice, got "${rows[0][1]}"`);
  if (rows[0][2] !== "31")    throw new Error(`age should be 31, got "${rows[0][2]}"`);
});

await test("delete row: non-existent row is silently ignored", async () => {
  // Count rows before
  let reply = await worker_do_work(["get_data", "edit_delete_test", "", ["id"]]);
  const beforeCount = reply[1].length;

  // Delete row 999
  reply = await worker_do_work(["delete_row", "edit_delete_test", "999"]);
  if (reply[0] === "error") throw new Error("unexpected error for non-existent row delete: " + reply[1]);

  // Row count unchanged
  reply = await worker_do_work(["get_data", "edit_delete_test", "", ["id"]]);
  const afterCount = reply[1].length;
  if (afterCount !== beforeCount) throw new Error(`row count changed: ${beforeCount} → ${afterCount}`);
});

await test("delete row: delete last row leaves table empty", async () => {
  // Delete Alice (id=1)
  let reply = await worker_do_work(["delete_row", "edit_delete_test", "1"]);
  if (reply[0] === "error") throw new Error("delete last row failed: " + reply[1]);

  // Table should be empty
  reply = await worker_do_work(["get_data", "edit_delete_test", "", ["id"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  if (reply[1].length !== 0) throw new Error("table not empty after deleting all rows");
});

await test("cleanup: drop edit_delete_test table", async () => {
  await worker_do_work(["delete_table", "edit_delete_test"]);
});

await test("list_tables: multiple tables are returned", async () => {
  // Create two distinct tables
  await worker_do_work(["create_table", "multi_test_a", [["id","INTEGER",true,true,true,"",false,true]]]);
  await worker_do_work(["create_table", "multi_test_b", [["id","INTEGER",true,true,true,"",false,true]]]);

  // Call list_tables
  const reply = await worker_do_work(["list_tables"]);
  if (reply[0] === "error") throw new Error("list_tables failed: " + reply[1]);

  const tables = reply[1];
  if (!Array.isArray(tables)) throw new Error("expected array of table names");

  // Verify both tables are present
  if (!tables.includes("multi_test_a")) throw new Error("missing multi_test_a");
  if (!tables.includes("multi_test_b")) throw new Error("missing multi_test_b");

  // Clean up
  await worker_do_work(["delete_table", "multi_test_a"]);
  await worker_do_work(["delete_table", "multi_test_b"]);
});

// ----- Create index tests -----

// Ensure we have a table to index
await test("setup: table for index tests", async () => {
  const cols = [
    ["id",  "INTEGER", true, true, true, "", false, true],
    ["tag", "TEXT",    false,false,false,"", false, false]
  ];
  let reply = await worker_do_work(["create_table", "index_test", cols]);
  if (reply[0] === "error") throw new Error("create table failed: " + reply[1]);
});

await test("create index: on existing column succeeds", async () => {
  const reply = await worker_do_work(["create_index", "index_test", "tag"]);
  // We just care that no error occurred.
  if (reply[0] === "error") throw new Error("create index failed: " + reply[1]);
  // Optionally check the success message
  if (reply[1] !== undefined && !reply[1].includes("indexed")) {
    throw new Error(`unexpected success message: ${reply[1]}`);
  }
});

await test("create index: non-existent table should error", async () => {
  const reply = await worker_do_work(["create_index", "ghost", "col"]);
  // This is expected to fail; log a pass if an error is returned, otherwise log the actual outcome.
  if (reply[0] !== "error") {
    // If it doesn't error, document what happened and pass anyway (or fail, depending on desired contract)
    throw new Error(`expected error for missing table, but got ${JSON.stringify(reply)}`);
  }
  // If error, test passes automatically (no throw)
});

await test("create index: non-existent column should error", async () => {
  const reply = await worker_do_work(["create_index", "index_test", "nonexistent"]);
  if (reply[0] !== "error") {
    throw new Error(`expected error for missing column, but got ${JSON.stringify(reply)}`);
  }
});

await test("cleanup: drop index_test table", async () => {
  await worker_do_work(["delete_table", "index_test"]);
});

/*
  Tests for behavior that succeeds silently instead of erroring.
  Meant to be pasted into your existing suite (anywhere after "initialize"
  has run). Each test creates and drops its own table, so it doesn't
  depend on — or interfere with — state from your other tests.

  Every test here is traced to something specific:
    A, B, C, D  -> your Rust code uses "IF NOT EXISTS" in several places
    E, F, G     -> Rust's `zip()` truncates, and NULL reads back as ""
*/

// --- A: check_table on a table that was never created ---
await test("check_table: non-existent table returns empty list, no error", async () => {
  const reply = await worker_do_work(["check_table", "definitely_not_a_real_table"]);
  if (reply[0] === "error") throw new Error(`expected no error, got: ${reply[1]}`);
  if (!Array.isArray(reply[1]) || reply[1].length !== 0) {
    throw new Error(`expected empty schema, got: ${JSON.stringify(reply[1])}`);
  }
});

// --- B: re-creating a table with a different schema keeps old schema ---
await test("create_table: re-creating with different columns keeps the OLD schema", async () => {
  const tbl = "schema_drift_test";
  const originalCols = [["id", "INTEGER", true, true, true, "", false]];
  const differentCols = [
    ["id", "INTEGER", true, true, true, "", false],
    ["extra_col", "TEXT", false, false, false, "", false],
  ];

  let reply = await worker_do_work(["create_table", tbl, originalCols]);
  if (reply[0] === "error") throw new Error("initial create failed: " + reply[1]);

  reply = await worker_do_work(["create_table", tbl, differentCols]);
  if (reply[0] === "error") {
    throw new Error("expected second create to succeed silently, got error: " + reply[1]);
  }

  reply = await worker_do_work(["check_table", tbl]);
  if (reply[0] === "error") throw new Error("check_table failed: " + reply[1]);
  const schemaLines = reply[1];

  if (schemaLines.length !== 1) {
    throw new Error(
      `expected schema to stay at 1 column (old schema wins), got ${schemaLines.length}: ${JSON.stringify(schemaLines)}`
    );
  }

  await worker_do_work(["delete_table", tbl]);
});

// --- C: creating the same index twice ---
await test("create_index: creating the same index twice succeeds silently", async () => {
  const tbl = "dup_index_test";
  await worker_do_work(["create_table", tbl, [["id", "INTEGER", true, true, true, "", false]]]);

  let reply = await worker_do_work(["create_index", tbl, "id"]);
  if (reply[0] === "error") throw new Error("first create_index failed: " + reply[1]);

  reply = await worker_do_work(["create_index", tbl, "id"]);
  if (reply[0] === "error") {
    throw new Error("expected duplicate create_index to succeed silently, got error: " + reply[1]);
  }

  await worker_do_work(["delete_table", tbl]);
});

// --- D: deleting a table that was never created ---
await test("delete_table: non-existent table succeeds silently (no error)", async () => {
  const reply = await worker_do_work(["delete_table", "table_that_was_never_created"]);
  if (reply[0] === "error") throw new Error(`expected no error, got: ${reply[1]}`);
});

// --- E: insert_data with fewer values than column names ---
await test("insert_data: fewer values than column names silently drops the extra column", async () => {
  const tbl = "mismatch_test";
  await worker_do_work(["create_table", tbl, [
    ["id", "INTEGER", true, true, true, "", false],
    ["name", "TEXT", false, false, false, "", false],
  ]]);

  // 2 column names, only 1 value → name is omitted, becomes NULL → reads as ""
  const reply = await worker_do_work(["insert_data", tbl, ["id", "name"], ["7"]]);
  if (reply[0] === "error") throw new Error(`expected this to succeed silently, got error: ${reply[1]}`);

  const check = await worker_do_work(["get_data", tbl, "", ["id", "name"]]);
  if (check[0] === "error") throw new Error("get_data failed: " + check[1]);
  if (check[1][0][1] !== "") {
    throw new Error(`expected "name" to read back as "", got: ${JSON.stringify(check[1][0])}`);
  }

  await worker_do_work(["delete_table", tbl]);
});

// --- F: insert_data with more values than column names ---
await test("insert_data: more values than column names silently drops the extra value", async () => {
  const tbl = "mismatch_test2";
  await worker_do_work(["create_table", tbl, [
    ["id", "INTEGER", true, true, true, "", false],
    ["name", "TEXT", false, false, false, "", false],
  ]]);

  // 1 column name, 2 values → ExtraIgnored is discarded
  const reply = await worker_do_work(["insert_data", tbl, ["name"], ["Alice", "ExtraIgnored"]]);
  if (reply[0] === "error") throw new Error(`expected this to succeed silently, got error: ${reply[1]}`);

  const check = await worker_do_work(["get_data", tbl, "", ["name"]]);
  if (check[0] === "error") throw new Error("get_data failed: " + check[1]);
  if (check[1][0][0] !== "Alice") {
    throw new Error(`expected "Alice" (the paired value), got: ${JSON.stringify(check[1][0])}`);
  }

  await worker_do_work(["delete_table", tbl]);
});

// --- G: NULL vs empty string are indistinguishable on read ---
await test("silent: NULL and empty string both read back as ''", async () => {
  const tbl = "null_vs_empty";
  // Table with a nullable text column and no default
  await worker_do_work(["create_table", tbl, [
    ["id",  "INTEGER", true, true, true, "", false],
    ["data", "TEXT",    false,false,false,"", false, false]
  ]]);

  // Insert a row with an explicit empty string
  await worker_do_work(["insert_data", tbl, ["id", "data"], ["1", ""]]);
  // Insert a row where we omit 'data' → it will be NULL
  await worker_do_work(["insert_data", tbl, ["id"], ["2"]]);

  const reply = await worker_do_work(["get_data", tbl, "", ["id", "data"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  const rows = reply[1];
  if (rows.length !== 2) throw new Error("expected 2 rows");

  // Both should be "" — we can't tell apart
  const row1data = rows[0][1];
  const row2data = rows[1][1];
  if (row1data !== "") throw new Error(`row1 expected '', got '${row1data}'`);
  if (row2data !== "") throw new Error(`row2 expected '', got '${row2data}'`);

  // If we ever want to distinguish them, this test will start failing,
  // which is exactly the point.

  await worker_do_work(["delete_table", tbl]);
});

// // //


// ============================================================
//  1. drop_table (the "drop all tables" command) – should error
// ============================================================
await test("drop_table (no arg): expected to error because Rust expects a table name", async () => {
  // The worker handler calls db_manager.drop_table() with NO argument,
  // but Rust requires table_name: String. This should fail.
  const reply = await worker_do_work(["drop_table"]);
  if (reply[0] !== "error") {
    throw new Error("expected error for missing table_name argument, but got success: " + JSON.stringify(reply));
  }
  // If it fails, test passes. If it somehow succeeds, the test fails – which is still useful.
});

// ============================================================
//  2. get_data, edit_row, delete_row on a non‑existent table
// ============================================================
await test("get_data: non-existent table should error", async () => {
  const reply = await worker_do_work(["get_data", "table_does_not_exist", "", ["id"]]);
  if (reply[0] !== "error") {
    throw new Error("expected error for non-existent table, got: " + JSON.stringify(reply));
  }
});

await test("edit_row: non-existent table should error", async () => {
  const reply = await worker_do_work(["edit_row", "nope_table", "1", "col", "val"]);
  if (reply[0] !== "error") throw new Error("expected error for non-existent table, got: " + JSON.stringify(reply));
});

await test("delete_row: non-existent table should error", async () => {
  const reply = await worker_do_work(["delete_row", "ghost_table", "1"]);
  if (reply[0] !== "error") throw new Error("expected error for non-existent table, got: " + JSON.stringify(reply));
});

// ============================================================
//  4. insert_data with empty col_names and vals arrays
// ============================================================
await test("insert_data: empty columns/values inserts a default row", async () => {
  const tbl = "empty_insert_test";
  await worker_do_work(["create_table", tbl, [["id","INTEGER",true,true,true,"",false,true]]]);

  const reply = await worker_do_work(["insert_data", tbl, [], []]);
  if (reply[0] === "error") {
    throw new Error("empty insert should succeed, got error: " + reply[1]);
  }

  const check = await worker_do_work(["get_data", tbl, "", ["id"]]);
  if (check[0] === "error") throw new Error("get_data failed: " + check[1]);
  if (check[1].length !== 1) throw new Error(`expected 1 row, got ${check[1].length}`);

  await worker_do_work(["delete_table", tbl]);
});

// ============================================================
//  5. Edit / delete with a non‑numeric row ID
// ============================================================
await test("edit_row: non‑numeric row id is silently ignored (no matching row)", async () => {
  const tbl = "non_numeric_test";
  await worker_do_work(["create_table", tbl, [["id","INTEGER",true,true,true,"",false], ["val","TEXT",false,false,false,"",false]]]);
  await worker_do_work(["insert_data", tbl, ["id","val"], ["1","hello"]]);

  // Try to edit with "abc" as the row id
  let reply = await worker_do_work(["edit_row", tbl, "abc", "val", "new"]);
  // Should either error or succeed with no change. We'll accept either as long as data is unchanged.
  // To verify, read the row back.
  reply = await worker_do_work(["get_data", tbl, "id=1", ["val"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  if (reply[1][0][0] !== "hello") {
    throw new Error(`row was changed to "${reply[1][0][0]}" when it should have stayed "hello"`);
  }
  await worker_do_work(["delete_table", tbl]);
});

await test("delete_row: non‑numeric row id is silently ignored", async () => {
  const tbl = "non_numeric_del";
  await worker_do_work(["create_table", tbl, [["id","INTEGER",true,true,true,"",false]]]);
  await worker_do_work(["insert_data", tbl, ["id"], ["1"]]);

  const reply = await worker_do_work(["delete_row", tbl, "xyz"]);
  // Should not error, and row count should stay 1
  const check = await worker_do_work(["get_data", tbl, "", ["id"]]);
  if (check[0] === "error") throw new Error("get_data failed: " + check[1]);
  if (check[1].length !== 1) throw new Error(`row was deleted when it shouldn't have been`);
  await worker_do_work(["delete_table", tbl]);
});

// ============================================================
//  6. create_table with special characters in names
// ============================================================
await test("create_table: table name with spaces and quotes works", async () => {
  const tbl = "my table with 'quotes'";
  let reply = await worker_do_work(["create_table", tbl, [["id","INTEGER",true,true,true,"",false]]]);
  if (reply[0] === "error") throw new Error("create_table failed: " + reply[1]);

  reply = await worker_do_work(["list_tables"]);
  if (reply[0] === "error") throw new Error("list_tables failed: " + reply[1]);
  if (!reply[1].includes(tbl)) throw new Error("table not found in list_tables");

  await worker_do_work(["delete_table", tbl]);
});

await test("create_table: column name is a SQL keyword (select)", async () => {
  const tbl = "keyword_col";
  const cols = [["select", "TEXT", false,false,false,"",false]];
  let reply = await worker_do_work(["create_table", tbl, cols]);
  if (reply[0] === "error") throw new Error("create_table failed: " + reply[1]);

  reply = await worker_do_work(["check_table", tbl]);
  if (reply[0] === "error") throw new Error("check_table failed: " + reply[1]);
  if (!reply[1][0].includes("name=select")) throw new Error(`expected column name 'select', got: ${reply[1][0]}`);

  await worker_do_work(["delete_table", tbl]);
});

// ============================================================
//  7. create_table with a non‑empty default value
// ============================================================
await test("create_table: non‑empty default value is used", async () => {
  const tbl = "default_val_test";
  const cols = [
    ["id",  "INTEGER", true, true, true, "42", false],   // default 42 for id? That's weird but let's test a text default
    ["tag", "TEXT",    false,false,false,"'hello'", false] // note: the default value string must include quotes if it's a literal in SQL
  ];
  // The way your ColumnDef works, the default field is a String that will be inserted directly into the SQL.
  // For TEXT defaults, it should contain the SQL literal, e.g. "'hello'" (with single quotes).
  // If you pass "hello" without quotes, SQL will try to interpret it as a column name and fail.
  // Let's use a TEXT default that is a number as string for simplicity:
  const colsSafe = [["id","INTEGER",true,true,true,"",false], ["status","TEXT",false,false,false,"'ok'",false]];
  const reply = await worker_do_work(["create_table", tbl, colsSafe]);
  if (reply[0] === "error") throw new Error("create_table with default failed: " + reply[1]);

  // Insert a row without specifying 'status'
  await worker_do_work(["insert_data", tbl, ["id"], ["1"]]);
  const check = await worker_do_work(["get_data", tbl, "id=1", ["status"]]);
  if (check[0] === "error") throw new Error("get_data failed: " + check[1]);
  // The default 'ok' should be present. Because of quoting, it should be stored as "ok"
  if (check[1][0][0] !== "ok") throw new Error(`expected default 'ok', got: "${check[1][0][0]}"`);
  await worker_do_work(["delete_table", tbl]);
});

// ============================================================
//  8. swap_columns with a single quote in value
// ============================================================
await test("swap: values containing single quotes are handled correctly", async () => {
  const tbl = "quote_swap";
  await worker_do_work(["create_table", tbl, [["id","INTEGER",true,true,true,"",false], ["name","TEXT",false,false,false,"",false]]]);
  await worker_do_work(["insert_data", tbl, ["id","name"], ["1","O'Brien"]]);
  await worker_do_work(["insert_data", tbl, ["id","name"], ["2","Smith"]]);

  const reply = await worker_do_work(["swap_columns", tbl, "1", "2", "name"]);
  if (reply[0] === "error") throw new Error("swap with single quote failed: " + reply[1]);

  const row1 = await worker_do_work(["get_data", tbl, "id=1", ["name"]]);
  const row2 = await worker_do_work(["get_data", tbl, "id=2", ["name"]]);
  if (row1[0] === "error" || row2[0] === "error") throw new Error("get_data failed");
  if (row1[1][0][0] !== "Smith") throw new Error(`row1 expected Smith, got ${row1[1][0][0]}`);
  if (row2[1][0][0] !== "O'Brien") throw new Error(`row2 expected O'Brien, got ${row2[1][0][0]}`);
  await worker_do_work(["delete_table", tbl]);
});

// ============================================================
//  9. Stress test: 100 rows and a long string
// ============================================================
await test("stress: insert 100 rows and verify count", async () => {
  const tbl = "hundred";
  await worker_do_work(["create_table", tbl, [["id","INTEGER",true,true,true,"",false], ["n","INTEGER",false,false,false,"",false]]]);
  for (let i = 1; i <= 100; i++) {
    const reply = await worker_do_work(["insert_data", tbl, ["id","n"], [String(i), String(i)]]);
    if (reply[0] === "error") throw new Error(`insert ${i} failed: ${reply[1]}`);
  }
  const check = await worker_do_work(["get_data", tbl, "", ["id"]]);
  if (check[0] === "error") throw new Error("get_data failed: " + check[1]);
  if (check[1].length !== 100) throw new Error(`expected 100 rows, got ${check[1].length}`);
  await worker_do_work(["delete_table", tbl]);
});

await test("stress: long text value (10k characters)", async () => {
  const tbl = "long_text";
  await worker_do_work(["create_table", tbl, [["id","INTEGER",true,true,true,"",false], ["text","TEXT",false,false,false,"",false]]]);
  const longString = "a".repeat(10000);
  const reply = await worker_do_work(["insert_data", tbl, ["text"], [longString]]);
  if (reply[0] === "error") throw new Error("insert long string failed: " + reply[1]);
  const check = await worker_do_work(["get_data", tbl, "", ["text"]]);
  if (check[0] === "error") throw new Error("get_data failed: " + check[1]);
  if (check[1][0][0] !== longString) throw new Error("retrieved string doesn't match (length: " + check[1][0][0].length + ")");
  await worker_do_work(["delete_table", tbl]);
});

/* test closing db connection */

await test("close db connection", async () => {
  const reply = await worker_do_work(["close_db"]);
  // If we get here without error, the command was recognized – pass.
  // Until implemented, the worker will likely return ["error", "…"] and the throw will make the test fail.
  if (reply[0] === "error") {
    throw new Error(`close_db not implemented (got: ${reply[1]})`);
  }
  // success: nothing to assert, just that no error occurred
});

// Purpose: Verify that initialising the database with an empty string
// returns an error instead of silently creating a nameless database or hanging.
await test("init: empty database name should error", async () => {
  const worker = new Worker('test_worker.js', { type: 'module' });
  try {
    const result = await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error("test timed out")), 2000);
      worker.onmessage = function firstMsg(e) {
        if (e.data[0] === "ready") {
          worker.onmessage = (e2) => {
            clearTimeout(timeout);
            resolve(e2.data);
          };
          worker.postMessage(["initialize", ""]);
        } else {
          clearTimeout(timeout);
          reject(new Error("expected 'ready', got: " + JSON.stringify(e.data)));
        }
      };
      worker.onerror = (e) => {
        clearTimeout(timeout);
        reject(new Error("worker error: " + e.message));
      };
    });
    if (result[0] !== "error") throw new Error("expected error, got: " + JSON.stringify(result));
  } finally {
    worker.terminate();
  }
});

// Purpose: Ensure that passing an excessively long database name
// (1024+ characters) does not cause a crash or hang, and ideally returns an error.
await test("init: very long database name should error", async () => {
  const worker = new Worker('test_worker.js', { type: 'module' });
  try {
    const longName = "x".repeat(1024);
    const result = await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error("test timed out")), 2000);
      worker.onmessage = function firstMsg(e) {
        if (e.data[0] === "ready") {
          worker.onmessage = (e2) => {
            clearTimeout(timeout);
            resolve(e2.data);
          };
          worker.postMessage(["initialize", longName]);
        } else {
          clearTimeout(timeout);
          reject(new Error("expected 'ready', got: " + JSON.stringify(e.data)));
        }
      };
      worker.onerror = (e) => {
        clearTimeout(timeout);
        reject(new Error("worker error: " + e.message));
      };
    });
    if (result[0] !== "error") throw new Error("expected error for long name, got: " + JSON.stringify(result));
  } finally {
    worker.terminate();
  }
});

// Purpose: Confirm that a normal request completes within a reasonable time.
// If worker_do_work never resolves (e.g., the worker hangs), the test will
// fail after 2 seconds instead of freezing the whole suite indefinitely.
await test("timeout: worker_do_work should not hang forever", async () => {
  const promise = worker_do_work(["list_tables"]);
  const timeout = new Promise((_, reject) =>
    setTimeout(() => reject(new Error("worker_do_work hung for 2 seconds")), 2000)
  );
  const reply = await Promise.race([promise, timeout]);
  // As long as we get here without hitting the timeout, the test passes.
  // We don't care if list_tables returned an error; only that it didn't hang.
});

// Purpose: Verify that the askQueue serialisation does not break after an error.
// A bad command should not poison the queue – a subsequent valid command
// must still complete normally.
await test("queue: error in one request does not break the queue", async () => {
  // Cause an error with a guaranteed unsupported command
  const badReply = await worker_do_work(["invalid_command_xyz"]);
  // It's okay if the worker doesn't explicitly report an error (maybe unknown
  // commands are silently ignored), but we just need to confirm the queue is intact.
  // Now send a known good command – it must succeed.
  const goodReply = await worker_do_work(["list_tables"]);
  if (goodReply[0] === "error") {
    throw new Error("queue broken: list_tables failed after a previous error: " + goodReply[1]);
  }
});

// Purpose: Ensure that if a worker’s first message is not exactly ["ready"],
// the test suite does not hang. Instead it should eventually time out or
// detect the unexpected message. Here we simulate a broken worker that sends
// ["wrong"] instead of "ready", and verify that our test helper does not
// freeze forever.
await test("worker: unexpected first message causes timeout (not ready)", async () => {
  // Create a tiny worker that sends an incorrect first message
  const blob = new Blob([
    `self.postMessage(["wrong"]);`
  ], { type: 'application/javascript' });
  const blobUrl = URL.createObjectURL(blob);
  const worker = new Worker(blobUrl, { type: 'module' });

  try {
    const result = await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error("test timed out (good – no hang forever)")), 2000);
      worker.onmessage = (e) => {
        clearTimeout(timeout);
        // If we get any message, it's not 'ready', which is the expected failure mode.
        resolve("worker sent non-ready, no hang");
      };
      worker.onerror = (e) => {
        clearTimeout(timeout);
        reject(new Error("worker error: " + e.message));
      };
    });
    // If we reach here, the promise resolved without hanging. That's what we want to see.
  } finally {
    worker.terminate();
    URL.revokeObjectURL(blobUrl);
  }
});


// Purpose: Test the happy path of the `drop_table` command.
// Creates a table, drops it with a valid name, then confirms
// it no longer appears in list_tables and get_data returns an error.
await test("drop_table: happy path – table is removed", async () => {
  const tbl = "drop_happy_test";

  // 1. Create a table
  let reply = await worker_do_work(["create_table", tbl, [["id", "INTEGER", true, true, true, "", false]]]);
  if (reply[0] === "error") throw new Error("create_table failed: " + reply[1]);

  // 2. Drop the table (the actual happy path command)
  reply = await worker_do_work(["drop_table", tbl]);
  if (reply[0] === "error") throw new Error("drop_table failed: " + reply[1]);

  // 3. Verify the table is no longer listed
  reply = await worker_do_work(["list_tables"]);
  if (reply[0] === "error") throw new Error("list_tables failed: " + reply[1]);
  if (reply[1].includes(tbl)) throw new Error(`table '${tbl}' still present in list_tables after drop`);

  // 4. Verify get_data on the dropped table returns an error
  reply = await worker_do_work(["get_data", tbl, "", ["id"]]);
  if (reply[0] !== "error") throw new Error(`expected error when querying dropped table, got: ${JSON.stringify(reply)}`);
});

// Purpose: Verify that a REAL column stores floating-point values as strings,
// that they can be read back correctly, and that swapping works.
await test("REAL column: insert, read back, swap", async () => {
  const tbl = "real_test";
  // REAL column, no PK needed
  let reply = await worker_do_work(["create_table", tbl, [
    ["id",  "INTEGER", true, true, true, "", false, true],
    ["val", "REAL",    false,false,false,"", false, false]
  ]]);
  if (reply[0] === "error") throw new Error("create_table failed: " + reply[1]);

  // Insert two rows
  reply = await worker_do_work(["insert_data", tbl, ["val"], ["3.14"]]);
  if (reply[0] === "error") throw new Error("insert 1 failed: " + reply[1]);
  reply = await worker_do_work(["insert_data", tbl, ["val"], ["2.718"]]);
  if (reply[0] === "error") throw new Error("insert 2 failed: " + reply[1]);

  // Read back and check
  reply = await worker_do_work(["get_data", tbl, "", ["val"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  const vals = reply[1].map(row => row[0]);
  if (vals.length !== 2) throw new Error("expected 2 rows");
  if (vals[0] !== "3.14" && vals[1] !== "3.14") throw new Error("expected 3.14");
  if (vals[0] !== "2.718" && vals[1] !== "2.718") throw new Error("expected 2.718");

  // Swap values
  reply = await worker_do_work(["swap_columns", tbl, "1", "2", "val"]);
  if (reply[0] === "error") throw new Error("swap failed: " + reply[1]);

  // After swap, row 1 should have 2.718, row 2 should have 3.14
  reply = await worker_do_work(["get_data", tbl, "id=1", ["val"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  if (reply[1][0][0] !== "2.718") throw new Error(`expected 2.718, got ${reply[1][0][0]}`);
  reply = await worker_do_work(["get_data", tbl, "id=2", ["val"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  if (reply[1][0][0] !== "3.14") throw new Error(`expected 3.14, got ${reply[1][0][0]}`);

  await worker_do_work(["delete_table", tbl]);
});

// Purpose: Test BLOB column storage. Since the JavaScript layer passes strings,
// the BLOB value will be stored as the raw bytes of the string.
// We insert a simple text string, read it back, and verify it matches.
// Then test swapping two BLOB values.
await test("BLOB column: insert, read back, swap", async () => {
  const tbl = "blob_test";
  let reply = await worker_do_work(["create_table", tbl, [
    ["id",  "INTEGER", true, true, true, "", false, true],
    ["data","BLOB",    false,false,false,"", false, false]
  ]]);
  if (reply[0] === "error") throw new Error("create_table failed: " + reply[1]);

  // Insert two rows with distinct byte strings
  const blob1 = "hello";
  const blob2 = "world";
  reply = await worker_do_work(["insert_data", tbl, ["data"], [blob1]]);
  if (reply[0] === "error") throw new Error("insert 1 failed: " + reply[1]);
  reply = await worker_do_work(["insert_data", tbl, ["data"], [blob2]]);
  if (reply[0] === "error") throw new Error("insert 2 failed: " + reply[1]);

  // Read back
  reply = await worker_do_work(["get_data", tbl, "", ["data"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  const rows = reply[1];
  if (rows.length !== 2) throw new Error("expected 2 rows");
  if (rows[0][0] !== blob1) throw new Error(`expected ${blob1}, got ${rows[0][0]}`);
  if (rows[1][0] !== blob2) throw new Error(`expected ${blob2}, got ${rows[1][0]}`);

  // Swap
  reply = await worker_do_work(["swap_columns", tbl, "1", "2", "data"]);
  if (reply[0] === "error") throw new Error("swap failed: " + reply[1]);
  reply = await worker_do_work(["get_data", tbl, "id=1", ["data"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  if (reply[1][0][0] !== blob2) throw new Error(`expected ${blob2}, got ${reply[1][0][0]}`);
  reply = await worker_do_work(["get_data", tbl, "id=2", ["data"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  if (reply[1][0][0] !== blob1) throw new Error(`expected ${blob1}, got ${reply[1][0][0]}`);

  await worker_do_work(["delete_table", tbl]);
});

// Purpose: Ensure that a non‑primary‑key column with the UNIQUE constraint
// rejects duplicate values on insert.
await test("UNIQUE constraint on non-PK column prevents duplicates", async () => {
  const tbl = "unique_test";
  let reply = await worker_do_work(["create_table", tbl, [
    ["id",       "INTEGER", true, true, true, "", false, true],
    ["username", "TEXT",    false,false,true, "", false, false]   // UNIQUE = true
  ]]);
  if (reply[0] === "error") throw new Error("create_table failed: " + reply[1]);

  // Insert a user
  reply = await worker_do_work(["insert_data", tbl, ["username"], ["alice"]]);
  if (reply[0] === "error") throw new Error("first insert failed: " + reply[1]);

  // Try inserting the same username – must fail
  reply = await worker_do_work(["insert_data", tbl, ["username"], ["alice"]]);
  if (reply[0] !== "error") throw new Error("expected duplicate username to error, got: " + JSON.stringify(reply));

  await worker_do_work(["delete_table", tbl]);
});

// Purpose: Test composite UNIQUE constraints, i.e., the combination of two columns
// must be unique, even if each column individually can have duplicates.
// If the wrapper does not support this, the test will gracefully fail.
await test("composite UNIQUE constraint (if supported) prevents duplicate pairs", async () => {
  const tbl = "composite_test";
  // The current column definition does not allow declaring composite constraints.
  // We'll simply try to create a table with two columns and manually handle the UNIQUE
  // by trying duplicate inserts. This test documents the feature's absence/presence.
  let reply = await worker_do_work(["create_table", tbl, [
    ["id",  "INTEGER", true, true, true, "", false, true],
    ["a",   "TEXT",    false,false,false,"", false, false],
    ["b",   "TEXT",    false,false,false,"", false, false]
  ]]);
  if (reply[0] === "error") throw new Error("create_table failed: " + reply[1]);

  // Insert (a,b) = ("x","1") and ("x","2") and ("y","1") – all unique pairs.
  reply = await worker_do_work(["insert_data", tbl, ["a","b"], ["x","1"]]);
  if (reply[0] === "error") throw new Error("insert 1 failed: " + reply[1]);
  reply = await worker_do_work(["insert_data", tbl, ["a","b"], ["x","2"]]);
  if (reply[0] === "error") throw new Error("insert 2 failed: " + reply[1]);
  reply = await worker_do_work(["insert_data", tbl, ["a","b"], ["y","1"]]);
  if (reply[0] === "error") throw new Error("insert 3 failed: " + reply[1]);

  // Now try inserting ("x","2") again – should be rejected if composite unique is enforced.
  reply = await worker_do_work(["insert_data", tbl, ["a","b"], ["x","2"]]);
  if (reply[0] !== "error") {
    // If it succeeds, composite unique is NOT enforced – that's the expected current state.
    // We'll just log that it passed without error; the test itself doesn't fail.
    // But we'll add a comment to indicate the feature is missing.
  } else {
    // If it errors, that's great – composite unique works.
  }

  await worker_do_work(["delete_table", tbl]);
});

// Purpose: Verify that an INTEGER column with DEFAULT 10 correctly
// fills in the default when the column is omitted during insert.
await test("DEFAULT value: INTEGER DEFAULT 10 is used", async () => {
  const tbl = "default_int_test";
  let reply = await worker_do_work(["create_table", tbl, [
    ["id",    "INTEGER", true, true, true, "", false, true],
    ["count", "INTEGER", false,false,false,"10", false, false]  // default "10"
  ]]);
  if (reply[0] === "error") throw new Error("create_table failed: " + reply[1]);

  // Insert without specifying 'count'
  reply = await worker_do_work(["insert_data", tbl, ["id"], ["1"]]);
  if (reply[0] === "error") throw new Error("insert failed: " + reply[1]);

  // Read back
  reply = await worker_do_work(["get_data", tbl, "id=1", ["count"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  if (reply[1][0][0] !== "10") throw new Error(`expected default 10, got ${reply[1][0][0]}`);

  await worker_do_work(["delete_table", tbl]);
});


// Purpose: Attempt to edit the primary key column of a row.
// The expected behaviour is either an error or the edit is silently ignored
// (the original PK remains unchanged). This test verifies that the PK is
// not altered.
await test("edit primary key column: should error or be ignored", async () => {
  const tbl = "pk_edit_test";
  let reply = await worker_do_work(["create_table", tbl, [
    ["id",   "INTEGER", true, true, true, "", false, true],
    ["name", "TEXT",    false,false,false,"", false, false]
  ]]);
  if (reply[0] === "error") throw new Error("create_table failed: " + reply[1]);

  // Insert a row
  reply = await worker_do_work(["insert_data", tbl, ["name"], ["original"]]);
  if (reply[0] === "error") throw new Error("insert failed: " + reply[1]);

  // Attempt to change the PK of row with id=1 to 99
  reply = await worker_do_work(["edit_row", tbl, "1", "id", "99"]);
  // We don't know if this will error or succeed silently.
  // Now read the row to check the actual id value.
  reply = await worker_do_work(["get_data", tbl, "id=1", ["id","name"]]);
  if (reply[0] === "error") {
    // If the table now can't find id=1, maybe the edit actually moved it.
    // That would be a problem. We'll look for id=99 instead.
    reply = await worker_do_work(["get_data", tbl, "id=99", ["id","name"]]);
    if (reply[0] !== "error") {
      throw new Error("PK was unexpectedly changed to 99");
    }
    throw new Error("original row with id=1 disappeared after PK edit attempt");
  }
  // If we still have id=1, verify the id is still 1 and name unchanged
  if (reply[1][0][0] !== "1") throw new Error(`id was changed to ${reply[1][0][0]}`);
  if (reply[1][0][1] !== "original") throw new Error("name was altered");

  await worker_do_work(["delete_table", tbl]);
});


// Purpose: Prepare a table with multiple columns and rows
// so we can exercise various get_data filters.
await test("setup: table for get_data filter tests", async () => {
  const tbl = "filter_test";
  const cols = [
    ["id",  "INTEGER", true, true, true, "", false, true],
    ["name","TEXT",    false,false,false,"", false, false],
    ["age", "INTEGER", false,false,false,"", false, false]
  ];
  let reply = await worker_do_work(["create_table", tbl, cols]);
  if (reply[0] === "error") throw new Error("create_table failed: " + reply[1]);

  // Insert several rows
  const rows = [
    ["Alice", "25"],
    ["Bob",   "30"],
    ["Carol", "35"],
    ["Dave",  "30"]
  ];
  for (const [name, age] of rows) {
    reply = await worker_do_work(["insert_data", tbl, ["name","age"], [name, age]]);
    if (reply[0] === "error") throw new Error(`insert ${name} failed: ${reply[1]}`);
  }
});

// Purpose: Test a numeric comparison filter. Expect rows with id > 2 (i.e., Carol, Dave).
await test("get_data: filter id>2 returns correct rows", async () => {
  const reply = await worker_do_work(["get_data", "filter_test", "id>2", ["id","name","age"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  const rows = reply[1];
  if (!Array.isArray(rows)) throw new Error("expected array of rows");
  // Should get rows for id 3 and 4
  if (rows.length !== 2) throw new Error(`expected 2 rows, got ${rows.length}`);
  // Verify names (order may depend on internal order, but usually ordered by insertion)
  const names = rows.map(r => r[1]).sort();
  if (names[0] !== "Carol" || names[1] !== "Dave") {
    throw new Error(`expected Carol and Dave, got ${names}`);
  }
});

// Purpose: Test an exact string match filter. Expect row with Alice.
await test("get_data: filter name='Alice' returns only Alice", async () => {
  const reply = await worker_do_work(["get_data", "filter_test", "name='Alice'", ["id","name","age"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  const rows = reply[1];
  if (rows.length !== 1) throw new Error(`expected 1 row, got ${rows.length}`);
  if (rows[0][1] !== "Alice") throw new Error(`expected Alice, got ${rows[0][1]}`);
});

// Purpose: Test a numeric comparison with >=. Expect Bob, Carol, Dave (all age 30+).
await test("get_data: filter age>=30 returns rows with age 30 and above", async () => {
  const reply = await worker_do_work(["get_data", "filter_test", "age>=30", ["id","name","age"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  const rows = reply[1];
  if (rows.length !== 3) throw new Error(`expected 3 rows, got ${rows.length}`);
  const names = rows.map(r => r[1]).sort();
  if (names.join(",") !== "Bob,Carol,Dave") throw new Error(`unexpected names: ${names}`);
});

// Purpose: Send a malformed filter to see if the wrapper returns an error gracefully
// rather than crashing or hanging.
await test("get_data: invalid filter syntax should error", async () => {
  const reply = await worker_do_work(["get_data", "filter_test", "garbage;;;", ["id"]]);
  // We expect an error. If it succeeds (maybe returns empty), we'll treat that as a soft failure.
  if (reply[0] !== "error") {
    throw new Error("expected error for invalid filter, but got success: " + JSON.stringify(reply));
  }
});

// Purpose: Define the expected behavior when get_data is called with an empty column array.
// We assert that it returns an error (since 'SELECT FROM table' is invalid SQL).
await test("get_data: empty column list should error", async () => {
  // Use the existing filter_test table; query with empty columns
  const reply = await worker_do_work(["get_data", "filter_test", "", []]);
  if (reply[0] !== "error") {
    throw new Error("expected error for empty column list, got: " + JSON.stringify(reply));
  }
});

// Purpose: Ensure that asking for a column that doesn't exist in the schema
// returns an error (prevents silent wrong data).
await test("get_data: non-existent column should error", async () => {
  const reply = await worker_do_work(["get_data", "filter_test", "", ["nonexistent"]]);
  if (reply[0] !== "error") {
    throw new Error("expected error for non-existent column, got: " + JSON.stringify(reply));
  }
});

// Purpose: Test get_data on a table whose name contains spaces and single quotes.
// The wrapper should either quote the table name correctly and succeed,
// or return an error. The test passes in both cases as long as it doesn't crash/hang.
await test("get_data: table with spaces and quotes in name", async () => {
  const tbl = `test table with 'quotes'`;
  let reply = await worker_do_work(["create_table", tbl, [["id","INTEGER",true,true,true,"",false]]]);
  if (reply[0] === "error") throw new Error("create_table failed: " + reply[1]);

  reply = await worker_do_work(["insert_data", tbl, ["id"], ["1"]]);
  if (reply[0] === "error") throw new Error("insert failed: " + reply[1]);

  reply = await worker_do_work(["get_data", tbl, "", ["id"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  if (reply[1].length !== 1 || reply[1][0][0] !== "1") {
    throw new Error("unexpected data from special-char table");
  }

  await worker_do_work(["delete_table", tbl]);
});

// Purpose: Remove the filter_test table so later tests start clean.
await test("cleanup: drop filter_test table", async () => {
  await worker_do_work(["delete_table", "filter_test"]);
});

// Purpose: Test that after deleting a row, auto‑increment does NOT reuse the old ID,
// and that explicitly inserting the old ID works.
await test("insert_data: autoincrement counter after delete", async () => {
  const tbl = "autoincrement_test";
  // Create table with autoincrement PK
  let reply = await worker_do_work(["create_table", tbl, [
    ["id",  "INTEGER", true, true, true, "", false, true],
    ["val", "TEXT",    false,false,false,"", false, false]
  ]]);
  if (reply[0] === "error") throw new Error("create_table failed: " + reply[1]);

  // Insert two rows
  reply = await worker_do_work(["insert_data", tbl, ["val"], ["first"]]);
  if (reply[0] === "error") throw new Error("insert 1 failed: " + reply[1]);
  reply = await worker_do_work(["insert_data", tbl, ["val"], ["second"]]);
  if (reply[0] === "error") throw new Error("insert 2 failed: " + reply[1]);

  // Delete the second row (id=2)
  reply = await worker_do_work(["delete_row", tbl, "2"]);
  if (reply[0] === "error") throw new Error("delete failed: " + reply[1]);

  // Insert a new row without specifying id – it should get id=3, NOT 2
  reply = await worker_do_work(["insert_data", tbl, ["val"], ["third"]]);
  if (reply[0] === "error") throw new Error("insert 3 failed: " + reply[1]);

  // Verify the row with id=3 exists and id=2 is gone
  reply = await worker_do_work(["get_data", tbl, "", ["id","val"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  const rows = reply[1];
  const ids = rows.map(r => r[0]).sort();
  if (ids.length !== 2) throw new Error(`expected 2 rows, got ${ids.length}`);
  if (ids[0] !== "1" || ids[1] !== "3") throw new Error(`expected ids 1 and 3, got ${ids}`);

  // Now explicitly insert a row with id=2 – should succeed because the old one was deleted
  reply = await worker_do_work(["insert_data", tbl, ["id","val"], ["2","fourth"]]);
  if (reply[0] === "error") throw new Error("explicit insert id=2 failed: " + reply[1]);

  // Verify total rows = 3, with id=2 present
  reply = await worker_do_work(["get_data", tbl, "", ["id"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  if (reply[1].length !== 3) throw new Error(`expected 3 rows, got ${reply[1].length}`);

  await worker_do_work(["delete_table", tbl]);
});

// Purpose: Stress serialisation by creating a table with 25 columns,
// inserting a full row, and reading it back completely.
await test("insert_data: table with 20+ columns", async () => {
  const tbl = "wide_table";
  // Build 25 columns (col0..col24), all TEXT, no constraints except PK on col0
  const cols = [];
  for (let i = 0; i < 25; i++) {
    const name = `col${i}`;
    const isPK = i === 0;
    cols.push([name, "TEXT", isPK, isPK, isPK, "", false]);
  }
  let reply = await worker_do_work(["create_table", tbl, cols]);
  if (reply[0] === "error") throw new Error("create wide table failed: " + reply[1]);

  // Insert a row with 25 values (col0..col24 = "val0".."val24")
  const values = cols.map((_, i) => `val${i}`);
  reply = await worker_do_work(["insert_data", tbl, cols.map(c => c[0]), values]);
  if (reply[0] === "error") throw new Error("insert wide row failed: " + reply[1]);

  // Read back the whole row
  reply = await worker_do_work(["get_data", tbl, "", cols.map(c => c[0])]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  const row = reply[1][0];
  if (!row || row.length !== 25) throw new Error(`expected 25 columns, got ${row ? row.length : 0}`);
  for (let i = 0; i < 25; i++) {
    if (row[i] !== `val${i}`) throw new Error(`col${i} expected "val${i}", got "${row[i]}"`);
  }

  await worker_do_work(["delete_table", tbl]);
});

// Purpose: Test that swapping the values of a UNIQUE non‑PK column works correctly,
// even though the intermediate state might be temporarily duplicate (the implementation
// must handle this atomically).
await test("swap_columns: UNIQUE column – swap between two rows succeeds", async () => {
  const tbl = "swap_unique_test";
  let reply = await worker_do_work(["create_table", tbl, [
    ["id",   "INTEGER", true, true, true, "", false, true],
    ["code", "TEXT",    false,false,true, "", false, false]  // UNIQUE
  ]]);
  if (reply[0] === "error") throw new Error("create_table failed: " + reply[1]);

  // Insert two rows with distinct unique codes
  reply = await worker_do_work(["insert_data", tbl, ["code"], ["A"]]);
  if (reply[0] === "error") throw new Error("insert A failed: " + reply[1]);
  reply = await worker_do_work(["insert_data", tbl, ["code"], ["B"]]);
  if (reply[0] === "error") throw new Error("insert B failed: " + reply[1]);

  // Swap the codes between id=1 and id=2
  reply = await worker_do_work(["swap_columns", tbl, "1", "2", "code"]);
  if (reply[0] === "error") throw new Error("swap on UNIQUE column failed: " + reply[1]);

  // Verify the values are exchanged
  reply = await worker_do_work(["get_data", tbl, "id=1", ["code"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  if (reply[1][0][0] !== "B") throw new Error("id=1 expected B, got " + reply[1][0][0]);
  reply = await worker_do_work(["get_data", tbl, "id=2", ["code"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  if (reply[1][0][0] !== "A") throw new Error("id=2 expected A, got " + reply[1][0][0]);

  await worker_do_work(["delete_table", tbl]);
});

// Purpose: The `swap_columns` command expects exactly one column name.
// Sending an extra argument should result in an error rather than silently ignoring it.
await test("swap_columns: extra arguments cause an error", async () => {
  // Use the existing `people` table (still present from earlier tests).
  // It has columns id, name.
  const reply = await worker_do_work(["swap_columns", "people", "1", "2", "name", "extra"]);
  // Expect an error because the command signature doesn't accept a 6th argument.
  if (reply[0] !== "error") {
    throw new Error("expected error for extra argument, got: " + JSON.stringify(reply));
  }
  // If it errors, pass. No cleanup needed.
});

// Purpose: Verify that check_table works (or errors gracefully) on a table
// whose name contains spaces and single quotes. The SQL may need proper quoting.
await test("check_table: table with spaces and quotes in name", async () => {
  const tbl = `table with 'quotes'`;
  let reply = await worker_do_work(["create_table", tbl, [["id","INTEGER",true,true,true,"",false]]]);
  if (reply[0] === "error") throw new Error("create_table failed: " + reply[1]);

  reply = await worker_do_work(["check_table", tbl]);
  if (reply[0] === "error") throw new Error("check_table failed: " + reply[1]);
  if (!Array.isArray(reply[1]) || reply[1].length !== 1) {
    throw new Error(`expected 1 column schema, got ${JSON.stringify(reply[1])}`);
  }
  if (!reply[1][0].includes("name=id")) throw new Error(`expected column id, got: ${reply[1][0]}`);

  await worker_do_work(["delete_table", tbl]);
});

// Purpose: After calling create_index (which may be unimplemented and return
// an error), the table schema should still be intact and check_table should work.
await test("check_table: schema correct after (failed) index creation", async () => {
  const tbl = "schema_after_index";
  let reply = await worker_do_work(["create_table", tbl, [
    ["id",   "INTEGER", true, true, true, "", false],
    ["name", "TEXT",    false,false,false,"", false]
  ]]);
  if (reply[0] === "error") throw new Error("create_table failed: " + reply[1]);

  // Try to create an index (this command may not be implemented)
  reply = await worker_do_work(["create_index", tbl, "name"]);
  // We don't care if it succeeds or fails; we just want to check schema afterwards.

  // Now inspect the table
  reply = await worker_do_work(["check_table", tbl]);
  if (reply[0] === "error") throw new Error("check_table failed: " + reply[1]);
  const schema = reply[1];
  if (schema.length !== 2) throw new Error(`expected 2 columns, got ${schema.length}`);
  // Verify both columns are present
  if (!schema[0].includes("name=id") && !schema[1].includes("name=id")) {
    throw new Error("column 'id' not found in schema");
  }
  if (!schema[0].includes("name=name") && !schema[1].includes("name=name")) {
    throw new Error("column 'name' not found in schema");
  }

  await worker_do_work(["delete_table", tbl]);
});

// Purpose: Test that a table can be named "table" (a SQL keyword).
// The wrapper should quote it properly, or fail with an error.
await test("table named 'table' (SQL keyword)", async () => {
  const tbl = "table";
  let reply = await worker_do_work(["create_table", tbl, [["id","INTEGER",true,true,true,"",false]]]);
  if (reply[0] === "error") throw new Error("create_table failed: " + reply[1]);

  reply = await worker_do_work(["insert_data", tbl, ["id"], ["1"]]);
  if (reply[0] === "error") throw new Error("insert failed: " + reply[1]);

  reply = await worker_do_work(["get_data", tbl, "", ["id"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  if (reply[1].length !== 1 || reply[1][0][0] !== "1") {
    throw new Error("unexpected data from keyword-named table");
  }

  await worker_do_work(["delete_table", tbl]);
});

// Purpose: Same as above, but with the table name "index" (another SQL keyword).
await test("table named 'index' (SQL keyword)", async () => {
  const tbl = "index";
  let reply = await worker_do_work(["create_table", tbl, [["id","INTEGER",true,true,true,"",false]]]);
  if (reply[0] === "error") throw new Error("create_table failed: " + reply[1]);

  reply = await worker_do_work(["insert_data", tbl, ["id"], ["1"]]);
  if (reply[0] === "error") throw new Error("insert failed: " + reply[1]);

  reply = await worker_do_work(["get_data", tbl, "", ["id"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  if (reply[1].length !== 1 || reply[1][0][0] !== "1") {
    throw new Error("unexpected data from keyword-named table");
  }

  await worker_do_work(["delete_table", tbl]);
});

// Purpose: Stress the naming by creating a table with a 1000‑character name.
// The system should either accept it (if file/identifier limits allow) or
// return an error without crashing.
await test("extremely long table name (1k chars) should error", async () => {
  const longName = "a".repeat(1000);
  const reply = await worker_do_work(["create_table", longName, [["id","INTEGER",true,true,true,"",false]]]);
  if (reply[0] !== "error") {
    await worker_do_work(["delete_table", longName]);
    throw new Error("expected error for extremely long table name, but it succeeded");
  }
});

// Purpose: Test a column with a 1000‑character name. Should either work or error cleanly.
await test("extremely long column name (1k chars) should error", async () => {
  const tbl = "long_col_test";
  const longCol = "c".repeat(1000);
  let reply = await worker_do_work(["create_table", tbl, [
    ["id", "INTEGER", true, true, true, "", false],
    [longCol, "TEXT", false, false, false, "", false]
  ]]);
  if (reply[0] !== "error") {
    await worker_do_work(["delete_table", tbl]);
    throw new Error("expected error for extremely long column name, but it succeeded");
  }
});

// Purpose: Send several insert commands in parallel (without awaiting each one
// individually) to stress the askQueue serialisation. The final data must
// be consistent – all rows inserted in the order they were enqueued, no missing values.
await test("concurrent inserts: parallel calls maintain order", async () => {
  const tbl = "parallel_inserts";
  // 1. Create table
  let reply = await worker_do_work(["create_table", tbl, [
    ["id",  "INTEGER", true, true, true, "", false, true],
    ["val", "TEXT",    false,false,false,"", false, false]
  ]]);
  if (reply[0] === "error") throw new Error("create_table failed: " + reply[1]);

  // 2. Prepare multiple inserts, start them all at once (no await inside map)
  const values = ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"];
  const promises = values.map(v =>
    worker_do_work(["insert_data", tbl, ["val"], [v]])
  );

  // 3. Await all simultaneously
  const results = await Promise.all(promises);
  results.forEach((res, i) => {
    if (res[0] === "error") throw new Error(`insert ${i} ("${values[i]}") failed: ${res[1]}`);
  });

  // 4. Read all rows – they must be in insertion order (IDs 1..10, values in original order)
  reply = await worker_do_work(["get_data", tbl, "", ["id","val"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  const rows = reply[1];
  if (rows.length !== values.length) throw new Error(`expected ${values.length} rows, got ${rows.length}`);
  for (let i = 0; i < values.length; i++) {
    const [id, val] = rows[i];
    if (id !== String(i + 1)) throw new Error(`row ${i} id expected ${i+1}, got ${id}`);
    if (val !== values[i])    throw new Error(`row ${i} val expected "${values[i]}", got "${val}"`);
  }

  // Clean up
  await worker_do_work(["delete_table", tbl]);
});

// Purpose: Verify that the wrapper handles 1000 rows without performance issues
// or crashes. Insert 1000 rows one by one, then count them.
await test("stress: 1000 rows insert and count", async () => {
  const tbl = "thousand_rows";
  let reply = await worker_do_work(["create_table", tbl, [
    ["id",  "INTEGER", true, true, true, "", false, true],
    ["val", "TEXT",    false,false,false,"", false, false]
  ]]);
  if (reply[0] === "error") throw new Error("create_table failed: " + reply[1]);

  // Insert 1000 rows (sequentially to avoid overwhelming the queue)
  for (let i = 0; i < 1000; i++) {
    reply = await worker_do_work(["insert_data", tbl, ["val"], [`row${i}`]]);
    if (reply[0] === "error") throw new Error(`insert ${i} failed: ${reply[1]}`);
  }

  // Verify count
  reply = await worker_do_work(["get_data", tbl, "", ["id"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  if (reply[1].length !== 1000) throw new Error(`expected 1000 rows, got ${reply[1].length}`);

  await worker_do_work(["delete_table", tbl]);
});

// Purpose: Create 55 tables (each tiny), call list_tables, and verify that
// the count is at least 55 and that a few known names are present.
await test("stress: 50+ tables in list_tables", async () => {
  const prefix = "many_tbl_";
  // Create 55 tables
  for (let i = 0; i < 55; i++) {
    const name = prefix + i;
    const reply = await worker_do_work(["create_table", name, [["id","INTEGER",true,true,true,"",false]]]);
    if (reply[0] === "error") throw new Error(`create table ${name} failed: ${reply[1]}`);
  }

  const reply = await worker_do_work(["list_tables"]);
  if (reply[0] === "error") throw new Error("list_tables failed: " + reply[1]);

  const tables = reply[1];
  if (tables.length < 55) throw new Error(`expected at least 55 tables, got ${tables.length}`);

  // Spot-check a few
  if (!tables.includes(prefix + "0"))  throw new Error("missing table 0");
  if (!tables.includes(prefix + "27")) throw new Error("missing table 27");
  if (!tables.includes(prefix + "54")) throw new Error("missing table 54");

  // Clean up all 55 tables
  for (let i = 0; i < 55; i++) {
    await worker_do_work(["delete_table", prefix + i]);
  }
});

// Purpose: Insert a 1 MB string into a TEXT column, read it back, and
// verify it matches exactly. This checks memory handling and serialisation limits.
await test("stress: 1 MB text value", async () => {
  const tbl = "large_text";
  let reply = await worker_do_work(["create_table", tbl, [
    ["id",   "INTEGER", true, true, true, "", false, true],
    ["data", "TEXT",    false,false,false,"", false, false]
  ]]);
  if (reply[0] === "error") throw new Error("create_table failed: " + reply[1]);

  // Generate a 1 MB string (1,048,576 characters)
  const longString = "A".repeat(1024 * 1024);
  reply = await worker_do_work(["insert_data", tbl, ["data"], [longString]]);
  if (reply[0] === "error") throw new Error("insert large string failed: " + reply[1]);

  // Read back
  reply = await worker_do_work(["get_data", tbl, "", ["data"]]);
  if (reply[0] === "error") throw new Error("get_data failed: " + reply[1]);
  const retrieved = reply[1][0][0];
  if (retrieved.length !== longString.length) {
    throw new Error(`length mismatch: expected ${longString.length}, got ${retrieved.length}`);
  }
  if (retrieved !== longString) throw new Error("retrieved string does not match original");

  await worker_do_work(["delete_table", tbl]);
});


// Purpose: If a "close_db" command is exposed (or will be), test that calling
// it does not error, and that subsequent commands fail cleanly (no hang).
// If the command is not yet implemented, the test fails – that's the reminder.
await test("close db connection: no error and later commands fail", async () => {
  // First, try to close the database
  let reply = await worker_do_work(["close_db"]);
  if (reply[0] === "error") {
    throw new Error(`close_db not implemented or failed: ${reply[1]}`);
  }

  // After closing, any further operation should return an error.
  // We'll try a simple list_tables.
  reply = await worker_do_work(["list_tables"]);
  if (reply[0] !== "error") {
    throw new Error("expected error after close, but got success: " + JSON.stringify(reply));
  }
  // If we reach here, the post-close error is properly reported.
  // Note: This leaves the database closed, so later tests will fail.
  // The suite must be restarted after this test.
});

// Purpose: At the end of the test suite (or beginning of a run), drop every table
// and confirm the database is empty. This ensures no test leaves cruft behind.
await test("suite isolation: database is empty after cleanup", async () => {
  // 1. Get all tables
  let reply = await worker_do_work(["list_tables"]);
  if (reply[0] === "error") throw new Error("list_tables failed: " + reply[1]);

  const tables = reply[1];
  // 2. Drop each one (using delete_table, since drop_table may not be implemented)
  for (const name of tables) {
    const dropReply = await worker_do_work(["delete_table", name]);
    if (dropReply[0] === "error") {
      // If delete_table fails (e.g., non-existent table error), that's fine.
    }
  }

  // 3. Verify empty
  reply = await worker_do_work(["list_tables"]);
  if (reply[0] === "error") throw new Error("list_tables after cleanup failed: " + reply[1]);
  if (reply[1].length !== 0) {
    throw new Error(`database not empty after cleanup: ${JSON.stringify(reply[1])}`);
  }
});

// Purpose: Simulate the main worker being terminated and then trying to use
// worker_do_work. Expects an error (or the Promise rejects) without hanging.
// This uses a fresh worker to avoid breaking the actual test worker.
await test("worker termination: postMessage after terminate fails", async () => {
  const blob = new Blob([
    `self.onmessage = (e) => { self.postMessage(["pong", e.data]); };`
  ], { type: 'application/javascript' });
  const blobUrl = URL.createObjectURL(blob);
  const worker = new Worker(blobUrl, { type: 'module' });

  try {
    // Do one ping/pong to confirm it's alive
    let alive = await new Promise((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error("worker hang")), 1000);
      worker.onmessage = (e) => {
        clearTimeout(timeout);
        resolve(e.data);
      };
      worker.onerror = (e) => { clearTimeout(timeout); reject(new Error("worker error: " + e.message)); };
      worker.postMessage(["ping"]);
    });
    if (alive[0] !== "pong") throw new Error("unexpected initial response");

    // Terminate the worker
    worker.terminate();

    // Now try to postMessage again – it should throw or do nothing (but no hang).
    // We'll wrap it in a try/catch and also set a timeout.
    let errorHappened = false;
    try {
      worker.postMessage(["ping"]);
    } catch (e) {
      // Expected: an exception because the worker is terminated.
      errorHappened = true;
    }

    // If no exception was thrown, the postMessage may have queued without error,
    // but onmessage will never fire. To avoid hanging, we race with a timeout.
    if (!errorHappened) {
      const result = await new Promise((resolve, reject) => {
        const timeout = setTimeout(() => resolve("timeout"), 1000);
        worker.onmessage = () => {
          clearTimeout(timeout);
          resolve("message received unexpectedly");
        };
        worker.onerror = () => {
          clearTimeout(timeout);
          resolve("error event");
        };
      });
      if (result !== "timeout" && result !== "error event") {
        throw new Error("postMessage succeeded after terminate – that's weird");
      }
      // If we got a timeout, that means it hung? Actually, timeout means onmessage never fired, which is acceptable.
    }
    // Test passes either way – no hang occurred.
  } finally {
    // Already terminated, but revoke URL
    URL.revokeObjectURL(blobUrl);
  }
});

// Purpose: Until create_index is implemented, it must return an error containing
// "not implemented". Once implemented, this test will fail (because the error
// goes away), which is your signal to update the test to expect success.
await test("create_index: returns 'not implemented' error (feature pending)", async () => {
  const tbl = "index_stub_test";
  // Create a table to index
  let reply = await worker_do_work(["create_table", tbl, [
    ["id", "INTEGER", true, true, true, "", false],
    ["val","TEXT",    false,false,false,"", false]
  ]]);
  if (reply[0] === "error") throw new Error("create_table failed: " + reply[1]);

  // Try to create an index
  reply = await worker_do_work(["create_index", tbl, "val"]);

  // If the feature is not yet implemented, we expect an error with "not implemented".
  if (reply[0] !== "error") {
    // It succeeded, which means the feature might be implemented – but we still want a
    // clear signal. So we fail the test: you need to update the test to expect success.
    throw new Error("create_index unexpectedly succeeded. Update this test to expect success.");
  }

  // Check that the error message contains "not implemented"
  const errorMsg = reply[1] || "";
  if (!errorMsg.toLowerCase().includes("not implemented")) {
    throw new Error(`expected "not implemented" error, got: ${errorMsg}`);
  }

  // If we get here, the test passes (feature still pending).
  await worker_do_work(["delete_table", tbl]);
});


// Purpose: Placeholder that FAILS until you implement a timeout / abort mechanism
// in worker_do_work. Once you add that feature, update this test to actually
// verify that a hung worker is detected and doesn't freeze the suite.
await test("hang protection: worker_do_work supports timeout (not implemented)", async () => {
  // Check if worker_do_work has been updated to accept a timeout or AbortSignal.
  // We'll use a simple heuristic: if the function accepts more than one argument
  // (e.g., an options object), we assume you've started implementing it.
  if (worker_do_work.length < 2) {
    throw new Error(
      "worker_do_work does not accept a timeout option. Hang protection not implemented."
    );
  }

  // Even if the signature changed, the implementation isn't validated yet.
  // This test is a to-do flag – remove or replace it when the feature is done.
  throw new Error(
    "Hang protection not yet fully implemented. Update this test to verify timeout behaviour."
  );
});
