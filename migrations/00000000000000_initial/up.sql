CREATE TABLE users (
    id            UUID        PRIMARY KEY,
    name          VARCHAR     NOT NULL,
    email         VARCHAR     NOT NULL UNIQUE,
    password_hash VARCHAR     NOT NULL
);

CREATE TABLE organizations (
    id   UUID    PRIMARY KEY,
    name VARCHAR NOT NULL
);

CREATE TABLE memberships (
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, organization_id)
);

CREATE TYPE pg_version AS ENUM ('v14', 'v15', 'v16', 'v17');

CREATE TABLE projects (
    id              UUID       NOT NULL PRIMARY KEY,
    organization_id UUID       NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name            VARCHAR    NOT NULL DEFAULT '',
    pg_version      pg_version NOT NULL DEFAULT 'v17'
);

CREATE TABLE branches (
    id               UUID    PRIMARY KEY,
    name             VARCHAR NOT NULL,
    parent_branch_id UUID    REFERENCES branches(id) ON DELETE SET NULL,
    timeline_id      UUID    NOT NULL,
    project_id       UUID    NOT NULL REFERENCES projects(id),
    password         TEXT    NOT NULL DEFAULT ''
);

CREATE TYPE endpoint_state AS ENUM ('stopped', 'running');

CREATE TABLE endpoints (
    branch_id     UUID           PRIMARY KEY REFERENCES branches(id) ON DELETE CASCADE,
    state         endpoint_state NOT NULL,
    endpoint_port INTEGER        NOT NULL
);
