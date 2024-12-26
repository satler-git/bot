CREATE TABLE IF NOT EXISTS merge (
    id INTEGER PRIMARY KEY,
    pr_number INTEGER NOT NULL,
    owner TEXT NOT NULL,
    repository TEXT NOT NULL,
    will_merged_at TEXT NOT NULL -- Stored in UTC
);
