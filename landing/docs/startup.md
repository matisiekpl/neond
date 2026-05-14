# Startup

When you start the neond container, it boots in a fixed order. Most of
the time you don't need to think about it — but when something goes
wrong on launch, knowing the order tells you where to look.

## What happens

1. Configuration is read from environment variables. `SERVER_SECRET`
   is required and is **permanent** — the first value used is baked
   into the data directory. Changing it later invalidates every token
   and password.
2. A preflight check verifies internal ports are free and that there
   is at least 3 GB of free disk. If S3 is configured, it verifies
   the bucket is reachable. Any failure here is loud and boot stops
   before anything is started.
3. A **lockfile** is acquired. See below.
4. The embedded Postgres servers used internally by neond are
   initialised on first boot. On subsequent boots they are reused,
   unless S3 backups exist and the local copies are missing — in that
   case they are automatically restored from S3 before continuing.
5. Pending schema migrations apply.
6. Any branch imports that were interrupted by a previous shutdown
   are marked as **failed**. They are not resumed.
7. The HTTP API binds to `PORT` (default 3000). The container is
   considered healthy once `GET /api/auth/setup` answers 200.

A healthy boot takes roughly 10–30 seconds on a warm data directory,
and up to a couple of minutes on first boot or after an S3 restore.
The compose example in the README uses `start_period: 5m` for exactly
this reason.

## The lockfile

neond is a **single-writer system**. Only one neond process is allowed
to use a given data directory at a time. This is enforced with a
lockfile:

- A lockfile is created inside the data directory at boot.
- If S3 is configured, a matching lock object is also created in the
  bucket under `control-plane/.lock`.
- Both locks are removed during a clean shutdown.

If you start a second container against the same data directory or
the same S3 bucket, the second one refuses to boot with a clear
`lease already held` error. This is intentional — running two neond
instances against the same data corrupts it.

### When the lockfile goes stale

If neond is killed hard (host crash, OOM kill, `docker kill`,
`kill -9`, power loss) the lockfile is **not** cleaned up. On the next
start you will see a lease error even though no neond is running.

There is no automatic recovery. You have to clear it manually.

::: warning
Only do this when you are certain no other neond instance is running
against this data directory or S3 bucket. Removing the lockfile while
another instance is alive will corrupt the data.
:::

To recover:

1. Confirm no neond process is running. Check `docker ps` on every
   host that might be pointed at this data directory or bucket.
2. Delete the lockfile inside the data directory.
3. If S3 is configured, also delete the `control-plane/.lock`
   object from the bucket.
4. Start neond again.

## Warnings

- `SERVER_SECRET` cannot be rotated. Choose it once.
- A preflight failure means nothing has started — the data directory
  is untouched and safe to retry against.
- Healthcheck failures during the first few minutes after launch are
  normal; the container takes a moment to bring everything up.
- Imports interrupted by a restart are not resumed. Delete the failed
  branch and re-import.
- Running two neond containers against the same data directory is the
  fastest way to corrupt a deployment. The lockfile is there to stop
  you — don't bypass it casually.
