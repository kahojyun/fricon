CREATE TABLE datasets (
    id INTEGER NOT NULL PRIMARY KEY,
    uuid TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    favorite BOOLEAN NOT NULL DEFAULT FALSE,
    index_columns TEXT NOT NULL, -- JSON: Vec<String>
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE tags (
    id INTEGER NOT NULL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE datasets_tags (
    dataset_id INTEGER NOT NULL REFERENCES datasets (id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
    PRIMARY KEY (dataset_id, tag_id)
);
