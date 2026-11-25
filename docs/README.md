# Deoxys docs

How the pieces of this repo fit together and how to work on them.

| Page | What it covers |
|---|---|
| [shared-types.md](shared-types.md) | The `shared/` crate, the Cargo workspace, and generating TypeScript types with ts-rs |
| [backend.md](backend.md) | Running the backend, its Docker image, and CI/CD to GHCR |
| [kubernetes.md](kubernetes.md) | The `k8s/` manifests, minikube on Windows, deploying and updating |
| [desktop.md](desktop.md) | The Tauri app: dev, IPC, `BACKEND_URL`, building installers |

## How a request flows

```
SolidJS page (desktop/src)
   │  invoke("get_hello")            ← Tauri IPC, typed with desktop/src/ipc/Hello.ts
   ▼
Tauri Rust side (desktop/src-tauri)
   │  reqwest GET {BACKEND_URL}/     ← HTTP, parsed into shared::Hello
   ▼
Backend (backend/, axum)
   │  returns Json(shared::Hello)
   ▼
{"message":"Hello, World!"}
```

The frontend never calls the backend directly. Only Rust code talks HTTP, so the
URL and any future tokens stay out of the webview.

## Repo layout

```
Deoxys/
├── Cargo.toml          # Cargo workspace: backend + shared (desktop is excluded)
├── Cargo.lock          # one lockfile for the workspace
├── .cargo/config.toml  # where ts-rs writes generated TypeScript
├── .dockerignore       # keeps the backend image build context small
├── shared/             # types used by backend and desktop
├── backend/            # axum server + Dockerfile
├── desktop/            # Tauri + SolidJS app (its own Cargo project)
│   ├── src/            # frontend; src/ipc/ is generated
│   └── src-tauri/      # Rust side of the app
├── k8s/                # Kubernetes manifests
└── .github/workflows/  # CI (build/test) and CD (push image to GHCR)
```

## Quick commands

All from the repo root unless noted.

| Task | Command |
|---|---|
| Run the backend | `cargo run -p backend` |
| Regenerate TS types | `cargo test -p shared --features ts` |
| Build the backend image | `docker build -f backend/Dockerfile -t deoxys-backend:dev .` |
| Run the desktop app (from `desktop/`) | `npm run tauri dev` |
| Build installers (from `desktop/`) | `npm run tauri build` |
| Deploy to the cluster | `kubectl apply -f k8s/deployment.yaml` |
| Roll out a new `:latest` | `kubectl rollout restart deployment/deoxys` |
