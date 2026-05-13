use crate::cast::ServerCastOrchestrator;
use crate::services::agent::dispatcher::AgentDispatcher;
use crate::services::agent::registry::AgentRegistry;
use crate::services::audiobookshelf::audiobook_scanner::AudiobookScanner;
use crate::services::audiobookshelf::book_service::AudiobookshelfBookService;
use crate::services::audiobookshelf::library_service::AudiobookshelfLibraryService;
use crate::services::audiobookshelf::listening_session_service::AudiobookshelfListeningSessionService;
use crate::services::audiobookshelf::login_service::AudiobookshelfLoginService;
use crate::services::audiobookshelf::media_progress_service::AudiobookshelfMediaProgressService;
use crate::services::audiobookshelf::playback_session_service::AudiobookshelfPlaybackSessionService;
use crate::services::cast::service::CastOrchestrator;
use crate::services::device::service::DeviceService;
use crate::services::device_sync_group::service::DeviceSyncGroupService;
use crate::services::favorite_podcast_episode::service::FavoritePodcastEpisodeService;
use crate::services::filter::service::FilterService;
use crate::services::gpodder_setting::service::GpodderSettingService;
use crate::services::invite::service::InviteService;
use crate::services::login::service::LoginService;
use crate::services::notification::service::NotificationService;
use crate::services::playlist::service::PlaylistService;
use crate::services::podcast_episode_chapter::service::PodcastEpisodeChapterService;
use crate::services::podcast_settings::service::PodcastSettingsService;
use crate::services::session::service::SessionService;
use crate::services::settings::service::SettingsService;
use crate::services::stats::service::StatsService;
use crate::services::subscription::service::SubscriptionService;
use crate::services::tag::service::TagService;
use crate::services::user_admin::service::UserAdminService;
use crate::services::user_auth::service::UserAuthService;
use crate::services::user_onboarding::service::UserOnboardingService;
use crate::usecases::watchtime::WatchtimeUseCase;
use common_infrastructure::config::EnvironmentService;
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use podfetch_cast::StubCastDriver;
use podfetch_persistence::adapters::AuthorRepositoryImpl;
use podfetch_persistence::adapters::BookAudioFileRepositoryImpl;
use podfetch_persistence::adapters::BookChapterRepositoryImpl;
use podfetch_persistence::adapters::BookRepositoryImpl;
use podfetch_persistence::adapters::DeviceRepositoryImpl;
use podfetch_persistence::adapters::DeviceSyncGroupRepositoryImpl;
use podfetch_persistence::adapters::LibraryRepositoryImpl;
use podfetch_persistence::adapters::ListeningSessionRepositoryImpl;
use podfetch_persistence::adapters::MediaProgressRepositoryImpl;
use podfetch_persistence::adapters::NarratorRepositoryImpl;
use podfetch_persistence::adapters::PlaybackSessionRepositoryImpl;
use podfetch_persistence::adapters::SeriesRepositoryImpl;
use podfetch_persistence::adapters::FavoritePodcastEpisodeRepositoryImpl;
use podfetch_persistence::adapters::FilterRepositoryImpl;
use podfetch_persistence::adapters::GpodderSettingRepositoryImpl;
use podfetch_persistence::adapters::InviteRepositoryImpl;
use podfetch_persistence::adapters::ListeningEventRepositoryImpl;
use podfetch_persistence::adapters::NotificationRepositoryImpl;
use podfetch_persistence::adapters::PlaylistRepositoryImpl;
use podfetch_persistence::adapters::PodcastEpisodeChapterRepositoryImpl;
use podfetch_persistence::adapters::PodcastSettingsRepositoryImpl;
use podfetch_persistence::adapters::SessionRepositoryImpl;
use podfetch_persistence::adapters::SettingsRepositoryImpl;
use podfetch_persistence::adapters::SubscriptionRepositoryImpl;
use podfetch_persistence::adapters::TagRepositoryImpl;
use podfetch_persistence::adapters::UserAdminRepositoryImpl;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub agent_dispatcher: Arc<AgentDispatcher>,
    pub agent_registry: Arc<AgentRegistry>,
    pub audiobookshelf_book_service: Arc<AudiobookshelfBookService>,
    pub audiobookshelf_library_service: Arc<AudiobookshelfLibraryService>,
    pub audiobookshelf_listening_session_service: Arc<AudiobookshelfListeningSessionService>,
    pub audiobookshelf_login_service: Arc<AudiobookshelfLoginService>,
    pub audiobookshelf_media_progress_service: Arc<AudiobookshelfMediaProgressService>,
    pub audiobookshelf_playback_session_service: Arc<AudiobookshelfPlaybackSessionService>,
    pub audiobookshelf_scanner: Arc<AudiobookScanner>,
    pub cast_orchestrator: Arc<ServerCastOrchestrator>,
    pub device_service: Arc<DeviceService>,
    pub device_sync_group_service: Arc<DeviceSyncGroupService>,
    pub environment: Arc<EnvironmentService>,
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
    pub stats_service: Arc<StatsService>,
    pub subscription_service: Arc<SubscriptionService>,
    pub tag_service: Arc<TagService>,
    pub user_admin_service: Arc<UserAdminService>,
    pub user_auth_service: Arc<UserAuthService>,
    pub user_onboarding_service: Arc<UserOnboardingService>,
    pub watchtime_service: Arc<WatchtimeUseCase>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let database = podfetch_persistence::db::database();
        let environment = Arc::new(ENVIRONMENT_SERVICE.clone());
        let device_service = Arc::new(DeviceService::new(Arc::new(DeviceRepositoryImpl::new(
            database.clone(),
        ))));
        let agent_registry = Arc::new(AgentRegistry::new());
        let agent_dispatcher = Arc::new(AgentDispatcher::new(agent_registry.clone()));
        let cast_orchestrator = Arc::new(CastOrchestrator::new(
            device_service.clone(),
            Arc::new(StubCastDriver),
            agent_dispatcher.clone(),
        ));
        let device_sync_group_service = Arc::new(DeviceSyncGroupService::new(Arc::new(
            DeviceSyncGroupRepositoryImpl::new(database.clone()),
        )));
        let favorite_podcast_episode_service = Arc::new(FavoritePodcastEpisodeService::new(
            Arc::new(FavoritePodcastEpisodeRepositoryImpl::new(database.clone())),
        ));
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
        let stats_service = Arc::new(StatsService::new(Arc::new(
            crate::services::listening_event::service::ListeningEventService::new(Arc::new(
                ListeningEventRepositoryImpl::new(database.clone()),
            )),
        )));
        let subscription_service = Arc::new(SubscriptionService::new(Arc::new(
            SubscriptionRepositoryImpl::new(database.clone()),
        )));
        let tag_service = Arc::new(TagService::new(Arc::new(TagRepositoryImpl::new(
            database.clone(),
        ))));
        let watchtime_service = Arc::new(WatchtimeUseCase::new());
        let user_admin_service = Arc::new(UserAdminService::new(
            Arc::new(UserAdminRepositoryImpl::new(database.clone())),
            environment.clone(),
        ));
        let user_onboarding_service = Arc::new(UserOnboardingService::new(
            invite_service.clone(),
            Arc::new(UserAdminRepositoryImpl::new(database.clone())),
        ));

        let audiobookshelf_book_repository =
            Arc::new(BookRepositoryImpl::new(database.clone()));
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
        let audiobookshelf_media_progress_service = Arc::new(
            AudiobookshelfMediaProgressService::new(Arc::new(MediaProgressRepositoryImpl::new(
                database.clone(),
            ))),
        );
        let audiobookshelf_playback_session_service = Arc::new(
            AudiobookshelfPlaybackSessionService::new(Arc::new(
                PlaybackSessionRepositoryImpl::new(database.clone()),
            )),
        );
        let audiobookshelf_listening_session_service = Arc::new(
            AudiobookshelfListeningSessionService::new(Arc::new(
                ListeningSessionRepositoryImpl::new(database),
            )),
        );
        let audiobookshelf_login_service = Arc::new(AudiobookshelfLoginService::new(
            user_auth_service.clone(),
            user_admin_service.clone(),
        ));

        Self {
            agent_dispatcher,
            agent_registry,
            audiobookshelf_book_service,
            audiobookshelf_library_service,
            audiobookshelf_listening_session_service,
            audiobookshelf_login_service,
            audiobookshelf_media_progress_service,
            audiobookshelf_playback_session_service,
            audiobookshelf_scanner,
            cast_orchestrator,
            device_service,
            device_sync_group_service,
            environment,
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
            stats_service,
            subscription_service,
            tag_service,
            user_admin_service,
            user_auth_service,
            user_onboarding_service,
            watchtime_service,
        }
    }
}
