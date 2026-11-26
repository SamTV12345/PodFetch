-- Event store: append-only log of domain events.
CREATE TABLE events (
    seq            INTEGER PRIMARY KEY AUTOINCREMENT,
    id             TEXT NOT NULL UNIQUE,
    event_type     TEXT NOT NULL,
    event_version  INTEGER NOT NULL,
    aggregate_id   TEXT NOT NULL,
    payload        TEXT NOT NULL CHECK (json_valid(payload)),
    occurred_at    TIMESTAMP NOT NULL,
    correlation_id TEXT
);
CREATE INDEX idx_events_type ON events (event_type);
CREATE INDEX idx_events_aggregate ON events (aggregate_id, seq);

-- One delivery per (event, handler); each handler retries on its own.
CREATE TABLE event_deliveries (
    event_seq       INTEGER NOT NULL REFERENCES events(seq) ON DELETE CASCADE,
    handler         TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'pending', -- pending|processing|done|failed|dead
    attempts        INTEGER NOT NULL DEFAULT 0,
    next_attempt_at TIMESTAMP NOT NULL,
    locked_until    TIMESTAMP,
    last_error      TEXT,
    updated_at      TIMESTAMP NOT NULL,
    PRIMARY KEY (event_seq, handler)
);
CREATE INDEX idx_event_deliveries_due ON event_deliveries (status, next_attempt_at);
