# Kubernetes

The backend runs on Kubernetes. The manifests in `k8s/` are plain Kubernetes
objects with nothing provider-specific, so the same files apply to any
conformant cluster: a local one (minikube, kind, k3d), a self-hosted k3s
server, or a managed service (EKS, GKE, AKS, DigitalOcean).

| File | Status |
|---|---|
| `k8s/deployment.yaml` | **in use**. Runs the backend |
| `k8s/pod.yaml` | deprecated. Superseded by the Deployment; kept for reference |

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
          ports:
            - containerPort: 3000   # a number; must match PORT by hand
```

`PORT` and `containerPort` must agree. YAML can't reference the env var, so
they're kept in sync by hand.

## Deploying and updating

```sh
kubectl apply -f k8s/deployment.yaml
kubectl get pods -l app=deoxys -o wide
kubectl port-forward deployment/deoxys 3000:3000
```

A running Pod never changes its image. The Deployment rolls out new Pods
instead: it starts a new Pod, waits for it, then removes the old one.

- **Using `:latest`:** a new push to GHCR doesn't change the manifest, so trigger the rollout yourself with `kubectl rollout restart deployment/deoxys`. `imagePullPolicy: Always` makes the new Pod download the new image.
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
| `kubectl exec -it <pod> -- sh` | a shell in the container (the trixie-slim base includes `sh`) |
