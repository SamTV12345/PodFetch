import {FC, PropsWithChildren, Suspense, useEffect, useRef, useState} from 'react'
import {createBrowserRouter, createRoutesFromElements, Navigate, Route} from 'react-router-dom'
import {useTranslation} from 'react-i18next'
import {enqueueSnackbar} from 'notistack'
import useCommon from './store/CommonSlice'
import useOpmlImport from './store/opmlImportSlice'
import {decodeHTMLEntities} from './utils/Utilities'
import {
    EpisodeSearchViewLazyLoad,
    HomepageViewLazyLoad,
    PlaylistViewLazyLoad,
    PodcastDetailViewLazyLoad,
    PodcastInfoViewLazyLoad,
    PodcastViewLazyLoad,
    SettingsViewLazyLoad,
    TimeLineViewLazyLoad,
    UserAdminViewLazyLoad
} from "./utils/LazyLoading"
import {Root} from "./routing/Root"
import {AcceptInvite} from "./pages/AcceptInvite"
import {Login} from "./pages/Login"
import "./App.css"
import './App.css'
import {HomePageSelector} from "./pages/HomePageSelector";
import {PlaylistPage} from "./pages/PlaylistPage";
import {SettingsData} from "./components/SettingsData";
import {SettingsOPMLExport} from "./components/SettingsOPMLExport";
import {SettingsNaming} from "./components/SettingsNaming";
import {SettingsPodcastDelete} from "./components/SettingsPodcastDelete";
import {UserAdminUsers} from "./components/UserAdminUsers";
import {UserAdminInvites} from "./components/UserAdminInvites";
import {UserManagementPage} from "./pages/UserManagement";
import {GPodderIntegration} from "./pages/GPodderIntegration";
import {TagsPage} from "./pages/TagsPage";
import {components} from "../schema";
import {client} from "./utils/http";
import io, {Socket} from "socket.io-client"
import {ClientToServerEvents, ServerToClientEvents} from "./models/socketioEvents";

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
            <Route path={"settings"} element={<Suspense><SettingsViewLazyLoad /></Suspense>}>
                <Route index element={<Navigate  to="retention"/>}/>
                <Route path="retention" element={<SettingsData/>}/>
                <Route path="opml" element={<SettingsOPMLExport/>}/>
                <Route path="naming" element={<SettingsNaming/>}/>
                <Route path="podcasts" element={<SettingsPodcastDelete/>}/>
                <Route path="gpodder" element={<GPodderIntegration/>}/>
            </Route>
            <Route path={"administration"} element={<Suspense><UserAdminViewLazyLoad /></Suspense>}>
                <Route index element={<Navigate to="users"/>}/>
                <Route path="users" element={<UserAdminUsers/>}/>
                <Route path="invites" element={<UserAdminInvites/>}/>
            </Route>
            <Route path={"profile"}>
                <Route index element={<UserManagementPage/>}/>
            </Route>
            <Route path="tags">
                <Route index element={<TagsPage/>}/>
            </Route>
        </Route>
        <Route path="/login" element={<Login />} />
        <Route path="/invite/:id" element={<AcceptInvite />}></Route>
    </>
), {
    basename: import.meta.env.BASE_URL
})

