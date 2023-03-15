import './App.css'
import {BrowserRouter, Route, Routes} from "react-router-dom";
import {SideBar} from "./components/SideBar";
import {Header} from "./components/Header";
import {useAppDispatch, useAppSelector} from "./store/hooks";
import {Podcasts} from "./pages/Podcasts";
import {PodcastDetailPage} from "./pages/PodcastDetailPage";
import {AudioPlayer} from "./components/AudioPlayer";
import {Homepage} from "./pages/Homepage";
import {Search} from "./components/Search";
import {apiURL, isJsonString, wsURL} from "./utils/Utilities";
import axios, {AxiosResponse} from "axios";
import {useEffect, useState} from "react";
import {Notification} from "./models/Notification";
import {PodcastEpisode, setNotifications, setPodcasts, setSelectedEpisodes} from "./store/CommonSlice";
import {checkIfPodcastAdded, checkIfPodcastEpisodeAdded} from "./utils/MessageIdentifier";
import useOnMount from "./hooks/useOnMount";
import {store} from "./store/store";

const App = () => {
    const dispatch = useAppDispatch()
    const sideBarCollapsed = useAppSelector(state => state.common.sideBarCollapsed)
    const currentPodcast = useAppSelector(state => state.audioPlayer.currentPodcastEpisode)
    const podcasts = useAppSelector(state => state.common.podcasts)
    const [socket] = useState(()=>new WebSocket(wsURL))

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
            if (checkIfPodcastAdded(parsed)) {
                const podcast = parsed.podcast
                dispatch(setPodcasts([...podcasts, podcast]))
            } else if (checkIfPodcastEpisodeAdded(parsed)) {
                const downloadedPodcastEpisode = parsed.podcast_episode
                let podcastUpdated = store.getState().common.selectedEpisodes.map(p => {
                    if (p.id === downloadedPodcastEpisode.id) {
                        const foundDownload = JSON.parse(JSON.stringify(p)) as PodcastEpisode
                        foundDownload.status = "D"
                        return foundDownload
                    }
                    return p
                })
                dispatch(setSelectedEpisodes(podcastUpdated))
            }
        }

        socket.onerror = (event) => {
            console.log("Error")
        }

        socket.onclose = (event) => {
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

    return (
        <BrowserRouter basename="/ui">
            <div className="grid  grid-rows-[auto_1fr] h-full md:grid-cols-[300px_1fr]">
                <Header/>
                <SideBar/>
                <div
                    className={`col-span-6 md:col-span-5 ${sideBarCollapsed ? 'xs:col-span-5' : 'hidden'} md:block w-full overflow-x-auto`}>
                    <div className="grid grid-rows-[1fr_auto] h-full ">
                        <Routes>
                            <Route path={"/home"} element={<Homepage/>}/>
                            <Route path={"/podcasts"} element={<Podcasts/>}/>
                            <Route path={"/podcasts/:id"} element={<PodcastDetailPage/>}/>
                            <Route path={"/podcasts/:id/episodes/:podcastid"} element={<PodcastDetailPage/>}/>
                        </Routes>
                        {currentPodcast && <AudioPlayer/>}
                    </div>
                </div>
            </div>
            <Search/>
        </BrowserRouter>
    )
}

export default App
