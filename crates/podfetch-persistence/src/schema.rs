// @generated automatically by Diesel CLI.

diesel::table! {
    device_sync_groups (id) {
        id -> Integer,
        user_id -> Integer,
        group_id -> Integer,
        device_id -> Text,
    }
}

diesel::table! {
    devices (id) {
        id -> Integer,
        deviceid -> Text,
        kind -> Text,
        name -> Text,
        user_id -> Integer,
        chromecast_uuid -> Nullable<Text>,
        agent_id -> Nullable<Text>,
        last_seen_at -> Nullable<Timestamp>,
        ip -> Nullable<Text>,
    }
}

diesel::table! {
    episodes (id) {
        id -> Integer,
        user_id -> Integer,
        device -> Text,
        podcast -> Text,
        episode -> Text,
        timestamp -> Timestamp,
        guid -> Nullable<Text>,
        action -> Text,
        started -> Nullable<Integer>,
        position -> Nullable<Integer>,
        total -> Nullable<Integer>,
    }
}

diesel::table! {
    favorite_podcast_episodes (user_id, episode_id) {
        user_id -> Integer,
        episode_id -> Integer,
        favorite -> Bool,
    }
}

diesel::table! {
    favorites (user_id, podcast_id) {
        user_id -> Integer,
        podcast_id -> Integer,
        favored -> Bool,
    }
}

diesel::table! {
    filters (user_id) {
        user_id -> Integer,
        title -> Nullable<Text>,
        ascending -> Bool,
        filter -> Nullable<Text>,
        only_favored -> Bool,
    }
}

diesel::table! {
    gpodder_settings (id) {
        id -> Integer,
        user_id -> Integer,
        scope -> Text,
        scope_id -> Nullable<Text>,
        data -> Text,
    }
}

diesel::table! {
    invites (id) {
        id -> Text,
        role -> Text,
        created_at -> Timestamp,
        accepted_at -> Nullable<Timestamp>,
        explicit_consent -> Bool,
        expires_at -> Timestamp,
    }
}

diesel::table! {
    notifications (id) {
        id -> Integer,
        type_of_message -> Text,
        message -> Text,
        created_at -> Text,
        status -> Text,
    }
}

diesel::table! {
    listening_events (id) {
        id -> Integer,
        user_id -> Integer,
        device -> Text,
        podcast_episode_id -> Text,
        podcast_id -> Integer,
        podcast_episode_db_id -> Integer,
        delta_seconds -> Integer,
        start_position -> Integer,
        end_position -> Integer,
        listened_at -> Timestamp,
    }
}

diesel::table! {
    playlist_items (playlist_id, episode) {
        playlist_id -> Text,
        episode -> Integer,
        position -> Integer,
    }
}

diesel::table! {
    playlists (id) {
        id -> Text,
        name -> Text,
        user_id -> Integer,
    }
}

diesel::table! {
    podcast_episode_chapters (id) {
        id -> Text,
        episode_id -> Integer,
        title -> Text,
        start_time -> Integer,
        end_time -> Integer,
        href -> Nullable<Text>,
        image -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        chapter_type -> Text,
    }
}

