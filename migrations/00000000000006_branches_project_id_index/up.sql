CREATE INDEX IF NOT EXISTS idx_branches_project_id ON branches (project_id);
CREATE INDEX IF NOT EXISTS idx_branches_parent_branch_id ON branches (parent_branch_id);
