use crate::services::episode_triage::service::EpisodeTriageService;
use crate::services::playlist::service::PlaylistService;
use crate::services::podcast_episode_chapter::service::PodcastEpisodeChapterService;
use crate::services::podcast_settings::service::PodcastSettingsService;
use crate::services::settings::service::SettingsService;
use crate::services::transcript::service::TranscriptService;
use crate::usecases::watchtime::WatchtimeUseCase;
use common_infrastructure::config::EnvironmentService;
use podfetch_audiobooks::persistence::AuthorRepositoryImpl;
use podfetch_audiobooks::persistence::BookAudioFileRepositoryImpl;
use podfetch_audiobooks::persistence::BookChapterRepositoryImpl;
use podfetch_audiobooks::persistence::BookRepositoryImpl;
use podfetch_audiobooks::persistence::LibraryRepositoryImpl;
use podfetch_audiobooks::persistence::ListeningSessionRepositoryImpl;
use podfetch_audiobooks::persistence::MediaProgressRepositoryImpl;
use podfetch_audiobooks::persistence::NarratorRepositoryImpl;
use podfetch_audiobooks::persistence::PlaybackSessionRepositoryImpl;
use podfetch_audiobooks::persistence::SeriesRepositoryImpl;
use podfetch_audiobooks::services::audiobook_scanner::AudiobookScanner;
use podfetch_audiobooks::services::book_service::AudiobookshelfBookService;
use podfetch_audiobooks::services::hls_transcoder::HlsTranscoder;
use podfetch_audiobooks::services::library_service::AudiobookshelfLibraryService;
use podfetch_audiobooks::services::listening_session_service::AudiobookshelfListeningSessionService;
use podfetch_audiobooks::services::login_service::AudiobookshelfLoginService;
use podfetch_audiobooks::services::media_progress_service::AudiobookshelfMediaProgressService;
use podfetch_audiobooks::services::playback_session_service::AudiobookshelfPlaybackSessionService;
use podfetch_cast::StubCastDriver;
use podfetch_casting::agent::dispatcher::AgentDispatcher;
use podfetch_casting::agent::registry::AgentRegistry;
use podfetch_casting::dto::ServerCastOrchestrator;
use podfetch_casting::mopidy::driver::{MopidyDriver, MopidyEvent};
use podfetch_casting::orchestrator::CastOrchestrator;
use podfetch_catalog::persistence::chapter::PodcastEpisodeChapterRepositoryImpl;
use podfetch_gpodder::persistence::DeviceRepositoryImpl;
use podfetch_gpodder::persistence::DeviceSyncGroupRepositoryImpl;
use podfetch_gpodder::persistence::GpodderSettingRepositoryImpl;
use podfetch_gpodder::persistence::SubscriptionRepositoryImpl;
use podfetch_gpodder::services::device::DeviceService;
use podfetch_gpodder::services::device_sync_group::DeviceSyncGroupService;
use podfetch_gpodder::services::gpodder_setting::GpodderSettingService;
use podfetch_gpodder::services::subscription::SubscriptionService;
use podfetch_identity::persistence::adapters::{
    InviteRepositoryImpl, SessionRepositoryImpl, UserAdminRepositoryImpl,
};
use podfetch_identity::services::invite::service::InviteService;
use podfetch_identity::services::login::service::LoginService;
use podfetch_identity::services::session::service::SessionService;
use podfetch_identity::services::user_admin::service::UserAdminService;
use podfetch_identity::services::user_auth::service::UserAuthService;
use podfetch_identity::services::user_onboarding::service::UserOnboardingService;
use podfetch_library::persistence::EpisodeTriageRepositoryImpl;
use podfetch_library::persistence::FavoritePodcastEpisodeRepositoryImpl;
use podfetch_library::persistence::FilterRepositoryImpl;
use podfetch_library::persistence::PlaylistRepositoryImpl;
use podfetch_library::persistence::TagRepositoryImpl;
use podfetch_library::services::favorite_podcast_episode::FavoritePodcastEpisodeService;
use podfetch_library::services::filter::FilterService;
use podfetch_library::services::tag::TagService;
use podfetch_listening::persistence::ListeningEventRepositoryImpl;
use podfetch_listening::services::stats::StatsService;
use podfetch_notifications::persistence::NotificationRepositoryImpl;
use podfetch_notifications::service::NotificationService;
use podfetch_persistence::adapters::PodcastEpisodeTranscriptRepositoryImpl;
use podfetch_persistence::adapters::PodcastSettingsRepositoryImpl;
use podfetch_persistence::adapters::SettingsRepositoryImpl;
use podfetch_persistence::adapters::TranscriptionJobRepositoryImpl;
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct AppState {
    pub event_bus: Arc<podfetch_events::EventBus>,
    pub identity: Arc<dyn podfetch_identity::api::IdentityApi>,
    pub agent_dispatcher: Arc<AgentDispatcher>,
    pub agent_registry: Arc<AgentRegistry>,
    pub audiobookshelf_book_service: Arc<AudiobookshelfBookService>,
    pub audiobookshelf_hls_transcoder: Arc<HlsTranscoder>,
    pub audiobookshelf_library_service: Arc<AudiobookshelfLibraryService>,
    pub audiobookshelf_listening_session_service: Arc<AudiobookshelfListeningSessionService>,
    pub audiobookshelf_login_service: Arc<AudiobookshelfLoginService>,
    pub audiobookshelf_media_progress_service: Arc<AudiobookshelfMediaProgressService>,
    pub audiobookshelf_playback_session_service: Arc<AudiobookshelfPlaybackSessionService>,
    pub audiobookshelf_scanner: Arc<AudiobookScanner>,
    pub cast_orchestrator: Arc<ServerCastOrchestrator>,
    pub device_service: Arc<DeviceService>,
    pub mopidy_event_rx: Arc<AsyncMutex<Option<mpsc::Receiver<MopidyEvent>>>>,
    pub device_sync_group_service: Arc<DeviceSyncGroupService>,
    pub environment: Arc<EnvironmentService>,
    pub episode_triage_service: Arc<EpisodeTriageService>,
    pub favorite_podcast_episode_service: Arc<FavoritePodcastEpisodeService>,
    pub filter_service: Arc<FilterService>,
    pub gpodder_setting_service: Arc<GpodderSettingService>,
    pub invite_service: Arc<InviteService>,
    pub login_service: Arc<LoginService>,
    pub notification_service: Arc<NotificationService>,
    pub playlist_service: Arc<PlaylistService>,
    pub podcast_episode_chapter_service: Arc<PodcastEpisodeChapterService>,
    pub podcast_settings_service: Arc<PodcastSettingsService>,
    pub session_service: Arc<SessionService>,
    pub settings_service: Arc<SettingsService>,
    pub media: crate::media::MediaServices,
    pub catalog: Arc<dyn podfetch_catalog::api::CatalogApi>,
    pub stats_service: Arc<StatsService>,
    pub subscription_service: Arc<SubscriptionService>,
    pub tag_service: Arc<TagService>,
    pub transcript_service: Arc<TranscriptService>,
    pub user_admin_service: Arc<UserAdminService>,
    pub user_auth_service: Arc<UserAuthService>,
    pub user_onboarding_service: Arc<UserOnboardingService>,
    pub watchtime_service: Arc<WatchtimeUseCase>,
}

