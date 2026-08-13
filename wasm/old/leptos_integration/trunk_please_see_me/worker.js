import init, { LiveForever } from './part4.js';
await init();
let db_manager = null;   // will hold the instance after successful creation
// each handler takes the raw message array and returns the payload to send back
const handlers = {
  initialize: async (msg) => {
      console.log("[worker] initialize handler called");
      try {
          console.log("[worker] calling LiveForever.new with db_conn_name: ", msg[1]);
          db_manager = await LiveForever.new(msg[1]);
          console.log("[worker] LiveForever.new resolved");
          return "ok";
      } catch (e) {
          console.error("[worker] LiveForever.new failed:", e);
          throw e;
      }
  },
  drop_table: async () => {
    await db_manager.drop_table();
    return "droppety woppetied all of it";
  },
  check_table: (msg) => db_manager.check_table(msg[1]),
  get_data: (msg) => db_manager.get_data(msg[1], msg[2], msg[3]),
  insert_data: async (msg) => {
      // msg = ["insert_data", table_name, col_names_array, vals_array]
      await db_manager.insert_data(msg[1], msg[2], msg[3]);
      return "ok";
  },
  // msg: ["edit_row", table_name, row_id, column, new_value]
  edit_row: async (msg) => { //log('edit', await ask(["edit_row", table, id, col, val]));
    /*
    pub async fn edit_col_in_row(
        &self,
        table_name: String,
        row_id: String,
        column: String,
        value: String,
    ) */
    await db_manager.edit_col_in_row(msg[1], msg[2], msg[3], msg[4]);
    return "ok";
  },
  // msg: ["delete_row", table_name, row_id]
  delete_row: async (msg) => {
    await db_manager.delete_row(msg[1], msg[2]);
    return "ok";
  },
  // msg: ["swap_columns", table_name, row_id_1, row_id_2, column]
  swap_columns: async (msg) => {
    await db_manager.swap_columns(msg[1], msg[2], msg[3], msg[4]);
    return "ok";
  },

  // msg: ["create_table", table_name, columns]
  // columns: array of [name, type, primaryKey, notNull, unique, defaultValue, autoincrement]
  create_table: async (msg) => {
    await db_manager.create_table(msg[1], msg[2]);
    return `created table: ${msg[1]}`;
  },

  // msg: ["delete_table", table_name]
  delete_table: async (msg) => {
    await db_manager.delete_table(msg[1]);
    return `deleted table: ${msg[1]}`;
  },

  // msg: ["create_index", table_name, column_name]
  create_index: async (msg) => {
    await db_manager.create_index(msg[1], msg[2]);
    return `indexed ${msg[2]} on ${msg[1]}`;
  },

  // msg: ["list_tables"]
  list_tables: async () => db_manager.list_tables(),
};

self.onmessage = async (e) => {
  const msg = e.data;
  const command = msg[0];
  const handler = handlers[command];

  if (!handler) {
    self.postMessage(["error", "unknown command"]);
    return;
  }
  if (command !== "initialize" && !db_manager) {
    self.postMessage(["error", "Database not initialised. Send an 'initialize' command first."]);
    return;
  }

  try {
    const data = await handler(msg);
    self.postMessage([command, data]);
  } catch (err) {
    self.postMessage(["error", err.toString()]);
  }
};
console.log("loaded worker js");
self.postMessage(["ready"]);
