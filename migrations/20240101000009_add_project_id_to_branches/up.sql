ALTER TABLE branches ADD COLUMN project_id UUID NOT NULL REFERENCES projects(id);
