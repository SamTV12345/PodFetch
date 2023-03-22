import {Podcast, PodcastEpisode, setDetailedAudioPlayerOpen} from "../store/CommonSlice";
import {FC} from "react";
import {useAppDispatch} from "../store/hooks";

type PreviewPlayerProps = {
    podcastEpisode?: PodcastEpisode,
    podcast?: Podcast|undefined
}

export const PreviewPlayer:FC<PreviewPlayerProps> = ({podcastEpisode, podcast}) => {
    const dispatch = useAppDispatch()

    return <div className="grid place-items-start ml-5 mb-2">
        <div className="grid grid-cols-[auto_1fr] gap-2">
            <div className="relative">
                <img src={podcastEpisode?.local_image_url} alt="" className="w-10 h-10"/>
                <div className="absolute left-0 top-0 w-full h-full hover:bg-gray-500 opacity-80 z-10 grid place-items-center play-button-background" onClick={()=>{dispatch(setDetailedAudioPlayerOpen(true))}}>
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-6 h-6 text-white opacity-0 hover:opacity-100">
                        <path strokeLinecap="round" strokeLinejoin="round" d="M3.75 3.75v4.5m0-4.5h4.5m-4.5 0L9 9M3.75 20.25v-4.5m0 4.5h4.5m-4.5 0L9 15M20.25 3.75h-4.5m4.5 0v4.5m0-4.5L15 9m5.25 11.25h-4.5m4.5 0v-4.5m0 4.5L15 15" />
                    </svg>
                </div>
                </div>
            <div>
                <div className="text-white">{podcastEpisode?.name}</div>
                <div className="text-white text-sm">{podcast?.name}</div>
            </div>
        </div>
    </div>
}
