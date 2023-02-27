import {useEffect, useState} from "react";
import axios, {AxiosResponse} from "axios";
import {apiURL} from "../utils/Utilities";
import {PodcastWatchedEpisodeModel} from "../models/PodcastWatchedEpisodeModel";

export const Homepage = () => {
    const [podcastWatched, setPodcastWatched] = useState<PodcastWatchedEpisodeModel[]>([])

    useEffect(()=>{
        axios.get(apiURL+"/podcast/episode/lastwatched")
            .then((v:AxiosResponse<PodcastWatchedEpisodeModel[]>)=>{
                setPodcastWatched(v.data)
            })
    },[])

    return <div className="p-3">
        <h1 className="font-bold text-2xl">Zuletzt gehört</h1>
        <div className="grid grid-cols-5 gap-4">
        {
            podcastWatched.map((v)=>{
                return <div key={v.episodeId}
                    className="max-w-sm rounded-lg shadow bg-gray-800 border-gray-700">
                    <div  className={`bg-[${v.imageUrl}] object-cover`}>
                        <div>test</div>
                    </div>
                    <div className="bg-blue-900 h-2" style={{width: (v.watchedTime/v.totalTime)*100+"%"}}></div>
                    <div className="p-5">
                            <h5 className="mb-2 text-2xl font-bold tracking-tight text-gray-900 dark:text-white">{v.name}</h5>
                    </div>
                </div>


                return <div key={v.episodeId}>
                    <img src={v.imageUrl} alt="" className=""/>
                    <span>{v.name}</span>
                </div>
            })
        }
        </div>
    </div>
}
