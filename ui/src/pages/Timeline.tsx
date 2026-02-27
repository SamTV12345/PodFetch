import {Fragment, useEffect, useState} from 'react'
import { useTranslation } from 'react-i18next'
import { formatTime, getFiltersDefault } from '../utils/Utilities'
import useCommon from '../store/CommonSlice'
import { Filter } from '../models/Filter'
import { Heading1 } from '../components/Heading1'
import { Loading } from '../components/Loading'
import { Switcher } from '../components/Switcher'
import { TimelineEpisode } from '../components/TimelineEpisode'
import {$api} from "../utils/http";
import {useQuery, useQueryClient} from "@tanstack/react-query";

export const Timeline = () => {
    const filter = $api.useQuery('get', '/api/v1/podcasts/filter')
    const [notListened, setNotListened] = useState(false)
    const [notFavored, setFavored] = useState(false)
    const queryClient = useQueryClient()
    const timeline = $api.useQuery('get', '/api/v1/podcasts/timeline', {
        params: {
            query: {
                favoredOnly: filter.data === undefined ? false : filter.data.onlyFavored,
                notListened: notListened,
                favoredEpisodes: notFavored
            }
        }
    })

    const { t } = useTranslation()

    return (
        <div>
            {/* Title and toggle */}
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-10">
                <Heading1>{t('timeline')}</Heading1>

                <div className="flex flex-row gap-5">
                    <div className="flex items-center gap-3">
                        <span className="text-xs text-(--fg-secondary-color)">{t('onlyFavored')}</span>

                        <Switcher loading={filter.isLoading} checked={filter.data?.onlyFavored}
                                  onChange={() => {
                                      queryClient.setQueryData(['get', '/api/v1/podcasts/filter'], (oldData: Filter) => ({
                                            ...oldData,
                                            onlyFavored: !oldData.onlyFavored
                                      }))}}/>
                    </div>
                    <div className="flex items-center gap-3">
                        <span className="text-xs text-(--fg-secondary-color)">{t('not-yet-played')}</span>

                        <Switcher checked={notListened} onChange={() => setNotListened(!notListened)}/>
                    </div>
                    <div className="flex items-center gap-3">
                        <span className="text-xs text-(--fg-secondary-color)">{t('onlyFavoredEpisodes')}</span>

                        <Switcher checked={notFavored} onChange={() => setFavored(!notFavored)}/>
                    </div>
                </div>
            </div>

            <div
                className="relative grid grid-cols-1 xs:grid-cols-2 sm:grid-cols-3 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 gap-x-8 gap-y-12 pl-6">
                {(timeline.isLoading || !timeline.data)? <></> : timeline.data.data.map((e, index) => (
                    <Fragment key={e.podcast_episode.episode_id+index + "Parent"}>
                        {/* Section start */
                        index === 0 || (formatTime(e.podcast_episode.date_of_recording) !== formatTime(timeline.data.data[index-1]!.podcast_episode.date_of_recording)) ? (<>
                            {/* Date */}
                            <span className="col-span-full bg-(--bg-color) -mb-4 -ml-6 py-2">
                                <span className="inline-block bg-(--accent-color) mr-4 outline outline-2 outline-offset-2 outline-(--accent-color) h-2 w-2 rounded-full"></span>
                                <span className="text-xs text-(--accent-color)">{formatTime(e.podcast_episode.date_of_recording)}</span>
                            </span>

                            {/* Left line */}
                            <div className="absolute h-full bg-(--border-color) ml-[0.1875rem] w-px -z-10"></div>
                        </>) : ''}

                        <TimelineEpisode
                            podcastHistoryItem={e.history!}
                            notListened={notListened}
                            podcastEpisode={e.podcast_episode!}
                            key={e.podcast_episode.episode_id+index + "Parent"}
                            index={index}
                            timelineLength={timeline.data.data.length}
                            timeLineEpisodes={timeline.data}
                            totalLength={timeline.data.totalElements}
                            favoredEpisodes={notFavored}
                            podcast={e.podcast!}
                        />
                    </Fragment>
                ))}
            </div>
        </div>
    )
}
