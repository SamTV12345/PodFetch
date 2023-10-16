import {AgnosticPodcastDataModel} from "../models/PodcastAddModel";
import {Notification} from "../models/Notification";
import {ConfigModel} from "../models/SysInfo";
import {LoginData} from "../pages/Login";
import {User} from "../models/User";
import {ConfirmModalProps} from "../components/ConfirmModal";
import {Invite} from "../components/UserAdminInvites";
import {TimelineHATEOASModel} from "../models/TimeLineModel";
import {Filter} from "../models/Filter";
import {EpisodesWithOptionalTimeline} from "../models/EpisodesWithOptionalTimeline";
import {PodcastWatchedModel} from "../models/PodcastWatchedModel";
import {create} from "zustand";

export type Podcast = {
    directory: string,
    id: number,
    name: string,
    rssfeed: string,
    image_url: string,
    favorites: boolean,
    summary?: string,
    language?: string,
    explicit?: boolean,
    keywords?: string,
    author?: string,
    last_build_date?: string,
    active:boolean
}

export type PodcastEpisode = {
    id: number,
    podcast_id: number,
    episode_id: string,
    name: string,
    url: string,
    date_of_recording: string,
    image_url: string,
    total_time: number,
    local_url: string,
    local_image_url:string,
    description: string,
    status: "D"|"N"|"P",
    time?: number
}

type PodcastEpisodeWithPodcastWatchModel = {
    podcastEpisode: EpisodesWithOptionalTimeline,
    podcastWatchModel: PodcastWatchedModel
}

// Define a type for the slice state
interface CommonProps {
    selectedEpisodes: EpisodesWithOptionalTimeline[]
    sidebarCollapsed: boolean,
    podcasts:Podcast[],
    searchedPodcasts: AgnosticPodcastDataModel[]|undefined,
    notifications: Notification[],
    infoModalPodcast: PodcastEpisode|undefined,
    infoModalPodcastOpen: boolean,
    podcastAlreadyPlayed: boolean,
    detailedAudioPlayerOpen: boolean,
    configModel: ConfigModel|undefined,
    currentDetailedPodcastId: number|undefined,
    loginData: Partial<LoginData>|undefined,
    confirmModalData: ConfirmModalProps|undefined
    selectedUser: User|undefined,
    users: User[],
    createInviteModalOpen: boolean,
    invites: Invite[],
    timeLineEpisodes: TimelineHATEOASModel|undefined
    filters: Filter|undefined,
    infoHeading: string|undefined,
    infoText: string|undefined,
    podcastEpisodeAlreadyPlayed: PodcastEpisodeWithPodcastWatchModel|undefined,
    setSidebarCollapsed: (sidebarCollapsed: boolean) => void,
    setPodcasts: (podcasts: Podcast[]) => void,
    updateLikePodcast: (id: number) => void,
    setSelectedEpisodes: (selectedEpisodes: EpisodesWithOptionalTimeline[]) => void,
    setSearchedPodcasts: (searchedPodcasts: AgnosticPodcastDataModel[]) => void,
    setNotifications: (notifications: Notification[]) => void,
    removeNotification: (id: number) => void,
    setInfoModalPodcast: (infoModalPodcast: PodcastEpisode) => void,
    setInfoModalPodcastOpen: (infoModalPodcastOpen: boolean) => void,
    setEpisodeDownloaded: (episode_id: string) => void,
    setDetailedAudioPlayerOpen: (detailedAudioPlayerOpen: boolean) => void,
    setConfigModel: (configModel: ConfigModel) => void,
    setCurrentDetailedPodcastId: (currentDetailedPodcastId: number) => void,
    addPodcast: (podcast: Podcast) => void,
    podcastDeleted: (id: number) => void,
    setLoginData: (loginData: Partial<LoginData>) => void,
    setConfirmModalData: (confirmModalData: ConfirmModalProps) => void,
    setSelectedUser: (selectedUser: User) => void,
    setUsers: (users: User[]) => void,
    setCreateInviteModalOpen: (createInviteModalOpen: boolean) => void,
    setInvites: (invites: Invite[]) => void,
    setTimeLineEpisodes: (timeLineEpisodes: TimelineHATEOASModel) => void,
    addTimelineEpisodes: (timeLineEpisodes: TimelineHATEOASModel) => void,
    addPodcastEpisodes: (selectedEpisodes: EpisodesWithOptionalTimeline[]) => void,
    setFilters: (filters: Filter) => void,
    setInfoHeading: (infoHeading: string) => void,
    setInfoText: (infoText: string) => void,
    setPodcastAlreadyPlayed: (podcastAlreadyPlayed: boolean) => void,
    setPodcastEpisodeAlreadyPlayed: (podcastEpisodeAlreadyPlayed: PodcastEpisodeWithPodcastWatchModel) => void
}

