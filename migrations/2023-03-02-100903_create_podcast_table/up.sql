-- Your SQL goes here
create table if not exists podcasts (
                                       id integer primary key not null,
                                       name text not null unique,
                                       directory text not null,
                                       rssfeed text not null,
                                       image_url text not null)