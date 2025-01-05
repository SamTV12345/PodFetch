import { FC } from 'react'
import { Waypoint } from 'react-waypoint'
import axios, { AxiosResponse } from 'axios'
import useCommon from '../store/CommonSlice'
import { TimelineHATEOASModel, TimeLineModel } from '../models/TimeLineModel'
import { EpisodeCard } from './EpisodeCard'
import {Episode} from "../models/Episode";

type TimelineEpisodeProps = {
    podcastEpisode: TimeLineModel,
    index: number,
    timelineLength: number,
    totalLength: number,
    timeLineEpisodes: TimelineHATEOASModel,
    notListened: boolean,
    podcastHistoryItem?: Episode,
    favoredEpisodes: boolean
}

export const TimelineEpisode: FC<TimelineEpisodeProps> = ({ podcastEpisode,podcastHistoryItem, notListened, index, timeLineEpisodes, favoredEpisodes }) => {
    const addTimelineEpisodes = useCommon(state => state.addTimelineEpisodes)

    return (
        <>
            <EpisodeCard watchedTime={podcastHistoryItem?.position} totalTime={podcastEpisode?.podcast_episode.total_time} podcast={podcastEpisode.podcast} podcastEpisode={podcastEpisode.podcast_episode} />

            {/*Infinite scroll */
            timeLineEpisodes.data.length === index + 1 &&
                <Waypoint key={index + 'waypoint'} onEnter={() => {
                    axios.get('/podcasts/timeline', {
                        params:{
                            lastTimestamp: podcastEpisode.podcast_episode.date_of_recording,
                            favoredOnly: useCommon.getState().filters?.onlyFavored,
                            notListened: notListened,
                            favoredEpisodes
                        }
                    })
                        .then((response: AxiosResponse<TimelineHATEOASModel>) => {
                            addTimelineEpisodes(response.data)
                        })
                }} />
            }
        </>
    )
}
