-- Structure only, observed WorkBuddy CN 5.7.7 edge v4, 2026-10-10.
-- No production records; not a write-compatibility allowlist.
-- Observation SHA256: debe65eaa2398142197da2fd7ba5b191a2233249c04c441a54f22c405c669d83

CREATE TABLE edge_sync_artifact_cache (
                    file_path     TEXT NOT NULL,
                    mtime_ms      INTEGER NOT NULL,
                    size          INTEGER NOT NULL,
                    file_hash     TEXT NOT NULL,
                    download_url  TEXT NOT NULL,
                    smh_path      TEXT NOT NULL,
                    content_type  TEXT NOT NULL DEFAULT '',
                    expires_at    INTEGER NOT NULL,
                    uploaded_at   INTEGER NOT NULL DEFAULT (unixepoch()),
                    PRIMARY KEY (file_path)
                );

CREATE TABLE edge_sync_image_mapping (
                    blob_id    TEXT NOT NULL PRIMARY KEY,
                    cos_uri    TEXT NOT NULL,
                    session_id TEXT NOT NULL,
                    created_at INTEGER NOT NULL DEFAULT (unixepoch())
                );

CREATE TABLE edge_sync_mapping (
                    session_id TEXT NOT NULL,
                    conversation_id TEXT NOT NULL,
                    msg_channel TEXT NOT NULL DEFAULT '',
                    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
                    PRIMARY KEY (session_id)
                );

CREATE UNIQUE INDEX idx_edge_sync_mapping_conv
                    ON edge_sync_mapping(conversation_id);
