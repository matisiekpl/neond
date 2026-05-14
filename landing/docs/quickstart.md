# Quickstart

This walks through your first project, branch, and connection. It
assumes you've followed [Installation](./installation.md) and neond
is healthy on `http://<host>:3000`.

## 1. Create the first account

Open the URL in a browser. neond shows a one-time setup screen
because the management database has no users yet:

1. Enter an email and password — this becomes the initial **owner**.
2. Name your **organization**. Organizations group projects and
   users; you can invite more people later.

You land on an empty dashboard.

## 2. Create a project

A project owns one or more **branches**. Each project pins a
PostgreSQL major version.

1. Click **New project**.
2. Give it a name (e.g. `acme-app`).
3. Pick a Postgres version. v14, v15, v16, and v17 are supported.
4. Submit.

neond creates the project and a default `production` branch. Nothing
is running yet — endpoints are started on demand to save resources.

## 3. Start an endpoint

A branch on its own is just a timeline. To connect to it, start a
compute **endpoint**:

1. Open the `production` branch.
2. Click the branch actions menu and choose **Start endpoint**.
3. Wait for the status to flip to **running** (a few seconds).

The endpoint is a real Postgres process behind PgBouncer. See
[PgBouncer](./pgbouncer.md) for what this means for your app.

## 4. Get a connection string

In the branch actions menu, click **Copy connection string**. You
get something shaped like:

```
postgresql://USER:PASSWORD@HOST:PORT/postgres?sslmode=require
```

The host depends on your deployment:

- **No SNI routing**: `host:PORT_RANGE_port` — e.g. `vps.example.com:50001`.
- **SNI routing**: `<instance-slug>.your-domain.com:5432` —
  e.g. `acme-app-production.example.com:5432`.

Connect:

```bash
psql 'postgresql://USER:PASSWORD@HOST:PORT/postgres?sslmode=require'
```

You're talking to a real Postgres. Create tables, load data, do
whatever you need.

```sql
CREATE TABLE notes (id serial PRIMARY KEY, body text);
INSERT INTO notes (body) VALUES ('hello from neond');
SELECT * FROM notes;
```

## 5. Branch it

The point of neond is that you can branch this state instantly.

1. Back in the dashboard, open the branch actions menu on `production`.
2. Click **Create branch**.
3. Name it (e.g. `preview-feature-x`).

The new branch starts from the current state of `production`. Start
its endpoint, copy its connection string, and your app talks to an
isolated copy. Writes on `preview-feature-x` do **not** affect
`production`.

See [Branching & PITR](./branching.md) for the full story including
point-in-time recovery.

## 6. Stop the endpoint when you're done

Idle endpoints still hold a Postgres process and a pool. Stop them
from the branch actions menu when you don't need them. The branch
data is preserved; starting the endpoint again brings everything
back.

## What's next

- [Configuration](./configuration.md) — all the env vars you can set.
- [Branching & PITR](./branching.md) — preview branches, restore to
  a moment.
- [Import](./import.md) — bring an existing Postgres database in.
- [Storage](./storage.md) — what S3 buys you, what it doesn't.
- [Startup](./startup.md) — read this **before** your first crash.

## Warnings

- Connection strings contain a password. Treat them like secrets.
- Stopped endpoints retain their port assignment; restarting reuses
  it. Don't write the port into your app config — re-read the
  connection string from the dashboard or the API.
- The user you create first is an org **owner** with full control.
  Add additional members as editors if you don't want them deleting
  things.
