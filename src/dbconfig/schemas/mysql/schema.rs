// @generated automatically by Diesel CLI.

diesel::table! {
    devices (id) {
        id -> Integer,
        deviceid -> Varchar,
        kind -> Varchar,
        name -> Varchar,
        username -> Varchar,
    }
}

diesel::table! {
    episodes (id) {
        id -> Integer,
        username -> Varchar,
        device -> Varchar,
        podcast -> Varchar,
        episode -> Varchar,
        timestamp -> Timestamp,
        guid -> Nullable<Varchar>,
        action -> Varchar,
        started -> Nullable<Integer>,
        position -> Nullable<Integer>,
        total -> Nullable<Integer>,
    }
}

diesel::table! {
    favorites (username, podcast_id) {
        username -> Varchar,
        podcast_id -> Integer,
        favored -> Bool,
    }
}

diesel::table! {
    filters (username) {
        username -> Varchar,
        title -> Nullable<Varchar>,
        ascending -> Bool,
        filter -> Nullable<Varchar>,
        only_favored -> Bool,
    }
}

diesel::table! {
    invites (id) {
        id -> Varchar,
        role -> Varchar,
        created_at -> Timestamp,
        accepted_at -> Nullable<Timestamp>,
        explicit_consent -> Bool,
        expires_at -> Timestamp,
    }
}

diesel::table! {
    notifications (id) {
        id -> Integer,
        type_of_message -> Varchar,
        message -> Varchar,
        created_at -> Varchar,
        status -> Varchar,
    }
}

diesel::table! {
    podcast_episodes (id) {
        id -> Integer,
        podcast_id -> Integer,
        episode_id -> Varchar,
        name -> Varchar,
        url -> Varchar,
        date_of_recording -> Varchar,
        image_url -> Varchar,
        total_time -> Integer,
        local_url -> Varchar,
        local_image_url -> Varchar,
        description -> Varchar,
        status -> Char,
        download_time -> Nullable<Timestamp>,
    }
}

diesel::table! {
    podcast_history_items (id) {
        id -> Integer,
        podcast_id -> Integer,
        episode_id -> Varchar,
        watched_time -> Integer,
        date -> Timestamp,
        username -> Varchar,
    }
}

diesel::table! {
    podcasts (id) {
        id -> Integer,
        name -> Varchar,
        directory_id -> Varchar,
        rssfeed -> Varchar,
        image_url -> Varchar,
        summary -> Nullable<Varchar>,
        language -> Nullable<Varchar>,
        explicit -> Nullable<Varchar>,
        keywords -> Nullable<Varchar>,
        last_build_date -> Nullable<Varchar>,
        author -> Nullable<Varchar>,
        active -> Bool,
        original_image_url -> Varchar,
        directory_name -> Varchar,
    }
}

diesel::table! {
    sessions (username, session_id) {
        username -> Varchar,
        session_id -> Varchar,
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
        replacement_strategy -> Varchar,
        episode_format -> Varchar,
        podcast_format -> Varchar,
    }
}

diesel::table! {
    subscriptions (id) {
        id -> Integer,
        username -> Varchar,
        device -> Varchar,
        podcast -> Varchar,
        created -> Timestamp,
        deleted -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        username -> Varchar,
        role -> Varchar,
        password -> Nullable<Varchar>,
        explicit_consent -> Bool,
        created_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    devices,
    episodes,
    favorites,
    filters,
    invites,
    notifications,
    podcast_episodes,
    podcast_history_items,
    podcasts,
    sessions,
    settings,
    subscriptions,
    users,
);
