CREATE TABLE compute_metric_samples (
    id          UUID             PRIMARY KEY,
    branch_id   UUID             NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    recorded_at TIMESTAMP        NOT NULL DEFAULT NOW(),
    slug        TEXT             NOT NULL,
    value       DOUBLE PRECISION NOT NULL
);

CREATE INDEX compute_metric_samples_branch_time_idx
    ON compute_metric_samples (branch_id, recorded_at DESC);

CREATE INDEX compute_metric_samples_branch_slug_time_idx
    ON compute_metric_samples (branch_id, slug, recorded_at DESC);
