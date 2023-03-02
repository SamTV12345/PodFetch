-- Your SQL goes here
create table if not exists queue (
                                     id integer primary key not null,
                                     podcast_id integer not null,
                                     download_url text not null,
                                     episode_id TEXT not null,
                                     status integer not null,
                                     FOREIGN KEY (podcast_id) REFERENCES podcasts(id),
                                     FOREIGN KEY (episode_id) REFERENCES podcast_episodes(episode_id))