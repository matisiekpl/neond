# Branching & PITR

Branching is the headline feature. A branch is a writable copy of a
timeline at a specific point. You can branch production into a
preview environment, run destructive migrations safely, or restore
production itself back to a moment a few minutes ago.

This is the same model as `neondatabase/neon`, packaged so you can
run it yourself.

## What a branch is

- A **timeline** of immutable layer files plus its WAL.
- A **parent** (except for the root `production` branch).
- A **branch point** — a moment in the parent's timeline where this
  branch diverged.

After a branch is created, writes on it produce new layers on the
child timeline. The parent is unaffected.

## Creating a branch

From the dashboard:

1. Open the source branch (e.g. `production`).
2. Open the branch actions menu → **Create branch**.
3. Name it.

The new branch:

- Starts from the **current** state of the parent.
- Has its own slug, password, and endpoint port (or SNI hostname).
- Is independent — writes do not flow back to the parent.

There is no copy step. Branching is metadata only and completes in
milliseconds regardless of database size.

## Point-in-time recovery

You can branch from a moment in the past, not just the current
state. From the source branch's actions menu, choose **Restore to a
point in time**:

- Pick a timestamp.
- Confirm.

neond either:

- Creates a **new branch** that starts at that moment (preferred —
  non-destructive), or
- Restores the **existing branch** to that moment (destructive —
  data after the chosen moment on this branch is gone).

The retention window depends on how long ago your layer files were
written and on your S3 lifecycle rules. There is no fixed retention
configured by neond itself.

## Suggested workflows

### Preview environments per PR

1. CI creates a branch off `production` named `pr-1234`.
2. CI starts its endpoint and gets a connection string.
3. The preview app deploys against that connection string.
4. When the PR closes, CI deletes the branch.

Branches are cheap to create and cheap to delete. Doing this per PR
is fine even on a small VPS.

### "Oops, I dropped the table"

1. Open the affected branch.
2. **Restore to a point in time** — pick a moment before the drop.
3. Choose **into a new branch** to be safe.
4. Connect to the recovered branch, copy the data back out, drop the
   branch.

You did not need backups for this. Layer files plus WAL are enough.

### Heavy migration rehearsal

1. Branch `production` into `migration-rehearsal`.
2. Run your destructive migration against the rehearsal branch.
3. Time it. Inspect the result. Throw the branch away.
4. Run it for real against `production` once you're happy.

## Lifecycle

- **Stopping an endpoint** does not affect the branch. Data is
  preserved. Restart later to reconnect.
- **Deleting a branch** removes its endpoint and its child timeline.
  Layer files unique to this branch are eventually garbage-collected.
- **Deleting the parent** is blocked while child branches still
  reference it. Delete the children first.

## Warnings

- A branch is **independent** the moment it's created. Writes on the
  child don't reach the parent and vice versa. You can't "merge a
  branch" — neond is not git.
- Destructive PITR (restoring the same branch in place) is
  irreversible. Prefer recovering into a new branch when in doubt.
- A long-lived branch with heavy writes pins layer files in the
  parent's timeline and grows S3 storage. Delete branches you don't
  need.
- Restoring to a point in time before the branch's creation does
  not exist — you can only PITR within the branch's own timeline,
  which starts at the parent's branch point.
- Don't rely on PITR as your only backup if you're on local-only
  mode. A lost data directory loses everything. Enable S3.
