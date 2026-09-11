# AI Lab Rules

1. **Isolation**: Code inside the `lab` package/folder must NEVER affect the core application's logic or stability.
2. **Independence**: If a Lab feature needs to interact with a production feature, **do not** import the production code directly if you intend to modify it for the lab. Instead, **COPY** the production code into the lab directory and modify the copy.
3. **Opt-in**: Lab features should be visually distinct (e.g., the Smart Button) and should be easy to "rip out" without leaving scars in the main codebase.
4. **UX First**: The Lab is for testing interactions, animations, and "feel". Functionality (DB, Sync, CRDT) comes after the UX is proven.
