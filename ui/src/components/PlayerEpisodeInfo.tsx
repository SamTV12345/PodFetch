import {FC} from "react"
import {useAppDispatch} from "../store/hooks"
import {Podcast, PodcastEpisode, setDetailedAudioPlayerOpen} from "../store/CommonSlice"
import "material-symbols/outlined.css"

type PlayerEpisodeInfoProps = {
    podcastEpisode?: PodcastEpisode,
    podcast?: Podcast|undefined
}

export const PlayerEpisodeInfo:FC<PlayerEpisodeInfoProps> = ({podcastEpisode, podcast}) => {
    const dispatch = useAppDispatch()

    return <div className="flex items-center gap-3">
        {/* Thumbnail */}
        <div className="hidden md:block relative aspect-square bg-center bg-cover h-full rounded-md" style={{backgroundImage: `url("${podcastEpisode?.local_image_url}")`}}>
            <div className="absolute inset-0 grid place-items-center bg-[rgba(0,0,0,0.5)] cursor-pointer opacity-0 hover:opacity-100 rounded-md transition-opacity" onClick={()=>{dispatch(setDetailedAudioPlayerOpen(true))}}>
                <span className="material-symbols-outlined text-white">open_in_full</span>
            </div>
        </div>

        {/* Titles */}
        <div className="text-center xs:text-left">
            <span className="line-clamp-3 md:line-clamp-2 font-bold leading-tight text-sm">{podcastEpisode?.name}</span>
            <span className="hidden md:inline text-xs">{podcast?.name}</span>
        </div>
    </div>
}
