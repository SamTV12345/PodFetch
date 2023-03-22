import {
    setCurrentPodcastEpisode,
    setCurrentTimeUpdate,
    setMetadata} from "../store/AudioPlayerSlice";
import {FC, RefObject, useEffect} from "react";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import useOnMount from "../hooks/useOnMount";
import {AudioAmplifier} from "../models/AudioAmplifier";
import axios, {AxiosResponse} from "axios";
import {apiURL} from "../utils/Utilities";
import {PodcastWatchedModel} from "../models/PodcastWatchedModel";
import {store} from "../store/store";

type HiddenAudioPlayerProps = {
    refItem: RefObject<HTMLAudioElement>,
    setAudioAmplifier: (audioAmplifier: AudioAmplifier)=>void
}

export const HiddenAudioPlayer:FC<HiddenAudioPlayerProps> = ({refItem, setAudioAmplifier}) => {
    const dispatch = useAppDispatch()
    const podcastEpisode = useAppSelector(state=>state.audioPlayer.currentPodcastEpisode)

    useEffect(
        ()=>{
            if(podcastEpisode && refItem && refItem.current){
                console.log("Switched to episode: "+podcastEpisode.local_url+ " at time: "+podcastEpisode.time)
                refItem.current.load()
                if(podcastEpisode.time===undefined){
                    //fetch time from server
                    axios.get(apiURL+"/podcast/episode/"+podcastEpisode.episode_id)
                        .then((response: AxiosResponse<PodcastWatchedModel>)=>{
                            store.dispatch(setCurrentPodcastEpisode({
                                ...podcastEpisode,
                                time: response.data.watchedTime
                            }))
                            refItem.current!.currentTime = podcastEpisode.time
                        })
                }
                else{
                    refItem.current!.currentTime = podcastEpisode.time
                }
                refItem.current.play()
            }
        }
        ,[podcastEpisode])

    useOnMount(()=>{
        setAudioAmplifier(new AudioAmplifier(refItem.current!))
    })

    return <audio crossOrigin={"anonymous"} ref={refItem} src={podcastEpisode?.local_url} id={'hiddenaudio'} onTimeUpdate={(e)=>{
        dispatch(setCurrentTimeUpdate(e.currentTarget.currentTime))
    }} onLoadedMetadata={(e)=>{
        dispatch(setMetadata({
            currentTime: e.currentTarget.currentTime,
            duration: e.currentTarget.duration,
            percentage: 0
        }))
    }}/>
}
