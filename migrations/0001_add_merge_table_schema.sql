-- Migration number: 0001 	 2024-12-27T09:18:59.929Z
DROP TABLE IF EXISTS merge;
CREATE TABLE merge (
    id INTEGER PRIMARY KEY,
    pr_number INTEGER NOT NULL,
    owner TEXT NOT NULL,
    repository TEXT NOT NULL,
    will_merged_at TEXT NOT NULL, -- Stored in UTC
    merged INTEGER NOT NULL DEFAULT 0
);
