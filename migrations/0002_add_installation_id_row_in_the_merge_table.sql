-- Migration number: 0002 	 2024-12-27T12:39:31.286Z
DROP TABLE IF EXISTS merge;
CREATE TABLE merge (
    id INTEGER PRIMARY KEY,
    pr_number INTEGER NOT NULL,
    owner TEXT NOT NULL,
    repository TEXT NOT NULL,
    will_merged_at TEXT NOT NULL, -- Stored in UTC
    merged INTEGER NOT NULL DEFAULT 0,
    installation_id INTEGER
);
