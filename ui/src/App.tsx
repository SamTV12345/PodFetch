import './App.css'
import {createBrowserRouter, createRoutesFromElements, Route} from "react-router-dom";
import {useAppDispatch, useAppSelector} from "./store/hooks";
import {Homepage} from "./pages/Homepage";
import axios, {AxiosResponse} from "axios";
import {FC, PropsWithChildren, Suspense, useEffect, useState} from "react";
import {Notification} from "./models/Notification";
import {addPodcast, PodcastEpisode, setNotifications, setPodcasts, setSelectedEpisodes} from "./store/CommonSlice";
import {checkIfPodcastAdded, checkIfPodcastEpisodeAdded} from "./utils/MessageIdentifier";
import {store} from "./store/store";
import {Root} from "./routing/Root";
import {PodcastWatchedEpisodeModel} from "./models/PodcastWatchedEpisodeModel";
import {
    PodcastDetailViewLazyLoad,
    PodcastInfoViewLazyLoad,
    PodcastViewLazyLoad,
    SettingsViewLazyLoad
} from "./utils/LazyLoading";
import {apiURL, configWSUrl, isJsonString} from "./utils/Utilities";
import {LoginComponent} from "./components/LoginComponent";


export const router =  createBrowserRouter(createRoutesFromElements(
    <>
    <Route path="/" element={<Root/>}>
        <Route index element={<Homepage/>}/>
        <Route path={"podcasts"}>
            <Route index  element={<Suspense><PodcastViewLazyLoad/></Suspense>}/>
            <Route path={":id/episodes"} element={<Suspense><PodcastDetailViewLazyLoad/></Suspense>}/>
            <Route path={":id/episodes/:podcastid"} element={<Suspense><PodcastDetailViewLazyLoad/></Suspense>}/>
        </Route>
        <Route path={"favorites"}>
            <Route element={<PodcastViewLazyLoad onlyFavorites={true}/>} index/>
            <Route path={":id/episodes"} element={<Suspense><PodcastDetailViewLazyLoad/></Suspense>}/>
            <Route path={":id/episodes/:podcastid"} element={<Suspense><PodcastDetailViewLazyLoad/></Suspense>}/>
        </Route>
        <Route path={"info"} element={<Suspense><PodcastInfoViewLazyLoad/></Suspense>}/>
        <Route path={"settings"} element={<Suspense><SettingsViewLazyLoad/></Suspense>}/>
    </Route>
    <Route path="/login" element={<LoginComponent/>}/>
    </>
), {
    basename: import.meta.env.BASE_URL
})

const App:FC<PropsWithChildren> = ({children}) => {
    const dispatch = useAppDispatch()
    const podcasts = useAppSelector(state => state.common.podcasts)
    const [socket, setSocket] = useState<any>()
    const config = useAppSelector(state=>state.common.configModel)

    useEffect(() => {
        if(socket) {
            socket.onopen = () => {
                console.log("Connected")
            }

            // @ts-ignore
            socket.onmessage = (event) => {
                if (!isJsonString(event.data)) {
                    return
                }
                const parsed = JSON.parse(event.data)
                if (checkIfPodcastAdded(parsed)) {
                    const podcast = parsed.podcast
                    dispatch(addPodcast(podcast))
                } else if (checkIfPodcastEpisodeAdded(parsed)) {
                    if (store.getState().common.currentDetailedPodcastId === parsed.podcast_episode.podcast_id) {
                        console.log("Episode added to current podcast")
                        const downloadedPodcastEpisode = parsed.podcast_episode
                        let res = store.getState().common.selectedEpisodes.find(p => p.id === downloadedPodcastEpisode.id)
                        if (res == undefined) {
                            // This is a completely new episode
                            dispatch(setSelectedEpisodes([...store.getState().common.selectedEpisodes, downloadedPodcastEpisode]))
                        }
                        let podcastUpdated = store.getState().common.selectedEpisodes.map(p => {
                            if (p.id === downloadedPodcastEpisode.id) {
                                const foundDownload = JSON.parse(JSON.stringify(p)) as PodcastEpisode
                                foundDownload.status = "D"
                                foundDownload.url = downloadedPodcastEpisode.url
                                foundDownload.local_url = downloadedPodcastEpisode.local_url
                                foundDownload.image_url = downloadedPodcastEpisode.image_url
                                foundDownload.local_image_url = downloadedPodcastEpisode.local_image_url
                                return foundDownload
                            }
                            return p
                        })
                        dispatch(setSelectedEpisodes(podcastUpdated))
                    }
                }
            }


            socket.onerror = () => {
                console.log("Error")
            }

            socket.onclose = () => {
                console.log("Closed")
            }
        }
    }, [podcasts, socket, config])

    useEffect(()=> {
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

    return <Suspense><div className="h-full">{children}</div></Suspense>
}

export default App
