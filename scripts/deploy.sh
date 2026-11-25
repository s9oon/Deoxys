#!/usr/bin/env bash
# Deploys or updates everything. Safe to run repeatedly.
# Run from anywhere: bash scripts/deploy.sh
set -euo pipefail
cd "$(dirname "$0")/.."

bash scripts/secret.sh
kubectl apply -f infrastructure/k8s/postgresql.yaml
kubectl apply -f infrastructure/k8s/deployment.yaml

# Pick up a new :latest image even if the YAML didn't change
kubectl rollout restart deployment/deoxys
kubectl rollout status deployment/deoxys
