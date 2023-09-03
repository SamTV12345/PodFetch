import {Fragment, useEffect, useState} from 'react'
import { useTranslation } from 'react-i18next'
import axios, { AxiosResponse } from 'axios'
import { apiURL, formatTime, getFiltersDefault } from '../utils/Utilities'
import { useAppDispatch, useAppSelector } from '../store/hooks'
import { setFilters, setTimeLineEpisodes } from '../store/CommonSlice'
import { Filter } from '../models/Filter'
import { TimelineHATEOASModel } from '../models/TimeLineModel'
import { Heading1 } from '../components/Heading1'
import { Loading } from '../components/Loading'
import { Switcher } from '../components/Switcher'
import { TimelineEpisode } from '../components/TimelineEpisode'

export const Timeline = () => {
    const dispatch = useAppDispatch()
    const timeLineEpisodes = useAppSelector(state => state.common.timeLineEpisodes)
    const filter = useAppSelector(state => state.common.filters)
    const { t } = useTranslation()
    const [notListened, setNotListened] = useState(false)

    useEffect(() => {
        !filter && axios.get(apiURL + '/podcasts/filter')
            .then((filterAxiosResponse: AxiosResponse<Filter>) => {
                filterAxiosResponse.data == null && dispatch(setFilters(getFiltersDefault()))

                filterAxiosResponse.data && dispatch(setFilters(filterAxiosResponse.data))
            })
    }, [])

    useEffect(() => {
        if (filter) {
            let favoredOnly = filter?.onlyFavored

            axios.get(apiURL + '/podcasts/timeline', {
                params: {
                    favoredOnly: favoredOnly === undefined ? false : favoredOnly,
                    notListened: notListened
                }
            })
                .then((c: AxiosResponse<TimelineHATEOASModel>) => {
                    dispatch(setTimeLineEpisodes(c.data))
                })
        }
    }, [filter,notListened])

    if(timeLineEpisodes === undefined){
        return <Loading/>
    }

    return (
        <div>
            {/* Title and toggle */}
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-10">
                <Heading1>{t('timeline')}</Heading1>

                <div className="flex flex-row gap-5">
                <div className="flex items-center gap-3">
                    <span className="text-xs text-[--fg-secondary-color]">{t('onlyFavored')}</span>

                    <Switcher checked={filter === undefined?true: filter.onlyFavored} setChecked={() => dispatch(setFilters({
                        ...filter as Filter,
                        onlyFavored: !filter?.onlyFavored
                    }))}/>
                </div>
                <div className="flex items-center gap-3">
                    <span className="text-xs text-[--fg-secondary-color]">{t('not-yet-played')}</span>

                    <Switcher checked={notListened} setChecked={() => setNotListened(!notListened)}/>
                </div>
                </div>
            </div>

            <div className="relative grid grid-cols-1 xs:grid-cols-2 sm:grid-cols-3 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 gap-x-8 gap-y-12 pl-6">
                {timeLineEpisodes.data.map((e, index) => (
                    <Fragment key={e.podcast_episode.episode_id+index + "Parent"}>
                        {/* Section start */
                        index === 0 || (formatTime(e.podcast_episode.date_of_recording) !== formatTime(timeLineEpisodes.data[index-1].podcast_episode.date_of_recording)) ? (<>
                            {/* Date */}
                            <span className="col-span-full bg-[--bg-color] -mb-4 -ml-6 py-2">
                                <span className="inline-block bg-[--accent-color] mr-4 outline outline-2 outline-offset-2 outline-[--accent-color] h-2 w-2 rounded-full"></span>
                                <span className="text-xs text-[--accent-color]">{formatTime(e.podcast_episode.date_of_recording)}</span>
                            </span>

                            {/* Left line */}
                            <div className="absolute h-full bg-[--border-color] ml-[0.1875rem] w-px -z-10"></div>
                        </>) : ''}

                        <TimelineEpisode
                            podcastHistoryItem={e.history}
                            notListened={notListened}
                            podcastEpisode={e}
                            key={e.podcast_episode.episode_id+index + "Parent"}
                            index={index}
                            timelineLength={timeLineEpisodes.data.length}
                            timeLineEpisodes={timeLineEpisodes}
                            totalLength={timeLineEpisodes.totalElements}
                        />
                    </Fragment>
                ))}
            </div>
        </div>
    )
}
