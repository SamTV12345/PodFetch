import { FC } from 'react'
import { Waypoint } from 'react-waypoint'
import axios, { AxiosResponse } from 'axios'
import { store } from '../store/store'
import { useAppDispatch } from '../store/hooks'
import { addTimelineEpisodes } from '../store/CommonSlice'
import { apiURL } from '../utils/Utilities'
import { TimelineHATEOASModel, TimeLineModel } from '../models/TimeLineModel'
import { EpisodeCard } from './EpisodeCard'
import {PodcastWatchedModel} from "../models/PodcastWatchedModel";

type TimelineEpisodeProps = {
    podcastEpisode: TimeLineModel,
    index: number,
    timelineLength: number,
    totalLength: number,
    timeLineEpisodes: TimelineHATEOASModel,
    notListened: boolean,
    podcastHistoryItem?: PodcastWatchedModel
}

export const TimelineEpisode: FC<TimelineEpisodeProps> = ({ podcastEpisode,podcastHistoryItem, notListened, index, timelineLength, timeLineEpisodes }) => {
    const dispatch = useAppDispatch()

    return (
        <>
            <EpisodeCard watchedTime={podcastHistoryItem?.watchedTime} totalTime={podcastEpisode?.podcast_episode.total_time} podcast={podcastEpisode.podcast} podcastEpisode={podcastEpisode.podcast_episode} />

            {/*Infinite scroll */
            timeLineEpisodes.data.length === index + 1 &&
                <Waypoint key={index + 'waypoint'} onEnter={() => {
                    axios.get(apiURL + '/podcasts/timeline', {
                        params:{
                            lastTimestamp: podcastEpisode.podcast_episode.date_of_recording,
                            favoredOnly: store.getState().common.filters?.onlyFavored,
                            notListened: notListened
                        }
                    })
                        .then((response: AxiosResponse<TimelineHATEOASModel>) => {
                            dispatch(addTimelineEpisodes(response.data))
                        })
                }} />
            }
        </>
    )
}
