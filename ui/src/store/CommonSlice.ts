import {LoginData} from "../pages/Login";
import {create} from "zustand";
import {components} from "../../schema";
import {AgnosticPodcastDataModel} from "../models/PodcastAddModel";

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
    sidebarCollapsed: boolean,
    searchedPodcasts: AgnosticPodcastDataModel[]|undefined,
    podcastAlreadyPlayed: boolean,
    detailedAudioPlayerOpen: boolean,
    detailedAudioPlayerTab: 'description' | 'chapters' | 'video',
    currentDetailedPodcastId: number|undefined,
    loginData: Partial<LoginData>|undefined,
    users: components["schemas"]["UserWithoutPassword"][],
    invites: components["schemas"]["Invite"][],
    timeLineEpisodes: components["schemas"]["TimeLinePodcastItem"]|undefined
    filters: components["schemas"]["Filter"]|undefined,
    podcastEpisodeAlreadyPlayed: components["schemas"]["PodcastEpisodeWithHistory"]|undefined,
    setSidebarCollapsed: (sidebarCollapsed: boolean) => void,
    tags: components["schemas"]["Tag"][],
    setPodcastTags: (t: components["schemas"]["Tag"][])=>void,
    setSelectedEpisodes: (selectedEpisodes: components["schemas"]["PodcastEpisodeWithHistory"][]) => void,
    setSearchedPodcasts: (searchedPodcasts: AgnosticPodcastDataModel[]) => void,
    setEpisodeDownloaded: (episode_id: string) => void,
    setDetailedAudioPlayerOpen: (detailedAudioPlayerOpen: boolean) => void,
    setDetailedAudioPlayerTab: (tab: 'description' | 'chapters' | 'video') => void,
    setCurrentDetailedPodcastId: (currentDetailedPodcastId: number) => void,
    setLoginData: (loginData: Partial<LoginData>) => void,
    setUsers: (users: components["schemas"]["UserWithoutPassword"][]) => void,
    setInvites: (invites: components["schemas"]["Invite"][]) => void,
    setTimeLineEpisodes: (timeLineEpisodes: components["schemas"]["TimeLinePodcastItem"]) => void,
    addTimelineEpisodes: (timeLineEpisodes: components["schemas"]["TimeLinePodcastItem"]) => void,
    setFilters: (filters: components["schemas"]["Filter"]) => void,
    setPodcastAlreadyPlayed: (podcastAlreadyPlayed: boolean) => void,
    setPodcastEpisodeAlreadyPlayed: (podcastEpisodeAlreadyPlayed: components["schemas"]["PodcastEpisodeWithHistory"]) => void,
    isAuthenticated: boolean
}

const useCommon = create<CommonProps>((set, get) => ({
    selectedEpisodes: [],
    sidebarCollapsed: true,
    podcasts: [],
    searchedPodcasts: undefined,
    detailedAudioPlayerOpen: false,
    detailedAudioPlayerTab: 'description',
    currentDetailedPodcastId: undefined,
    loginData: undefined,
    users: [],
    invites: [],
    timeLineEpisodes:undefined,
    filters: undefined,
    podcastAlreadyPlayed: false,
    podcastEpisodeAlreadyPlayed: undefined,
    setSidebarCollapsed: (sidebarCollapsed: boolean) => set({sidebarCollapsed}),
    setSelectedEpisodes: (selectedEpisodes: components["schemas"]["PodcastEpisodeWithHistory"][]) => set({selectedEpisodes}),
    setSearchedPodcasts: (searchedPodcasts) => set({searchedPodcasts}),
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
    setDetailedAudioPlayerTab: (tab) => set({detailedAudioPlayerTab: tab}),
    setCurrentDetailedPodcastId: (currentDetailedPodcastId: number) => set({currentDetailedPodcastId}),
    setLoginData: (loginData: Partial<LoginData>) => set({loginData}),
    setUsers: (users: components["schemas"]["UserWithoutPassword"][]) => set({users}),
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
    setFilters: (filters: components["schemas"]["Filter"]) => set({filters}),
    setPodcastAlreadyPlayed: (podcastAlreadyPlayed: boolean) => set({podcastAlreadyPlayed}),
    setPodcastEpisodeAlreadyPlayed: (podcastEpisodeAlreadyPlayed) => set({podcastEpisodeAlreadyPlayed}),
    tags: [],
    setPodcastTags: (t)=>set({tags: t}),
    isAuthenticated: false,
}))

export default useCommon
