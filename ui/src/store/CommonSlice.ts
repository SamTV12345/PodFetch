import {LoginData} from "../pages/Login";
import {User} from "../models/User";
import {ConfirmModalProps} from "../components/ConfirmModal";
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

type InfoModalPodcast = components["schemas"]["PodcastEpisodeDto"] & {
    podcast: components['schemas']['PodcastDto']
}

// Define a type for the slice state
interface CommonProps {
    selectedEpisodes: components["schemas"]["PodcastEpisodeWithHistory"][],
    sidebarCollapsed: boolean,
    searchedPodcasts: AgnosticPodcastDataModel[]|undefined,
    infoModalPodcast: components["schemas"]["PodcastEpisodeDto"]|undefined,
    infoModalPodcastOpen: boolean,
    podcastAlreadyPlayed: boolean,
    detailedAudioPlayerOpen: boolean,
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
    tags: components["schemas"]["Tag"][],
    setPodcastTags: (t: components["schemas"]["Tag"][])=>void,
    setSelectedEpisodes: (selectedEpisodes: components["schemas"]["PodcastEpisodeWithHistory"][]) => void,
    setSearchedPodcasts: (searchedPodcasts: AgnosticPodcastDataModel[]) => void,
    setInfoModalPodcast: (infoModalPodcast: components["schemas"]["PodcastEpisodeDto"]) => void,
    setInfoModalPodcastOpen: (infoModalPodcastOpen: boolean) => void,
    setEpisodeDownloaded: (episode_id: string) => void,
    setDetailedAudioPlayerOpen: (detailedAudioPlayerOpen: boolean) => void,
    setCurrentDetailedPodcastId: (currentDetailedPodcastId: number) => void,
    setLoginData: (loginData: Partial<LoginData>) => void,
    setConfirmModalData: (confirmModalData: ConfirmModalProps) => void,
    setSelectedUser: (selectedUser: User) => void,
    setUsers: (users: components["schemas"]["UserWithoutPassword"][]) => void,
    setCreateInviteModalOpen: (createInviteModalOpen: boolean) => void,
    setInvites: (invites: components["schemas"]["Invite"][]) => void,
    setTimeLineEpisodes: (timeLineEpisodes: components["schemas"]["TimeLinePodcastItem"]) => void,
    addTimelineEpisodes: (timeLineEpisodes: components["schemas"]["TimeLinePodcastItem"]) => void,
    setFilters: (filters: components["schemas"]["Filter"]) => void,
    setInfoHeading: (infoHeading: string) => void,
    setInfoText: (infoText: string) => void,
    setPodcastAlreadyPlayed: (podcastAlreadyPlayed: boolean) => void,
    setPodcastEpisodeAlreadyPlayed: (podcastEpisodeAlreadyPlayed: components["schemas"]["PodcastEpisodeWithHistory"]) => void,
    isAuthenticated: boolean
}

const useCommon = create<CommonProps>((set, get) => ({
    selectedEpisodes: [],
    sidebarCollapsed: true,
    podcasts: [],
    searchedPodcasts: undefined,
    infoModalPodcast: undefined,
    infoModalPodcastOpen: false,
    detailedAudioPlayerOpen: false,
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
    setSelectedEpisodes: (selectedEpisodes: components["schemas"]["PodcastEpisodeWithHistory"][]) => set({selectedEpisodes}),
    setSearchedPodcasts: (searchedPodcasts) => set({searchedPodcasts}),
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
    setCurrentDetailedPodcastId: (currentDetailedPodcastId: number) => set({currentDetailedPodcastId}),
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
    setFilters: (filters: components["schemas"]["Filter"]) => set({filters}),
    setInfoHeading: (infoHeading: string) => set({infoHeading}),
    setInfoText: (infoText: string) => set({infoText}),
    setPodcastAlreadyPlayed: (podcastAlreadyPlayed: boolean) => set({podcastAlreadyPlayed}),
    setPodcastEpisodeAlreadyPlayed: (podcastEpisodeAlreadyPlayed) => set({podcastEpisodeAlreadyPlayed}),
    tags: [],
    setPodcastTags: (t)=>set({tags: t}),
    isAuthenticated: false,
}))

export default useCommon
