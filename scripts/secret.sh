#!/usr/bin/env bash
# Creates or updates the deoxys-db Secret from the root .env.
# Run from anywhere: bash scripts/secret.sh
set -euo pipefail

# Run from the repo root, whatever folder you call this from
cd "$(dirname "$0")/.."

kubectl create secret generic deoxys-db \
  --from-env-file=.env \
  --dry-run=client -o yaml | kubectl apply -f -
