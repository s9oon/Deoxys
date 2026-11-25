# Kubernetes

The backend runs on Kubernetes. The manifests in `infrastructure/k8s/` are plain Kubernetes
objects with nothing provider-specific, so the same files apply to any
conformant cluster: a local one (minikube, kind, k3d), a self-hosted k3s
server, or a managed service (EKS, GKE, AKS, DigitalOcean).

| File | Status |
|---|---|
| `infrastructure/k8s/deployment.yaml` | **in use**. Runs the backend |
| `infrastructure/k8s/postgresql.yaml` | **in use**. Postgres: a Service and a StatefulSet |
| `infrastructure/k8s/pod.yaml` | deprecated. Superseded by the Deployment; kept for reference |

## How the pieces relate

- **Cluster:** a set of nodes plus the control plane (API server, scheduler, controllers). Which one you're talking to is set by your kubectl **context**. The manifests never name a cluster.
- **Node:** a machine that runs Pods. Nodes come from whatever provides the cluster, not from these manifests.
- **Deployment:** a declaration that says "keep N Pods like this running". It's stored in the cluster and acted on by the control plane. It isn't tied to any node.
- **Pod:** one running copy of the backend container. The scheduler places each Pod on a node with room for it.

We declare **how many** Pods there are and **what each Pod is**. Kubernetes
decides **where** they run, and replaces them when they die.

## `deployment.yaml`

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: deoxys
spec:
  replicas: 1                 # how many backend Pods to keep running
  selector:
    matchLabels:
      app: deoxys             # the Pods it owns = Pods carrying this label
  template:                   # blueprint for each Pod
    metadata:
      labels:
        app: deoxys           # must match the selector
    spec:
      containers:
        - name: backend
          image: ghcr.io/s9oon/deoxys-backend:latest
          imagePullPolicy: Always   # re-pull :latest whenever a Pod starts
          env:
            - name: PORT
              value: "3000"         # read by the backend at startup; always a string
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:       # from the deoxys-db Secret, not written here
                  name: deoxys-db
                  key: DATABASE_URL
          ports:
            - containerPort: 3000   # a number; must match PORT by hand
```

`PORT` and `containerPort` must agree. YAML can't reference the env var, so
they're kept in sync by hand.

## `postgresql.yaml`

- **Service `postgres`** gives the database the in-cluster name `postgres:5432`. That's the host in `DATABASE_URL`.
- **StatefulSet `postgres`** runs one `postgres:17` pod with a 1Gi volume at `/var/lib/postgresql/data`, so data survives restarts. It reads `POSTGRES_PASSWORD` from the `deoxys-db` Secret.
- **Keep `replicas: 1`.** More replicas would be separate databases with separate data, not copies of one.
- Postgres only reads `POSTGRES_PASSWORD` when its volume is empty. Changing it later does nothing until the volume is deleted.

## The `deoxys-db` Secret

Its values come from `.env` at the repo root (git-ignored):

```
POSTGRES_PASSWORD=...
DATABASE_URL=postgres://postgres:...@postgres:5432/postgres
```

No quotes, no spaces around `=`, no comments after a value. The password
appears in both lines. `scripts/secret.sh` turns the file into the Secret and
can be rerun after editing it. Pods only read Secret env vars when they start,
so restart them afterwards.

## Deploying and updating

```sh
bash scripts/deploy.sh
```

It applies the Secret, `postgresql.yaml` and `deployment.yaml`, then restarts
the backend and waits for it. Run it with **Git Bash**: in PowerShell, `bash`
is WSL's bash, which can't reach minikube on Windows' localhost.

```sh
kubectl get pods
kubectl port-forward deployment/deoxys 3000:3000
curl localhost:3000/health
```

The backend may restart once or twice on a fresh deploy while Postgres is
still starting (`PoolTimedOut`).

A running Pod never changes its image. The Deployment rolls out new Pods
instead: it starts a new Pod, waits for it, then removes the old one.

- **Using `:latest`:** a new push to GHCR doesn't change the manifest, so the rollout has to be triggered with `kubectl rollout restart deployment/deoxys` (`deploy.sh` does this). `imagePullPolicy: Always` makes the new Pod download the new image.
- **Using a SHA tag:** setting `image: ghcr.io/s9oon/deoxys-backend:<sha>` changes the manifest, so `kubectl apply` rolls it out on its own. Roll back with `kubectl rollout undo deployment/deoxys`.

## Private images

If the GHCR package is private, the cluster needs pull credentials. Without
them, Pods sit in `ImagePullBackOff`.

```sh
kubectl create secret docker-registry ghcr-creds \
  --docker-server=ghcr.io --docker-username=<github user> --docker-password=<token with read:packages>
```

```yaml
# in the Pod template's spec
imagePullSecrets:
  - name: ghcr-creds
```

## Debugging

| Command | Shows |
|---|---|
| `kubectl get pods -o wide` | status, restarts, and which node each Pod is on |
| `kubectl describe pod <name>` | events: pull errors, crashes, scheduling failures |
| `kubectl logs deployment/deoxys` | backend stdout |
| `kubectl logs postgres-0` | Postgres output |
| `kubectl exec -it <pod> -- sh` | a shell in the container (the trixie-slim base includes `sh`) |
