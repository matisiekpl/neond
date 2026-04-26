UPDATE compute_metric_samples SET branch_id = '00000000-0000-0000-0000-000000000000' WHERE branch_id IS NULL;
ALTER TABLE compute_metric_samples ALTER COLUMN branch_id SET NOT NULL;