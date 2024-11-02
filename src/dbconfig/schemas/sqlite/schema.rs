// @generated automatically by Diesel CLI.

diesel::table! {
    devices (id) {
        id -> Integer,
        deviceid -> Text,
        kind -> Text,
        name -> Text,
        username -> Text,
    }
}

diesel::table! {
    episodes (id) {
        id -> Integer,
        username -> Text,
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
    favorites (username, podcast_id) {
        username -> Text,
        podcast_id -> Integer,
        favored -> Bool,
    }
}

diesel::table! {
    filters (username) {
        username -> Text,
        title -> Nullable<Text>,
        ascending -> Bool,
        filter -> Nullable<Text>,
        only_favored -> Bool,
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
    podcast_episodes (id) {
        id -> Integer,
        podcast_id -> Integer,
        episode_id -> Text,
        name -> Text,
        url -> Text,
        date_of_recording -> Text,
        image_url -> Text,
        total_time -> Integer,
        local_url -> Text,
        local_image_url -> Text,
        description -> Text,
        status -> Text,
        download_time -> Nullable<Timestamp>,
        guid -> Text,
        deleted -> Bool,
        file_episode_path -> Nullable<Text>,
        file_image_path -> Nullable<Text>,
        episode_numbering_processed -> Bool,
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
    }
}

diesel::table! {
    sessions (username, session_id) {
        username -> Text,
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
        jwt_key -> Nullable<Binary>,
    }
}

diesel::table! {
    subscriptions (id) {
        id -> Integer,
        username -> Text,
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
        username -> Text,
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
    }
}

diesel::table! {
    watch_together_users (subject) {
        subject -> Text,
        name -> Nullable<Text>,
        user_id -> Nullable<Integer>,
    }
}

diesel::table! {
    watch_together_users_to_room_mappings (room_id, subject) {
        room_id -> Integer,
        subject -> Text,
        status -> Text,
        role -> Text,
    }
}

diesel::table! {
    watch_togethers (id) {
        id -> Integer,
        room_id -> Text,
        room_name -> Text,
    }
}

diesel::joinable!(favorites -> podcasts (podcast_id));
diesel::joinable!(playlist_items -> playlists (playlist_id));
diesel::joinable!(playlist_items -> podcast_episodes (episode));
diesel::joinable!(podcast_episodes -> podcasts (podcast_id));
diesel::joinable!(tags_podcasts -> podcasts (podcast_id));
diesel::joinable!(tags_podcasts -> tags (tag_id));
diesel::joinable!(watch_together_users -> users (user_id));
diesel::joinable!(watch_together_users_to_room_mappings -> watch_togethers (room_id));

diesel::allow_tables_to_appear_in_same_query!(
    devices,
    episodes,
    favorites,
    filters,
    invites,
    notifications,
    playlist_items,
    playlists,
    podcast_episodes,
    podcast_settings,
    podcasts,
    sessions,
    settings,
    subscriptions,
    tags,
    tags_podcasts,
    users,
    watch_together_users,
    watch_together_users_to_room_mappings,
    watch_togethers,
);
