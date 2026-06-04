-- Per-user triage state for podcast episodes.
--
-- A missing row means the episode is still in the user's *inbox* (a newly
-- released, not-yet-triaged episode). Once the user acts on it the status
-- captures that decision:
--   * queued    -> picked to listen to (shown in the waiting list)
--   * archived  -> listened / kept for the archive only
--   * dismissed -> not interesting, removed from the inbox
CREATE TABLE episode_triages (
    user_id TEXT NOT NULL,
    episode_id TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('queued', 'archived', 'dismissed')),
    updated_at TIMESTAMP NOT NULL,
    PRIMARY KEY (user_id, episode_id)
);

CREATE INDEX idx_episode_triages_user_status ON episode_triages (user_id, status);
