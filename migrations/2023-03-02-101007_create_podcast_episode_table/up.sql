-- Your SQL goes here
create table if not exists podcast_episodes (
                                                id integer primary key not null,
                                                podcast_id integer not null,
                                                episode_id TEXT not null,
                                                name text not null,
                                                url text not null,
                                                date_of_recording text not null,
                                                image_url text not null,
                                                total_time integer DEFAULT 0 not null,
                                                local_url text DEFAULT '' not null,
                                                local_image_url text DEFAULT '' not null,
                                                description text DEFAULT '' not null,
                                                FOREIGN KEY (podcast_id) REFERENCES podcasts(id));
CREATE INDEX IF NOT EXISTS podcast_episodes_podcast_id_index ON podcast_episodes (podcast_id);