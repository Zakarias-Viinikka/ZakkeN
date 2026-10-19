# explained for a human

We manually downloaded a dependency, put it in the project, and edited its Cargo.toml.

The dependency is one leptos uses: `server_fn_macro_default`. It was conflicting with leptos itself over which version of `syn` to use. Leptos wanted `3.0`. `server_fn_macro_default` wanted `2.0.104`. Forcing either one broke the other.

So we took a copy of `server_fn_macro_default`, changed its syn line to `3.0`, and told cargo to use our copy instead of the official one.

`server_fn_macro_default` comes in through leptos even though this is a csr app with no server code. The chain: leptos uses `leptos_macro`, and `leptos_macro` imports `server_fn_macro` unconditionally in its own source. That import doesn't go away when the ssr feature is off — cargo features gate what code gets used, not what dependencies exist. So the server-related crates get compiled into a csr app regardless of whether the app ever writes a `#[server]` function.

The rest of this file is the same story with more detail.

---

The `syn` crate is a tool that reads Rust code so other tools can understand it. It acts as a parser.

The build failed because two different versions of this tool, version 2 and version 3, were present in the project at the same time. The `leptos` framework required version 3. A smaller tool called `server_fn_macro_default` required version 2.

When the compiler tried to build `server_fn_macro_default`, it saw the parsing rules from both versions. It did not know which set of rules to apply. This caused a trait bound error, which means the code asked for a specific rule that the compiler could not uniquely identify.

Forcing the whole project to use version 2 broke `leptos`. Forcing it to use version 3 broke `server_fn_macro_default`.

The solution was to download the source code for `server_fn_macro_default` into a local folder. We changed its configuration file to accept version 3 of `syn`. Then we added a patch instruction to the main project configuration. This instruction told the compiler to use our local, modified copy instead of the official one from the internet. This unified the project to use only one version of the tool.
