# Cyber-Kube Architecture

## Overview

Cyber-Kube is a Zero-Trust Kubernetes security platform that combines eBPF monitoring, AI-powered threat detection, and automated remediation.

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     React Dashboard                          │
│  (Port 3000) - Real-time visualization & management         │
└──────────────────────┬──────────────────────────────────────┘
                       │ HTTP/WebSocket
┌──────────────────────▼──────────────────────────────────────┐
│                    Go API Server                             │
│  (Port 8081) - REST API, WebSocket, Authentication          │
└──────┬──────────────────────┬──────────────────────┬────────┘
       │                      │                      │
       │ gRPC                 │ Database             │ WebSocket
┌──────▼────────┐   ┌────────▼────────┐   ┌────────▼────────┐
│ Rust Engine   │   │  PostgreSQL      │   │  WebSocket       │
│ (Port 7000)   │   │  (Port 5432)     │   │  Manager         │
│ Policy Engine │   │  Data Storage    │   │  Real-time Events│
│ ML Inference  │   └──────────────────┘   └──────────────────┘
└──────┬────────┘
       │
       │ eBPF Events
┌──────▼──────────────────────────────────────┐
│         eBPF Program (Linux)                │
│  - Syscall tracing (execve, connect, bind) │
│  - Container escape detection              │
│  - XDP network filtering                    │
│  - Perf buffer events                       │
└─────────────────────────────────────────────┘
```

## Components

### 1. Frontend Dashboard (React + TypeScript)

**Location:** `dashboard/`

**Technologies:**
- React 18 with TypeScript
- Three.js for 3D cluster visualization
- Recharts for data visualization
- TailwindCSS for styling
- Zustand for state management
- React Router for navigation

**Features:**
- Real-time security dashboard
- Policy management UI
- Alert monitoring
- 3D cluster topology visualization
- Dark/Light mode
- WebSocket integration for live updates

### 2. Go API Server

**Location:** `api-go/`

**Technologies:**
- Gin web framework
- GORM for database ORM
- JWT for authentication
- Gorilla WebSocket for real-time events
- Prometheus for metrics

**Endpoints:**
- `GET /api/v1/health` - Health check
- `GET /api/v1/metrics` - Prometheus metrics
- `POST /api/v1/auth/login` - Authentication
- `GET/POST/PUT/DELETE /api/v1/policies` - Policy CRUD
- `GET/POST /api/v1/alerts` - Alert management
- `GET /api/v1/events` - WebSocket endpoint

**Security:**
- JWT authentication
- Rate limiting (Tollbooth)
- CORS configuration
- Input validation

### 3. Rust Security Engine

**Location:** `engine/`

**Technologies:**
- Tokio for async runtime
- Tonic for gRPC
- Wasmtime for WASM policy execution
- ONNX Runtime for ML inference

**Features:**
- Policy evaluation engine
- ML-based anomaly detection
- eBPF event processing
- gRPC API for policy decisions

### 4. eBPF Program

**Location:** `ebpf-aya/`

**Technologies:**
- Aya framework (Rust)
- eBPF for kernel-level monitoring

**Capabilities:**
- Syscall tracing (execve, connect, bind)
- Container escape detection
- Network traffic monitoring (XDP)
- Perf buffer event streaming

### 5. Database (PostgreSQL)

**Models:**
- Policies
- Alerts
- Events
- Users

**Migrations:** Auto-migrated via GORM

## Data Flow

### 1. Threat Detection Flow

```
eBPF Program → Perf Buffer → Rust Engine → Policy Evaluation
                                              ↓
                                    ML Anomaly Detection
                                              ↓
                                    Alert Generation
                                              ↓
                                    Go API → WebSocket → Dashboard
```

### 2. Policy Enforcement Flow

```
Dashboard → Go API → Rust Engine → Policy Decision
                                      ↓
                              eBPF Program Update
                                      ↓
                              Kernel Enforcement
```

## Security Features

1. **Zero-Trust Architecture**
   - All communications authenticated
   - Least privilege access
   - Network segmentation

2. **Threat Detection**
   - Container escape detection
   - Privilege escalation monitoring
   - Network anomaly detection
   - Crypto-mining detection

3. **Automated Response**
   - Policy-based blocking
   - Alert generation
   - Real-time notifications

## Deployment

### Development
```bash
docker-compose up
```

### Production
- Kubernetes manifests in `deployments/k8s/`
- Helm charts in `deployments/helm/`
- CI/CD via GitHub Actions

## Monitoring

- Prometheus metrics at `/api/v1/metrics`
- Structured logging
- Distributed tracing (planned)

## Performance

- eBPF: <1ms overhead per event
- API: <10ms average response time
- WebSocket: <100ms latency
- ML Inference: <50ms per prediction
