import {useTranslation} from "react-i18next";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {useEffect, useMemo} from "react";
import axios, {AxiosResponse} from "axios";
import {apiURL, formatTime} from "../utils/Utilities";
import {PodcastEpisode, setTimeLineEpisodes} from "../store/CommonSlice";
import {PodcastEpisodeTimeLine} from "../components/PodcastEpisodeTimeLine";
import {TimeLineModel} from "../models/TimeLineModel";

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

export const Timeline = ()=>{
    const {t} = useTranslation()
    const dispatch = useAppDispatch()
    const timeLineEpisodes = useAppSelector(state=>state.common.timeLineEpisodes)
    const mappedEpisodes = useMemo(()=>convertToTimeLineEpisodes(timeLineEpisodes),[timeLineEpisodes])

    useEffect(()=>{
        axios.get(apiURL+"/podcasts/timeline")
            .then((c:AxiosResponse<TimeLineModel[]>)=>{
                dispatch(setTimeLineEpisodes(c.data))
            })
        },
        [])

    console.log(convertToTimeLineEpisodes(timeLineEpisodes))

    return <div className="p-3">
        <h1 className="font-bold text-3xl">{t('timeline')}</h1>
        <div className="">{
            Object.keys(mappedEpisodes).map((e)=> {
                // @ts-ignore
                let episodesOnDate = mappedEpisodes[e] as TimeLineModel[]
                return <div key={e} className="bg-gray-800 mb-5 p-3 rounded"><h2 className="text-xl text-white">{formatTime(e)}</h2><div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 xl:grid-cols-4 gap-4">{episodesOnDate.map((v)=><PodcastEpisodeTimeLine podcastEpisode={v} key={v.podcast_episode.episode_id+"Parent"}/>)}</div></div>
                }
            )

        }</div>
    </div>
}
