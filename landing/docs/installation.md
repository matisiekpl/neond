# Installation

neond ships as a single Docker image. The recommended deployment is
Docker Compose on a bare-metal VPS. There is no Helm chart, no
operator, no separate database to provision.

You need:

- A Linux host with Docker and Docker Compose.
- ~3 GB of free disk for the data directory (more if you intend to
  hold real data).
- Open ports on the host: at minimum the management UI port, plus a
  range for compute endpoints **or** a single port if you use TLS
  SNI routing.

## 1. Prepare the data directory

```bash
mkdir neond_data
sudo chown -R 600:600 neond_data   # skip on macOS
```

This directory will hold everything neond owns. Back it up like you
would back up any other database directory — or use S3 mode (below)
to make it disposable.

## 2. Local-only deployment

A minimal `docker-compose.yaml` that exposes the UI on port 3000 and
a small range of ports for compute endpoints:

```yaml
services:
  neond:
    image: neond/neond:latest
    environment:
      PORT: 3000
      SERVER_SECRET: "change-me-once-and-never-again"
      PORT_RANGE: 50000-50010
    ports:
      - "3000:3000"
      - "50000-50010:50000-50010"
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

Bring it up:

```bash
docker compose up -d
docker compose logs -f neond
```

When the healthcheck flips to healthy, open `http://<host>:3000` and
create the first user.

::: tip
`stop_grace_period: 1h` is intentional. On shutdown neond flushes the
last checkpoint, uploads any pending state, and releases its lockfile.
Don't shorten this on a real deployment.
:::

## 3. TLS SNI routing (optional)

If you want every endpoint to live under a wildcard subdomain — e.g.
`my-branch.example.com` instead of `host:50001` — point a wildcard DNS
record at the host and set `PG_HOSTNAME`:

```yaml
services:
  neond:
    image: neond/neond:latest
    environment:
      PORT: 3000
      SERVER_SECRET: "change-me-once-and-never-again"
      PG_PROXY_PORT: "5432"
      PG_HOSTNAME: example.com
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

DNS:

```
*.example.com.   A   <host-ip>
```

Now every endpoint neond starts is reachable as
`<instance-slug>.example.com:5432`. TLS certificates are managed by
neond itself; you don't run a separate reverse proxy.

See the [TLS SNI routing](./tls-sni.md) page for the details.

## 4. S3 durability (optional, recommended for production)

Pass four extra environment variables and neond starts streaming
layer files and management-DB backups to your bucket:

```yaml
services:
  neond:
    image: neond/neond:latest
    environment:
      PORT: 3000
      SERVER_SECRET: "change-me-once-and-never-again"
      PG_PROXY_PORT: "5432"
      PG_HOSTNAME: example.com
      AWS_ACCESS_KEY_ID: "..."
      AWS_SECRET_ACCESS_KEY: "..."
      AWS_S3_BUCKET: "neond-prod"
      AWS_REGION: "eu-central-1"
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

All four S3 variables must be set together. Once on, the data
directory becomes a cache: you can wipe the host and the next neond
boot pulls everything back from S3. See [Storage](./storage.md) and
[Backups](./backups.md) for what is and isn't covered.

::: warning
S3 mode is sticky. Once layers live in your bucket, the next boot
expects them there. Don't unset the `AWS_*` variables on a deployment
that has been running with them.
:::

## 5. Updating

```bash
docker compose pull
docker compose up -d
```

Schema migrations apply automatically on boot.

## Warnings

- `SERVER_SECRET` is **permanent**. The first value used is baked
  into the data directory and into every password derived from it.
  Changing it later invalidates everything.
- Don't run two neond containers against the same `neond_data`
  directory or the same S3 bucket. The lockfile prevents this, but
  bypassing it corrupts data — see
  [Startup](./startup.md#the-lockfile).
- The data directory is owned by uid `600:600` inside the container.
  On Linux hosts, `chown` it to match or Docker won't be able to
  write.
- The compose `start_period: 5m` exists because first boot (and
  boot-after-S3-restore) can take a couple of minutes. Don't shorten
  it for marginal startup-time savings.
