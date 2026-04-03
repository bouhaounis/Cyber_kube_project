#!/bin/bash

set -e

echo "🚀 Cyber-Kube Demo Script"
echo "=========================="
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check dependencies
echo -e "${BLUE}Checking dependencies...${NC}"
command -v docker >/dev/null 2>&1 || { echo -e "${RED}Docker is required but not installed.${NC}" >&2; exit 1; }
command -v docker-compose >/dev/null 2>&1 || { echo -e "${RED}docker-compose is required but not installed.${NC}" >&2; exit 1; }

# Start services
echo -e "${GREEN}Starting services...${NC}"
docker-compose up -d

# Wait for services to be ready
echo -e "${BLUE}Waiting for services to be ready...${NC}"
sleep 10

# Check API health
echo -e "${BLUE}Checking API health...${NC}"
for i in {1..30}; do
    if curl -s http://localhost:8081/api/v1/health > /dev/null; then
        echo -e "${GREEN}✓ API is ready${NC}"
        break
    fi
    if [ $i -eq 30 ]; then
        echo -e "${RED}✗ API failed to start${NC}"
        exit 1
    fi
    sleep 1
done

# Create test policies
echo -e "${BLUE}Creating test policies...${NC}"
curl -X POST http://localhost:8081/api/v1/policies \
  -H "Content-Type: application/json" \
  -d '{
    "id": "policy-1",
    "name": "Block Container Escape",
    "description": "Detect and block container escape attempts"
  }' | jq .

curl -X POST http://localhost:8081/api/v1/policies \
  -H "Content-Type: application/json" \
  -d '{
    "id": "policy-2",
    "name": "Network Isolation",
    "description": "Enforce network policies between namespaces"
  }' | jq .

# Create test alerts
echo -e "${BLUE}Creating test alerts...${NC}"
curl -X POST http://localhost:8081/api/v1/alerts \
  -H "Content-Type: application/json" \
  -d '{
    "kind": "Container Escape",
    "severity": "high",
    "message": "Suspicious process detected attempting to escape container"
  }' | jq .

curl -X POST http://localhost:8081/api/v1/alerts \
  -H "Content-Type: application/json" \
  -d '{
    "kind": "Network Scan",
    "severity": "medium",
    "message": "Multiple connection attempts detected"
  }' | jq .

# Display summary
echo ""
echo -e "${GREEN}=========================="
echo -e "Demo Setup Complete!"
echo -e "==========================${NC}"
echo ""
echo -e "${BLUE}Services:${NC}"
echo "  • API: http://localhost:8081"
echo "  • Dashboard: http://localhost:3000"
echo "  • Metrics: http://localhost:8081/api/v1/metrics"
echo ""
echo -e "${BLUE}Test Endpoints:${NC}"
echo "  • Health: curl http://localhost:8081/api/v1/health"
echo "  • Policies: curl http://localhost:8081/api/v1/policies"
echo "  • Alerts: curl http://localhost:8081/api/v1/alerts"
echo ""
echo -e "${YELLOW}To stop services: docker-compose down${NC}"
