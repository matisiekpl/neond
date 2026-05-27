# Storage

neond runs out of a single mounted data directory. Everything it owns lives
there: the Postgres data for each branch, the layer files that back
point-in-time recovery, and the two small internal databases that hold
projects, branches, users, and orgs.

You only ever interact with one directory. To move neond to a new host, stop
the container, copy the directory, and start it on the other side. There is
no separate state to migrate.

## What's in the data directory

After first boot the data directory contains a handful of things worth
knowing about:

- **Layer files** — the immutable building blocks of every branch. These
  grow over time and are the largest thing on disk.
- **Write-ahead log** — recent WAL not yet folded into layer files.
- **Embedded internal databases** — two small Postgres instances neond uses
  to track its own state and coordinate storage. Treat them as internal; you
  never connect to them directly.
- **The lockfile** — see the [Startup](./startup.md) page.

You should not need to touch any of this by hand.

## What S3 changes

If you set the `AWS_*` environment variables, neond switches into S3 mode.
Two things start happening:

1. Layer files (and the checkpointed state of every branch) are continuously
   uploaded to your bucket.
2. The internal management database is dumped to the bucket on a schedule
   and on graceful shutdown.

With S3 on, the data directory becomes a **cache plus working set**, not the
source of truth.

### What S3 protects

- Branch data: every committed transaction up to the last checkpoint.
- Layer files for point-in-time recovery within your retention.
- The internal management database (projects, branches, users, orgs), at the
  granularity of the backup interval.

### What S3 does not protect

- In-memory state at the moment of a crash.
- WAL written after the last checkpoint that has not yet been uploaded —
  losing the local disk between checkpoints loses that window.
- The lockfile. The S3 copy of the lock is for cross-host mutual exclusion,
  not data durability.

## Recovery: wipe local, restore from S3

If the host disk is lost, point a new neond at the same bucket and the same
`SERVER_SECRET`. On boot it will:

1. Acquire the S3 lock.
2. Restore the internal management database from the latest backup.
3. Fetch layer files on demand as branches are accessed.

You will lose the WAL written after the last checkpoint that didn't make it
to S3. Everything else comes back.

## Warnings

- Never delete or modify files inside the data directory while neond is
  running.
- Don't share an S3 bucket between two different neond deployments. Both
  will think they own the data and one will lose its lease.
- Switching a running deployment from local-only to S3 is fine. Switching
  **back** is not — once layers live in S3 the pageserver expects them on
  every subsequent boot.
- "Move neond by copying the directory" only works when neond is stopped
  first. Copying a live data directory produces a corrupt snapshot.
- S3 retention is governed by your bucket lifecycle rules, not by neond. If
  you delete objects from the bucket, the corresponding branch history is
  gone.
