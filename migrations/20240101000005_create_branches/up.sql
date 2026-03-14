CREATE TABLE branches (
    id               UUID    PRIMARY KEY,
    name             VARCHAR NOT NULL,
    parent_branch_id UUID    REFERENCES branches(id) ON DELETE SET NULL
);
