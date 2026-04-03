# Cyber-Kube Deployment Guide

## Prerequisites

- Docker and Docker Compose
- Kubernetes cluster (for production)
- PostgreSQL (optional, in-memory fallback available)
- Redis (optional, for caching)

## Development Deployment

### Quick Start

```bash
# Clone repository
git clone https://github.com/your-org/cyber-kube.git
cd cyber-kube

# Start all services
docker-compose up -d

# Or use the demo script
./scripts/demo/demo.sh  # Linux/Mac
.\scripts\demo\demo.ps1  # Windows
```

### Services

- **API Server**: http://localhost:8081
- **Dashboard**: http://localhost:3000
- **Metrics**: http://localhost:8081/api/v1/metrics

### Environment Variables

Create `.env` file:

```env
# API Configuration
ADDR=:8081
GIN_MODE=release

# Database
DATABASE_URL=postgres://user:password@localhost:5432/cyberkube

# Redis
REDIS_URL=redis://localhost:6379

# JWT
JWT_SECRET=your-secret-key-change-in-production

# Engine
ENGINE_URL=http://localhost:7000

# Kubernetes
KUBECONFIG=/path/to/kubeconfig
```

## Production Deployment

### Kubernetes

```bash
# Apply manifests
kubectl apply -f deployments/k8s/

# Or use Helm
helm install cyber-kube deployments/helm/
```

### Docker Compose

```bash
docker-compose -f docker-compose.prod.yml up -d
```

## Configuration

### API Server

Configuration via environment variables (see `api-go/internal/config/config.go`)

### Dashboard

Set `VITE_API_URL` environment variable:

```bash
VITE_API_URL=http://api.example.com:8081 npm run build
```

## Monitoring

### Prometheus

Metrics endpoint: `/api/v1/metrics`

### Grafana

Import dashboard from `deployments/grafana/dashboard.json`

## Security

1. Change default JWT secret
2. Enable HTTPS/TLS
3. Configure CORS properly
4. Set up firewall rules
5. Use secrets management (Kubernetes secrets, Vault, etc.)

## Troubleshooting

### API not starting

- Check port 8081 is available
- Verify database connection (if using PostgreSQL)
- Check logs: `docker-compose logs api-go`

### Dashboard not connecting

- Verify `VITE_API_URL` is correct
- Check CORS configuration
- Verify API is running

### WebSocket not working

- Check firewall rules
- Verify WebSocket endpoint is accessible
- Check browser console for errors

## Backup & Recovery

### Database Backup

```bash
pg_dump -U user cyberkube > backup.sql
```

### Restore

```bash
psql -U user cyberkube < backup.sql
```
