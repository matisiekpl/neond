ALTER TABLE branches DROP COLUMN created_at;
ALTER TABLE branches DROP COLUMN updated_at;

ALTER TABLE organizations DROP COLUMN created_at;
ALTER TABLE organizations DROP COLUMN updated_at;

ALTER TABLE projects DROP COLUMN created_at;
ALTER TABLE projects DROP COLUMN updated_at;

ALTER TABLE memberships DROP COLUMN created_at;
ALTER TABLE memberships DROP COLUMN updated_at;

CREATE TYPE endpoint_state AS ENUM ('stopped', 'running');

CREATE TABLE endpoints (
    branch_id UUID NOT NULL PRIMARY KEY REFERENCES branches(id),
    state endpoint_state NOT NULL,
    endpoint_port INTEGER NOT NULL
);