const useCommon = create<CommonProps>((set, get) => ({
    selectedEpisodes: [],
    sidebarCollapsed: true,
    podcasts: [],
    searchedPodcasts: undefined,
    notifications: [],
    infoModalPodcast: undefined,
    infoModalPodcastOpen: false,
    detailedAudioPlayerOpen: false,
    configModel: undefined,
    currentDetailedPodcastId: undefined,
    loginData: undefined,
    confirmModalData: undefined,
    selectedUser: undefined,
    users: [],
    createInviteModalOpen: false,
    invites: [],
    timeLineEpisodes:undefined,
    filters: undefined,
    infoHeading: undefined,
    infoText: undefined,
    podcastAlreadyPlayed: false,
    podcastEpisodeAlreadyPlayed: undefined,
    setSidebarCollapsed: (sidebarCollapsed: boolean) => set({sidebarCollapsed}),
    setPodcasts: (podcasts: Podcast[]) => set({podcasts}),
    updateLikePodcast: (id: number) => {
        const podcasts = get().podcasts
        if(podcasts){
            set({podcasts: podcasts.map((podcast) => {
                    if(podcast.id === id) {
                        podcast.favorites = !podcast.favorites
                    }
                    return podcast
                })})
        }
    },
    setSelectedEpisodes: (selectedEpisodes: EpisodesWithOptionalTimeline[]) => set({selectedEpisodes}),
    setSearchedPodcasts: (searchedPodcasts: AgnosticPodcastDataModel[]) => set({searchedPodcasts}),
    setNotifications: (notifications: Notification[]) => set({notifications}),
    removeNotification: (id: number) => {
        const notifications = get().notifications
        if(notifications){
            set({notifications: notifications.filter((notification) => notification.id !== id)})
        }
    },
    setInfoModalPodcast: (infoModalPodcast: PodcastEpisode) => set({infoModalPodcast}),
    setInfoModalPodcastOpen: (infoModalPodcastOpen: boolean) => set({infoModalPodcastOpen}),
    setEpisodeDownloaded: (episode_id: string) => {
        const selectedEpisodes = get().selectedEpisodes
        if(selectedEpisodes){
            set({selectedEpisodes: selectedEpisodes.map((episode) => {
                    if(episode.podcastEpisode.episode_id === episode_id) {
                        episode.podcastEpisode.status = 'D'
                    }
                    return episode
                })})
        }
    },
    setDetailedAudioPlayerOpen: (detailedAudioPlayerOpen: boolean) => set({detailedAudioPlayerOpen}),
    setConfigModel: (configModel: ConfigModel) => set({configModel}),
    setCurrentDetailedPodcastId: (currentDetailedPodcastId: number) => set({currentDetailedPodcastId}),
    addPodcast: (podcast: Podcast) => {
        set({podcasts: [...get().podcasts, podcast]})
    },
    podcastDeleted: (id: number) => {
        set({podcasts: get().podcasts.filter((podcast) => podcast.id !== id)})
    },
    setLoginData: (loginData: Partial<LoginData>) => set({loginData}),
    setConfirmModalData: (confirmModalData: ConfirmModalProps) => set({confirmModalData}),
    setSelectedUser: (selectedUser: User) => set({selectedUser}),
    setUsers: (users: User[]) => set({users}),
    setCreateInviteModalOpen: (createInviteModalOpen: boolean) => set({createInviteModalOpen}),
    setInvites: (invites: Invite[]) => set({invites}),
    setTimeLineEpisodes: (timeLineEpisodes: TimelineHATEOASModel) => set({timeLineEpisodes}),
    addTimelineEpisodes: (timeLineEpisodes: TimelineHATEOASModel) => {
        const currentTimeline = get().timeLineEpisodes
        if(currentTimeline){
            set({timeLineEpisodes: {
                    totalElements: timeLineEpisodes.totalElements,
                    data: [...currentTimeline.data, ...timeLineEpisodes.data]
                }})
        }
        else{
            set({timeLineEpisodes})
        }
    },
    addPodcastEpisodes: (selectedEpisodes: EpisodesWithOptionalTimeline[]) => {
        set({selectedEpisodes: [...get().selectedEpisodes, ...selectedEpisodes]})
    },
    setFilters: (filters: Filter) => set({filters}),
    setInfoHeading: (infoHeading: string) => set({infoHeading}),
    setInfoText: (infoText: string) => set({infoText}),
    setPodcastAlreadyPlayed: (podcastAlreadyPlayed: boolean) => set({podcastAlreadyPlayed}),
    setPodcastEpisodeAlreadyPlayed: (podcastEpisodeAlreadyPlayed: PodcastEpisodeWithPodcastWatchModel) => set({podcastEpisodeAlreadyPlayed})
}))

export default useCommon
