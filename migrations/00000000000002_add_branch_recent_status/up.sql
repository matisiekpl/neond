CREATE TYPE compute_endpoint_status AS ENUM ('starting', 'running', 'stopping', 'stopped', 'failed');
ALTER TABLE branches ADD COLUMN recent_status compute_endpoint_status;
