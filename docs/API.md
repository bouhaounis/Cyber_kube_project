# Cyber-Kube API Documentation

## Base URL

```
http://localhost:8081/api/v1
```

## Authentication

Most endpoints require JWT authentication. Include the token in the Authorization header:

```
Authorization: Bearer <token>
```

### Login

**POST** `/auth/login`

Request body:
```json
{
  "username": "admin",
  "password": "password"
}
```

Response:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_at": "2024-01-01T12:00:00Z",
  "username": "admin"
}
```

## Policies

### List Policies

**GET** `/policies`

Response:
```json
[
  {
    "id": "policy-1",
    "name": "Block Container Escape",
    "description": "Detect and block container escape attempts",
    "created_at": "2024-01-01T12:00:00Z"
  }
]
```

### Get Policy

**GET** `/policies/:id`

### Create Policy

**POST** `/policies`

Request body:
```json
{
  "id": "policy-1",
  "name": "Block Container Escape",
  "description": "Detect and block container escape attempts"
}
```

### Update Policy

**PUT** `/policies/:id`

### Delete Policy

**DELETE** `/policies/:id`

## Alerts

### List Alerts

**GET** `/alerts`

Query parameters:
- `severity`: Filter by severity (high, medium, low)
- `resolved`: Filter by resolved status (true/false)

Response:
```json
[
  {
    "id": "1",
    "kind": "Container Escape",
    "severity": "high",
    "message": "Suspicious process detected",
    "created_at": "2024-01-01T12:00:00Z"
  }
]
```

### Create Alert

**POST** `/alerts`

Request body:
```json
{
  "kind": "Container Escape",
  "severity": "high",
  "message": "Suspicious process detected"
}
```

### Resolve Alert

**PUT** `/alerts/:id/resolve`

## Events

### WebSocket Connection

**GET** `/events` (WebSocket)

Connect to receive real-time events:

```javascript
const ws = new WebSocket('ws://localhost:8081/api/v1/events');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log(data);
};
```

Event types:
- `alert`: New security alert
- `event`: Security event

## Health & Metrics

### Health Check

**GET** `/health`

Response:
```json
{
  "status": "ok",
  "timestamp": "2024-01-01T12:00:00Z",
  "version": "1.0.0"
}
```

### Prometheus Metrics

**GET** `/metrics`

Returns Prometheus-formatted metrics.

## Rate Limiting

- Standard endpoints: 10 requests/second, burst of 20
- Authentication: 1 request/second, burst of 5
- Health/Metrics: No rate limiting

## Error Responses

```json
{
  "error": "Error message"
}
```

Status codes:
- `200` - Success
- `201` - Created
- `400` - Bad Request
- `401` - Unauthorized
- `404` - Not Found
- `429` - Too Many Requests
- `500` - Internal Server Error
