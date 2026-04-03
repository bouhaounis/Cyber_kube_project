#!/usr/bin/env python3
"""Simulate security attacks for Cyber-Kube demo"""

import requests
import time
import json

API_URL = "http://localhost:8080"

def simulate_container_escape():
    """Simulate container escape attempt"""
    print("Simulating container escape attack...")
    alert = {
        "kind": "CONTAINER_ESCAPE_ATTEMPT",
        "pid": 12345,
        "tgid": 12345,
        "uid": 1000,
        "message": "Process attempting to access /proc/self/exe from container"
    }
    response = requests.post(f"{API_URL}/api/v1/alerts", json=alert)
    print(f"Alert sent: {response.status_code}")

def simulate_crypto_mining():
    """Simulate crypto mining detection"""
    print("Simulating crypto mining attack...")
    alert = {
        "kind": "CRYPTO_MINING",
        "pid": 12346,
        "message": "High CPU usage detected, possible crypto mining"
    }
    response = requests.post(f"{API_URL}/api/v1/alerts", json=alert)
    print(f"Alert sent: {response.status_code}")

def simulate_network_scan():
    """Simulate network scanning"""
    print("Simulating network scan attack...")
    alert = {
        "kind": "NETWORK_SCAN",
        "src_ip": "10.0.0.100",
        "dst_ip": "10.0.0.1",
        "message": "Port scan detected from pod"
    }
    response = requests.post(f"{API_URL}/api/v1/alerts", json=alert)
    print(f"Alert sent: {response.status_code}")

if __name__ == "__main__":
    print("Starting attack simulation...")
    time.sleep(2)
    simulate_container_escape()
    time.sleep(2)
    simulate_crypto_mining()
    time.sleep(2)
    simulate_network_scan()
    print("Attack simulation complete!")
