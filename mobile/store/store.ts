import { create } from 'zustand'
import {components} from "@/schema";

type ZustandStore = {
    podcastEpisodeRecord: components["schemas"]["PodcastWatchedEpisodeModelWithPodcastEpisode"]|undefined
}

export const useStore = create<ZustandStore>((set, get)=>({
    podcastEpisodeRecord: undefined
}))
