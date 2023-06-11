import {useTranslation} from "react-i18next";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {useEffect} from "react";
import axios, {AxiosResponse} from "axios";
import {apiURL, formatTime, getFiltersDefault} from "../utils/Utilities";
import {setFilters, setTimeLineEpisodes} from "../store/CommonSlice";
import {PodcastEpisodeTimeLine} from "../components/PodcastEpisodeTimeLine";
import {TimelineHATEOASModel} from "../models/TimeLineModel";
import {Switcher} from "../components/Switcher";
import {Filter} from "../models/Filter";
import {Loading} from "../components/Loading";

export const Timeline = () => {
    const {t} = useTranslation()
    const dispatch = useAppDispatch()
    const timeLineEpisodes = useAppSelector(state => state.common.timeLineEpisodes)
    const filter = useAppSelector(state => state.common.filters)
    let currentTime = ""

    useEffect(() => {
        !filter && axios.get(apiURL + "/podcasts/filter")
            .then((filterAxiosResponse: AxiosResponse<Filter>) => {
                filterAxiosResponse.data == null && dispatch(setFilters(getFiltersDefault()))

                filterAxiosResponse.data &&dispatch(setFilters(filterAxiosResponse.data))
            })
    }, [])


    useEffect(() => {
        if (filter) {
            let favoredOnly = filter?.onlyFavored

            axios.get(apiURL + "/podcasts/timeline", {
                params: {
                    favoredOnly: favoredOnly === undefined ? false : favoredOnly
                }
            })
                .then((c: AxiosResponse<TimelineHATEOASModel>) => {
                    dispatch(setTimeLineEpisodes(c.data))
                })
        }
    }, [filter])

    if(timeLineEpisodes === undefined){
        return <Loading/>
    }

    return <div className="p-3">
        <div className="grid-cols-1 grid md:grid-cols-[1fr_auto] ">
            <h1 className="font-bold text-3xl">{t('timeline')}</h1>
            <div>
                <div className="grid grid-cols-[1fr_auto] gap-2">
                    <span className="grid">{t('onlyFavored')}</span>
                    <div className="static">
                        <Switcher checked={filter === undefined?true: filter.onlyFavored} setChecked={() => dispatch(setFilters({
                            ...filter as Filter,
                            onlyFavored: !filter?.onlyFavored
                        }))}/>
                    </div>
                </div>
            </div>
        </div>
        <div className="">{
            timeLineEpisodes.data.map((e, index) => {
                if (currentTime.length==0|| e.podcast_episode.date_of_recording.split('T')[0] !== currentTime) {
                    return <div key={e.podcast_episode.episode_id} className="bg-gray-800 mb-5 p-3 rounded"><h2 className="text-xl text-white">
                        {formatTime(e.podcast_episode.date_of_recording)}</h2>
                        <div
                            className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 xl:grid-cols-4 gap-4">
                            <PodcastEpisodeTimeLine podcastEpisode={e} key={e.podcast_episode.episode_id+index + "Parent"}
                                                    index={index} timelineLength={timeLineEpisodes.data.length} timeLineEpisodes={timeLineEpisodes} totalLength={timeLineEpisodes.totalElements}/>
                        </div>
                    </div>
                }
                else {

                    return <PodcastEpisodeTimeLine podcastEpisode={e}
                                                   key={e.podcast_episode.episode_id + index + "Parent"}
                                                   index={index} timelineLength={timeLineEpisodes.data.length}
                                                   timeLineEpisodes={timeLineEpisodes}
                                                   totalLength={timeLineEpisodes.totalElements}/>
                }})}
                </div>
    </div>
}
