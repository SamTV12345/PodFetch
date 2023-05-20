import {useTranslation} from "react-i18next";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {useEffect, useMemo, useState} from "react";
import axios, {AxiosResponse} from "axios";
import {apiURL, formatTime, getFiltersDefault} from "../utils/Utilities";
import {addTimelineEpisodes, setFilters, setTimeLineEpisodes} from "../store/CommonSlice";
import {PodcastEpisodeTimeLine} from "../components/PodcastEpisodeTimeLine";
import {TimeLineModel} from "../models/TimeLineModel";
import {Switcher} from "../components/Switcher";
import {Filter} from "../models/Filter";
import {Loading} from "../components/Loading";
import {Waypoint} from "react-waypoint";
import {store} from "../store/store";

const convertToTimeLineEpisodes = (podcastEpisodes: TimeLineModel[]) => {
    return podcastEpisodes.reduce((groups, game) => {
        const date = game.podcast_episode.date_of_recording.split('T')[0];
        // @ts-ignore
        if (!groups[date]) {
            // @ts-ignore
            groups[date] = [];
        }
        // @ts-ignore
        groups[date].push(game);
        return groups;
    }, {});
}

export const Timeline = () => {
    const {t} = useTranslation()
    const dispatch = useAppDispatch()
    const timeLineEpisodes = useAppSelector(state => state.common.timeLineEpisodes)
    const mappedEpisodes = useMemo(() => convertToTimeLineEpisodes(timeLineEpisodes), [timeLineEpisodes])
    const filter = useAppSelector(state => state.common.filters)
    let seenEpisodes = 0
    let totalEpisodesGrouped = 0

    useEffect(() => {
        !filter && axios.get(apiURL + "/podcasts/filter")
            .then((filterAxiosResponse: AxiosResponse<Filter>) => {
                filterAxiosResponse.data == null && dispatch(setFilters(getFiltersDefault()))

                dispatch(setFilters(filterAxiosResponse.data))
            })
    }, [])

    useEffect(() => {
        if (filter) {
            axios.get(apiURL + "/podcasts/timeline", {
                params: {
                    favoredOnly: filter.onlyFavored
                }
            })
                .then((c: AxiosResponse<TimeLineModel[]>) => {
                    dispatch(setTimeLineEpisodes(c.data))
                })
        }
    }, [filter])

    if (filter == null) {
        return <Loading/>
    }

    return <div className="p-3">
        <div className="grid-cols-1 grid md:grid-cols-[1fr_auto] ">
            <h1 className="font-bold text-3xl">{t('timeline')}</h1>
            <div>
                <div className="grid grid-cols-[1fr_auto] gap-2">
                    <span className="grid">{t('onlyFavored')}</span>
                    <div className="static">
                        <Switcher checked={filter.onlyFavored} setChecked={() => dispatch(setFilters({
                            ...filter as Filter,
                            onlyFavored: !filter?.onlyFavored
                        }))}/>
                    </div>
                </div>
            </div>
        </div>
        <div className="">{
            Object.keys(mappedEpisodes).map((e, index) => {
                    // @ts-ignore
                    let episodesOnDate = mappedEpisodes[e] as TimeLineModel[]
                    seenEpisodes = seenEpisodes + episodesOnDate.length
                    totalEpisodesGrouped = Object.keys(mappedEpisodes).length
                    return <div key={e} className="bg-gray-800 mb-5 p-3 rounded"><h2
                        className="text-xl text-white">{formatTime(e)}</h2>
                        {
                            seenEpisodes == totalEpisodesGrouped&&<Waypoint key={index+"waypoint"} onEnter={()=>{
                                axios.get(apiURL+"/podcasts/timeline", {
                                    params:{
                                        lastTimestamp: episodesOnDate[episodesOnDate.length-1].podcast_episode.date_of_recording,
                                        favoredOnly: store.getState().common.filters?.onlyFavored
                                    }
                                })
                                    .then((response:AxiosResponse<TimeLineModel[]>)=>{
                                        dispatch(addTimelineEpisodes(response.data))
                                    })
                            }
                            }/>
                        }
                        <div
                            className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 xl:grid-cols-4 gap-4">{episodesOnDate.map((v) =>
                            <PodcastEpisodeTimeLine podcastEpisode={v} key={v.podcast_episode.episode_id+index + "Parent"}
                                                    index={index} timelineLength={timeLineEpisodes.length}/>)}
                        </div>
                    </div>
                }
            )

        }</div>
    </div>
}
