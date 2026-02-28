import { FC } from 'react'
import useCommon, { Podcast, PodcastEpisode} from '../store/CommonSlice'
import 'material-symbols/outlined.css'
import {components} from "../../schema";

type PlayerEpisodeInfoProps = {
    podcastEpisode?: components["schemas"]["PodcastEpisodeDto"],
    podcast?: components["schemas"]["PodcastDto"] | undefined
}

export const PlayerEpisodeInfo: FC<PlayerEpisodeInfoProps> = ({ podcastEpisode, podcast }) => {
    const setDetailedAudioPlayerOpen = useCommon(state => state.setDetailedAudioPlayerOpen)

    return (
        <div className="flex items-center gap-3">
            {/* Thumbnail */}
            <div className="hidden md:block relative aspect-square bg-center bg-cover h-full rounded-md" style={{backgroundImage: `url("${podcastEpisode?.local_image_url}")`}}>
                <div className="absolute inset-0 grid place-items-center bg-[rgba(0,0,0,0.5)] cursor-pointer opacity-0 hover:opacity-100 rounded-md transition-opacity"
                     onClick={()=>{setDetailedAudioPlayerOpen(true)}}>
                    <span className="material-symbols-outlined text-white">open_in_full</span>
                </div>
            </div>

            {/* Titles */}
            <div className="text-center xs:text-left">
                <span className="line-clamp-3 md:line-clamp-2 font-bold leading-tight text-sm ui-text">{podcastEpisode?.name}</span>
                <span className="hidden md:inline text-xs ui-text">{podcast?.name}</span>
            </div>
        </div>
    )
}
