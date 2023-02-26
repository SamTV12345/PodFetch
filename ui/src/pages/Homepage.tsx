import {useEffect} from "react";
import axios, {AxiosResponse} from "axios";
import {apiURL} from "../utils/Utilities";
import {PodcastWatchedEpisodeModel} from "../models/PodcastWatchedEpisodeModel";

export const Homepage = () => {
    useEffect(()=>{
        axios.get(apiURL+"/podcast/episode/lastwatched")
            .then((v:AxiosResponse<PodcastWatchedEpisodeModel>)=>{
                console.log(v.data)
            })
    },[])

    return <div>
        test
    </div>
}
