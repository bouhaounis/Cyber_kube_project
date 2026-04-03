#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ENGINE_DIR="$ROOT_DIR/engine"
EBPF_DIR="$ROOT_DIR/ebpf-aya"
POLICY_DIR_DEFAULT="$ROOT_DIR/policies"
EBPF_OBJECT_DEFAULT="$EBPF_DIR/target/bpfel-unknown-none/release/cyber_kube_ebpf"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "This script must be run on Linux." >&2
  exit 1
fi

require_cmd cargo
require_cmd rustc

if [[ $EUID -ne 0 ]] && ! command -v sudo >/dev/null 2>&1; then
  echo "sudo is required when not running as root because the engine needs eBPF privileges." >&2
  exit 1
fi

echo "Project root: $ROOT_DIR"
echo "Checking Rust toolchain..."
cargo --version
rustc --version

if ! rustup target list --installed | grep -qx "bpfel-unknown-none"; then
  echo "Installing Rust BPF target..."
  rustup target add bpfel-unknown-none
fi

echo "Building eBPF object..."
(cd "$EBPF_DIR" && cargo build --release --target bpfel-unknown-none)

if [[ ! -f "$EBPF_OBJECT_DEFAULT" ]]; then
  echo "Expected eBPF object not found: $EBPF_OBJECT_DEFAULT" >&2
  exit 1
fi

export POLICY_DIR="${POLICY_DIR:-$POLICY_DIR_DEFAULT}"
export POLICY_FAILURE_MODE="${POLICY_FAILURE_MODE:-alert}"
export POLICY_RELOAD_ENABLED="${POLICY_RELOAD_ENABLED:-true}"
export POLICY_RELOAD_INTERVAL_SECS="${POLICY_RELOAD_INTERVAL_SECS:-5}"

export ENGINE_METRICS_ENABLED="${ENGINE_METRICS_ENABLED:-true}"
export ENGINE_METRICS_ADDR="${ENGINE_METRICS_ADDR:-127.0.0.1:9465}"

export EBPF_ENABLED="${EBPF_ENABLED:-true}"
export EBPF_REQUIRED="${EBPF_REQUIRED:-true}"
export EBPF_PROGRAM_PATH="${EBPF_PROGRAM_PATH:-$EBPF_OBJECT_DEFAULT}"

export TLS_OBSERVABILITY_ENABLED="${TLS_OBSERVABILITY_ENABLED:-false}"
export TLS_OBSERVABILITY_REQUIRED="${TLS_OBSERVABILITY_REQUIRED:-false}"
export TLS_OBSERVABILITY_READ_SYMBOL="${TLS_OBSERVABILITY_READ_SYMBOL:-SSL_read}"
export TLS_OBSERVABILITY_WRITE_SYMBOL="${TLS_OBSERVABILITY_WRITE_SYMBOL:-SSL_write}"
export TLS_OBSERVABILITY_MAX_CAPTURE_BYTES="${TLS_OBSERVABILITY_MAX_CAPTURE_BYTES:-512}"

if [[ "$TLS_OBSERVABILITY_ENABLED" == "true" && -z "${TLS_OBSERVABILITY_LIBRARY_PATH:-}" ]]; then
  TLS_PATH="$(ldconfig -p 2>/dev/null | awk '/libssl\.so/ {print $NF; exit}')"
  if [[ -n "$TLS_PATH" ]]; then
    export TLS_OBSERVABILITY_LIBRARY_PATH="$TLS_PATH"
  else
    echo "TLS observability is enabled but libssl could not be auto-detected." >&2
    echo "Set TLS_OBSERVABILITY_LIBRARY_PATH manually." >&2
    exit 1
  fi
fi

echo "Using policy directory: $POLICY_DIR"
echo "Using eBPF object: $EBPF_PROGRAM_PATH"
echo "Metrics endpoint: http://$ENGINE_METRICS_ADDR/metrics"
if [[ "$TLS_OBSERVABILITY_ENABLED" == "true" ]]; then
  echo "TLS observability enabled with library: $TLS_OBSERVABILITY_LIBRARY_PATH"
else
  echo "TLS observability disabled"
fi

echo "Starting engine..."
if [[ $EUID -eq 0 ]]; then
  cd "$ENGINE_DIR"
  exec cargo run
else
  cd "$ENGINE_DIR"
  exec sudo -E cargo run
fi
