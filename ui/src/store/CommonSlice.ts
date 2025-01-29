import {LoginData} from "../pages/Login";
import {User} from "../models/User";
import {ConfirmModalProps} from "../components/ConfirmModal";
import {create} from "zustand";
import {components} from "../../schema";
import {AgnosticPodcastDataModel} from "../models/PodcastAddModel";
import {addHeader} from "../utils/http";
import io, {Socket} from "socket.io-client";
import {ClientToServerEvents, ServerToClientEvents} from "../models/socketioEvents";

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
    active:boolean,
    episode_numbering: boolean,
    tags: components["schemas"]["Tag"][][]
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
    status: boolean,
    time?: number,
    favored?: boolean
}

// Define a type for the slice state
interface CommonProps {
    selectedEpisodes: components["schemas"]["PodcastEpisodeWithHistory"][],
    loggedInUser: components["schemas"]["UserWithAPiKey"]|undefined,
    setLoggedInUser: (loggedInUser: components["schemas"]["UserWithAPiKey"]) => void,
    sidebarCollapsed: boolean,
    podcasts:   components["schemas"]["PodcastDto"][],
    searchedPodcasts: AgnosticPodcastDataModel[]|undefined,
    notifications: components["schemas"]["Notification"][],
    infoModalPodcast: components["schemas"]["PodcastEpisodeDto"]|undefined,
    infoModalPodcastOpen: boolean,
    podcastAlreadyPlayed: boolean,
    detailedAudioPlayerOpen: boolean,
    configModel: components["schemas"]["ConfigModel"]|undefined,
    currentDetailedPodcastId: number|undefined,
    loginData: Partial<LoginData>|undefined,
    confirmModalData: ConfirmModalProps|undefined
    selectedUser: components["schemas"]["UserWithoutPassword"]|undefined,
    users: components["schemas"]["UserWithoutPassword"][],
    createInviteModalOpen: boolean,
    invites: components["schemas"]["Invite"][],
    timeLineEpisodes: components["schemas"]["TimeLinePodcastItem"]|undefined
    filters: components["schemas"]["Filter"]|undefined,
    infoHeading: string|undefined,
    infoText: string|undefined,
    podcastEpisodeAlreadyPlayed: components["schemas"]["PodcastEpisodeWithHistory"]|undefined,
    setSidebarCollapsed: (sidebarCollapsed: boolean) => void,
    setPodcasts: (podcasts: components["schemas"]["PodcastDto"][]) => void,
    tags: components["schemas"]["Tag"][],
    setPodcastTags: (t: components["schemas"]["Tag"][])=>void,
    updateLikePodcast: (id: number) => void,
    setSelectedEpisodes: (selectedEpisodes: components["schemas"]["PodcastEpisodeWithHistory"][]) => void,
    setSearchedPodcasts: (searchedPodcasts: AgnosticPodcastDataModel[]) => void,
    setNotifications: (notifications: components["schemas"]["Notification"][]) => void,
    removeNotification: (id: number) => void,
    setInfoModalPodcast: (infoModalPodcast: components["schemas"]["PodcastEpisodeDto"]) => void,
    setInfoModalPodcastOpen: (infoModalPodcastOpen: boolean) => void,
    setEpisodeDownloaded: (episode_id: string) => void,
    setDetailedAudioPlayerOpen: (detailedAudioPlayerOpen: boolean) => void,
    setConfigModel: (configModel: components["schemas"]["ConfigModel"]) => void,
    setCurrentDetailedPodcastId: (currentDetailedPodcastId: number) => void,
    addPodcast: (podcast: components["schemas"]["PodcastDto"]) => void,
    podcastDeleted: (id: number) => void,
    setLoginData: (loginData: Partial<LoginData>) => void,
    setConfirmModalData: (confirmModalData: ConfirmModalProps) => void,
    setSelectedUser: (selectedUser: User) => void,
    setUsers: (users: components["schemas"]["UserWithoutPassword"][]) => void,
    setCreateInviteModalOpen: (createInviteModalOpen: boolean) => void,
    setInvites: (invites: components["schemas"]["Invite"][]) => void,
    setTimeLineEpisodes: (timeLineEpisodes: components["schemas"]["TimeLinePodcastItem"]) => void,
    addTimelineEpisodes: (timeLineEpisodes: components["schemas"]["TimeLinePodcastItem"]) => void,
    addPodcastEpisodes: (selectedEpisodes: components["schemas"]["PodcastEpisodeWithHistory"][]) => void,
    setFilters: (filters: components["schemas"]["Filter"]) => void,
    setInfoHeading: (infoHeading: string) => void,
    setInfoText: (infoText: string) => void,
    setPodcastAlreadyPlayed: (podcastAlreadyPlayed: boolean) => void,
    setPodcastEpisodeAlreadyPlayed: (podcastEpisodeAlreadyPlayed: components["schemas"]["PodcastEpisodeWithHistory"]) => void,
    updatePodcast: (podcast: components["schemas"]["PodcastDto"]) => void,
    headers: Record<string,string>,
    setHeaders:(headers: Record<string,string>)=>void,
    isAuthenticated: boolean,
    socketIo: Socket<ServerToClientEvents, ClientToServerEvents> | undefined
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
    setPodcasts: (podcasts) => set({podcasts}),
    updatePodcast: (podcast) => {
        const podcasts = get().podcasts
        if(podcasts){
            set({podcasts: podcasts.map((p) => {
                    if(p.id === podcast.id) {
                        return podcast
                    }
                    return p
                })})
        }
    },
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
    setSelectedEpisodes: (selectedEpisodes: components["schemas"]["PodcastEpisodeWithHistory"][]) => set({selectedEpisodes}),
    setSearchedPodcasts: (searchedPodcasts) => set({searchedPodcasts}),
    setNotifications: (notifications) => set({notifications}),
    removeNotification: (id: number) => {
        const notifications = get().notifications
        if(notifications){
            set({notifications: notifications.filter((notification) => notification.id !== id)})
        }
    },
    setInfoModalPodcast: (infoModalPodcast) => set({infoModalPodcast}),
    setInfoModalPodcastOpen: (infoModalPodcastOpen: boolean) => set({infoModalPodcastOpen}),
    setEpisodeDownloaded: (episode_id: string) => {
        const selectedEpisodes = get().selectedEpisodes
        if(selectedEpisodes){
            set({selectedEpisodes: selectedEpisodes.map((episode) => {
                    if(episode.podcastEpisode.episode_id === episode_id) {
                        episode.podcastEpisode.status = true
                    }
                    return episode
                })})
        }
    },
    setDetailedAudioPlayerOpen: (detailedAudioPlayerOpen: boolean) => set({detailedAudioPlayerOpen}),
    setConfigModel: (configModel: components["schemas"]["ConfigModel"]) => {
        set({configModel, socketIo: io(new URL(configModel.wsUrl).host + "/main", {
                transports: ['websocket'],
                path: new URL(configModel.wsUrl).pathname,
                hostname: new URL(configModel.wsUrl).host,
            })})
    },
    setCurrentDetailedPodcastId: (currentDetailedPodcastId: number) => set({currentDetailedPodcastId}),
    addPodcast: (podcast: components["schemas"]["PodcastDto"]) => {
        set({podcasts: [...get().podcasts, podcast]})
    },
    podcastDeleted: (id: number) => {
        set({podcasts: get().podcasts.filter((podcast) => podcast.id !== id)})
    },
    setLoginData: (loginData: Partial<LoginData>) => set({loginData}),
    setConfirmModalData: (confirmModalData: ConfirmModalProps) => set({confirmModalData}),
    setSelectedUser: (selectedUser: User) => set({selectedUser}),
    setUsers: (users: components["schemas"]["UserWithoutPassword"][]) => set({users}),
    setCreateInviteModalOpen: (createInviteModalOpen: boolean) => set({createInviteModalOpen}),
    setInvites: (invites) => set({invites}),
    setTimeLineEpisodes: (timeLineEpisodes: components["schemas"]["TimeLinePodcastItem"]) => set({timeLineEpisodes}),
    addTimelineEpisodes: (timeLineEpisodes: components["schemas"]["TimeLinePodcastItem"]) => {
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
    addPodcastEpisodes: (selectedEpisodes) => {
        set({selectedEpisodes: [...get().selectedEpisodes, ...selectedEpisodes!]})
    },
    setFilters: (filters: components["schemas"]["Filter"]) => set({filters}),
    setInfoHeading: (infoHeading: string) => set({infoHeading}),
    setInfoText: (infoText: string) => set({infoText}),
    setPodcastAlreadyPlayed: (podcastAlreadyPlayed: boolean) => set({podcastAlreadyPlayed}),
    setPodcastEpisodeAlreadyPlayed: (podcastEpisodeAlreadyPlayed) => set({podcastEpisodeAlreadyPlayed}),
    setLoggedInUser: (loggedInUser: components["schemas"]["UserWithoutPassword"]) => set({loggedInUser}),
    loggedInUser: undefined,
    tags: [],
    setPodcastTags: (t)=>set({tags: t}),
    headers:{
        "Content-Type":"application/json"
    },
    isAuthenticated: false,
    setHeaders:(headers)=>{

        set({...get().headers, ...headers})
        if (headers["Authorization"]) {
            addHeader("Authorization", headers["Authorization"])
            set({isAuthenticated: true})

        }
    },
    socketIo: undefined
}))

export default useCommon
