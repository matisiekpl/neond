CREATE TYPE pg_version AS ENUM ('v14', 'v15', 'v16', 'v17');
ALTER TABLE branches ADD COLUMN password TEXT NOT NULL DEFAULT '';
ALTER TABLE branches ADD COLUMN pg_version pg_version NOT NULL DEFAULT 'v17';
