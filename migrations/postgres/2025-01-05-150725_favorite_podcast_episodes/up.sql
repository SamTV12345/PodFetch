-- Your SQL goes here
CREATE TABLE favorite_podcast_episodes (
                                           username TEXT NOT NULL,
                                           episode_id INT NOT NULL,
                                           favorite BOOLEAN NOT NULL DEFAULT FALSE,
                                           PRIMARY KEY (username, episode_id),
                                           FOREIGN KEY (episode_id) REFERENCES podcast_episodes(id)
);