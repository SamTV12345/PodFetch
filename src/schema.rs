// @generated automatically by Diesel CLI.

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
    }
}

diesel::table! {
    podcast_history_items (id) {
        id -> Integer,
        podcast_id -> Integer,
        episode_id -> Text,
        watched_time -> Integer,
        date -> Text,
    }
}

diesel::table! {
    podcasts (id) {
        id -> Integer,
        name -> Text,
        directory -> Text,
        rssfeed -> Text,
        image_url -> Text,
        favored -> Integer,
        summary -> Nullable<Text>,
        language -> Nullable<Text>,
        explicit -> Nullable<Text>,
        keywords -> Nullable<Text>,
        last_build_date -> Nullable<Text>,
        author -> Nullable<Text>,
        active -> Bool,
    }
}

diesel::table! {
    settings (id) {
        id -> Integer,
        auto_download -> Bool,
        auto_update -> Bool,
        auto_cleanup -> Bool,
        auto_cleanup_days -> Integer,
    }
}

diesel::joinable!(podcast_episodes -> podcasts (podcast_id));
diesel::joinable!(podcast_history_items -> podcasts (podcast_id));

diesel::allow_tables_to_appear_in_same_query!(
    notifications,
    podcast_episodes,
    podcast_history_items,
    podcasts,
    settings,
);
