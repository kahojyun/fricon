CREATE TABLE data_indices (
    id INTEGER NOT NULL PRIMARY KEY,
    name TEXT NULL,
    description TEXT NULL,
    marked BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE tags (
    id INTEGER NOT NULL PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE data_index_tag (
    data_index_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (data_index_id, tag_id),
    FOREIGN KEY (data_index_id) REFERENCES data_indices (id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags (id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX ix_tags_name ON tags (name);