impl AppState {
    /// Wires every service onto `database`. Only the
    /// [`crate::application::Application`] (and test support) builds one.
    pub(crate) fn build(
        database: podfetch_persistence::db::Database,
        environment: Arc<EnvironmentService>,
    ) -> Self {
        let device_service = Arc::new(DeviceService::new(Arc::new(DeviceRepositoryImpl::new(
            database.clone(),
        ))));
        let agent_registry = Arc::new(AgentRegistry::new());
        let agent_dispatcher = Arc::new(AgentDispatcher::new(agent_registry.clone()));
        let (mopidy_tx, mopidy_rx) = mpsc::channel::<MopidyEvent>(64);
        let mopidy_driver = Arc::new(MopidyDriver::new(mopidy_tx));
        let cast_orchestrator = Arc::new(CastOrchestrator::new(
            Arc::new(
                podfetch_casting::persistence::CastDeviceRepositoryImpl::new(database.clone()),
            ),
            Arc::new(StubCastDriver),
            agent_dispatcher.clone(),
            mopidy_driver,
        ));
        let device_sync_group_service = Arc::new(DeviceSyncGroupService::new(Arc::new(
            DeviceSyncGroupRepositoryImpl::new(database.clone()),
        )));
        let favorite_podcast_episode_service = Arc::new(FavoritePodcastEpisodeService::new(
            Arc::new(FavoritePodcastEpisodeRepositoryImpl::new(database.clone())),
        ));
        let episode_triage_service = Arc::new(EpisodeTriageService::new(Arc::new(
            EpisodeTriageRepositoryImpl::new(database.clone()),
        )));
        let filter_service = Arc::new(FilterService::new(Arc::new(FilterRepositoryImpl::new(
            database.clone(),
        ))));
        let gpodder_setting_service = Arc::new(GpodderSettingService::new(Arc::new(
            GpodderSettingRepositoryImpl::new(database.clone()),
        )));
        let invite_service = Arc::new(InviteService::new(Arc::new(InviteRepositoryImpl::new(
            database.clone(),
        ))));
        let user_auth_service = Arc::new(UserAuthService::new(
            Arc::new(UserAdminRepositoryImpl::new(database.clone())),
            environment.clone(),
        ));
        let login_service = Arc::new(LoginService::new(
            environment.clone(),
            user_auth_service.clone(),
        ));
        let notification_service = Arc::new(NotificationService::new(Arc::new(
            NotificationRepositoryImpl::new(database.clone()),
        )));
        let playlist_service = Arc::new(PlaylistService::new(Arc::new(
            PlaylistRepositoryImpl::new(database.clone()),
        )));
        let podcast_episode_chapter_service = Arc::new(PodcastEpisodeChapterService::new(
            Arc::new(PodcastEpisodeChapterRepositoryImpl::new(database.clone())),
        ));
        let podcast_settings_service = Arc::new(PodcastSettingsService::new(Arc::new(
            PodcastSettingsRepositoryImpl::new(database.clone()),
        )));
        let session_service = Arc::new(SessionService::new(Arc::new(SessionRepositoryImpl::new(
            database.clone(),
        ))));
        let settings_service = Arc::new(SettingsService::new(Arc::new(
            SettingsRepositoryImpl::new(database.clone()),
        )));
        let catalog: Arc<dyn podfetch_catalog::api::CatalogApi> =
            Arc::new(podfetch_catalog::api::LocalCatalog::new(database.clone()));
        let listening_events = Arc::new(
            podfetch_listening::services::listening_event::ListeningEventService::new(Arc::new(
                ListeningEventRepositoryImpl::new(database.clone()),
            )),
        );
        let stats_service = Arc::new(StatsService::new(
            listening_events.clone(),
            Arc::new(crate::listening::CatalogPodcasts),
        ));
        let event_bus = podfetch_events::EventBus::new(
            database.clone(),
            podfetch_events::EventBusConfig::default(),
        );
        let media = crate::media::MediaServices::build(
            database.clone(),
            environment.clone(),
            settings_service.clone(),
            Arc::new(crate::media::BusEvents(Some(event_bus.clone()))),
        );
        let subscription_service = Arc::new(SubscriptionService::new(
            Arc::new(SubscriptionRepositoryImpl::new(database.clone())),
            Arc::new(DeviceSyncGroupRepositoryImpl::new(database.clone())),
            event_bus.clone(),
        ));
        let tag_service = Arc::new(TagService::new(Arc::new(TagRepositoryImpl::new(
            database.clone(),
        ))));
        let transcript_service = Arc::new(TranscriptService::new(
            Arc::new(PodcastEpisodeTranscriptRepositoryImpl::new(
                database.clone(),
            )),
            Arc::new(TranscriptionJobRepositoryImpl::new(database.clone())),
        ));
        let watchtime_service = Arc::new(WatchtimeUseCase::new());
        let identity: Arc<dyn podfetch_identity::api::IdentityApi> =
            Arc::new(podfetch_identity::api::Identity::new(Arc::new(
                UserAdminRepositoryImpl::new(database.clone()),
            )));
        podfetch_notifications::handlers::subscribe(
            &event_bus,
            notification_service.clone(),
            environment
                .telegram_api
                .clone()
                .map(podfetch_notifications::telegram::TelegramNotifier::new),
        );
        podfetch_casting::handlers::subscribe(&event_bus, cast_orchestrator.clone());
        podfetch_listening::handlers::subscribe(
            &event_bus,
            Arc::new(
                podfetch_listening::persistence::episode::DieselEpisodeRepository::new(
                    database.clone(),
                ),
            ),
            listening_events.clone(),
        );
        podfetch_gpodder::handlers::subscribe_user_cleanup(
            &event_bus,
            Arc::new(podfetch_gpodder::handlers::UserData {
                devices: Arc::new(DeviceRepositoryImpl::new(database.clone())),
                subscriptions: Arc::new(SubscriptionRepositoryImpl::new(database.clone())),
                sync_groups: Arc::new(DeviceSyncGroupRepositoryImpl::new(database.clone())),
                settings: Arc::new(
                    podfetch_gpodder::persistence::GpodderSettingRepositoryImpl::new(
                        database.clone(),
                    ),
                ),
            }),
        );
        let favorites: Arc<
            dyn podfetch_library::domain::favorite::FavoriteRepository<
                    Error = common_infrastructure::error::CustomError,
                >,
        > = Arc::new(podfetch_library::persistence::FavoriteRepositoryImpl::new(
            database.clone(),
        ));
        podfetch_library::handlers::subscribe_user_cleanup(&event_bus, favorites.clone());
        podfetch_library::handlers::subscribe_favorite_sync(
            &event_bus,
            favorites,
            Arc::new(crate::catalog_ports::CatalogFeeds(catalog.clone())),
        );
        if environment.gpodder_integration_enabled {
            podfetch_gpodder::handlers::subscribe_favorite_sync(
                &event_bus,
                subscription_service.clone(),
                Arc::new(crate::catalog_ports::CatalogFeeds(catalog.clone())),
            );
        }
        podfetch_audiobooks::handlers::subscribe(&event_bus, database.clone());
        podfetch_media::handlers::subscribe_user_cleanup(&event_bus, media.sponsorblock.clone());
        podfetch_media::handlers::subscribe_auto_download(&event_bus, media.auto_download.clone());
        crate::podcast_events::install(event_bus.clone());
        let user_admin_service = Arc::new(UserAdminService::new(
            Arc::new(UserAdminRepositoryImpl::new(database.clone())),
            environment.clone(),
            event_bus.clone(),
        ));
        let user_onboarding_service = Arc::new(UserOnboardingService::new(
            invite_service.clone(),
            Arc::new(UserAdminRepositoryImpl::new(database.clone())),
        ));

        let audiobookshelf_book_repository = Arc::new(BookRepositoryImpl::new(database.clone()));
        let audiobookshelf_author_repository =
            Arc::new(AuthorRepositoryImpl::new(database.clone()));
        let audiobookshelf_narrator_repository =
            Arc::new(NarratorRepositoryImpl::new(database.clone()));
        let audiobookshelf_series_repository =
            Arc::new(SeriesRepositoryImpl::new(database.clone()));
        let audiobookshelf_audio_file_repository =
            Arc::new(BookAudioFileRepositoryImpl::new(database.clone()));
        let audiobookshelf_chapter_repository =
            Arc::new(BookChapterRepositoryImpl::new(database.clone()));
        let audiobookshelf_library_service = Arc::new(AudiobookshelfLibraryService::new(Arc::new(
            LibraryRepositoryImpl::new(database.clone()),
        )));
        let audiobookshelf_book_service = Arc::new(AudiobookshelfBookService {
            book_repository: audiobookshelf_book_repository.clone(),
            author_repository: audiobookshelf_author_repository.clone(),
            narrator_repository: audiobookshelf_narrator_repository.clone(),
            series_repository: audiobookshelf_series_repository.clone(),
            audio_file_repository: audiobookshelf_audio_file_repository.clone(),
            chapter_repository: audiobookshelf_chapter_repository.clone(),
        });
        let audiobookshelf_scanner = Arc::new(AudiobookScanner {
            library_service: audiobookshelf_library_service.clone(),
            book_repository: audiobookshelf_book_repository,
            audio_file_repository: audiobookshelf_audio_file_repository,
            chapter_repository: audiobookshelf_chapter_repository,
            author_repository: audiobookshelf_author_repository,
            narrator_repository: audiobookshelf_narrator_repository,
            series_repository: audiobookshelf_series_repository,
            environment: environment.clone(),
        });
        let audiobookshelf_media_progress_service =
            Arc::new(AudiobookshelfMediaProgressService::new(Arc::new(
                MediaProgressRepositoryImpl::new(database.clone()),
            )));
        let audiobookshelf_playback_session_service =
            Arc::new(AudiobookshelfPlaybackSessionService::new(Arc::new(
                PlaybackSessionRepositoryImpl::new(database.clone()),
            )));
        let audiobookshelf_listening_session_service =
            Arc::new(AudiobookshelfListeningSessionService::new(Arc::new(
                ListeningSessionRepositoryImpl::new(database),
            )));
        let audiobookshelf_login_service =
            Arc::new(AudiobookshelfLoginService::new(user_auth_service.clone()));
        let audiobookshelf_hls_transcoder = Arc::new(HlsTranscoder::new(environment.clone()));

        Self {
            event_bus,
            identity,
            agent_dispatcher,
            agent_registry,
            audiobookshelf_book_service,
            audiobookshelf_hls_transcoder,
            audiobookshelf_library_service,
            audiobookshelf_listening_session_service,
            audiobookshelf_login_service,
            audiobookshelf_media_progress_service,
            audiobookshelf_playback_session_service,
            audiobookshelf_scanner,
            cast_orchestrator,
            device_service,
            mopidy_event_rx: Arc::new(AsyncMutex::new(Some(mopidy_rx))),
            device_sync_group_service,
            environment,
            episode_triage_service,
            favorite_podcast_episode_service,
            filter_service,
            gpodder_setting_service,
            invite_service,
            login_service,
            notification_service,
            playlist_service,
            podcast_episode_chapter_service,
            podcast_settings_service,
            session_service,
            settings_service,
            media,
            catalog,
            stats_service,
            subscription_service,
            tag_service,
            transcript_service,
            user_admin_service,
            user_auth_service,
            user_onboarding_service,
            watchtime_service,
        }
    }
}
