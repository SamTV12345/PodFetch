import {Fragment, useEffect, useState} from 'react'
import {useParams} from 'react-router-dom'
import {useTranslation} from 'react-i18next'
import {prependAPIKeyOnAuthEnabled, removeHTML} from '../utils/Utilities'
import useCommon, {Podcast} from '../store/CommonSlice'
import useAudioPlayer from '../store/AudioPlayerSlice'
import {Chip} from '../components/Chip'
import {Heading1} from '../components/Heading1'
import {Heading2} from '../components/Heading2'
import {PodcastDetailItem} from '../components/PodcastDetailItem'
import {PodcastInfoModal} from '../components/PodcastInfoModal'
import {Switcher} from '../components/Switcher'
import 'material-symbols/outlined.css'
import {PodcastEpisodeAlreadyPlayed} from "../components/PodcastEpisodeAlreadyPlayed";
import {ErrorIcon} from "../icons/ErrorIcon";
import {PodcastSettingsModal} from "../components/PodcastSettingsModal";
import { client } from '../utils/http'

export const PodcastDetailPage = () => {
    const configModel = useCommon(state => state.configModel)
    const currentPodcast = useAudioPlayer(state => state.currentPodcast)
    const selectedEpisodes = useCommon(state => state.selectedEpisodes)
    const setCurrentPodcast = useAudioPlayer(state => state.setCurrentPodcast)
    const params = useParams()
    const [lineClamp, setLineClamp] = useState(true)
    const {t} = useTranslation()
    const setCurrentDetailedPodcastId = useCommon(state => state.setCurrentDetailedPodcastId)
    const setInfoModalPodcastOpen = useCommon(state => state.setInfoModalPodcastOpen)
    const setSelectedEpisodes = useCommon(state => state.setSelectedEpisodes)
    const [openSettingsMenu, setOpenSettingsMenu] = useState<boolean>(false)
    const [onlyUnplayed, setOnlyUnplayed] = useState<boolean>(false)

    useEffect(() => {
        if (params && !isNaN(parseFloat(params.id as string))) {
            setCurrentDetailedPodcastId(Number(params.id))
        }
        client.GET("/api/v1/podcasts/{id}", {
            params: {
                path: {
                    id: params.id!
                }
            }
        }).then(resp=>{
            setCurrentPodcast(resp.data!)
        })
        .then(() => {
            client.GET("/api/v1/podcasts/{id}/episodes", {
                params: {
                    query: {
                        only_unlistened: onlyUnplayed
                    },
                    path: {
                        id: params.id!
                    }
                }
            }).then((response) => {
                setSelectedEpisodes(response.data!)

                if (params.podcastid) {
                    const element = document.getElementById('episode_' + params.podcastid)

                    if (element) {
                        element.scrollIntoView({behavior: 'smooth', block: 'start', inline: 'nearest'})
                    }
                }
            })
        })
    }, [onlyUnplayed])

    useEffect(() => {
        if (currentPodcast?.summary) {
            const summary = document.getElementById('summary')!

            summary.querySelectorAll('a').forEach((a) => {
                a.setAttribute('target', '_blank')
            })
        }
    }, [currentPodcast?.summary])

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

    if (currentPodcast === undefined) {
        return <div className="w-full md:w-3/4 mx-auto">
            <ErrorIcon text={t('podcast-not-found')}/>
        </div>
    }

    return (
        <Fragment key={'detail'}>
            <div className="max-w-4xl">
                <PodcastInfoModal/>
                <PodcastEpisodeAlreadyPlayed/>
                <PodcastSettingsModal open={openSettingsMenu} setOpen={setOpenSettingsMenu} podcast={currentPodcast}/>

                {/* Header */}
                <div className="
                    flex flex-col
                    xs:grid
                    xs:grid-cols-[auto_1fr_auto] xs:grid-rows-[auto_auto_auto]
                    gap-x-4 gap-y-2 lg:gap-x-8 lg:gap-y-1 items-center mb-8
                ">
                    {/* Thumbnail */}
                    <img className="
                        order-4
                        xs:col-start-1 xs:col-end-2 row-start-3 row-end-4
                        lg:col-start-1 lg:col-end-2 lg:row-start-2 lg:row-end-4
                        w-full xs:w-24 md:w-32 lg:w-40 rounded-xl
                    " src={prependAPIKeyOnAuthEnabled(currentPodcast.image_url)} alt=""/>

                    {/* Title and refresh icon */}
                    <div className="
                        order-2
                        col-start-1 col-end-4 row-start-2 row-end-3
                        sm:col-start-1 sm:col-end-3
                        lg:col-start-2 lg:col-end-3
                        self-start xs:self-end
                    ">
                        <Heading1 className="inline align-middle mr-2">{currentPodcast.name}</Heading1>

                        <span
                            className="material-symbols-outlined inline cursor-pointer align-middle text-[--fg-icon-color] hover:text-[--fg-icon-color-hover]"
                            onClick={() => {
                                client.POST("/api/v1/podcasts/{id}/refresh", {
                                    params: {
                                        path: {
                                            id: params.id!
                                        }
                                    }
                                })
                            }}>refresh</span>
                        <span>
                            <button className="material-symbols-outlined inline cursor-pointer align-middle text-[--fg-icon-color] hover:text-[--fg-icon-color-hover]"
                                    onClick={() => {
                                        setOpenSettingsMenu(true)
                                    }}>settings</button>
                        </span>
                    </div>

                    {/* Metadata */}
                    <div className="
                        order-3
                        col-start-2 col-end-4 row-start-3 row-end-4
                        sm:col-start-2 sm:col-end-3
                        self-start flex flex-col items-start gap-2
                    ">
                        <span className="block text-[--fg-secondary-color]">{currentPodcast.author}</span>

                        <div className="flex gap-2">
                            {currentPodcast.keywords && currentPodcast.keywords?.split(',').map((keyword, index) => (
                                <Chip key={"keyword"+index} index={index}>{keyword}</Chip>
                            ))}
                        </div>

                        <span className="grid grid-cols-2 md:grid-cols-3">
                        <button className="flex gap-4" rel="noopener noreferrer"
                                onClick={() => window.open(prependAPIKeyOnAuthEnabled(configModel?.rssFeed + '/' + params.id))}>
                            <a rel="noopener noreferrer"
                               className="material-symbols-outlined cursor-pointer text-[--fg-icon-color] hover:text-[--fg-icon-color-hover]"
                               target="_blank"
                               href={prependAPIKeyOnAuthEnabled(configModel?.rssFeed + '/' + params.id)}>rss_feed</a>
                            <span className="text-[--fg-color]">PodFetch</span>
                        </button>

                        <button className="flex gap-4" rel="noopener noreferrer"
                                onClick={() => window.open(currentPodcast.rssfeed)}>
                            <a className="material-symbols-outlined cursor-pointer text-[--fg-icon-color] hover:text-[--fg-icon-color-hover]"
                               target="_blank" rel="noopener noreferrer" href={currentPodcast.rssfeed}>rss_feed</a>
                            <span className="text-[--fg-color]">{t('original-rss-feed')}</span>
                        </button>
                            <div className="flex gap-4 justify-end">
                                <Switcher checked={onlyUnplayed} onChange={setOnlyUnplayed}/>
                                <span className=" text-[--fg-color] mt-auto">{t('active')}</span>
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
                        <span className="text-xs text-[--fg-secondary-color]">{t('active')}</span>

                        <Switcher checked={currentPodcast.active} onChange={() => {
                            client.PUT("/api/v1/podcasts/{id}/active", {
                                params: {
                                    path: {
                                        id: params.id!
                                    }
                                }
                            }).then(()=>{
                                setCurrentPodcast({...currentPodcast, active: !currentPodcast?.active})
                            })
                        }}/>
                    </div>
                </div>

                {/* Description */
                    currentPodcast.summary &&
                    <div className="relative leading-[1.75] mb-8 text-sm text-[--fg-color]">
                        <div id="summary" className={lineClamp ? 'line-clamp-3' : ''}
                             dangerouslySetInnerHTML={removeHTML(currentPodcast.summary)}/>
                        {(isOverflown('summary') || lineClamp) && <div
                            className="cursor-pointer underline text-[--accent-color] hover:text-[--accent-color-hover]"
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

                    {selectedEpisodes.map((episode, index) => (
                        <PodcastDetailItem episode={episode} key={episode.podcastEpisode.id} index={index}
                                           onlyUnplayed={onlyUnplayed}
                                           episodesLength={selectedEpisodes.length}/>
                    ))}
                </div>
            </div>

        </Fragment>
    )
}
