import {createSlice, PayloadAction} from '@reduxjs/toolkit'
import {GeneralModel} from "../models/PodcastAddModel";
import {Notification} from "../models/Notification";

export type Podcast = {
    directory: string,
    id: number,
    name: string,
    rssfeed: string,
    image_url: string
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
    searchedPodcasts: GeneralModel|undefined,
    notifications: Notification[],
    infoModalPodcast: PodcastEpisode|undefined,
    infoModalPodcastOpen: boolean,
    detailedAudioPlayerOpen: boolean
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
    detailedAudioPlayerOpen: false
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
        setSelectedEpisodes: (state, action:PayloadAction<PodcastEpisode[]>) => {
            state.selectedEpisodes = action.payload
        },
        setSearchedPodcasts: (state, action:PayloadAction<GeneralModel>) => {
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
        }
}})

export const {setSideBarCollapsed, setPodcasts,setSelectedEpisodes, setSearchedPodcasts, setInfoModalDownloaded,
    setNotifications, removeNotification, setInfoModalPodcast, setInfoModalPodcastOpen, setDetailedAudioPlayerOpen} = commonSlice.actions

export default commonSlice.reducer
