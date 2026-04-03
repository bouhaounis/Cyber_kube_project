#!/bin/bash
set -e

echo "Starting Cyber-Kube demo..."

# Start services
docker-compose up -d

# Wait for services to be ready
echo "Waiting for services to start..."
sleep 10

# Deploy vulnerable apps
kubectl apply -f deployments/kustomize/base/

# Run attack simulation
python3 scripts/demo/simulate-attacks.py

echo "Demo started! Dashboard: http://localhost:3000"
