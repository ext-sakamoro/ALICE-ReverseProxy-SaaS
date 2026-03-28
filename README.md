# ALICE ReverseProxy SaaS

L7 reverse proxy management powered by ALICE. Manage routes, upstreams, circuit breakers, and health checks via a simple REST API.

[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

## Status

| Check | Status |
|-------|--------|
| `cargo check` | passing |
| API health | `/health` |

## Quick Start

```bash
docker compose up -d
```

API Gateway: http://localhost:8130

## Architecture

```
Client
  |
  v
API Gateway          :8130
  |
  v
ReverseProxy Engine  :8131-internal
(route management, upstream pool, circuit breaker)
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v1/proxy/routes` | Register a new proxy route |
| `POST` | `/api/v1/proxy/upstream` | Configure upstream pool |
| `POST` | `/api/v1/proxy/circuit-breaker` | Configure circuit breaker |
| `POST` | `/api/v1/proxy/health-check` | Run upstream health check |
| `GET`  | `/api/v1/proxy/stats` | Retrieve proxy statistics |
| `GET`  | `/health` | Service health check |

### routes

```json
POST /api/v1/proxy/routes
{
  "host": "api.example.com",
  "path_prefix": "/v1",
  "upstream": "backend-pool",
  "strip_prefix": true
}
```

### upstream

```json
POST /api/v1/proxy/upstream
{
  "name": "backend-pool",
  "targets": ["http://10.0.0.1:8080", "http://10.0.0.2:8080"],
  "strategy": "round-robin"
}
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PROXY_ADDR` | `0.0.0.0:8130` | Core engine bind address |
| `GATEWAY_ADDR` | `0.0.0.0:8129` | API gateway bind address |

## License

AGPL-3.0. Commercial dual-license available — contact for pricing.