diesel::table! {
    podcast_episodes (id) {
        id -> Integer,
        podcast_id -> Integer,
        episode_id -> Text,
        name -> Text,
        url -> Text,
        date_of_recording -> Text,
        image_url -> Text,
        total_time -> Integer,
        description -> Text,
        download_time -> Nullable<Timestamp>,
        guid -> Text,
        deleted -> Bool,
        file_episode_path -> Nullable<Text>,
        file_image_path -> Nullable<Text>,
        episode_numbering_processed -> Bool,
        download_location -> Nullable<Text>,
        sponsorblock_fetched_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    podcast_settings (podcast_id) {
        podcast_id -> Integer,
        episode_numbering -> Bool,
        auto_download -> Bool,
        auto_update -> Bool,
        auto_cleanup -> Bool,
        auto_cleanup_days -> Integer,
        replace_invalid_characters -> Bool,
        use_existing_filename -> Bool,
        replacement_strategy -> Text,
        episode_format -> Text,
        podcast_format -> Text,
        direct_paths -> Bool,
        activated -> Bool,
        podcast_prefill -> Integer,
        use_one_cover_for_all_episodes -> Bool,
        sponsorblock_enabled -> Bool,
        sponsorblock_categories -> Text,
    }
}

diesel::table! {
    podcasts (id) {
        id -> Integer,
        name -> Text,
        directory_id -> Text,
        rssfeed -> Text,
        image_url -> Text,
        summary -> Nullable<Text>,
        language -> Nullable<Text>,
        explicit -> Nullable<Text>,
        keywords -> Nullable<Text>,
        last_build_date -> Nullable<Text>,
        author -> Nullable<Text>,
        active -> Bool,
        original_image_url -> Text,
        directory_name -> Text,
        download_location -> Nullable<Text>,
        guid -> Nullable<Text>,
        added_by -> Nullable<Integer>,
    }
}

diesel::table! {
    sessions (user_id, session_id) {
        username -> Text,
        user_id -> Integer,
        session_id -> Text,
        expires -> Timestamp,
    }
}

diesel::table! {
    settings (id) {
        id -> Integer,
        auto_download -> Bool,
        auto_update -> Bool,
        auto_cleanup -> Bool,
        auto_cleanup_days -> Integer,
        podcast_prefill -> Integer,
        replace_invalid_characters -> Bool,
        use_existing_filename -> Bool,
        replacement_strategy -> Text,
        episode_format -> Text,
        podcast_format -> Text,
        direct_paths -> Bool,
        auto_transcode_opus -> Bool,
        use_one_cover_for_all_episodes -> Bool,
        sponsorblock_enabled -> Bool,
        sponsorblock_categories -> Text,
    }
}

diesel::table! {
    subscriptions (id) {
        id -> Integer,
        user_id -> Integer,
        device -> Text,
        podcast -> Text,
        created -> Timestamp,
        deleted -> Nullable<Timestamp>,
    }
}

diesel::table! {
    tags (id) {
        id -> Text,
        name -> Text,
        user_id -> Integer,
        description -> Nullable<Text>,
        created_at -> Timestamp,
        color -> Text,
    }
}

diesel::table! {
    tags_podcasts (tag_id, podcast_id) {
        tag_id -> Text,
        podcast_id -> Integer,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        username -> Text,
        role -> Text,
        password -> Nullable<Text>,
        explicit_consent -> Bool,
        created_at -> Timestamp,
        api_key -> Nullable<Text>,
        country -> Nullable<Text>,
        language -> Nullable<Text>,
    }
}

diesel::joinable!(favorite_podcast_episodes -> podcast_episodes (episode_id));
diesel::joinable!(listening_events -> podcast_episodes (podcast_episode_db_id));
diesel::joinable!(listening_events -> podcasts (podcast_id));
diesel::joinable!(favorites -> podcasts (podcast_id));
diesel::joinable!(playlist_items -> playlists (playlist_id));
diesel::joinable!(playlist_items -> podcast_episodes (episode));
diesel::joinable!(podcast_episode_chapters -> podcast_episodes (episode_id));
diesel::joinable!(podcast_episodes -> podcasts (podcast_id));
diesel::joinable!(tags_podcasts -> podcasts (podcast_id));
diesel::joinable!(tags_podcasts -> tags (tag_id));

diesel::allow_tables_to_appear_in_same_query!(
    device_sync_groups,
    devices,
    episodes,
    favorite_podcast_episodes,
    favorites,
    filters,
    gpodder_settings,
    invites,
    listening_events,
    notifications,
    playlist_items,
    playlists,
    podcast_episode_chapters,
    podcast_episodes,
    podcast_settings,
    podcasts,
    sessions,
    settings,
    subscriptions,
    tags,
    tags_podcasts,
    users,
);
