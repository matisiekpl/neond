# Exporting a branch

You can download the database of any branch as a standard `pg_dump` file and
restore it on any PostgreSQL server: another neond, a managed Postgres, or a
local one. neond runs `pg_dump` against the branch's compute on the server,
writes the dump to disk, and lets you download it when it's done.

## How to start an export

From the dashboard, open a project, open the branch's menu (⋮) and choose
**Export (pg_dump)**. You pick:

- The **format**:
  - **Custom (`.dump`)**: a compressed `pg_dump` archive. Restore it with
    `pg_restore`, which can also restore single tables or schemas, or run in
    parallel with `-j`. Usually the better choice.
  - **Plain SQL (`.sql`)**: a readable SQL script. Restore it with `psql`.
- The **database**. Defaults to `postgres`. When the branch's compute is
  running, the dialog lists every database of the branch.
- **Skip ownership** (`--no-owner`) and **skip privileges**
  (`--no-privileges`). Turn them on when you restore into a server where the
  roles of this branch don't exist.

If the branch's compute is stopped, the export starts it.

## What you see

The dialog shows a progress bar for the running export with:

- the phase: starting the compute, dumping the schema, the table being copied,
  then writing the file;
- the number of tables dumped so far;
- the bytes written.

Progress is estimated from the on-disk size of each table. A plain dump is
often larger than that estimate and a compressed one much smaller, so the
percentage measures how far `pg_dump` has got, not the size of the file.

You can close the dialog: the export keeps running on the server. The branch
row shows an **Exporting N%** badge, and a notification with a **Download**
button appears when it finishes. `pg_dump`'s own output is in the branch's
**export** log view.

When the export is done, **Download** saves the file through the browser's
normal download. You can also delete the file, or start a new export, which
replaces it.

## Restoring

```sh
# custom format
pg_restore --no-owner -d "postgresql://user:password@host:5432/dbname" export.dump

# plain SQL
psql "postgresql://user:password@host:5432/dbname" -f export.sql
```

## Postgres versions

The dump is made with the `pg_dump` of the **project's own Postgres version**:
a Postgres 15 project is dumped with `pg_dump` 15. Restore it on a server of
the same major version or a newer one, using `pg_restore`/`psql` of that
version or newer:

- an archive from a newer `pg_dump` cannot be read by an older `pg_restore`
  ("unsupported version in file header");
- a newer `pg_dump` writes SQL that older servers reject, e.g.
  `SET transaction_timeout` from `pg_dump` 17.

The dialog and the finished export show the `pg_dump` version used.

## What is exported

Everything `pg_dump` exports from the chosen database, except Neon's internal
objects, which don't exist on a vanilla Postgres:

- the `neon` and `neon_migration` schemas;
- the `neon` extension;
- the compute's `public.health_check` table. Like any `pg_dump`, an export
contains a single database, and no roles or other cluster-wide objects.

The dump is transactionally consistent: `pg_dump` reads everything in one
snapshot. Writes to the branch while the export runs are not included.

## Limits and housekeeping

- **One export per branch at a time**, and at most **two running on the whole
  server**. Starting another returns an error until one of them finishes.
- The file is stored in `neon_daemon_data/exports/` on the server, with access
  restricted to the neond user. Before starting, neond checks that the disk has
  room for the estimated dump plus 1 GiB. A running export is aborted if free
  space drops below 512 MiB.
- A finished export is **kept for one hour**, then deleted. Download it within
  that time or start a new one.
- Exports are tracked in memory. A running export is aborted when neond stops,
  and every export file is deleted when neond starts again.
- Deleting the branch cancels its export and deletes the file.
- Resetting the branch, restoring it from PITR, or changing its password
  restarts its compute. A running export then fails; start it again.

## Security

- Only members of the branch's organization can start, watch, cancel, or
  download exports.
- The dump runs on the server against the branch's compute. The branch's
  password is not used, and nothing you type in the dialog is passed to
  `pg_dump` as a command-line option.
- Downloads use a **single-use token valid for one minute**. The dashboard
  requests the token right before the download. The token is bound to one
  export of one branch, and your access to the branch is checked again when it
  is redeemed. Your login token never appears in a download URL, and a
  download token cannot be used to call the rest of the API.

## API

| Method   | Path                                                          | What it does                                                     |
|----------|---------------------------------------------------------------|------------------------------------------------------------------|
| `POST`   | `/api/organizations/{org}/projects/{project}/branches/{branch}/export` | Starts an export. Body: `{"format": "custom" \| "plain", "database": "postgres", "no_owner": false, "no_privileges": false}`. Returns `202` and the status. |
| `GET`    | `…/export`                                                    | Status: `state` (`idle`, `preparing`, `dumping`, `finalizing`, `completed`, `failed`, `cancelled`), `progress` (0–1), `current_table`, `tables_done`/`tables_total`, `bytes_written`, `pg_dump_version`, `error`, `expires_at`. |
| `DELETE` | `…/export`                                                    | Cancels the running export, or deletes the finished file.       |
| `POST`   | `…/export/download-token`                                     | Returns `{"token", "expires_at"}` for a finished export.         |
| `GET`    | `…/export/download?token=…`                                   | Downloads the file. Needs only the download token.              |

A script can download an export with:

```sh
TOKEN=$(curl -s -X POST -H "Authorization: Bearer $ACCESS_TOKEN" \
  "$NEOND/api/organizations/$ORG/projects/$PROJECT/branches/$BRANCH/export/download-token" | jq -r .token)
curl -fOJ "$NEOND/api/organizations/$ORG/projects/$PROJECT/branches/$BRANCH/export/download?token=$TOKEN"
```
