import {createSlice, PayloadAction} from '@reduxjs/toolkit'
import {AgnosticPodcastDataModel} from "../models/PodcastAddModel";
import {Notification} from "../models/Notification";
import {ConfigModel} from "../models/SysInfo";
import {LoginData} from "../pages/Login";
import {User} from "../models/User";
import {ConfirmModalProps} from "../components/ConfirmModal";
import {Invite} from "../components/UserAdminInvites";
import {TimelineHATEOASModel, TimeLineModel} from "../models/TimeLineModel";
import {Filter} from "../models/Filter";
import {EpisodesWithOptionalTimeline} from "../models/EpisodesWithOptionalTimeline";
import {PodcastWatchedModel} from "../models/PodcastWatchedModel";

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
    podcastEpisodeAlreadyPlayed: PodcastEpisodeWithPodcastWatchModel|undefined
}

// Define the initial state using that type
const initialState: CommonProps = {
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
    podcastEpisodeAlreadyPlayed: undefined
}

export const commonSlice = createSlice({
    name: 'commonSlice',
    // `createSlice` will infer the state type from the `initialState` argument
    initialState,
    reducers: {
        setSidebarCollapsed: (state, action) => {
            state.sidebarCollapsed = action.payload
        },
        setPodcasts: (state, action: PayloadAction<Podcast[]>) => {
            state.podcasts = action.payload
        },
        updateLikePodcast:(state, action: PayloadAction<number>)=>{
          state.podcasts = state.podcasts.map((podcast) => {
                if(podcast.id === action.payload) {
                    podcast.favorites = !podcast.favorites
                }
                return podcast
          })
        },
        setSelectedEpisodes: (state, action:PayloadAction<EpisodesWithOptionalTimeline[]>) => {
            state.selectedEpisodes = action.payload
        },
        setSearchedPodcasts: (state, action:PayloadAction<AgnosticPodcastDataModel[]>) => {
            state.searchedPodcasts = action.payload
        },
        setNotifications: (state, action:PayloadAction<Notification[]>) => {
            state.notifications = action.payload
        },
        removeNotification: (state, action:PayloadAction<number>) => {
            state.notifications = state.notifications.filter((notification) => notification.id !== action.payload)
        },
        setInfoModalPodcast: (state, action:PayloadAction<PodcastEpisode>) => {
            state.infoModalPodcast = action.payload
        },
        setInfoModalPodcastOpen: (state, action:PayloadAction<boolean>) => {
            state.infoModalPodcastOpen = action.payload
        },
        setEpisodeDownloaded: (state, action:PayloadAction<string>) => {
            state.selectedEpisodes = state.selectedEpisodes.map((episode) => {
                if(episode.podcastEpisode.episode_id === action.payload) {
                    episode.podcastEpisode.status = 'D'
                }
                return episode
            })
        },
        setDetailedAudioPlayerOpen: (state, action:PayloadAction<boolean>) => {
            state.detailedAudioPlayerOpen = action.payload
        },
        setConfigModel: (state, action:PayloadAction<ConfigModel>) => {
            state.configModel = action.payload
        },
        setCurrentDetailedPodcastId: (state, action:PayloadAction<number>) => {
            state.currentDetailedPodcastId = action.payload
        },
        addPodcast: (state, action:PayloadAction<Podcast>) => {
            state.podcasts = [...state.podcasts, action.payload]
        },
        podcastDeleted: (state, action:PayloadAction<number>) => {
            state.podcasts = state.podcasts.filter((podcast) => podcast.id !== action.payload)
        },
        setLoginData: (state, action:PayloadAction<Partial<LoginData>>) => {
            state.loginData = action.payload
        },
        setConfirmModalData: (state, action:PayloadAction<ConfirmModalProps>) => {
            state.confirmModalData = action.payload
        },
        setSelectedUser: (state, action:PayloadAction<User>) => {
            state.selectedUser = action.payload
        },
        setUsers: (state, action:PayloadAction<User[]>) => {
            state.users = action.payload
        },
        setCreateInviteModalOpen: (state, action:PayloadAction<boolean>) => {
            state.createInviteModalOpen = action.payload
        },
        setInvites: (state, action:PayloadAction<Invite[]>) => {
            state.invites = action.payload
        },
        setTimeLineEpisodes: (state, action:PayloadAction<TimelineHATEOASModel>) => {
            state.timeLineEpisodes = action.payload
        },
        addTimelineEpisodes: (state, action:PayloadAction<TimelineHATEOASModel>) => {
            if (!state.timeLineEpisodes) {
                state.timeLineEpisodes = action.payload
                return
            }
            state.timeLineEpisodes = {
                totalElements: action.payload.totalElements,
                data: [...state.timeLineEpisodes.data, ...action.payload.data]
            }  satisfies TimelineHATEOASModel
            //[...state.timeLineEpisodes, ...action.payload]
        },
        addPodcastEpisodes: (state, action:PayloadAction<EpisodesWithOptionalTimeline[]>) => {
            state.selectedEpisodes = [...state.selectedEpisodes, ...action.payload]
        },
        setFilters: (state, action:PayloadAction<Filter>) => {
            state.filters = action.payload
        },
        setInfoHeading: (state, action:PayloadAction<string>) => {
            state.infoHeading = action.payload
        },
        setInfoText: (state, action:PayloadAction<string>) => {
            state.infoText = action.payload
        },
        setPodcastAlreadyPlayed: (state, action:PayloadAction<boolean>) => {
            state.podcastAlreadyPlayed = action.payload
        },
        setPodcastEpisodeAlreadyPlayed: (state, action:PayloadAction<PodcastEpisodeWithPodcastWatchModel>) => {
            state.podcastEpisodeAlreadyPlayed = action.payload
        }
}})

export const {
    setSidebarCollapsed, setInfoHeading,setInfoText, addTimelineEpisodes, setFilters, addPodcastEpisodes, setTimeLineEpisodes,setInvites, setCreateInviteModalOpen, setUsers, setSelectedUser, setConfirmModalData, setLoginData, addPodcast, podcastDeleted, setCurrentDetailedPodcastId, setConfigModel, setPodcasts,
    setSelectedEpisodes, setSearchedPodcasts, updateLikePodcast, setEpisodeDownloaded, setNotifications,setPodcastEpisodeAlreadyPlayed, removeNotification, setInfoModalPodcast,setPodcastAlreadyPlayed, setInfoModalPodcastOpen, setDetailedAudioPlayerOpen,
} = commonSlice.actions

export default commonSlice.reducer
