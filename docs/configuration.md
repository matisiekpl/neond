# Configuration

All configuration is via environment variables. There is no config file —
everything is set on the container.

## Required

| Variable | What it does |
|---|---|
| `SERVER_SECRET` | The master secret used to derive tokens and endpoint passwords. **Permanent** — set once on first boot and never change. |

## Networking

| Variable | Default | What it does |
|---|---|---|
| `PORT` | `3000` | HTTP port for the dashboard and API. |
| `PORT_RANGE` | _(unset)_ | Inclusive port range for compute endpoints, e.g. `50000-50010`. Use this when you don't want TLS SNI routing. |
| `PG_PROXY_PORT` | _(unset)_ | Port for the TLS SNI proxy. Typically `5432`. Required when using `PG_HOSTNAME`. |
| `PG_HOSTNAME` | _(unset)_ | Base domain for SNI-routed endpoints. With `PG_HOSTNAME=example.com`, endpoints are reachable as `<slug>.example.com:<PG_PROXY_PORT>`. |

You pick **one** addressing model: either `PORT_RANGE` for raw per-endpoint
ports, or `PG_HOSTNAME` + `PG_PROXY_PORT` for SNI routing. You can run both,
but in practice you don't need to.

## S3 durability

All four must be set together. If any one is missing, neond stays in
local-only mode.

| Variable | What it does |
|---|---|
| `AWS_S3_BUCKET` | Bucket to use for layer uploads and management-DB backups. |
| `AWS_REGION` | Bucket region, e.g. `eu-central-1`. |
| `AWS_ACCESS_KEY_ID` | IAM access key. Needs `s3:GetObject`, `s3:PutObject`, `s3:DeleteObject` on the bucket, and `s3:ListBucket`. |
| `AWS_SECRET_ACCESS_KEY` | Corresponding secret key. |

## Backups

| Variable | Default | What it does |
|---|---|---|
| `BACKUP_INTERVAL` | `30m` | How often the management DB is dumped to S3. Accepts Go duration strings (`10m`, `1h`, `6h`). |

The backup interval is also a window of acceptable metadata loss on S3
restore. See [Backups](./backups.md).

## Compose example with all the knobs

```yaml
services:
  neond:
    image: neond/neond:latest
    environment:
      PORT: 3000
      SERVER_SECRET: "change-me-once-and-never-again"

      # SNI routing
      PG_PROXY_PORT: "5432"
      PG_HOSTNAME: example.com

      # S3 durability
      AWS_ACCESS_KEY_ID: "..."
      AWS_SECRET_ACCESS_KEY: "..."
      AWS_S3_BUCKET: "neond-prod"
      AWS_REGION: "eu-central-1"

      # Backup cadence
      BACKUP_INTERVAL: "15m"
    ports:
      - "3000:3000"
      - "5432:5432"
    restart: unless-stopped
    stop_grace_period: 1h
    healthcheck:
      test: ["CMD", "curl", "-fsS", "http://127.0.0.1:3000/api/auth/setup"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 5m
    volumes:
      - ./neond_data:/neond
```

## Health and lifecycle

- **Healthcheck endpoint**: `GET /api/auth/setup` returns 200 once the API is
  ready. Use it in your compose / Kubernetes config.
- **Graceful shutdown**: `SIGTERM` triggers a final checkpoint, a last backup
  to S3 (if configured), and lockfile release. Give this at least
  `stop_grace_period: 1h` on a real deployment.
- **First boot**: takes longer than subsequent boots — embedded Postgres
  instances need to be initialised. The `5m` `start_period` in the
  healthcheck covers this.

## Warnings

- `SERVER_SECRET` cannot be rotated. Generate it from a CSPRNG once and store
  it in your secret manager.
- Don't enable S3 by setting only some of the `AWS_*` variables. neond
  requires all four or it stays local.
- Setting `BACKUP_INTERVAL` very high to save S3 calls makes disaster
  recovery lossier. 15–30 minutes is a reasonable balance.
- `PG_HOSTNAME` is baked into endpoint URLs neond hands out. Don't change it
  after users have started copying connection strings into their apps.
- Don't expose the embedded internal databases on the host. The only port you
  expose to the outside world is `PORT` (HTTP) and either `PG_PROXY_PORT`
  (SNI) or `PORT_RANGE` (direct).
