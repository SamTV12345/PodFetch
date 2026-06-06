DROP TABLE sponsorblock_user_settings;
DROP TABLE episode_sponsor_segments;
ALTER TABLE settings DROP COLUMN sponsorblock_enabled;
ALTER TABLE podcast_episodes DROP COLUMN youtube_video_id;
