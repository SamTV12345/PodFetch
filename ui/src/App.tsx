import { FC, PropsWithChildren, Suspense, useEffect, useState } from 'react'
import {createBrowserRouter, createRoutesFromElements, Navigate, Route} from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import axios, { AxiosResponse } from 'axios'
import { enqueueSnackbar } from 'notistack'
import { store } from './store/store'
import { useAppDispatch, useAppSelector } from './store/hooks'
import { addPodcast, PodcastEpisode, setNotifications, setSelectedEpisodes } from './store/CommonSlice'
import { setMessages, setProgress } from './store/opmlImportSlice'
import { apiURL, configWSUrl, isJsonString } from './utils/Utilities'
import {
    UserAdminViewLazyLoad,
    EpisodeSearchViewLazyLoad,
    PodcastDetailViewLazyLoad,
    PodcastInfoViewLazyLoad,
    PodcastViewLazyLoad,
    SettingsViewLazyLoad,
    TimeLineViewLazyLoad, PlaylistViewLazyLoad, HomepageViewLazyLoad
} from "./utils/LazyLoading"
import {
    checkIfOpmlAdded,
    checkIfOpmlErrored,
    checkIfPodcastAdded,
    checkIfPodcastEpisodeAdded,
    checkIfPodcastEpisodeDeleted,
    checkIfPodcastRefreshed
} from "./utils/MessageIdentifier"
import {Notification} from "./models/Notification"
import {Root} from "./routing/Root"
import {AcceptInvite} from "./pages/AcceptInvite"
import {Login} from "./pages/Login"
import "./App.css"
import './App.css'
import {HomePageSelector} from "./pages/HomePageSelector";
import {EpisodesWithOptionalTimeline} from "./models/EpisodesWithOptionalTimeline";
import {PlaylistPage} from "./pages/PlaylistPage";

export const router = createBrowserRouter(createRoutesFromElements(
    <>
        <Route path="/" element={<Root/>}>
            <Route index element={<Navigate to="home"/>}/>
            <Route path="home" element={<HomePageSelector/>}>
                <Route index element={<Navigate to="view"/>}/>
                <Route path="view" element={<Suspense><HomepageViewLazyLoad /></Suspense>}/>
                <Route path={"playlist"}>
                    <Route index element={<Suspense><PlaylistPage/></Suspense>}></Route>
                    <Route path={":id"} element={<Suspense><PlaylistViewLazyLoad/></Suspense>}/>
                </Route>
            </Route>
            <Route path={"podcasts"}>
                <Route index element={<Suspense><PodcastViewLazyLoad/></Suspense>}/>
                <Route path={":id/episodes"} element={<Suspense><PodcastDetailViewLazyLoad/></Suspense>}/>
                <Route path={":id/episodes/:podcastid"} element={<Suspense><PodcastDetailViewLazyLoad/></Suspense>}/>
                <Route path={"search"} element={<Suspense><EpisodeSearchViewLazyLoad/></Suspense>}/>
            </Route>

            <Route path="timeline" element={<Suspense><TimeLineViewLazyLoad /></Suspense>} />
            <Route path={"favorites"}>
                <Route element={<PodcastViewLazyLoad onlyFavorites={true} />} index />
                <Route path={":id/episodes"} element={<Suspense><PodcastDetailViewLazyLoad /></Suspense>} />
                <Route path={":id/episodes/:podcastid"} element={<Suspense><PodcastDetailViewLazyLoad /></Suspense>} />
            </Route>
            <Route path={"info"} element={<Suspense><PodcastInfoViewLazyLoad /></Suspense>} />
            <Route path={"settings"} element={<Suspense><SettingsViewLazyLoad /></Suspense>} />
            <Route path={"administration"} element={<Suspense><UserAdminViewLazyLoad /></Suspense>} />
        </Route>
        <Route path="/login" element={<Login />} />
        <Route path="/invite/:id" element={<AcceptInvite />}></Route>
    </>
), {
    basename: import.meta.env.BASE_URL
})

