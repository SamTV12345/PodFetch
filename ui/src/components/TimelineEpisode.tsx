import { FC } from 'react'
import { Waypoint } from 'react-waypoint'
import useCommon from '../store/CommonSlice'
import { EpisodeCard } from './EpisodeCard'
import {components} from "../../schema";
import {client} from "../utils/http";

type TimelineEpisodeProps = {
    podcastEpisode: components["schemas"]["PodcastEpisodeDto"],
    index: number,
    timelineLength: number,
    totalLength: number,
    timeLineEpisodes: components["schemas"]["TimeLinePodcastItem"],
    notListened: boolean,
    podcastHistoryItem?: components["schemas"]["Episode"],
    favoredEpisodes: boolean,
    podcast: components["schemas"]["PodcastDto"]
}

export const TimelineEpisode: FC<TimelineEpisodeProps> = ({ podcastEpisode,podcastHistoryItem, notListened, index, timeLineEpisodes, favoredEpisodes, podcast }) => {
    const addTimelineEpisodes = useCommon(state => state.addTimelineEpisodes)

    return (
        <>
            <EpisodeCard watchedTime={podcastHistoryItem?.position!} totalTime={podcastEpisode?.total_time} podcast={podcast} podcastEpisode={podcastEpisode} />

            {/*Infinite scroll */
            timeLineEpisodes.data.length === index + 1 &&
                <Waypoint key={index + 'waypoint'} onEnter={() => {
                    client.GET("/api/v1/podcasts/timeline", {
                        params: {
                            query: {
                                lastTimestamp: podcastEpisode.date_of_recording,
                                favoredOnly: useCommon.getState().filters?.onlyFavored!,
                                notListened: notListened,
                                favoredEpisodes
                            }
                        }
                    }).then((resp)=>addTimelineEpisodes(resp.data!))
                }} />
            }
        </>
    )
}
