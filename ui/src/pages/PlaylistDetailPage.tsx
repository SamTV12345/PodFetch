import {Heading2} from "../components/Heading2";
import {PodcastDetailItem} from "../components/PodcastDetailItem";
import {useTranslation} from "react-i18next";
import {useParams} from "react-router-dom";
import {prepareOnlinePodcastEpisode, preparePodcastEpisode} from "../utils/Utilities";
import usePlaylist from "../store/PlaylistSlice";
import {useEffect} from "react";
import useAudioPlayer from "../store/AudioPlayerSlice";
import {PodcastInfoModal} from "../components/PodcastInfoModal";
import {PodcastEpisodeAlreadyPlayed} from "../components/PodcastEpisodeAlreadyPlayed";
import {client} from "../utils/http";

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
                client.DELETE("/api/v1/playlist/{playlist_id}/episode/{episode_id}", {
                    params: {
                        path: {
                            playlist_id: params.id!,
                            episode_id: current_podcast_episode!.podcastEpisode.id
                        }
                    }
                }).then(()=>{
                    const currentIndex = selectedPlaylist!.items.findIndex(i=>i.podcastEpisode.id===current_podcast_episode!.podcastEpisode.id)
                    if(currentIndex === selectedPlaylist!.items.length-1){
                        return
                    }
                    const nextEpisode = selectedPlaylist!.items[currentIndex+1]!
                    client.GET("/api/v1/podcasts/episode/{id}", {
                        params: {
                            path: {
                                id: nextEpisode.podcastEpisode.episode_id
                            }
                        }
                    })
                        .then((response) => {
                            nextEpisode.podcastEpisode.status
                                ? setCurrentPodcastEpisode(preparePodcastEpisode(nextEpisode.podcastEpisode, response.data! as any))
                                : setCurrentPodcastEpisode(prepareOnlinePodcastEpisode(nextEpisode.podcastEpisode, response.data! as any))

                            setPlaying(true)
                        })

                    setSelectedPlaylist({
                        id: selectedPlaylist!.id,
                        name: selectedPlaylist!.name,
                        items: selectedPlaylist!.items.filter(i=>i.podcastEpisode.id!==current_podcast_episode!.podcastEpisode.id)
                    })
                })

            }
        }
    }, [metadata])


    useEffect(()=>{
        client.GET("/api/v1/playlist/{playlist_id}", {
            params: {
                path: {
                    playlist_id: String(params.id)
                }
            }
        }).then((response)=>{
            setSelectedPlaylist(response.data!)
        })
    },[])

    return selectedPlaylist&&<div>
        <Heading2 className="mb-8">{t('available-episodes')}</Heading2>
        <PodcastInfoModal/>
        <PodcastEpisodeAlreadyPlayed/>
        {selectedPlaylist.items.map((episode, index) => {
            return <PodcastDetailItem onlyUnplayed={false} episode={episode} key={episode.podcastEpisode.id} index={index} episodesLength={selectedPlaylist.items.length}/>
        })}
    </div>
}