const App: FC<PropsWithChildren> = ({ children }) => {
    const dispatch = useAppDispatch()
    const config = useAppSelector(state => state.common.configModel)
    const podcasts = useAppSelector(state => state.common.podcasts)
    const [socket, setSocket] = useState<WebSocket>()
    const { t } = useTranslation()

    useEffect(() => {
        if (socket) {
            socket.onopen = () => {
            }

            socket.onmessage = (event) => {
                if (!isJsonString(event.data)) return

                const parsed = JSON.parse(event.data)

                if (checkIfPodcastAdded(parsed)) {
                    const podcast = parsed.podcast

                    dispatch(addPodcast(podcast))
                    enqueueSnackbar(t('new-podcast-added', { name: podcast.name }), { variant: 'success' })
                } else if (checkIfPodcastEpisodeAdded(parsed)) {
                    if (store.getState().common.currentDetailedPodcastId === parsed.podcast_episode.podcast_id) {
                        enqueueSnackbar(t('new-podcast-episode-added', { name: parsed.podcast_episode.name }), { variant: 'success' })

                        const downloadedPodcastEpisode = parsed.podcast_episode
                        let res = store.getState().common.selectedEpisodes.find(p => p.podcastEpisode.id === downloadedPodcastEpisode.id)

                        if (res == undefined) {
                            // This is a completely new episode
                            dispatch(setSelectedEpisodes([...store.getState().common.selectedEpisodes, {
                                podcastEpisode: downloadedPodcastEpisode
                            }]))
                        }

                        let podcastUpdated:EpisodesWithOptionalTimeline[] = store.getState().common.selectedEpisodes
                            .map(p => {
                            if (p.podcastEpisode.id === downloadedPodcastEpisode.id) {
                                const foundDownload = JSON.parse(JSON.stringify(p)) as PodcastEpisode

                                foundDownload.status = 'D'
                                foundDownload.url = downloadedPodcastEpisode.url
                                foundDownload.local_url = downloadedPodcastEpisode.local_url
                                foundDownload.image_url = downloadedPodcastEpisode.image_url
                                foundDownload.local_image_url = downloadedPodcastEpisode.local_image_url

                                return {
                                    podcastEpisode: foundDownload
                                } satisfies EpisodesWithOptionalTimeline
                            }

                            return p
                        })

                        dispatch(setSelectedEpisodes(podcastUpdated))
                    }
                } else if (checkIfPodcastEpisodeDeleted(parsed)) {
                    const updatedPodcastEpisodes = store.getState().common.selectedEpisodes.map(e => {
                        if (e.podcastEpisode.episode_id === parsed.podcast_episode.episode_id) {
                            const clonedPodcast = Object.assign({}, parsed.podcast_episode)

                            clonedPodcast.status = 'N'

                            return {
                                podcastEpisode: clonedPodcast
                            }
                        }

                        return e
                    })

                    enqueueSnackbar(t('podcast-episode-deleted', { name: parsed.podcast_episode.name }), { variant: 'success' })
                    dispatch(setSelectedEpisodes(updatedPodcastEpisodes))
                } else if (checkIfPodcastRefreshed(parsed)) {
                    const podcast = parsed.podcast

                    enqueueSnackbar(t('podcast-refreshed', { name: podcast.name }), { variant: 'success' })
                } else if (checkIfOpmlAdded(parsed)) {
                    dispatch(setProgress([...store.getState().opmlImport.progress,true]))
                } else if (checkIfOpmlErrored(parsed)) {
                    const podcast = parsed

                    dispatch(setProgress([...store.getState().opmlImport.progress,false]))
                    dispatch(setMessages([...store.getState().opmlImport.messages, podcast.message]))
                }
            }

            socket.onerror = () => {
            }

            socket.onclose = () => {
            }
        }
    }, [podcasts, socket, config])

    useEffect(() => {
        if (config) {
            setSocket(new WebSocket(configWSUrl(config?.serverUrl!)))
        }
    }, [config])

    const getNotifications = () => {
        axios.get(apiURL + '/notifications/unread')
            .then((response: AxiosResponse<Notification[]>) => {
                dispatch(setNotifications(response.data))
            })
    }

    useEffect(() => {
        getNotifications()
    }, [])

    return (
        <Suspense>
            {children}
        </Suspense>
    )
}

export default App
