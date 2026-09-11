# AI Lab Rules

1. **Isolation**: Code inside the `lab` package/folder must NEVER affect the core application's logic or stability.
2. **Modularity**: Every experiment must live in its own sub-folder under `lab/experiments/`. Do not monolith code into a single manager.
3. **Navigation**: Use the `LabNavigator` in `lab/core/` to define routes and state for experiments.
4. **Independence**: If a Lab feature needs to interact with a production feature, **do not** import the production code directly if you intend to modify it for the lab. Instead, **COPY** the production code into the lab directory and modify the copy.
5. **Opt-in**: Lab features should be visually distinct (e.g., the Smart Button) and should be easy to "rip out" without leaving scars in the main codebase.
6. **UX First**: The Lab is for testing interactions, animations, and "feel". Functionality (DB, Sync, CRDT) comes after the UX is proven.
