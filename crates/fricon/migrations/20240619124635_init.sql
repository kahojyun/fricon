CREATE TABLE datasets (
    id INTEGER NOT NULL PRIMARY KEY,
    uid TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    favorite BOOLEAN NOT NULL,
    index_columns TEXT NOT NULL, -- JSON: Vec<String>
    path TEXT NOT NULL, -- Relative to the data directory
    created_at TEXT NOT NULL -- chrono::DateTime<Utc>
);

CREATE TABLE tags (
    id INTEGER NOT NULL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE dataset_tag (
    dataset_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (dataset_id, tag_id),
    FOREIGN KEY (dataset_id) REFERENCES datasets (id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags (id) ON DELETE CASCADE
);
