ALTER TABLE branches ADD COLUMN shared_buffers_mb INTEGER NOT NULL DEFAULT 128;
ALTER TABLE branches ADD COLUMN effective_cache_size_mb INTEGER;
ALTER TABLE branches ADD COLUMN maintenance_work_mem_mb INTEGER;
ALTER TABLE branches ADD COLUMN work_mem_mb INTEGER;
ALTER TABLE branches ADD COLUMN max_connections INTEGER;
