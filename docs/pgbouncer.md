# PgBouncer

Every running endpoint in neond sits behind PgBouncer. You don't configure
it; you don't run it separately. When you copy a connection string from the
dashboard, it points at the pooled port, not the raw Postgres process behind
it.

This is mostly invisible — but it has a few sharp edges if you come from a
world where you talk straight to Postgres.

## What you get for free

- A pool per endpoint, with sensible default sizing for small VPS
  deployments.
- Faster connection setup for short-lived clients (web request handlers,
  serverless functions) — you're handed an already-warm backend.
- Survival of brief connection spikes that would otherwise overload a small
  Postgres.

## What changes vs. talking to Postgres directly

PgBouncer runs in **transaction pooling** mode. That means a single client
connection is mapped to a backend only for the duration of a transaction,
and the same backend can serve a different client on the next transaction.

A few Postgres features assume a stable, dedicated backend connection and
don't survive that:

- **`LISTEN` / `NOTIFY`.** Notifications are delivered to whichever backend
  is currently bound — usually not the one you issued `LISTEN` on. Don't
  rely on it through the pooled port.
- **Session-level `SET`.** A `SET` outside an explicit transaction is
  forgotten as soon as the backend is returned to the pool. Use `SET LOCAL`
  inside a transaction, or `SET` parameters per query.
- **Prepared statements.** Some drivers cache server-side prepared
  statements per connection. Behind a transaction pooler the cache is wrong
  as soon as a different backend is handed out. Either disable server-side
  prepares in your driver or use a driver that re-prepares automatically.
- **Advisory locks at session scope.** `pg_advisory_lock` survives only as
  long as the backend is bound. Prefer transaction-scoped advisory locks
  (`pg_advisory_xact_lock`).
- **`SET ROLE` / `SET SESSION AUTHORIZATION`.** Same story — scoped to the
  backend, lost on release.

If you need any of the above, you need a direct connection to Postgres, not
the pooled one.

## Warnings

- The connection string the dashboard gives you is the **pooled** one. If
  your app needs `LISTEN/NOTIFY` or session-level state, build it knowing
  you're behind a transaction pooler.
- Don't try to "fix" the pooler by issuing your own `DISCARD ALL` —
  PgBouncer resets the backend between transactions anyway.
- Pool sizing is per endpoint, not per project. Many idle endpoints cost
  more than a few busy ones.
- If you see "server login has been failing" errors in your app, the backend
  Postgres is unreachable — usually because the endpoint was stopped from
  the dashboard. Start the endpoint again.

For PgBouncer's own documented constraints, see the upstream
[PgBouncer documentation](https://www.pgbouncer.org/features.html).
