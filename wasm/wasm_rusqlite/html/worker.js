import init, { LiveForever } from '../pkg/part4.js';

let db_manager = null;
let wasmInitPromise = null;

// Catch uncaught errors and unhandled rejections so they don't disappear silently.
self.addEventListener('error', (e) => {
  console.error('[worker] UNCAUGHT ERROR:', e.message, e.filename, e.lineno, e.colno, e.error);
});

self.addEventListener('unhandledrejection', (e) => {
  console.error('[worker] UNHANDLED PROMISE REJECTION:', e.reason);
});

// OPFS directory check – separate async IIFE, does not block worker startup.
(async () => {
  try {
    const root = await navigator.storage.getDirectory();
    console.log('[worker] OPFS root handle:', root);
  } catch (e) {
    console.error('[worker] OPFS NOT AVAILABLE:', e);
  }
})();

// Define handlers before onconnect; function declarations are hoisted,
// but `handlers` object is a normal variable, so it must be defined here.
const handlers = {
  initialize: async (msg) => {
    console.log('[worker] initialize handler called, full message:', JSON.stringify(msg));
    try {
      console.log('[worker] calling LiveForever.new with db_conn_name:', msg[1]);
      db_manager = await LiveForever.new(msg[1]);
      console.log('[worker] LiveForever.new resolved successfully');
      return 'ok';
    } catch (e) {
      console.error('[worker] LiveForever.new failed:', e);
      throw e;
    }
  },

  drop_table: async () => {
    console.log('[worker] drop_table handler called');
    await db_manager.drop_table();
    console.log('[worker] drop_table completed');
    return 'droppety woppetied all of it';
  },

  check_table: (msg) => {
    console.log('[worker] check_table handler called, args:', msg[1]);
    return db_manager.check_table(msg[1]);
  },

  get_data: (msg) => {
    console.log('[worker] get_data handler called, args:', msg[1], msg[2], msg[3]);
    return db_manager.get_data(msg[1], msg[2], msg[3]);
  },

  get_data_by_order: (msg) => {
    console.log('[worker] get_data_by_order handler called, args:', msg[1], msg[2], msg[3], msg[4]);
    return db_manager.get_data_ordered(msg[1], msg[2], msg[3], msg[4]);
  },

  insert_data: async (msg) => {
    console.log('[worker] insert_data handler called, args:', msg[1], msg[2], msg[3]);
    await db_manager.insert_data(msg[1], msg[2], msg[3]);
    console.log('[worker] insert_data completed');
    return 'ok';
  },

  edit_row: async (msg) => {
    console.log('[worker] edit_row handler called, args:', msg[1], msg[2], msg[3], msg[4]);
    await db_manager.edit_col_in_row(msg[1], msg[2], msg[3], msg[4]);
    console.log('[worker] edit_row completed');
    return 'ok';
  },

  delete_row: async (msg) => {
    console.log('[worker] delete_row handler called, args:', msg[1], msg[2]);
    await db_manager.delete_row(msg[1], msg[2]);
    console.log('[worker] delete_row completed');
    return 'ok';
  },

  swap_columns: async (msg) => {
    console.log('[worker] swap_columns handler called, args:', msg[1], msg[2], msg[3], msg[4]);
    await db_manager.swap_columns(msg[1], msg[2], msg[3], msg[4]);
    console.log('[worker] swap_columns completed');
    return 'ok';
  },

  create_table: async (msg) => {
    console.log('[worker] create_table handler called, table:', msg[1], 'columns:', JSON.stringify(msg[2]));
    await db_manager.create_table(msg[1], msg[2]);
    console.log('[worker] create_table completed for table:', msg[1]);
    return `created table: ${msg[1]}`;
  },

  delete_table: async (msg) => {
    console.log('[worker] delete_table handler called, table:', msg[1]);
    await db_manager.delete_table(msg[1]);
    console.log('[worker] delete_table completed for table:', msg[1]);
    return `deleted table: ${msg[1]}`;
  },

  create_index: async (msg) => {
    console.log('[worker] create_index handler called, table:', msg[1], 'column:', msg[2]);
    await db_manager.create_index(msg[1], msg[2]);
    console.log('[worker] create_index completed');
    return `indexed ${msg[2]} on ${msg[1]}`;
  },

  list_tables: async () => {
    console.log('[worker] list_tables handler called');
    const result = await db_manager.list_tables();
    console.log('[worker] list_tables result:', JSON.stringify(result));
    return result;
  },
};

// Register onconnect immediately – BEFORE awaiting anything – so we never miss it.
self.onconnect = (e) => {
  console.log('[worker] onconnect fired');
  const port = e.ports[0];

  if (!port) {
    console.error('[worker] no port found in connect event');
    return;
  }

  port.onmessage = async (event) => {
    console.log('[worker] onmessage received:', JSON.stringify(event.data));

    // Start WASM init on first message, if not already started.
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
      await wasmInitPromise; // ensure WASM is ready before handling any command
    } catch (err) {
      console.error(`[worker] WASM init failed while handling command "${command}" (message: ${JSON.stringify(msg)})`, err);
      port.postMessage(['error', `WASM initialization failed: ${err.toString()}`]);
      return;
    }

    const msg = event.data;
    const command = msg[0];
    const handler = handlers[command];

    if (!handler) {
      console.warn('[worker] unknown command:', command);
      port.postMessage(['error', 'unknown command']);
      return;
    }

    if (command !== 'initialize' && !db_manager) {
      console.warn('[worker] db not initialized for command:', command);
      port.postMessage(['error', "Database not initialised. Send an 'initialize' command first."]);
      return;
    }

    try {
      const data = await handler(msg);
      console.log('[worker] handler succeeded for:', command, 'result:', JSON.stringify(data));
      port.postMessage([command, data]);
    } catch (err) {
      console.error('[worker] handler error for:', command, err);
      port.postMessage(['error', err.toString()]);
    }
  };

  console.log('[worker] port message listener attached; starting port...');
  port.start();
  console.log('[worker] port started');
};

console.log('[worker] worker.js loaded, onconnect registered');
