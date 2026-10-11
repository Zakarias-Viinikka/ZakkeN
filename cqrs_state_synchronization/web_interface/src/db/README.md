# db

## init.rs

Gets called in main and creates the tables if they don't exist.

## ui_actions.rs

If the user interacts with a UI, and it needs to do something with the
db, the request gets routed to ui_actions which is the dumb facade the UI
interacts with.

## ui_actions_helper.rs

ui_actions_helper debloats ui_actions, refer to HELPER_EXPLAINED.md for
further details.

## db_helpers_for_web_client.rs

The dumb facade for direct db calls.

## db_out_destruct_helpers.rs

The db out values need destructuring. That's condensed into methods, and
the methods are in destruct_helper.

## dev_actions.rs

dev_actions is ui_actions but for dev stuff.
