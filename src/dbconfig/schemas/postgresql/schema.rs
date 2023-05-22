// @generated automatically by Diesel CLI.

diesel::table! {
    devices (id) {
        id -> Int4,
        deviceid -> Varchar,
        kind -> Text,
        name -> Varchar,
        username -> Varchar,
    }
}

diesel::table! {
    episodes (id) {
        id -> Int4,
        username -> Varchar,
        device -> Varchar,
        podcast -> Varchar,
        episode -> Varchar,
        timestamp -> Timestamp,
        guid -> Nullable<Varchar>,
        action -> Varchar,
        started -> Nullable<Int4>,
        position -> Nullable<Int4>,
        total -> Nullable<Int4>,
    }
}

diesel::table! {
    favorites (username, podcast_id) {
        username -> Text,
        podcast_id -> Int4,
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
        id -> Varchar,
        role -> Text,
        created_at -> Timestamp,
        accepted_at -> Nullable<Timestamp>,
        explicit_consent -> Bool,
        expires_at -> Timestamp,
    }
}

diesel::table! {
    notifications (id) {
        id -> Int4,
        type_of_message -> Text,
        message -> Text,
        created_at -> Text,
        status -> Text,
    }
}

diesel::table! {
    podcast_episodes (id) {
        id -> Int4,
        podcast_id -> Int4,
        episode_id -> Text,
        name -> Text,
        url -> Text,
        date_of_recording -> Text,
        image_url -> Text,
        total_time -> Int4,
        local_url -> Text,
        local_image_url -> Text,
        description -> Text,
        status -> Bpchar,
        download_time -> Nullable<Timestamp>,
    }
}

diesel::table! {
    podcast_history_items (id) {
        id -> Int4,
        podcast_id -> Int4,
        episode_id -> Text,
        watched_time -> Int4,
        date -> Timestamp,
        username -> Text,
    }
}

diesel::table! {
    podcasts (id) {
        id -> Int4,
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
        id -> Int4,
        auto_download -> Bool,
        auto_update -> Bool,
        auto_cleanup -> Bool,
        auto_cleanup_days -> Int4,
        podcast_prefill -> Int4,
        replace_invalid_characters -> Bool,
        use_existing_filename -> Bool,
        replacement_strategy -> Text,
        episode_format -> Text,
        podcast_format -> Text,
    }
}

diesel::table! {
    subscriptions (id) {
        id -> Int4,
        username -> Text,
        device -> Text,
        podcast -> Text,
        created -> Timestamp,
        deleted -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        role -> Text,
        password -> Nullable<Varchar>,
        explicit_consent -> Bool,
        created_at -> Timestamp,
    }
}

diesel::joinable!(favorites -> podcasts (podcast_id));
diesel::joinable!(podcast_episodes -> podcasts (podcast_id));
diesel::joinable!(podcast_history_items -> podcasts (podcast_id));

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
