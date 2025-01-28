import {FC} from 'react'
import {handlePlayofEpisode} from "../utils/PlayHandler";
import {client} from "../utils/http";
import {components} from "../../schema";

type EpisodeCardProps = {
    podcast: components["schemas"]["PodcastDto"],
    podcastEpisode: components["schemas"]["PodcastEpisodeDto"],
    totalTime?: number,
    watchedTime?: number
}

export const EpisodeCard: FC<EpisodeCardProps> = ({ podcast, podcastEpisode, totalTime, watchedTime }) => {

    return (
        <div className="group cursor-pointer" key={podcastEpisode.episode_id+"dv"} onClick={()=>{
            client.GET("/api/v1/podcasts/episode/{id}", {
                params: {
                    path:{
                        id: podcastEpisode.episode_id
                    }
                }
            }).then(resp => handlePlayofEpisode(podcastEpisode, resp.data!))
                .catch(() => handlePlayofEpisode(podcastEpisode, undefined))
        }}>

            {/* Thumbnail */}
            <div className="relative aspect-square bg-center bg-cover mb-2 overflow-hidden rounded-xl transition-shadow group-hover:shadow-[0_4px_32px_rgba(0,0,0,0.3)] w-full" key={podcastEpisode.episode_id} style={{backgroundImage: `url("${podcastEpisode.local_image_url}")`}}>
                <div className="absolute inset-0 grid place-items-center bg-[rgba(0,0,0,0.5)] opacity-0 group-hover:opacity-100 transition-opacity">
                    <span className="material-symbols-outlined !text-7xl text-white group-active:scale-90" key={podcastEpisode.episode_id+"icon"}>play_circle</span>
                </div>

                {/* Progress bar */
                totalTime && watchedTime && (
                    <div className="absolute bottom-0 inset-x-0 bg-stone-900">
                        <div className="bg-[--accent-color] h-1.5" style={{width: (watchedTime/totalTime)*100+"%"}}></div>
                    </div>
                )}
            </div>

            {/* Titles */}
            <div>
                <span className="block font-bold leading-[1.2] mb-2 text-sm text-[--fg-color] transition-colors group-hover:text-[--fg-color-hover]">{podcastEpisode.name}</span>
                <span className="block leading-[1.2] text-xs text-[--fg-color]">{podcast.name}</span>
            </div>
        </div>
    )
}
