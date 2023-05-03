-- This file should undo anything in `up.sql`

create table podcasts2
(
    id                 integer                   not null
        primary key,
    name               text                      not null
        unique,
    directory_id       text                      not null,
    rssfeed            text                      not null,
    image_url          text                      not null,
    summary            TEXT,
    language           TEXT,
    explicit           TEXT,
    keywords           TEXT,
    last_build_date    TEXT,
    author             TEXT,
    active             BOOLEAN      default TRUE not null,
    original_image_url VARCHAR(255) default ''   not null,
    directory_name     VARCHAR(255) default ''   not null
);

INSERT INTO podcasts2 SELECT * FROM podcasts;

DROP TABLE podcasts;
ALTER TABLE podcasts2 RENAME TO podcasts;