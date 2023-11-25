create table episodes
(
    id        SERIAL      not null
        primary key ,
    username  VARCHAR(255) not null,
    device    VARCHAR(255) not null,
    podcast   VARCHAR(255) not null,
    episode   VARCHAR(255) not null,
    timestamp TIMESTAMP     not null,
    guid      VARCHAR(255),
    action    VARCHAR(255) not null,
    started   INTEGER,
    position  INTEGER,
    total     INTEGER,
    unique (username, device, podcast, episode, timestamp)
);

create table filters
(
    username     TEXT                  not null
        primary key,
    title        TEXT,
    ascending    BOOLEAN default FALSE not null,
    filter       TEXT,
    only_favored BOOLEAN default TRUE  not null,
    check (filter IN ('PublishedDate', 'Title'))
);

create table invites
(
    id               VARCHAR(255)                       not null  primary key,
    role             TEXT                               not null,
    created_at       TIMESTAMP default CURRENT_TIMESTAMP not null,
    accepted_at      TIMESTAMP,
    explicit_consent BOOLEAN  default FALSE                 not null,
    expires_at       TIMESTAMP                           not null,
    check (role IN ('admin', 'uploader', 'user'))
);

create table notifications
(
    id              SERIAL not null
        primary key,
    type_of_message TEXT    not null,
    message         TEXT    not null,
    created_at      TEXT    not null,
    status          TEXT    not null
);

create table podcasts
(
    id                 SERIAL                   not null
        primary key,
    name               text                      not null,
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

create table favorites
(
    username   TEXT              not null,
    podcast_id INTEGER           not null
        references podcasts
            on delete cascade,
    favored    BOOLEAN default FALSE not null,
    primary key (username, podcast_id)
);

create table podcast_episodes
(
    id                SERIAL             not null
        primary key,
    podcast_id        integer             not null
        references podcasts,
    episode_id        TEXT                not null,
    name              text                not null,
    url               text                not null,
    date_of_recording text                not null,
    image_url         text                not null,
    total_time        integer default 0   not null,
    local_url         text    default ''  not null,
    local_image_url   text    default ''  not null,
    description       text    default ''  not null,
    status            CHAR(1) default 'N' not null,
    download_time     TIMESTAMP
);

create index podcast_episode_url_index
    on podcast_episodes (url);

create index podcast_episodes_podcast_id_index
    on podcast_episodes (podcast_id);

create table sessions
(
    username   VARCHAR(255) not null,
    session_id VARCHAR(255) not null,
    expires    TIMESTAMP     not null,
    primary key (username, session_id)
);

create table settings
(
    id                         SERIAL                                            not null
        primary key,
    auto_download              BOOLEAN default TRUE                               not null,
    auto_update                BOOLEAN default TRUE                               not null,
    auto_cleanup               BOOLEAN default FALSE                              not null,
    auto_cleanup_days          INTEGER default -1                                 not null,
    podcast_prefill            INTEGER default 5                                  not null,
    replace_invalid_characters BOOLEAN default TRUE                               not null,
    use_existing_filename      BOOLEAN default FALSE                              not null,
    replacement_strategy       TEXT    default 'replace-with-dash-and-underscore' not null,
    episode_format             TEXT    default '{}'                               not null,
    podcast_format             TEXT    default '{}'                               not null,
    check (replacement_strategy IN
           ('replace-with-dash-and-underscore', 'remove', 'replace-with-dash'))
);


create table subscriptions
(
    id      SERIAL  not null
        primary key,
    username TEXT     not null,
    device   TEXT     not null,
    podcast  TEXT     not null,
    created  TIMESTAMP not null,
    deleted  TIMESTAMP,
    unique (username, device, podcast)
);

create table users
(
    id      SERIAL                             not null
        primary key,
    username         VARCHAR(255)                       not null
        unique,
    role             TEXT                               not null,
    password         VARCHAR(255),
    explicit_consent BOOLEAN  default FALSE                 not null,
    created_at       TIMESTAMP default CURRENT_TIMESTAMP not null,
    check (role IN ('admin', 'uploader', 'user'))
);

create table devices
(
    id       SERIAL not null
        primary key,
    deviceid VARCHAR(255) not null,
    kind     TEXT         not null,
    name     VARCHAR(255) not null,
    username VARCHAR(255) not null
        references users (username),
    check (kind IN ('desktop', 'laptop', 'server', 'mobile', 'Other'))
);

