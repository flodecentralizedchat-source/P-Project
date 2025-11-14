## p-project Kubernetes Manifests

### Prereqs
- Push images to your registry or use a pull secret.
- Create required Secrets:
  - `p-project-secrets` with `DATABASE_URL`.
  - Optional `p-project-bridge-secrets` with bridge envs (see `secrets.example.yaml`).

### Build and push images
```
# From repo root
# API
docker build -f p-project-api/Dockerfile -t your-registry/p-project-api:latest .
docker push your-registry/p-project-api:latest

# Web (nginx)
docker build -f p-project-web/Dockerfile -t your-registry/p-project-web:latest .
docker push your-registry/p-project-web:latest

# Bridge relayer
docker build -f p-project-bridge/Dockerfile -t your-registry/p-project-bridge-relayer:latest .
docker push your-registry/p-project-bridge-relayer:latest

# Airdrop cron
docker build -f p-project-airdrop/Dockerfile -t your-registry/p-project-airdrop-cron:latest .
docker push your-registry/p-project-airdrop-cron:latest
```

### Apply
```
kubectl apply -f k8s/secrets.example.yaml   # edit values first or create your own
kubectl apply -f k8s/api.yaml
kubectl apply -f k8s/web.yaml
kubectl apply -f k8s/bridge-relayer.yaml
kubectl apply -f k8s/airdrop-cronjob.yaml
kubectl apply -f k8s/ingress.yaml
```

### Notes
- API listens on port 3000; web on port 80. Ingress routes:
  - Web: http://p-project.local/
  - API: http://api.p-project.local/
- For production, point `DATABASE_URL` at managed MySQL. Redis/Mongo are not deployed here.
- Bridge relayer requires chain RPCs and keys in `p-project-bridge-secrets` to enable Ethereum support.
-- Replace `your-ghcr-namespace` in images with your GitHub user/org (matches GHCR pushes from the workflow).
