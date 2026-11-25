# Backend

An axum server in `backend/`, backed by Postgres through sqlx.

| Route | Returns |
|---|---|
| `GET /` | `{"message":"Hello, World!"}`, a `shared::Hello` |
| `GET /health` | `200` if `SELECT 1` succeeds on the database, else `503`. No body. |

## State

`backend/src/state.rs` holds `AppState { database: PgPool }`. `main` creates it
once at startup, wraps it in an `Arc`, and passes it to the router with
`.with_state(...)`. A handler gets it by taking `State<Arc<AppState>>` as an
argument. Handlers that don't need it (like `GET /`) leave it out.

## Running locally

It needs a Postgres to connect to. Locally, the quickest is a container:

```sh
docker run -d --name pg -e POSTGRES_PASSWORD=dev -p 5432:5432 postgres:17
DATABASE_URL=postgres://postgres:dev@localhost:5432/postgres cargo run -p backend
```

It listens on `0.0.0.0:3000`. It binds `0.0.0.0` rather than `127.0.0.1` so
it's reachable from outside a container.

### Configuration

| Env var | Default | Read with |
|---|---|---|
| `PORT` | `3000` | `std::env::var("PORT")` at startup |
| `DATABASE_URL` | none, required | `std::env::var("DATABASE_URL")` at startup |

`DATABASE_URL` has the form `postgres://USER:PASSWORD@HOST:PORT/DATABASE`.
Without it the backend exits with `Error: "DATABASE_URL not set"`. If the
database can't be reached, `PgPool::connect` gives up after about 30 s with
`Error: PoolTimedOut`.

In Kubernetes both are set by the Deployment's `env` (see [kubernetes.md](kubernetes.md)).

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
