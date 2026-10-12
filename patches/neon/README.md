# Neon patches

Patches applied on top of the `neon` submodule before it is built
(`Dockerfile` neon stage, `make neon-patch`). They are applied by
`patches/apply-neon-patches.sh`, which skips patches that are already applied
and fails the build if a patch no longer applies, e.g. after bumping the
submodule.

## 0001-gc-compaction-retry-preempted-subcompaction-jobs.patch

`pageserver/src/tenant/timeline/compaction.rs` (`compact_legacy`,
`compact_with_gc`, `GcCompactionQueue`).

Automatic gc-compaction is scheduled with `YieldForL0`. When L0 layers pile up
while a sub-compaction job runs, the job is preempted ("preempt gc-compaction in
the main loop: too many L0 layers") before it writes anything. Upstream then
loses that outcome: `compact_legacy` discards the `YieldForL0` returned by
`compact_with_gc` and reports `Done`, so the queue counts the job as succeeded
(and even a `YieldForL0` that does reach the queue drops the job for good). The
meta job then advances the timeline's `l2_lsn`
(`gc_compaction.last_completed_lsn`). The skipped key range
is then treated as already compacted: its layers are counted as `l2`, the
`l1/l2` ratio trigger fires less and less often, and the uncompacted history
keeps growing. With child branches that are reset at the parent's tip, legacy GC
cannot remove those layers either, so physical size grows unbounded (seen as
~31 GB of layers for ~2 GB of data).

The patch:

- propagates `YieldForL0` from `compact_with_gc` through `compact_legacy`, so a
  preempted job is reported as preempted instead of done;
- re-queues a preempted sub-compaction job (before the meta job's `Notify`) up to
  3 attempts instead of dropping it;
- counts preemptions in `yielded_sub_compaction_jobs` and give-ups in
  `abandoned_sub_compaction_jobs` (both in the "gc-compaction meta statistics"
  log line), and no longer counts preempted jobs as succeeded;
- does not advance `l2_lsn` if any sub-compaction job failed or was abandoned.

Refreshing after a submodule bump: apply the patch in `neon/` (resolve
conflicts), then regenerate it with
`git -C neon diff -- pageserver/ > patches/neon/0001-gc-compaction-retry-preempted-subcompaction-jobs.patch`
and check that `./patches/apply-neon-patches.sh neon` reports it as applied.
Limit the diff to `pageserver/`, otherwise it also picks up the other patches
applied in the same working tree.

## 0002-safekeeper-remove-wal-without-wal-backup.patch

`safekeeper/src/remove_wal.rs` (`calc_horizon_lsn`),
`safekeeper/src/timeline_manager.rs` (`Manager::update_wal_removal`),
`safekeeper/src/pull_timeline.rs` (`start_snapshot`).

The safekeeper removes WAL segments below
`min(remote_consistent_lsn, backup_lsn, commit_lsn, flush_lsn)`. `backup_lsn` only
advances when the WAL backup task uploads segments to remote storage, and that
task only runs when the safekeeper is started with `--remote-storage`. neond
never gives the safekeeper remote storage (only the pageserver uploads to S3), so
`backup_lsn` stays at the timeline start and WAL is never removed, in both local
and S3 mode. Upstream documents `--disable-wal-backup` as "removes WAL ignoring
WAL backup horizon", but does not implement it. On a production install this
kept every WAL segment since the timeline was created (~142 GB for one busy
timeline, growing ~1.5 GB a day).

The patch:

- ignores `backup_lsn` in `calc_horizon_lsn` when WAL backup has no remote
  storage (no `--remote-storage`, or `--disable-wal-backup`), so WAL is removed
  once the pageserver has made it durable (`remote_consistent_lsn`). With remote
  storage configured the horizon is unchanged;
- makes `start_snapshot` (pull_timeline) stream WAL from `remote_consistent_lsn`
  in the same case, so a snapshot never asks for segments that were removed;
- adds unit tests for `calc_horizon_lsn`.

Consequence: without WAL backup there is no remote copy to fall back to, so a
reader that needs WAL older than `remote_consistent_lsn` (e.g. a logical
replication subscriber that has fallen that far behind) can no longer get it.
Pageserver ingest, branching, reset and point-in-time restore do not read
safekeeper WAL below that LSN.

Giving the safekeeper `--remote-storage` instead would only move the growth to
the bucket: the safekeeper never deletes offloaded WAL, and neond has no
recovery path that reads it.

Refreshing after a submodule bump: apply the patch in `neon/` (resolve
conflicts), then regenerate it with
`git -C neon diff -- safekeeper/ > patches/neon/0002-safekeeper-remove-wal-without-wal-backup.patch`
and check that `./patches/apply-neon-patches.sh neon` reports it as applied.
