// worker.js – dedicated worker that owns the OPFS SQLite connection
import init, { LiveForever } from './db_wrapper.js';

let db_manager = null;
let wasmInitPromise = null;
let connection_name = null;

self.addEventListener('error', (e) => {
  console.error('[worker] UNCAUGHT ERROR:', e.message, e.filename, e.lineno, e.colno, e.error);
});

self.addEventListener('unhandledrejection', (e) => {
  console.error('[worker] UNHANDLED PROMISE REJECTION:', e.reason);
});

(async () => {
  try {
    const root = await navigator.storage.getDirectory();
    console.log('[worker] OPFS available in js environment:', root);
  } catch (e) {
    console.error('[worker] OPFS NOT AVAILABLE:', e);
  }
})();

async function tryReconnect() {
  if (!connection_name) {
    return false;
  }

  let attempts = 10;
  for (let attempt = 0; attempt < attempts; attempt++) {
    try {
      console.log(`[worker] reconnect attempt ${attempt + 1}/${attempts}`);
      db_manager = await LiveForever.new(connection_name);
      console.log('[worker] reconnect succeeded');
      return true;
    } catch (e) {
      console.warn('[worker] reconnect attempt failed:', e);
    }

    if (attempt < attempts - 1) {
      await new Promise((r) => setTimeout(r, 500));
    }
  }

  return false;
}

// These commands receive a serialized Uint8Array in msg[1].
const serializedCommands = {
  create_table: (msg) => db_manager.create_table(msg[1]),
  drop_table: (msg) => db_manager.drop_table(msg[1]),
  delete_table: (msg) => db_manager.drop_table(msg[1]), // alias for old command name
  check_table: (msg) => db_manager.check_table(msg[1]),
  get_data: (msg) => db_manager.get_data(msg[1]),
  get_data_ordered: (msg) => db_manager.get_data_ordered(msg[1]),
  get_data_by_order: (msg) => db_manager.get_data_ordered(msg[1]), // alias
  insert_data: (msg) => db_manager.insert_data(msg[1]),
  edit_col_in_row: (msg) => db_manager.edit_col_in_row(msg[1]),
  edit_row: (msg) => db_manager.edit_col_in_row(msg[1]), // alias
  delete_row: (msg) => db_manager.delete_row(msg[1]),
  swap_columns: (msg) => db_manager.swap_columns(msg[1]),
  create_index: (msg) => db_manager.create_index(msg[1]),
  check_index: (msg) => db_manager.check_index(msg[1]),
  add_column: (msg) => db_manager.add_column(msg[1]),
  remove_column: (msg) => db_manager.remove_column(msg[1]),
  export_tables: (msg) => db_manager.export_tables(msg[1]),
  create_table_from_export: (msg) => db_manager.create_table_from_export(msg[1]),
  copy_table: (msg) => db_manager.copy_table(msg[1]),
};

// These commands take no serialized payload.
const noInputCommands = {
  list_tables: () => db_manager.list_tables(),
  export_database: () => db_manager.export_database(new Uint8Array(0)),
};

self.onmessage = async (event) => {
  console.log('[worker] onmessage received:', JSON.stringify(event.data));

  const msg = event.data;
  const command = msg[0];

  if (!wasmInitPromise) {
    console.log('[worker] starting wasm init...');
    wasmInitPromise = init()
      .then(() => {
        console.log('[worker] wasm init complete');
      })
      .catch((err) => {
        console.error('[worker] wasm init FAILED:', err);
        throw err;
      });
  }

  try {
    await wasmInitPromise;
  } catch (err) {
    console.error(
      `[worker] WASM init failed while handling command "${command}" (message: ${JSON.stringify(msg)})`,
      err
    );
    self.postMessage(['error', `WASM initialization failed: ${err.toString()}`]);
    return;
  }

  // Special lifecycle commands.
  if (command === 'initialize') {
    try {
      connection_name = msg[1];
      console.log('[worker] calling LiveForever.new with db_conn_name:', connection_name);
      db_manager = await LiveForever.new(connection_name);
      console.log('[worker] LiveForever.new resolved successfully');
      self.postMessage(['initialize', 'ok']);
    } catch (err) {
      console.error('[worker] LiveForever.new failed:', err);
      self.postMessage(['error', err.toString()]);
    }
    return;
  }

  if (command === 'close_conn') {
    try {
      if (db_manager) {
        console.log('[worker] attempting to give up conn');
        await db_manager.close_conn_js();
        db_manager = null;
        self.postMessage(['close_conn', 'closed']);
      } else {
        self.postMessage(['close_conn', 'already closed']);
      }
    } catch (err) {
      console.error('[worker] close_conn_js failed:', err);
      self.postMessage(['error', err.toString()]);
    }
    return;
  }

  // Ensure connection exists for DB commands.
  if (!db_manager) {
    console.log('[worker] no active connection, requesting want_conn');
    self.postMessage(['want_conn']);

    const ok = await tryReconnect();
    if (!ok) {
      self.postMessage(['error', "couldn't get sahpool back"]);
      return;
    }
  }

  // Find the command handler.
  const handler = serializedCommands[command] || noInputCommands[command];

  if (!handler) {
    console.warn('[worker] unknown command:', command);
    self.postMessage(['error', 'unknown command']);
    return;
  }

  try {
    const result = await handler(msg);
    console.log('[worker] handler succeeded for:', command, 'result type:', typeof result);
    self.postMessage([command, result]);
  } catch (err) {
    console.error('[worker] handler error for:', command, err);
    self.postMessage(['error', err.toString()]);
  }
};

console.log('[worker] worker.js loaded');
self.postMessage(['ready']);
