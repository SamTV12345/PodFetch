-- This file should undo anything in `up.sql`
ALTER TABLE podcast_episodes DROP file_episode_path;
ALTER TABLE podcast_episodes DROP file_image_path;