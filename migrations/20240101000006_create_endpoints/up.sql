CREATE TYPE endpoint_state AS ENUM ('stopped', 'running');

CREATE TABLE endpoints (
    branch_id     UUID           PRIMARY KEY REFERENCES branches(id) ON DELETE CASCADE,
    state         endpoint_state NOT NULL,
    endpoint_port INTEGER        NOT NULL
);
