import {Fragment, useEffect, useMemo, useState} from 'react'
import {useParams} from 'react-router-dom'
import {useTranslation} from 'react-i18next'
import {removeHTML} from '../utils/Utilities'
import useCommon, {Podcast} from '../store/CommonSlice'
import useAudioPlayer from '../store/AudioPlayerSlice'
import {Chip} from '../components/Chip'
import {Heading2} from '../components/Heading2'
import {PodcastDetailItem} from '../components/PodcastDetailItem'
import {PodcastInfoModal} from '../components/PodcastInfoModal'
import {Switcher} from '../components/Switcher'
import 'material-symbols/outlined.css'
import {PodcastEpisodeAlreadyPlayed} from "../components/PodcastEpisodeAlreadyPlayed";
import {PodcastSettingsModal} from "../components/PodcastSettingsModal";
import {$api, client} from '../utils/http'
import {EditableHeading} from "../components/EditableHeading";
import {ADMIN_ROLE} from "../models/constants";
import {Loading} from "../components/Loading";
import {useQueryClient} from "@tanstack/react-query";
import {components} from "../../schema";

export const PodcastDetailPage = () => {
    const setCurrentPodcast = useAudioPlayer(state => state.setCurrentPodcast)
    const params = useParams()
    const [lineClamp, setLineClamp] = useState(true)
    const {t} = useTranslation()
    const setInfoModalPodcastOpen = useCommon(state => state.setInfoModalPodcastOpen)
    const queryClient = useQueryClient()

    const [onlyUnplayed, setOnlyUnplayed] = useState<boolean>(false)
    const {data, error, isLoading} = $api.useQuery('get', '/api/v1/users/{username}', {
        params: {
            path: {
                username: 'me'
            }
        },
    })

    const currentDetailedPodcastId = useMemo(()=>{
        if (params && !isNaN(parseFloat(params.id as string))) {
            return params.id ?? ""
        }
        return ""
    }, [params])

    const currentPodcast = $api.useQuery('get', '/api/v1/podcasts/{id}', {
        params: {
            path: {
                id: currentDetailedPodcastId
            }
        }
    })
    const currentPodcastEpisodes = $api.useQuery('get', '/api/v1/podcasts/{id}/episodes', {
        params: {
            query: {
                only_unlistened: onlyUnplayed
            },
            path: {
                id: currentDetailedPodcastId
            }
        }
    })

    const refreshPodcastEpisodes = $api.useMutation('post','/api/v1/podcasts/{id}/refresh')

    useEffect(() => {
        if (params.podcastid) {
            const element = document.getElementById('episode_' + params.podcastid)

            if (element) {
                        element.scrollIntoView({behavior: 'smooth', block: 'start', inline: 'nearest'})
            }
        }
    }, [onlyUnplayed])

    useEffect(() => {
        if (currentPodcast.data?.summary) {
            const summary = document.getElementById('summary')!

            summary.querySelectorAll('a').forEach((a) => {
                a.setAttribute('target', '_blank')
            })
        }
    }, [currentPodcast.data?.summary])

    const isOverflown = (element: string) => {
        let foundElement = document.getElementById(element)

        if (foundElement) {
            return foundElement.scrollHeight > foundElement.clientHeight || foundElement.scrollWidth > foundElement.clientWidth
        }

        return false
    }

    useEffect(() => {
        if (params.podcastid) {
            const element = document.getElementById('episode_' + params.podcastid)

            if (element) {
                element.scrollIntoView({behavior: 'smooth', block: 'start', inline: 'nearest'})
            }
        }
    }, [params])

    useEffect(() => {
        return () => {
            setInfoModalPodcastOpen(false)
        }
    }, []);


    return (
        <Fragment key={'detail'}>
            <div className="max-w-4xl">
                <PodcastInfoModal/>
                <PodcastEpisodeAlreadyPlayed/>

                {/* Header */}
                <div className="
                    flex flex-col
                    xs:grid
                    xs:grid-cols-[auto_1fr_auto] xs:grid-rows-[auto_auto_auto]
                    gap-x-4 gap-y-2 lg:gap-x-8 lg:gap-y-1 items-center mb-8
                ">
                    {/* Thumbnail */}
                    {currentPodcast.data && <img className="
                        order-4
                        xs:col-start-1 xs:col-end-2 row-start-3 row-end-4
                        lg:col-start-1 lg:col-end-2 lg:row-start-2 lg:row-end-4
                        w-full xs:w-24 md:w-32 lg:w-40 rounded-xl
                    " src={currentPodcast.data.image_url} alt=""/>}

                    {/* Title and refresh icon */}
                    <div className="
                        order-2
                        col-start-1 col-end-4 row-start-2 row-end-3
                        sm:col-start-1 sm:col-end-3
                        lg:col-start-2 lg:col-end-3
                        self-start xs:self-end
                    ">

                        {currentPodcast.data && <EditableHeading podcastId={Number(currentDetailedPodcastId)} initialText={currentPodcast.data.name} allowedToEdit={data?.role == "admin"}></EditableHeading>}

                        {currentPodcast.data && data?.role === ADMIN_ROLE && refreshPodcastEpisodes.isPending ? <Loading className="inline-block h-auto w-auto"/>:  <span
                            className="material-symbols-outlined inline cursor-pointer align-middle text-(--fg-icon-color) hover:text-(--fg-icon-color-hover)"
                            onClick={() => {


                                refreshPodcastEpisodes.mutate({
                                    params: {
                                        path: {
                                            id: params.id!
                                        }
                                    }
                                })
                            }}>refresh</span> }
                        <span>
                         {currentPodcast.data && data?.role === ADMIN_ROLE && <PodcastSettingsModal podcast={currentPodcast.data}/>}
                        </span>
                    </div>

                    {/* Metadata */}
                    <div className="
                        order-3
                        col-start-2 col-end-4 row-start-3 row-end-4
                        sm:col-start-2 sm:col-end-3
                        self-start flex flex-col items-start gap-2
                    ">
                        <span className="block text-(--fg-secondary-color)">{currentPodcast.data?.author}</span>

                        <div className="flex gap-2">
                            {currentPodcast.data?.keywords && currentPodcast.data?.keywords?.split(',').map((keyword, index) => (
                                <Chip key={"keyword"+index} index={index}>{keyword}</Chip>
                            ))}
                        </div>

                        <span className="grid grid-cols-2 md:grid-cols-3">
                        <a className="flex gap-4" rel="noopener noreferrer" href={currentPodcast.data?.podfetch_feed}
                           target="_blank">
                            <span
                               className="material-symbols-outlined cursor-pointer text-(--fg-icon-color) hover:text-(--fg-icon-color-hover)">rss_feed</span>
                            <span className="text-(--fg-color)">PodFetch</span>
                        </a>

                        <button className="flex gap-4" rel="noopener noreferrer"
                                onClick={() => window.open(currentPodcast.data?.rssfeed)}>
                            <a className="material-symbols-outlined cursor-pointer text-(--fg-icon-color) hover:text-(--fg-icon-color-hover)"
                               target="_blank" rel="noopener noreferrer" href={currentPodcast.data?.rssfeed}>rss_feed</a>
                            <span className="text-(--fg-color)">{t('original-rss-feed')}</span>
                        </button>
                            <div className="flex gap-4 justify-end">
                                <Switcher checked={onlyUnplayed} onChange={setOnlyUnplayed}/>
                                <span className=" text-(--fg-color) mt-auto">{t('unplayed')}</span>
                            </div>
                        </span>
                    </div>

                    {/* Toggle */}
                    <div className="
                        order-1
                        col-start-1 col-end-4 row-start-1 row-end-2
                        sm:col-start-3 sm:col-end-4 sm:row-start-2 sm:row-end-3
                        justify-self-end self-end sm:self-start
                        flex gap-3 items-center
                    ">
                        <span className="text-xs text-(--fg-secondary-color)">{t('active')}</span>

                        <Switcher checked={currentPodcast.data?.active} onChange={() => {
                            client.PUT("/api/v1/podcasts/{id}/active", {
                                params: {
                                    path: {
                                        id: params.id!
                                    }
                                }
                            }).then(()=>{
                                setCurrentPodcast({...currentPodcast.data!, active: !currentPodcast.data?.active})
                                for (const cache of queryClient.getQueryCache().getAll()) {
                                    if (cache.queryKey[0] === 'get' && (cache.queryKey[1] as string) === '/api/v1/podcasts/{id}' && (cache.queryKey[2] as any).params.path.id == currentDetailedPodcastId) {
                                        queryClient.setQueryData(cache.queryKey, (oldData: components["schemas"]["PodcastDto"]) => {
                                            return {
                                                ...oldData,
                                                active: !oldData.active
                                            }
                                        })
                                    }
                                }
                            })
                        }}/>
                    </div>
                </div>

                {/* Description */
                    currentPodcast.data?.summary &&
                    <div className="relative leading-[1.75] mb-8 text-sm text-(--fg-color)">
                        <div id="summary" className={lineClamp ? 'line-clamp-3' : ''}
                             dangerouslySetInnerHTML={removeHTML(currentPodcast.data.summary)}/>
                        {(isOverflown('summary') || lineClamp) && <div
                            className="cursor-pointer underline text-(--accent-color) hover:text-(--accent-color-hover)"
                            onClick={() => {
                                setLineClamp(!lineClamp)
                            }}>
                            {lineClamp ? t('show-more') : t('show-less')}
                        </div>}
                    </div>
                }

                {/* Episode list */}
                <div>
                    <Heading2 className="mb-8">{t('available-episodes')}</Heading2>

                    {currentPodcastEpisodes.data?.map((episode, index) => (
                        <PodcastDetailItem episode={episode} currentEpisodes={currentPodcastEpisodes.data ?? []}
                                           key={episode.podcastEpisode.id} index={index}
                                           onlyUnplayed={onlyUnplayed}/>
                    ))}
                </div>
            </div>

        </Fragment>
    )
}
