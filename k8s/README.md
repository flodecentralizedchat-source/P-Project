## p-project Kubernetes Manifests

### Prereqs
- Push images to your registry or use a pull secret.
- Create required Secrets:
  - `p-project-secrets` with `DATABASE_URL`.
  - Optional `p-project-bridge-secrets` with bridge envs (see `secrets.example.yaml`).

### Build and push images
```
# From repo root
# Replace `your-ghcr-namespace` with your GH org/user

# API
docker build -f p-project-api/Dockerfile -t ghcr.io/your-ghcr-namespace/p-project-api:latest .
docker push ghcr.io/your-ghcr-namespace/p-project-api:latest

# Web (nginx)
docker build -f p-project-web/Dockerfile -t ghcr.io/your-ghcr-namespace/p-project-web:latest .
docker push ghcr.io/your-ghcr-namespace/p-project-web:latest

# Bridge relayer
docker build -f p-project-bridge/Dockerfile -t ghcr.io/your-ghcr-namespace/p-project-bridge-relayer:latest .
docker push ghcr.io/your-ghcr-namespace/p-project-bridge-relayer:latest

# Airdrop cron
docker build -f p-project-airdrop/Dockerfile -t ghcr.io/your-ghcr-namespace/p-project-airdrop-cron:latest .
docker push ghcr.io/your-ghcr-namespace/p-project-airdrop-cron:latest
```

### Apply
Using kustomize to create the namespace and deploy all resources:
```
# Create real secrets (or edit a copy of secrets.example.yaml) first
kubectl apply -f k8s/secrets.example.yaml    # dev only; replace values

# Apply all manifests into the `p-project` namespace
kubectl apply -k k8s
```

Validate without changing the cluster:
```
kubectl kustomize k8s | kubectl apply --dry-run=client -f -
```

### Notes
- API listens on port 3000; web on port 80. Ingress routes:
  - Web: http://p-project.local/
  - API: http://api.p-project.local/
- For production, point `DATABASE_URL` at managed MySQL. Redis/Mongo are not deployed here.
- Bridge relayer requires chain RPCs and keys in `p-project-bridge-secrets` to enable Ethereum support.
-- Replace `your-ghcr-namespace` in images with your GitHub user/org (matches GHCR pushes from the workflow).
-- All resources are namespaced under `p-project` via kustomization. Use `-n p-project` when querying.
