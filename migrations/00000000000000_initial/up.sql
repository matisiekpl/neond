CREATE TABLE users (
    id            UUID      PRIMARY KEY,
    name          VARCHAR   NOT NULL,
    email         VARCHAR   NOT NULL UNIQUE,
    password_hash VARCHAR   NOT NULL,
    created_at    TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE organizations (
    id         UUID      PRIMARY KEY,
    name       VARCHAR   NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE memberships (
    user_id         UUID      NOT NULL REFERENCES users(id),
    organization_id UUID      NOT NULL REFERENCES organizations(id),
    created_at      TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMP NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, organization_id)
);

CREATE TYPE pg_version AS ENUM ('v14', 'v15', 'v16', 'v17');

CREATE TABLE projects (
    id              UUID       NOT NULL PRIMARY KEY,
    organization_id UUID       NOT NULL REFERENCES organizations(id),
    name            VARCHAR    NOT NULL DEFAULT '',
    pg_version      pg_version NOT NULL DEFAULT 'v17',
    created_at      TIMESTAMP  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMP  NOT NULL DEFAULT NOW()
);

CREATE TYPE compute_endpoint_status AS ENUM ('starting', 'running', 'stopping', 'stopped', 'failed');

CREATE TABLE branches (
    id               UUID                    PRIMARY KEY,
    name             VARCHAR                 NOT NULL,
    slug             VARCHAR                 NOT NULL UNIQUE DEFAULT '',
    parent_branch_id UUID                    REFERENCES branches(id) ON DELETE SET NULL,
    timeline_id      UUID                    NOT NULL,
    project_id       UUID                    NOT NULL REFERENCES projects(id),
    password         TEXT                    NOT NULL DEFAULT '',
    recent_status    compute_endpoint_status,
    created_at       TIMESTAMP               NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMP               NOT NULL DEFAULT NOW()
);
