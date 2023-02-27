import {Podcast, PodcastEpisode} from "../store/CommonSlice";
import {FC} from "react";

type PreviewPlayerProps = {
    podcastEpisode?: PodcastEpisode,
    podcast?: Podcast|undefined
}

export const PreviewPlayer:FC<PreviewPlayerProps> = ({podcastEpisode, podcast}) => {
    return <div className="grid place-items-start ml-5 mb-2">
        <div className="grid grid-cols-[auto_1fr] gap-2">
            <img src={podcastEpisode?.local_image_url} alt="" className="w-10 h-10"/>
            <div>
                <div className="text-white">{podcastEpisode?.name}</div>
                <div className="text-white text-sm">{podcast?.name}</div>
            </div>
        </div>
    </div>
}
