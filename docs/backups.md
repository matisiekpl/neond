# Backups

neond's branch data is durable through the same S3 mechanism that backs
point-in-time recovery — there is no separate "backup" for branches. What
this page is about is the **internal management database**: the small
Postgres instance that holds your projects, branches, users, and
organisations. It is dumped to S3 on a schedule so you can rebuild a neond
host from nothing.

If you are not running with S3, none of this applies. Your only backup is
the data directory itself.

## When backups run

- On a fixed interval, default **every 30 minutes**. Configurable via the
  `BACKUP_INTERVAL` environment variable.
- On graceful shutdown (`SIGTERM` followed by `stop_grace_period`).

Each run uploads the latest dump to your bucket. Only the most recent copy
is retained — older dumps are overwritten.

## When restore runs

Restore happens **automatically and only** when the local management
database is missing on boot. The flow is:

1. neond boots, acquires its lockfile.
2. It sees no local management database.
3. It downloads the latest dump from S3 and loads it.
4. Boot continues normally.

You don't trigger this by hand. To force a restore, stop neond and delete
the local Postgres data directories that neond owns. On the next boot it
will treat the local copies as missing and pull from S3.

## Forcing a fresh restore

1. Stop the container.
2. Clear the [lockfile](./startup.md#when-the-lockfile-goes-stale) if the
   shutdown was not graceful.
3. Remove the internal Postgres data directories inside your data mount.
4. Start the container.

This is the supported way to "reset to whatever S3 says".

## What's not covered

- **No point-in-time recovery for the management DB.** You get the last
  snapshot, not arbitrary moments. Branch data itself still supports PITR —
  this caveat is only about projects/branches/users metadata.
- **Retention is "latest only".** If you want historical copies of the
  management DB, set up versioning on the S3 bucket. neond does not do this
  for you.
- **Backups are not encrypted by neond.** Use bucket-level encryption
  (SSE-S3 or SSE-KMS) if you need at-rest encryption.

## Warnings

- A backup contains user records and connection passwords. Restrict bucket
  access accordingly.
- Don't point two neond deployments at the same bucket "just for sharing
  backups". They will fight over the lockfile and one will fail to boot.
- A management-DB restore brings back the *list* of branches; the branch
  data itself still has to be reachable in S3 for those branches to start.
  If you wipe layer files but keep the management backup, you'll see ghost
  branches that fail to start.
- The backup interval is a window of acceptable metadata loss. Setting it
  very high to save S3 calls makes restores more lossy.
