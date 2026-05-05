# Service Monitoring Workflow

This repository now uses a simple status workflow for service visibility.

## Console message flow

1. Backend process starts and registers `backend`, `api`, and `frontend` in the service monitor.
2. Startup checks update service health:
   - backend: database connectivity and local runtime readiness
   - api: auth configuration and HTTP listener readiness
   - frontend: marked as monitored through the API edge until a runtime is attached
3. Every API request emits:
   - `request-start`
   - auth outcome when applicable
   - route-level operation event such as `push-start`, `pull-finish`, `init-repo-failed`
   - `request-finish` with HTTP status code and duration
4. `/health` exposes:
   - overall system status
   - service-by-service health and last status message
   - recent activity log entries

## Message format

Rust backend and API logs follow this structure:

```text
[LEVEL][service][event] message
```

Examples:

```text
[INFO][backend][status-update] Database connection OK
[INFO][api][request-start] POST /push
[WARN][backend][push-failed] [ERROR] Missing branch 'main'
```

## Frontend logging utility

`frontend/voor/src/common/service-monitor.js` provides a small monitoring helper for future frontend runtime code. It supports:

- `setStatus(health, status, message)`
- `logInfo(event, message, details)`
- `logWarn(event, message, details)`
- `logError(event, message, details)`
- `snapshot()`

The frontend utility uses the same `[LEVEL][service][event] message` console workflow so browser logs align with backend/API logs.
