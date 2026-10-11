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

## Compaction and disk usage

Layer files are never edited in place. Old page versions are dropped by two
background processes in the pageserver:

- **Garbage collection** removes whole layer files that are older than the
  PITR window. It cannot remove anything a child branch might still read, so
  any layer that starts before a child's branch point is kept. A branch that
  lives for a long time — or a `dev` branch that is regularly **reset to its
  parent** — pins the parent's history this way.
- **gc-compaction** rewrites the history of the root branch and keeps only
  the page versions that PITR and child branches actually need. This is what
  reclaims the space pinned by branches. It starts automatically once enough
  new layers have accumulated.

If a project's physical size is much larger than its logical size, run
**Compact storage** from the branch menu of the root branch. It schedules
the same gc-compaction in the background without yielding to other
compaction work, so it always runs to completion. The project page shows
"Compacting storage…" until it finishes and then the new physical size.
Expect:

- minutes for small projects, up to hours for tens of gigabytes of layers;
- slower reads and writes while it runs (L0 compaction of that project waits
  until it finishes), so prefer a quiet period;
- extra free disk space for the rewritten layers. In S3 mode, layers that
  are not on the local disk are downloaded first.

neond ships a patch for the pageserver (`patches/neon/`) that fixes automatic
gc-compaction silently skipping parts of the history: upstream it abandons
work preempted by L0 compaction while still marking the history as compacted,
which makes disk usage grow over time on busy projects with branches.

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
