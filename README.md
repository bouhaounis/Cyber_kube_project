# 🛡️ Cyber-Kube

[![CI/CD](https://github.com/your-org/cyber-kube/workflows/CI/badge.svg)](https://github.com/your-org/cyber-kube/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Go Version](https://img.shields.io/badge/go-1.22-blue.svg)](https://golang.org)
[![Rust Version](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://rust-lang.org)

**Zero-Trust Kubernetes Security Platform with eBPF Monitoring and AI**

Cyber-Kube is a comprehensive security platform that provides real-time threat detection, policy enforcement, and automated remediation for Kubernetes clusters using eBPF, machine learning, and Zero-Trust principles.

## ✨ Features

- 🔍 **eBPF-Powered Monitoring**: Kernel-level syscall tracing and network monitoring
- 🤖 **AI Threat Detection**: ML-based anomaly detection and threat classification
- 🛡️ **Zero-Trust Policies**: WASM-based policy engine with 10+ predefined policies
- 📊 **Real-Time Dashboard**: Interactive 3D cluster visualization with React + Three.js
- 🔔 **Real-Time Alerts**: WebSocket-based event streaming
- 🔐 **Security**: JWT authentication, rate limiting, CORS protection
- 📈 **Observability**: Prometheus metrics, structured logging
- 🚀 **Production-Ready**: Complete CI/CD, Docker, Kubernetes deployment

## 🏗️ Architecture

```
┌─────────────┐
│   React     │  Dashboard (Port 3000)
│  Dashboard  │
└──────┬──────┘
       │ HTTP/WebSocket
┌──────▼──────────┐
│   Go API        │  REST API (Port 8081)
│   Server        │  - JWT Auth
└──────┬──────────┘  - Rate Limiting
       │            - PostgreSQL
┌──────▼──────────┐
│  Rust Engine    │  Policy Engine (Port 7000)
│  - Policy Eval  │  - ML Inference
│  - ML Models    │  - gRPC API
└──────┬──────────┘
       │
┌──────▼──────────┐
│  eBPF Program   │  Kernel Monitoring
│  (Linux only)   │  - Syscall Tracing
└─────────────────┘  - XDP Filtering
```

## 🚀 Quick Start

### Prerequisites

- Docker & Docker Compose
- Node.js 20+ (for dashboard)
- Go 1.22+ (for API)
- Rust 1.70+ (for engine, optional)

### Development

```bash
# Clone repository
git clone https://github.com/your-org/cyber-kube.git
cd cyber-kube

# Start all services
docker-compose up -d

# Or use demo script
./scripts/demo/demo.sh  # Linux/Mac
.\scripts\demo\demo.ps1  # Windows
```

### Access Services

- **Dashboard**: http://localhost:3000
- **API**: http://localhost:8081
- **API Health**: http://localhost:8081/api/v1/health
- **Metrics**: http://localhost:8081/api/v1/metrics

## 📖 Documentation

- [Architecture Guide](docs/ARCHITECTURE.md)
- [API Documentation](docs/API.md)
- [Deployment Guide](docs/DEPLOYMENT.md)
- [Quick Start Guide](README_QUICKSTART.md)

## 🧪 Testing

```bash
# Test Go API
cd api-go
go test -v -coverprofile=coverage.out ./...

# Test Dashboard
cd dashboard
npm test
```

## 🛠️ Development

### API Server

```bash
cd api-go
go run .
```

### Dashboard

```bash
cd dashboard
npm install
npm run dev
```

### Rust Engine

```bash
cd engine
cargo run
```

## 📦 Components

### 1. Frontend Dashboard (`dashboard/`)

React + TypeScript dashboard with:
- Real-time security monitoring
- 3D cluster visualization (Three.js)
- Policy management UI
- Alert dashboard
- Dark/Light mode

### 2. Go API Server (`api-go/`)

RESTful API with:
- JWT authentication
- Rate limiting
- PostgreSQL integration
- WebSocket support
- Prometheus metrics

### 3. Rust Security Engine (`engine/`)

Policy engine with:
- WASM runtime for policies
- ML model inference (ONNX)
- gRPC API
- eBPF event processing

### 4. eBPF Program (`ebpf-aya/`)

Kernel-level monitoring:
- Syscall tracing
- Container escape detection
- Network filtering (XDP)
- Perf buffer events

## 🔒 Security Features

- **Zero-Trust Architecture**: All communications authenticated
- **Container Escape Detection**: PID namespace monitoring
- **Network Anomaly Detection**: ML-based traffic analysis
- **Privilege Escalation Prevention**: Syscall monitoring
- **Automated Remediation**: Policy-based blocking

## 📊 Monitoring

- Prometheus metrics at `/api/v1/metrics`
- Structured logging
- Real-time WebSocket events
- Grafana dashboards (planned)

## 🚢 Deployment

### Docker Compose

```bash
docker-compose up -d
```

### Kubernetes

```bash
kubectl apply -f deployments/k8s/
```

### Helm

```bash
helm install cyber-kube deployments/helm/
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📝 License

MIT License - see [LICENSE](LICENSE) file for details

## 🙏 Acknowledgments

- [Aya](https://github.com/aya-rs/aya) - eBPF framework
- [Gin](https://github.com/gin-gonic/gin) - Web framework
- [Three.js](https://threejs.org/) - 3D graphics

## 📧 Contact

For questions or support, please open an issue on GitHub.---**Built with ❤️ for Kubernetes Security**