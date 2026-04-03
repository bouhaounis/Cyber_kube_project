#!/bin/bash
set -e

echo "Setting up Cyber-Kube development environment..."

# Check prerequisites
command -v rustc >/dev/null 2>&1 || { echo "Rust not found. Install from https://rustup.rs/"; exit 1; }
command -v go >/dev/null 2>&1 || { echo "Go not found. Install from https://golang.org/dl/"; exit 1; }
command -v node >/dev/null 2>&1 || { echo "Node.js not found. Install from https://nodejs.org/"; exit 1; }
command -v python3 >/dev/null 2>&1 || { echo "Python3 not found. Install Python 3.10+"; exit 1; }

# Install Rust dependencies
echo "Installing Rust dependencies..."
cd ebpf-aya && cargo build --release && cd ..
cd engine && cargo build --release && cd ..

# Install Go dependencies
echo "Installing Go dependencies..."
cd api-go && go mod download && cd ..

# Install Node.js dependencies
echo "Installing Node.js dependencies..."
cd dashboard && npm install && cd ..

# Install Python dependencies
echo "Installing Python dependencies..."
python3 -m venv .venv
source .venv/bin/activate
pip install -r ml-models/requirements.txt

echo "Setup complete!"
