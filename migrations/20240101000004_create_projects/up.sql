CREATE TABLE projects (
    id              UUID NOT NULL PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE
);
