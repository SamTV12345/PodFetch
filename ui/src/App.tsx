import './App.css'
import {
    BrowserRouter,
    createBrowserRouter,
    createRoutesFromElements,
    Outlet,
    Route,
    RouterProvider,
    Routes
} from "react-router-dom";
import {SideBar} from "./components/SideBar";
import {Header} from "./components/Header";
import {useAppDispatch, useAppSelector} from "./store/hooks";
import {Podcasts} from "./pages/Podcasts";
import {PodcastDetailPage} from "./pages/PodcastDetailPage";
import {Homepage} from "./pages/Homepage";
import {Search} from "./components/Search";
import {apiURL, isJsonString, wsURL} from "./utils/Utilities";
import axios, {AxiosResponse} from "axios";
import {FC, useEffect, useState} from "react";
import {Notification} from "./models/Notification";
import {PodcastEpisode, setNotifications, setPodcasts, setSelectedEpisodes} from "./store/CommonSlice";
import {checkIfPodcastAdded, checkIfPodcastEpisodeAdded} from "./utils/MessageIdentifier";
import {store} from "./store/store";
import {PodcastInfoPage} from "./pages/PodcastInfoPage";
import {SettingsPage} from "./pages/SettingsPage";
import {Root} from "./routing/Root";


const router =  createBrowserRouter(createRoutesFromElements(
    <Route path="/ui" element={<Root/>}>
        <Route index element={<Homepage/>}/>
        <Route path={"podcasts"}>
            <Route index  element={<Podcasts/>}/>
            <Route path={":id/episodes"} element={<PodcastDetailPage/>}/>
            <Route path={":id/episodes/:podcastid"} element={<PodcastDetailPage/>}/>
        </Route>
        <Route path={"favorites"}>
            <Route element={<Podcasts onlyFavorites={true}/>} index/>
            <Route path={":id/episodes"} element={<PodcastDetailPage/>}/>
            <Route path={":id/episodes/:podcastid"} element={<PodcastDetailPage/>}/>
        </Route>
        <Route path={"info"} element={<PodcastInfoPage/>}/>
        <Route path={"settings"} element={<SettingsPage/>}/>
    </Route>


))

const App = () => {
    const dispatch = useAppDispatch()
    const podcasts = useAppSelector(state => state.common.podcasts)
    const [socket] = useState(()=>new WebSocket(wsURL))
    const currentPodcast = useAppSelector(state => state.audioPlayer.currentPodcast)

    useEffect(() => {
        socket.onopen = () => {
            console.log("Connected")
            socket.send("Hello")
        }

        socket.onmessage = (event) => {
            if (!isJsonString(event.data)) {
                return
            }
            const parsed = JSON.parse(event.data)
            console.log(parsed)
            if (checkIfPodcastAdded(parsed)) {
                const podcast = parsed.podcast
                dispatch(setPodcasts([...podcasts, podcast]))
            } else if (checkIfPodcastEpisodeAdded(parsed)) {
                if(currentPodcast?.id === parsed.podcast_episode.podcast_id) {
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
    }, [podcasts, socket])

    const getNotifications = () => {
        axios.get(apiURL + '/notifications/unread')
            .then((response: AxiosResponse<Notification[]>) => {
                dispatch(setNotifications(response.data))
            })
    }

    useEffect(() => {
        getNotifications()
    }, [])








    return <RouterProvider router={router}/>
}

export default App
