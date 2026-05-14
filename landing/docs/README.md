# User Guide

Everything you need to run neond in production. The guide is split
into three parts:

## Getting started

- [Installation](./installation.md) — Docker Compose snippets for
  local-only, TLS SNI, and S3 modes.
- [Quickstart](./quickstart.md) — first account, project, branch,
  and connection in a few minutes.
- [Configuration](./configuration.md) — every environment variable
  neond reads.

## Using neond

- [Branching & PITR](./branching.md) — preview branches and
  point-in-time recovery.
- [Importing PostgreSQL](./import.md) — bring an existing database
  into a new branch.
- [PgBouncer](./pgbouncer.md) — what the pooler in front of every
  endpoint changes for your app.
- [TLS SNI routing](./tls-sni.md) — wildcard hostnames per endpoint.

## Operating neond

- [Startup](./startup.md) — boot order, the lockfile, and what to
  do when it goes stale.
- [Storage](./storage.md) — what's in the data directory, what S3
  protects, the recovery story.
- [Backups](./backups.md) — management-DB dumps and automatic
  restore-on-boot.

Every page ends with a **Warnings** section. Read those — they are
the things that bite people.
