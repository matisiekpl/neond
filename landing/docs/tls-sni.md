# TLS SNI routing

You can run neond with a single public port for all endpoints and
have it route incoming connections to the right Postgres process
based on the TLS Server Name Indication header. With this set up,
every branch endpoint gets a clean DNS name like
`my-branch.example.com:5432`.

This is optional. If you don't want it, set `PORT_RANGE` instead and
expose a small range of raw ports — see [Installation](./installation.md).

## What you get

- One public port (`5432`) instead of a port range.
- A predictable hostname per endpoint:
  `<instance-slug>.<your-domain>`.
- TLS certificates managed by neond itself.

## DNS

Point a **wildcard** A (or AAAA) record at your host:

```
*.example.com.   A   <host-ip>
```

That single record covers every endpoint neond will ever hand out.
You don't need to provision DNS per branch.

## Compose configuration

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
    volumes:
      - ./neond_data:/neond
```

After this, connection strings the dashboard hands out look like:

```
postgresql://USER:PASSWORD@my-branch.example.com:5432/postgres?sslmode=require
```

The `sslmode=require` matters: the SNI proxy needs TLS to figure out
which backend to route to. A plain unencrypted connection on this
port will not reach the right Postgres.

## How routing works (briefly)

A client opens a TLS connection to `<slug>.example.com:5432`. The
TLS ClientHello carries the hostname. neond's proxy reads it,
terminates TLS, and forwards the connection to the right internal
Postgres backend. From the client's point of view it's one TLS hop
to one host.

## Certificates

neond manages certificates for you. You do not need to run cert-manager,
Caddy, Traefik, or a separate reverse proxy in front of port 5432.

::: tip
You still want a reverse proxy (Caddy / Traefik / Nginx) in front of
the **HTTP** port for the dashboard. SNI routing applies only to the
Postgres port.
:::

## Warnings

- The hostname `PG_HOSTNAME` is baked into the connection strings
  neond hands out. Changing it later invalidates every saved
  connection string in your users' apps.
- Don't put another TLS-terminating proxy in front of the Postgres
  port. neond needs the original ClientHello to route. Layer-4
  passthrough proxies (haproxy in TCP mode, an L4 load balancer)
  are fine.
- Clients **must** use `sslmode=require` or stronger. Without TLS,
  there is no SNI header and the connection is dropped.
- A wildcard cert is one cert. If you ever need per-branch
  certificates with different CAs or constraints, SNI routing is
  the wrong model — use `PORT_RANGE` and a per-endpoint terminator.
- SNI routing and `PORT_RANGE` can both be on at once, but in
  practice you should pick one. Mixed modes confuse users about
  which connection string to use.
