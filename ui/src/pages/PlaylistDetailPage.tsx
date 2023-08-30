import {Heading2} from "../components/Heading2";
import {PodcastDetailItem} from "../components/PodcastDetailItem";
import {useTranslation} from "react-i18next";
import {useParams} from "react-router-dom";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import axios, {AxiosResponse} from "axios";
import {apiURL, prepareOnlinePodcastEpisode, preparePodcastEpisode} from "../utils/Utilities";
import {PlaylistDto} from "../models/Playlist";
import {setSelectedPlaylist} from "../store/PlaylistSlice";
import {useEffect} from "react";
import {setCurrentPodcastEpisode, setPlaying} from "../store/AudioPlayerSlice";
import {PodcastWatchedModel} from "../models/PodcastWatchedModel";
import {store} from "../store/store";
import {PodcastInfoModal} from "../components/PodcastInfoModal";
import {PodcastEpisodeAlreadyPlayed} from "../components/PodcastEpisodeAlreadyPlayed";

export const PlaylistDetailPage = ()=>{
    const {t} = useTranslation()
    const params = useParams()
    const selectedPlaylist = useAppSelector(state=>state.playlist.selectedPlaylist)
    const dispatch = useAppDispatch()
    const metadata = useAppSelector(state=>state.audioPlayer.metadata)
    let current_podcast_episode = useAppSelector(state=>state.audioPlayer.currentPodcastEpisode)

    useEffect(() => {
        if(metadata){
            if(metadata.percentage>99){
                axios.delete(apiURL+"/playlist/"+params.id+"/episode/"+current_podcast_episode!.id)
                    .then(()=>{
                        const currentIndex = selectedPlaylist!.items.findIndex(i=>i.podcastEpisode.id===current_podcast_episode!.id)
                        if(currentIndex === selectedPlaylist!.items.length-1){
                            return
                        }
                        const nextEpisode = selectedPlaylist!.items[currentIndex+1]
                        axios.get(apiURL + "/podcast/episode/" + nextEpisode.podcastEpisode.episode_id)
                            .then((response: AxiosResponse<PodcastWatchedModel>) => {
                                nextEpisode.podcastEpisode.status === 'D'
                                    ? store.dispatch(setCurrentPodcastEpisode(preparePodcastEpisode(nextEpisode.podcastEpisode, response.data)))
                                    : store.dispatch(setCurrentPodcastEpisode(prepareOnlinePodcastEpisode(nextEpisode.podcastEpisode, response.data)))

                                dispatch(setPlaying(true))
                            })

                    dispatch(setSelectedPlaylist({
                        id: selectedPlaylist!.id,
                        name: selectedPlaylist!.name,
                        items: selectedPlaylist!.items.filter(i=>i.podcastEpisode.id!==current_podcast_episode!.id)
                    }))
                })
            }
        }
    }, [metadata])


    useEffect(()=>{
            axios.get(apiURL+"/playlist/"+params.id)
                .then((response:AxiosResponse<PlaylistDto>)=>{
                dispatch(setSelectedPlaylist(response.data))
        })
    },[])

    return selectedPlaylist&&<div>
        <Heading2 className="mb-8">{t('available-episodes')}</Heading2>
        <PodcastInfoModal/>
        <PodcastEpisodeAlreadyPlayed/>
        {selectedPlaylist.items.map((episode, index) => {
            return <PodcastDetailItem episode={episode} key={episode.podcastEpisode.id} index={index} episodesLength={selectedPlaylist.items.length}/>
        })}
    </div>
}
