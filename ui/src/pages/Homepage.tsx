import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { PodcastWatchedEpisodeModel } from '../models/PodcastWatchedEpisodeModel'
import { EpisodeCard } from '../components/EpisodeCard'
import { Heading2 } from '../components/Heading2'
import {PodcastEpisodeAlreadyPlayed} from "../components/PodcastEpisodeAlreadyPlayed";
import {client} from "../utils/http";
import {components} from "../../schema";

export const Homepage = () => {
    const [podcastWatched, setPodcastWatched] = useState<components["schemas"]["PodcastWatchedEpisodeModelWithPodcastEpisode"][]>([])
    const [latestEpisodes, setLatestEpisodes] = useState<components["schemas"]["TimeLinePodcastItem"]["data"]>([])
    const { t } = useTranslation()

    useEffect(()=>{
        client.GET("/api/v1/podcasts/episode/lastwatched")
            .then(response=>setPodcastWatched(response.data!))
        client.GET("/api/v1/podcasts/timeline", {
            params:{
                query: {
                    favoredOnly: false,
                    notListened: false,
                    favoredEpisodes: false
                }
            },

        })
            .then(response=>setLatestEpisodes(response.data!.data))
    }, [])

    return (
        <>
            <PodcastEpisodeAlreadyPlayed/>
            <div className="mb-8">
                <Heading2 className="mb-2">{t('last-listened')}</Heading2>

                <div className={`
                    scrollbox-x
                    pb-4 pt-8
                    w-[calc(100vw-2rem)] ${/* viewport - padding */ ''}
                    xs:w-[calc(100vw-4rem)] ${/* viewport - padding */ ''}
                    md:w-[calc(100vw-18rem-4rem)] ${/* viewport - sidebar - padding */ ''}
                `}>
                    <div className="flex gap-8">
                        {podcastWatched.map((v)=>{
                            return (
                                <div className="basis-40 shrink-0 whitespace-normal" key={v.episodeId}>
                                    <EpisodeCard podcast={v.podcast} podcastEpisode={v.podcastEpisode} watchedTime={v.watchedTime} totalTime={v.totalTime} />
                                </div>
                            )
                        })}
                    </div>
                </div>
            </div>
            <div>
                <div className="flex items-center gap-4 mb-2">
                    <Heading2>{t('latest-episodes')}</Heading2>
                    <Link className="text-sm text-(--accent-color) hover:text-(--accent-color-hover)" to="/timeline">{t('view-more')}</Link>
                </div>

                <div className={`
                    scrollbox-x
                    pb-4 pt-8
                    w-[calc(100vw-2rem)] ${/* viewport - padding */ ''}
                    xs:w-[calc(100vw-4rem)] ${/* viewport - padding */ ''}
                    md:w-[calc(100vw-18rem-4rem)] ${/* viewport - sidebar - padding */ ''}
                `}>
                    <div className="flex gap-8">
                        {latestEpisodes.map((episode)=>{
                            return (
                                <div className="basis-40 shrink-0 whitespace-normal" key={episode.podcast_episode.episode_id}>
                                    <EpisodeCard podcast={episode.podcast} podcastEpisode={episode.podcast_episode} />
                                </div>
                            )
                        })}
                    </div>
                </div>
            </div>
        </>
    )
}
