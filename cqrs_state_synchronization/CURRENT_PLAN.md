## Testing and sync checks

Add testing support using testing_wasm32_template.

Build a way to run methods that check if stuff is in sync, for various things: active pages agree, the number of blocks in a page agrees, the content in blocks agrees. The content check is what tests will assert on later.