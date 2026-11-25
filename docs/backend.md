# Backend

An axum server in `backend/`. It currently has one route: `GET /` returns
`{"message":"Hello, World!"}` (a `shared::Hello`).

## Running locally

```sh
cargo run -p backend
```

It listens on `0.0.0.0:3000`. It binds `0.0.0.0` rather than `127.0.0.1` so
it's reachable from outside a container.

### Configuration

| Env var | Default | Read with |
|---|---|---|
| `PORT` | `3000` | `std::env::var("PORT")` at startup |

PowerShell example: `$env:PORT = "4000"; cargo run -p backend`

## Docker image

`backend/Dockerfile` is a multi-stage build. **Build from the repo root**,
since the image needs the workspace files:

```sh
docker build -f backend/Dockerfile -t deoxys-backend:dev .
```

| Stage | Base | Does |
|---|---|---|
| `chef` | `rust:1-trixie` | installs cargo-chef |
| `planner` | chef | writes `recipe.json`, the dependency list |
| `builder` | chef | `cargo chef cook` compiles dependencies only (cached layer), then `cargo build --release -p backend` |
| `runtime` | `debian:trixie-slim` | non-root `app` user plus the binary; nothing else |

- **The Dockerfile copies explicitly:** `Cargo.toml`, `Cargo.lock`, `shared/` and `backend/`. There's no `COPY . .`, so new files elsewhere never end up in the build.
- **`.dockerignore`** (at the repo root) excludes `.git`, `target`, `node_modules`, `.env` and all of `desktop/`. That keeps the build context, which is uploaded to Docker before any `COPY`, at a few KB.
- **Same Debian release in both stages** (trixie = Debian 13), so the glibc the binary was built against matches the one it runs on.
- **`--package backend`** compiles only the backend's dependency tree. No Tauri crates are built.

### Size

About 29 MB compressed (what nodes download) and about 90 MB unpacked.
Nearly all of it is `debian:trixie-slim`; the binary is under 2 MB. If size
ever matters, use `gcr.io/distroless/cc-debian13`, or a static musl binary on
`scratch`. The cost is losing the shell for `kubectl exec`.

`docker image ls` may show about 120 MB. Docker Desktop counts the compressed
and unpacked copies together.

## CI/CD

Both workflows run on changes to `backend/**`, `shared/**`, the root
`Cargo.toml`/`Cargo.lock`, or the workflow file itself.

| Workflow | When | Does |
|---|---|---|
| `ci-backend.yml` | every push and PR | `cargo build --locked -p backend`, `cargo test --locked -p backend -p shared` |
| `cd-backend.yml` | push to `main` | builds `backend/Dockerfile` with the repo root as context, and pushes to GHCR |

CD logs into GHCR with the built-in `GITHUB_TOKEN`, so no secret needs
setting up. It pushes two tags:

- `ghcr.io/s9oon/deoxys-backend:latest`
- `ghcr.io/s9oon/deoxys-backend:<commit sha>`

**Prefer the SHA tag when deploying.** `:latest` changes under you, can't be
rolled back, and a new push doesn't trigger a rollout by itself. See
[kubernetes.md](kubernetes.md#updating-to-a-new-image).

GHCR packages are private by default. Either make the package public in its
settings, or give the cluster pull credentials (see
[kubernetes.md](kubernetes.md#private-images)).

## Rust toolchain

The Dockerfile uses `rust:1-trixie` (the latest stable 1.x) and CI uses
`stable`. Pin both to the same `1.xx` version, plus a `rust-toolchain.toml`,
once builds need to be reproducible exactly. Edition 2024 needs Rust 1.85 or
newer.
