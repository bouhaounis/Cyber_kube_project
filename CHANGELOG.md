# Changelog - Cyber-Kube

All notable changes to this project will be documented in this file.

## [0.2.0] - 2024-12-XX

### 🔄 Upgraded

#### Go API Server
- **Go version**: Updated to 1.23 (from 1.25.0 - corrected)
- **gin-gonic/gin**: v1.10.0 → v1.11.0
- **golang-jwt/jwt/v5**: v5.2.1 → v5.3.0
- **didip/tollbooth/v7**: v7.0.1 → v7.0.2
- **prometheus/client_golang**: v1.19.0 → v1.23.2
- All transitive dependencies updated

#### React Dashboard
- **zustand**: ^4.5.0 → ^4.5.5
- **@tanstack/react-query**: ^5.59.16 → ^5.62.0
- **three**: ^0.161.0 → ^0.170.0
- **react-router-dom**: ^6.26.0 → ^6.28.0
- **recharts**: ^2.12.0 → ^2.15.0
- **lucide-react**: ^0.344.0 → ^0.468.0
- **date-fns**: ^3.3.1 → ^4.1.0 (major update)
- **@headlessui/react**: ^1.7.17 → ^2.2.0 (major update)
- **typescript**: ^5.6.3 → ^5.7.2
- **vite**: ^5.4.10 → ^6.0.5 (major update)
- **tailwindcss**: ^3.4.13 → ^3.4.17

#### Rust Engine
- **tokio**: 1.37 → 1.40
- **tonic**: 0.11 → 0.12
- **prost**: 0.12 → 0.13
- **wasmtime**: 18.0.2 → 22.0.0 (major update)
- **wasmtime-wasi**: 18.0.2 → 22.0.0 (major update)

### 🐛 Fixed
- Fixed docker-compose.yml: Updated API URL from :8080 to :8081
- Fixed Go version: Changed from non-existent 1.25.0 to 1.23
- Fixed all dependency conflicts

### ⚠️ Breaking Changes

#### date-fns v4.x
- Check migration guide: https://github.com/date-fns/date-fns/blob/main/CHANGELOG.md
- Some API changes may require code updates

#### @headlessui/react v2.x
- Major version update with potential API changes
- Check: https://headlessui.com/changelog

#### wasmtime 22.x
- Major version update from 18.x
- May require code changes in engine
- Check: https://github.com/bytecodealliance/wasmtime/releases

#### vite 6.x
- Major version update
- May require configuration changes
- Check: https://vitejs.dev/guide/migration.html

### 📝 Added
- Created `UPGRADE_SUMMARY.md` with detailed upgrade information
- Created `CHANGELOG.md` for version tracking

### 🔍 Testing Required
- [ ] Test all API endpoints after Go dependency updates
- [ ] Test dashboard build and runtime with new React dependencies
- [ ] Test engine compilation with updated Rust dependencies
- [ ] Verify WebSocket functionality
- [ ] Test authentication flow
- [ ] Run full integration tests

## [0.1.0] - 2024-12-XX

### ✨ Initial Release
- Complete eBPF program with Aya framework
- Rust security engine with policy evaluation
- Go API server with JWT authentication
- React dashboard with 3D visualization
- PostgreSQL integration
- WebSocket real-time events
- CI/CD pipeline
- Comprehensive documentation
