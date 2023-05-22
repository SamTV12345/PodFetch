-- This file should undo anything in `up.sql`
DROP INDEX podcast_episodes_podcast_id_index;
DROP TABLE IF EXISTS 'podcast_episodes';