import {createSlice, PayloadAction} from '@reduxjs/toolkit'
import {AgnosticPodcastDataModel} from "../models/PodcastAddModel";
import {Notification} from "../models/Notification";
import {ConfigModel} from "../models/SysInfo";
import {LoginData} from "../components/LoginComponent";
import {User} from "../models/User";
import {ConfirmModalProps} from "../components/ConfirmModal";
import {Invite} from "../pages/InviteAdministrationUserPage";

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
    time: number,
    local_url: string,
    local_image_url:string,
    description: string,
    status: "D"|"N"|"P"
}

// Define a type for the slice state
interface CommonProps {
    selectedEpisodes: PodcastEpisode[]
    sideBarCollapsed: boolean,
    podcasts:Podcast[],
    searchedPodcasts: AgnosticPodcastDataModel[]|undefined,
    notifications: Notification[],
    infoModalPodcast: PodcastEpisode|undefined,
    infoModalPodcastOpen: boolean,
    detailedAudioPlayerOpen: boolean,
    configModel: ConfigModel|undefined,
    currentDetailedPodcastId: number|undefined,
    loginData: Partial<LoginData>|undefined,
    confirmModalData: ConfirmModalProps|undefined
    selectedUser: User|undefined,
    users: User[],
    addInviteModalOpen: boolean,
    invites: Invite[]
}

// Define the initial state using that type
const initialState: CommonProps = {
    selectedEpisodes: [],
    sideBarCollapsed: false,
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
    addInviteModalOpen: false,
    invites: []
}

export const commonSlice = createSlice({
    name: 'commonSlice',
    // `createSlice` will infer the state type from the `initialState` argument
    initialState,
    reducers: {
        setSideBarCollapsed: (state, action) => {
            state.sideBarCollapsed = action.payload
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
        setSelectedEpisodes: (state, action:PayloadAction<PodcastEpisode[]>) => {
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
        setInfoModalDownloaded: (state, action:PayloadAction<string>) => {
            if(state.infoModalPodcast) {
                state.infoModalPodcast.status = 'D'
                state.selectedEpisodes = state.selectedEpisodes.map((episode) => {
                    if(episode.episode_id === action.payload) {
                        episode.status = 'D'
                    }
                    return episode
                })
            }
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
        setAddInviteModalOpen: (state, action:PayloadAction<boolean>) => {
            state.addInviteModalOpen = action.payload
        },
        setInvites: (state, action:PayloadAction<Invite[]>) => {
            state.invites = action.payload
        }
}})

export const {setSideBarCollapsed,setInvites,setAddInviteModalOpen, setUsers, setSelectedUser, setConfirmModalData,setLoginData, addPodcast, setCurrentDetailedPodcastId, setConfigModel, setPodcasts,setSelectedEpisodes, setSearchedPodcasts,updateLikePodcast, setInfoModalDownloaded,
    setNotifications, removeNotification, setInfoModalPodcast, setInfoModalPodcastOpen, setDetailedAudioPlayerOpen} = commonSlice.actions

export default commonSlice.reducer
