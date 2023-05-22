-- Your SQL goes here
CREATE table if not exists podcast_history_items (
                                               id integer primary key not null,
                                               podcast_id integer not null,
                                               episode_id TEXT not null,
                                               watched_time integer not null,
                                               date text not null,
                                               FOREIGN KEY (podcast_id) REFERENCES podcasts(id))