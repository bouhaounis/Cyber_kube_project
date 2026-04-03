# Cyber-Kube Policies

The engine loads JSON policies from this directory by default. You can override the location with:

```bash
POLICY_DIR=/path/to/policies
```

Supported rule condition syntax is intentionally small and deterministic:

- `event_type == "connect"`
- `metadata.namespace == "security"`
- `payload.message contains "escape"`
- `exists(payload.message)`
- `&&` and `||` combinations

Example:

```json
{
  "id": "alert-connect",
  "name": "Alert on Connect",
  "description": "Alerts on runtime connect events",
  "enabled": true,
  "rules": [
    {
      "id": "rule-1",
      "condition": "event_type == \"connect\" && metadata.namespace == \"security\"",
      "action": "Alert",
      "priority": 50
    }
  ]
}
```

If no policy matches, the engine uses its configured fallback mode and default decisions.

The directory also includes a disabled-by-default WASM example:

- `sample_wasm_policy.json`
- `sample_wasm_policy.wat`

That sample demonstrates the supported module shape:

- export `memory`
- export `alloc(i32) -> i32`
- export `evaluate(i32, i32) -> i32`

Policy hot reload is enabled by default when a Tokio runtime is available. You can control it with:

```bash
POLICY_RELOAD_ENABLED=true
POLICY_RELOAD_INTERVAL_SECS=5
```
