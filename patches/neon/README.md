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
`git -C neon diff > patches/neon/0001-gc-compaction-retry-preempted-subcompaction-jobs.patch`
and check that `./patches/apply-neon-patches.sh neon` reports it as applied.
