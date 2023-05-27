create table episodes
(
    id        INTEGER      not null AUTO_INCREMENT
        primary key,
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
    UNIQUE episode_unique(episode)
);

create table filters
(
    username     VARCHAR(255)                  not null
        primary key,
    title        VARCHAR(255),
    ascending    BOOLEAN default FALSE not null,
    filter       VARCHAR(255),
    only_favored BOOLEAN default TRUE  not null,
    check (filter IN ('PublishedDate', 'Title'))
);

create table invites
(
    id               VARCHAR(255)                       not null  primary key,
    role             VARCHAR(255)                               not null,
    created_at       TIMESTAMP default CURRENT_TIMESTAMP not null,
    accepted_at      TIMESTAMP,
    explicit_consent BOOLEAN  default FALSE                 not null,
    expires_at       TIMESTAMP                           not null,
    check (role IN ('admin', 'uploader', 'user'))
);

create table notifications
(
    id              INTEGER not null AUTO_INCREMENT
        primary key,
    type_of_message VARCHAR(255)    not null,
    message         VARCHAR(255)    not null,
    created_at      VARCHAR(255)    not null,
    status          VARCHAR(255)    not null
);

create table podcasts
(
    id                 INTEGER                   not null AUTO_INCREMENT
        primary key,
    name               VARCHAR(255)                      not null,
    directory_id       VARCHAR(255)                      not null,
    rssfeed            VARCHAR(255)                      not null,
    image_url          VARCHAR(255)                      not null,
    summary            VARCHAR(255),
    language           VARCHAR(255),
    explicit           VARCHAR(255),
    keywords           VARCHAR(255),
    last_build_date    VARCHAR(255),
    author             VARCHAR(255),
    active             BOOLEAN      default TRUE not null,
    original_image_url VARCHAR(255) default ''   not null,
    directory_name     VARCHAR(255) default ''   not null
);

create table favorites
(
    username   VARCHAR(255)              not null,
    podcast_id INTEGER           not null
        references podcasts
            on delete cascade,
    favored    BOOLEAN default FALSE not null,
    primary key (username, podcast_id)
);

create table podcast_episodes
(
    id                INTEGER             not null AUTO_INCREMENT
        primary key,
    podcast_id        integer             not null
        references podcasts,
    episode_id        VARCHAR(255)                not null,
    name              VARCHAR(255)                not null,
    url               VARCHAR(255)                not null,
    date_of_recording VARCHAR(255)                not null,
    image_url         VARCHAR(255)                not null,
    total_time        integer default 0   not null,
    local_url         VARCHAR(255)    default ''  not null,
    local_image_url   VARCHAR(255)    default ''  not null,
    description       VARCHAR(255)    default ''  not null,
    status            CHAR(1) default 'N' not null,
    download_time     TIMESTAMP
);

create index podcast_episode_url_index
    on podcast_episodes (url);

create index podcast_episodes_podcast_id_index
    on podcast_episodes (podcast_id);

create table podcast_history_items
(
    id           INTEGER  not null AUTO_INCREMENT
        primary key,
    podcast_id   integer  not null
        references podcasts,
    episode_id   VARCHAR(255)     not null,
    watched_time integer  not null,
    date         TIMESTAMP not null,
    username     VARCHAR(255)     not null
);

create table sessions
(
    username   VARCHAR(255) not null,
    session_id VARCHAR(255) not null,
    expires    TIMESTAMP     not null,
    primary key (username, session_id)
);

create table settings
(
    id                         INTEGER                                            not null AUTO_INCREMENT
        primary key,
    auto_download              BOOLEAN default TRUE                               not null,
    auto_update                BOOLEAN default TRUE                               not null,
    auto_cleanup               BOOLEAN default FALSE                              not null,
    auto_cleanup_days          INTEGER default -1                                 not null,
    podcast_prefill            INTEGER default 5                                  not null,
    replace_invalid_characters BOOLEAN default TRUE                               not null,
    use_existing_filename      BOOLEAN default FALSE                              not null,
    replacement_strategy       VARCHAR(255)    default 'replace-with-dash-and-underscore' not null,
    episode_format             VARCHAR(255)    default '{}'                               not null,
    podcast_format             VARCHAR(255)    default '{}'                               not null,
    check (replacement_strategy IN
           ('replace-with-dash-and-underscore', 'remove', 'replace-with-dash'))
);


create table subscriptions
(
    id      INTEGER  not null AUTO_INCREMENT
        primary key,
    username VARCHAR(255)     not null,
    device   VARCHAR(255)     not null,
    podcast  VARCHAR(255)     not null,
    created  TIMESTAMP not null,
    deleted  TIMESTAMP,
    unique (username, device, podcast)
);

create table users
(
    id      INTEGER                             not null AUTO_INCREMENT
        primary key,
    username         VARCHAR(255)                       not null
        unique,
    role             VARCHAR(255)                               not null,
    password         VARCHAR(255),
    explicit_consent BOOLEAN  default FALSE                 not null,
    created_at       TIMESTAMP default CURRENT_TIMESTAMP not null,
    check (role IN ('admin', 'uploader', 'user'))
);

create table devices
(
    id       INTEGER not null AUTO_INCREMENT
        primary key,
    deviceid VARCHAR(255) not null,
    kind     VARCHAR(255)         not null,
    name     VARCHAR(255) not null,
    username VARCHAR(255) not null
        references users (username),
    check (kind IN ('desktop', 'laptop', 'server', 'mobile', 'Other'))
);