const App: FC<PropsWithChildren> = ({ children }) => {
    const config = useCommon(state => state.configModel)
    const addPodcast = useCommon(state => state.addPodcast)
    const { t } = useTranslation()
    const socket = useCommon(state=>state.socketIo)
    const setProgress = useOpmlImport(state => state.setProgress)
    const setNotifications = useCommon(state => state.setNotifications)
    const setSelectedEpisodes = useCommon(state => state.setSelectedEpisodes)
    const wasAlreadyRequested = useRef(false);

    useEffect(() => {
        if (!socket) {
            return
        }

        wasAlreadyRequested.current = true;


        socket.on('offlineAvailable', (data) => {
            if (!data) {
                return
            }
            console.log("I am ", typeof data === "string")

            if (useCommon.getState().currentDetailedPodcastId === data.podcast.id) {
            console.log("setting local url")
                enqueueSnackbar(t('new-podcast-episode-added', {name: decodeHTMLEntities(data.podcast_episode.name)}), {variant: 'success'})

                const downloadedPodcastEpisode = data.podcast_episode
                let res = useCommon.getState().selectedEpisodes
                    .find(p => p.podcastEpisode.id === downloadedPodcastEpisode.id)

                if (res == undefined) {
                    // This is a completely new episode
                    useCommon.getState().setSelectedEpisodes([...useCommon.getState().selectedEpisodes, {
                        podcastEpisode: downloadedPodcastEpisode
                    }])
                }

                let podcastUpdated = useCommon.getState().selectedEpisodes
                    .map(p => {
                            if (p.podcastEpisode.id === downloadedPodcastEpisode.id) {
                                const foundDownload = JSON.parse(JSON.stringify(p)) as components["schemas"]["PodcastEpisodeWithHistory"]

                                foundDownload.podcastEpisode.status = true
                                foundDownload.podcastEpisode.url = downloadedPodcastEpisode.url
                                foundDownload.podcastEpisode.local_url = downloadedPodcastEpisode.local_url
                                foundDownload.podcastEpisode.image_url = downloadedPodcastEpisode.image_url
                                foundDownload.podcastEpisode.local_image_url = downloadedPodcastEpisode.local_image_url

                                return foundDownload
                            }

                            return p
                        }) satisfies  components["schemas"]["PodcastEpisodeWithHistory"][]

                useCommon.getState().setSelectedEpisodes(podcastUpdated)
            }
        })

        socket.on('opmlError', (data) => {

            useOpmlImport.getState().setProgress([...useOpmlImport.getState().progress, false])
            useOpmlImport.getState().setMessages([...useOpmlImport.getState().messages, data.message])
        })

        socket.on('refreshedPodcast', (data) => {
            const podcast = data.podcast

            enqueueSnackbar(t('podcast-refreshed', {name: decodeHTMLEntities(podcast.name)}), {variant: 'success'})
        })

        socket.on('addedEpisodes', (data) => {
            enqueueSnackbar(t('new-podcast-episode-added', {name: decodeHTMLEntities(data.podcast.name)}), {variant: 'success'})
        })

        socket.on('addedPodcast', (data) => {
            const podcast = data.podcast

            addPodcast(podcast)
            enqueueSnackbar(t('new-podcast-added', {name: decodeHTMLEntities(podcast.name)}), {variant: 'success'})
        })

        socket.on('deletedPodcastEpisodeLocally', (data) => {
            const updatedPodcastEpisodes = useCommon.getState().selectedEpisodes.map(e => {
                if (e.podcastEpisode.episode_id === data.podcast_episode.episode_id) {
                    const clonedPodcast = Object.assign({}, data.podcast_episode)

                    clonedPodcast.status = false

                    return {
                        podcastEpisode: clonedPodcast
                    }
                }

                return e
            })

            enqueueSnackbar(t('podcast-episode-deleted', {name: decodeHTMLEntities(data.podcast_episode.name)}), {variant: 'success'})
            setSelectedEpisodes(updatedPodcastEpisodes)
        })

        socket.on('opmlAdded', () => {
            setProgress([...useOpmlImport.getState().progress, true])
        })
    }, [socket])

    useEffect(() => {
        if (config?.basicAuth||config?.oidcConfigured||config?.reverseProxy){
            client.GET("/api/v1/users/{username}", {
                params: {
                    path: {
                        username: "me"
                    }
                }
            }).then((c)=>useCommon.getState().setLoggedInUser(c.data!))
                .catch(() => enqueueSnackbar(t('not-admin'), { variant: 'error' }))
        }
    }, []);

    const getNotifications = () => {
        client.GET("/api/v1/notifications/unread")
            .then((response) => {
                setNotifications(response.data!)
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
