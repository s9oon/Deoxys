# Shared types

`shared/` is a small Rust library holding the structs that cross a boundary:
backend → desktop over HTTP, and Tauri → frontend over IPC.

```rust
// shared/src/lib.rs
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Hello {
    pub message: String,
}
```

Both the struct and its fields must be `pub`, or other crates can't use them.

## Who uses it and how

| Crate | Depends on shared via | Uses it for |
|---|---|---|
| `backend` | `shared = { workspace = true }` | `Json(Hello { .. })` responses |
| `desktop/src-tauri` | `shared = { path = "../../shared" }` | `.json::<Hello>()` when calling the backend |
| frontend (`desktop/src`) | generated `ipc/Hello.ts` | the type of `invoke<Hello>(...)` |

Because backend and desktop compile against the same struct, renaming a field
breaks the build on both sides until both are fixed. That's the point: the
compiler keeps the API in sync instead of an OpenAPI spec. (We don't use
utoipa/OpenAPI; add it only if a non-Rust client needs the API.)

## The Cargo workspace

The root `Cargo.toml`:

```toml
[workspace]
resolver = "3"
members = ["backend", "shared"]
exclude = ["desktop"]

[workspace.dependencies]
shared = { path = "shared" }
serde = { version = "1", features = ["derive"] }
```

- **Members** share one `Cargo.lock` and one `target/`. Build a single one with `-p`, e.g. `cargo build -p backend`.
- **Desktop is not a member.** It has its own `Cargo.lock`, `target/` and `[profile.release]`, and reaches `shared` by path. This keeps the backend's Docker build from needing anything under `desktop/`.
- **`exclude = ["desktop"]`** is required. Without it, Cargo walks up from `desktop/src-tauri`, finds the root `[workspace]`, and errors with "current package believes it's in a workspace when it's not".
- **`resolver = "3"`** has to be set explicitly, since a workspace root has no `edition` to infer it from.
- **Profiles** (`[profile.release]`) are only read from the workspace root, or from a project's own root like desktop's. A profile in a member crate is ignored.

## Generating TypeScript with ts-rs

ts-rs turns Rust structs into TypeScript types. It sits behind an optional
`ts` feature in `shared/Cargo.toml`, so the backend never compiles it:

```toml
[dependencies]
ts-rs = { version = "10", optional = true }

[features]
ts = ["dep:ts-rs"]
```

`#[cfg_attr(feature = "ts", ...)]` only adds the derive when that feature is on.

`.cargo/config.toml` sets where the files go:

```toml
[env]
TS_RS_EXPORT_DIR = { value = "desktop/src/ipc", relative = true }
```

To regenerate, from the repo root:

```sh
cargo test -p shared --features ts
```

ts-rs exports from generated tests, which is why it's `cargo test`. This
writes `desktop/src/ipc/Hello.ts`:

```ts
export type Hello = { message: string, };
```

The frontend imports it:

```ts
import type { Hello } from "./ipc/Hello";
```

### Rules

- **Re-run it after changing anything in `shared/`.** Otherwise `ipc/*.ts` is stale and TypeScript won't notice the change.
- **`desktop/src/ipc/` is committed**, so the frontend builds without running cargo first.
- **Don't edit `ipc/` by hand.** It's overwritten on every export.

### Adding a new shared type

1. Add the struct to `shared/src/lib.rs` with the same two derive lines as `Hello`.
2. Run `cargo test -p shared --features ts`.
3. Use it in the backend handler, the Tauri command, and the frontend (`import type { X } from "./ipc/X"`).
