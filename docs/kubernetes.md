# Kubernetes

The manifests live in `k8s/`. Locally we run them on **minikube**, using
Docker Desktop as the driver.

| File | Status |
|---|---|
| `k8s/deployment.yaml` | **in use**. Manages the backend Pods |
| `k8s/pod.yaml` | deprecated. A bare Pod from before we used a Deployment; kept for reference |

## Concepts, as used here

- **Cluster:** everything kubectl talks to. Ours is the `minikube` cluster.
- **Node:** a machine in the cluster. minikube has one node, itself a Docker container. Nodes aren't defined in YAML; they join the cluster (minikube, or the k3s installer on a server).
- **Pod:** one running copy of the backend container, on one node.
- **Deployment:** a record saying "keep N Pods like this running". Kubernetes picks the nodes and replaces Pods that die. One Deployment per app, not per node.

The YAML never names a cluster. kubectl sends it to the **current context** in
`~/.kube/config`.

## `deployment.yaml`, field by field

```yaml
spec:
  replicas: 1                 # how many Pods to keep running
  selector:
    matchLabels:
      app: deoxys             # the Pods it owns = Pods with this label
  template:                   # blueprint for each Pod
    metadata:
      labels:
        app: deoxys           # must match the selector
    spec:
      containers:
        - name: backend
          image: ghcr.io/s9oon/deoxys-backend:latest
          imagePullPolicy: Always   # re-download :latest on every Pod start
          env:
            - name: PORT
              value: "3000"         # env values are always strings
          ports:
            - containerPort: 3000   # a number; must match PORT by hand
```

YAML can't reference variables, so `containerPort` can't say `PORT`. Keep the
two in sync yourself.

## Local setup (Windows 11 Home)

Home has no Hyper-V, so minikube must use the Docker driver.

1. Start Docker Desktop and wait for "Engine running".
2. Set the driver once: `minikube config set driver docker`
3. Start the cluster: `minikube start`
4. Point kubectl at it: `kubectl config use-context minikube`

**Gotcha:** if Docker Desktop's own Kubernetes is enabled, it creates a
`docker-desktop` context and may make it current. Then kubectl talks to that
cluster instead of minikube. Check with `kubectl config current-context`, and
turn it off in Docker Desktop → Settings → Kubernetes.

If minikube's container shows `Exited (255)`, Docker was stopped under it
(for example by sleep or quitting Docker Desktop). Run `minikube start` again.

## Deploying

```sh
kubectl apply -f k8s/deployment.yaml
kubectl get pods -l app=deoxys -o wide
kubectl port-forward deployment/deoxys 3000:3000
```

Then open `http://localhost:3000`.

### Testing a local image without GHCR

Build straight into minikube:

```sh
minikube image build -t deoxys-backend:dev -f backend/Dockerfile .
```

Then in the manifest use `image: deoxys-backend:dev` with
`imagePullPolicy: Never`.

## Updating to a new image

A running Pod never swaps its image. The Deployment replaces Pods instead.

**With `:latest` plus `imagePullPolicy: Always`** (current setup):

```sh
kubectl rollout restart deployment/deoxys
kubectl rollout status deployment/deoxys
```

**With a SHA tag** (better): set
`image: ghcr.io/s9oon/deoxys-backend:<sha>` and run `kubectl apply`. The image
name changed, so Kubernetes rolls it out on its own. To roll back, put the old
SHA back, or run `kubectl rollout undo deployment/deoxys`.

Later, CD can deploy itself (`kubectl set image ...` or GitOps with Argo
CD/Flux). That needs a cluster GitHub can reach, so it can't target minikube
on a laptop.

## Private images

If the GHCR package is private, Pods fail with `ImagePullBackOff`. Create a
pull secret from a GitHub token with the `read:packages` scope:

```sh
kubectl create secret docker-registry ghcr-creds \
  --docker-server=ghcr.io --docker-username=<github user> --docker-password=<token>
```

Reference it in the Pod template's `spec`:

```yaml
imagePullSecrets:
  - name: ghcr-creds
```

## Debugging

| Command | Shows |
|---|---|
| `kubectl get pods -o wide` | status, restarts, which node |
| `kubectl describe pod <name>` | events at the bottom: pull errors, crashes, scheduling |
| `kubectl logs deployment/deoxys` | the backend's stdout |
| `kubectl exec -it <pod> -- sh` | a shell in the container (trixie-slim has `sh`) |

## Beyond minikube

minikube is for local development only: one machine, local-only networking,
and storage that disappears with `minikube delete`. For a real server, use
**k3s**. The same `k8s/*.yaml` applies unchanged; only the kubectl context
differs. Extra nodes join with the k3s agent (`K3S_URL` + `K3S_TOKEN`), which
can be automated with Ansible or cloud-init.
