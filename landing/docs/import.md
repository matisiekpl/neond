# Importing an existing PostgreSQL

You can bring an existing Postgres database into neond as a new
branch. neond connects to the source, dumps it, and streams the dump
into a freshly-created branch. When it's done, that branch behaves
like any other neond branch — you can connect to it, fork it, restore
it to a point in time.

## How to start an import

From the dashboard, open a project and use **Import branch**. You
provide:

- A **name** for the new branch.
- A **connection string** to the source database. Any standard
  `postgresql://user:password@host:5432/dbname` works.

That's it. There's no per-table selection — the whole database is
imported, minus internal neon bookkeeping schemas.

## What you see

The branch shows up immediately with status **importing**. neond:

1. Creates a fresh, empty branch.
2. Starts a compute endpoint behind it.
3. Pipes `pg_dump` from your source into `pg_restore` against the
   new endpoint.
4. Streams logs into the branch's log view while this is happening.

Depending on size and network this can take seconds (small databases)
or hours (large ones). The endpoint stays running for the duration.

When the dump completes successfully, the status flips to **ready**
and the branch becomes a normal branch you can use.

## When something goes wrong

If anything fails — bad credentials, network drop, missing extension
on the target, schema conflict — the branch transitions to **failed**.
The error message is captured on the branch and the import logs are
preserved in the branch's log view.

Imports are **not idempotent and are not resumed**. A failed import
leaves an empty branch behind:

1. Open the failed branch.
2. Read the logs to understand why.
3. Delete the branch.
4. Fix the underlying issue and start a fresh import.

Imports that were running when neond was restarted are marked
**failed** on boot — they are not picked up where they left off.

## Common failure modes

- **Authentication / network.** The source connection string is
  wrong, unreachable, or blocked by a firewall. neond surfaces the
  underlying `pg_dump` error.
- **Missing extensions on the target.** If your source uses an
  extension that isn't installed in neond, the restore fails partway.
  Check the [README](https://github.com/matisiekpl/neond) for the
  bundled extension list (pgvector and Postgres contrib extensions
  are included).
- **Schema conflicts.** If you somehow created objects in the target
  branch before the import finished, restore fails. Don't touch a
  branch that is in `importing` state.
- **Source moved while we were reading it.** Long imports hold a
  read transaction on the source for the whole duration. Heavy DDL
  on the source during the import can fail it.

## Warnings

- Importing from a **busy production database without a snapshot**
  produces a transactionally-consistent copy only for the duration
  of the single dump transaction. If your source workload writes
  heavily across many tables, prefer importing from a replica or a
  paused database.
- Long imports hold an **open transaction on the source** for the
  whole duration. On a heavily-written source this delays autovacuum
  and bloats tables. Monitor the source.
- The new branch is **independent** from the source after import.
  Subsequent writes on the source are not replicated.
- Imports cannot be paused or resumed. If you need to cancel, delete
  the branch — neond will tear down the in-flight import.
