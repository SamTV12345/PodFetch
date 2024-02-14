import {Heading2} from "../components/Heading2";
import {PodcastDetailItem} from "../components/PodcastDetailItem";
import {useTranslation} from "react-i18next";
import {useParams} from "react-router-dom";
import axios, {AxiosResponse} from "axios";
import {prepareOnlinePodcastEpisode, preparePodcastEpisode} from "../utils/Utilities";
import {PlaylistDto} from "../models/Playlist";
import usePlaylist from "../store/PlaylistSlice";
import {useEffect} from "react";
import useAudioPlayer from "../store/AudioPlayerSlice";
import {PodcastInfoModal} from "../components/PodcastInfoModal";
import {PodcastEpisodeAlreadyPlayed} from "../components/PodcastEpisodeAlreadyPlayed";
import {Episode} from "../models/Episode";

export const PlaylistDetailPage = ()=>{
    const {t} = useTranslation()
    const params = useParams()
    const selectedPlaylist = usePlaylist(state=>state.selectedPlaylist)
    const metadata = useAudioPlayer(state=>state.metadata)
    const setCurrentPodcastEpisode = useAudioPlayer(state=>state.setCurrentPodcastEpisode)
    let current_podcast_episode = useAudioPlayer(state=>state.currentPodcastEpisode)
    const setPlaying = useAudioPlayer(state=>state.setPlaying)
    const setSelectedPlaylist = usePlaylist(state=>state.setSelectedPlaylist)

    useEffect(() => {
        if(metadata){
            if(metadata.percentage>99){
                axios.delete("/playlist/"+params.id+"/episode/"+current_podcast_episode!.id)
                    .then(()=>{
                        const currentIndex = selectedPlaylist!.items.findIndex(i=>i.podcastEpisode.id===current_podcast_episode!.id)
                        if(currentIndex === selectedPlaylist!.items.length-1){
                            return
                        }
                        const nextEpisode = selectedPlaylist!.items[currentIndex+1]
                        axios.get("/podcast/episode/" + nextEpisode.podcastEpisode.episode_id)
                            .then((response: AxiosResponse<Episode>) => {
                                nextEpisode.podcastEpisode.status === 'D'
                                    ? setCurrentPodcastEpisode(preparePodcastEpisode(nextEpisode.podcastEpisode, response.data))
                                    : setCurrentPodcastEpisode(prepareOnlinePodcastEpisode(nextEpisode.podcastEpisode, response.data))

                               setPlaying(true)
                            })

                   setSelectedPlaylist({
                        id: selectedPlaylist!.id,
                        name: selectedPlaylist!.name,
                        items: selectedPlaylist!.items.filter(i=>i.podcastEpisode.id!==current_podcast_episode!.id)
                    })
                })
            }
        }
    }, [metadata])


    useEffect(()=>{
            axios.get("/playlist/"+params.id)
                .then((response:AxiosResponse<PlaylistDto>)=>{
                setSelectedPlaylist(response.data)
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
