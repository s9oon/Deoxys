# Backend

An axum server in `backend/`. It has one route: `GET /` returns
`{"message":"Hello, World!"}`, a `shared::Hello`.

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

In Kubernetes it's set by the Deployment's `env` (see [kubernetes.md](kubernetes.md)).

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

The image is about 29 MB compressed and about 90 MB unpacked. Nearly all of
that is `debian:trixie-slim`; the binary is under 2 MB.

## CI/CD

Both workflows run on changes to `backend/**`, `shared/**`, the root
`Cargo.toml`/`Cargo.lock`, or the workflow file itself.

| Workflow | When | Does |
|---|---|---|
| `ci-backend.yml` | every push and PR | `cargo build --locked -p backend`, `cargo test --locked -p backend -p shared` |
| `cd-backend.yml` | push to `main` | builds `backend/Dockerfile` with the repo root as context, and pushes to GHCR |

CD logs into GHCR with the built-in `GITHUB_TOKEN`, so no secret is needed.
It pushes two tags:

- `ghcr.io/s9oon/deoxys-backend:latest`
- `ghcr.io/s9oon/deoxys-backend:<commit sha>`. This one never changes, so it identifies an exact build.

See [kubernetes.md](kubernetes.md#deploying-and-updating) for how each tag is
rolled out, and [kubernetes.md](kubernetes.md#private-images) if the package
is private.
