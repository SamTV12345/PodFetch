import {useAppDispatch, useAppSelector} from "../store/hooks";
import {useParams} from "react-router-dom";
import {Fragment, useEffect, useState} from "react";
import {apiURL, removeHTML} from "../utils/Utilities";
import axios, {AxiosResponse} from "axios";
import {Podcast, setSelectedEpisodes} from "../store/CommonSlice";
import {PodcastInfoModal} from "../components/PodcastInfoModal";
import {useTranslation} from "react-i18next";
import {setCurrentPodcast} from "../store/AudioPlayerSlice";
import {Chip} from "../components/Chip";
import {PodcastDetailItem} from "../components/PodcastDetailItem";
import {Switcher} from "../components/Switcher";

export const PodcastDetailPage = () => {
    const currentPodcast = useAppSelector(state => state.audioPlayer.currentPodcast)
    const params = useParams()
    const selectedEpisodes = useAppSelector(state => state.common.selectedEpisodes)
    const dispatch = useAppDispatch()
    const [lineClamp, setLineClamp] = useState(true)
    const {t} = useTranslation()

    useEffect(() => {
        axios.get(apiURL + "/podcast/" + params.id).then((response: AxiosResponse<Podcast>) => {
            dispatch(setCurrentPodcast(response.data))
        }).then(() => {
            axios.get(apiURL + "/podcast/" + params.id + "/episodes")
                .then((response) => {
                    dispatch(setSelectedEpisodes(response.data))
                    if (params.podcastid) {
                        const element = document.getElementById("episode_" + params.podcastid)
                        if (element) {
                            element.scrollIntoView({behavior: "smooth", block: "start", inline: "nearest"})
                        }
                    }
                })
        })
    }, [])

    useEffect(()=>{
        if(currentPodcast?.summary) {
            const summary = document.getElementById('summary')!
            summary.querySelectorAll('a').forEach((a) => {
                a.setAttribute('target', '_blank')
            })
        }
    },[currentPodcast?.summary])

    const isOverflown = (element: string)=>{
        let foundElement = document.getElementById(element)
        if(foundElement) {
            return foundElement.scrollHeight > foundElement.clientHeight || foundElement.scrollWidth > foundElement.clientWidth;
        }
        return false
    }

    useEffect(() => {

            if (params.podcastid) {
                const element = document.getElementById("episode_" + params.podcastid)
                if (element) {
                    element.scrollIntoView({behavior: "smooth", block: "start", inline: "nearest"})
                }
            }
        },
        [params])


    if (currentPodcast === undefined) {
        return <div>"Nicht gefunden"</div>
    }

    return <Fragment key={"detail"}>
        <div className="pl-5 pt-5 overflow-y-scroll">
            <PodcastInfoModal/>
            <div className="grid grid-cols-[auto_1fr] gap-3">
                <div className="grid place-items-center">
                    <img className="w-60 rounded" src={currentPodcast.image_url} alt=""/>
                </div>
                <div className="grid place-items-center">
                    <div className=" w-full">
                        <h1 className="movie-text">{currentPodcast.name}
                            <i className="fa-solid fa-arrows-rotate hover:text-slate-400 active:text-slate-800 active:scale-95 ml-1"
                               onClick={() => {
                                   axios.post(apiURL + "/podcast/" + params.id + "/refresh")
                                       .then(() => {
                                           console.log("Refreshed")
                                       })
                               }}></i></h1>
                        <div className="flex gap-3">
                            <div>{t('active')}</div>
                            <Switcher checked={currentPodcast.active} setChecked={()=>{
                                axios.put(apiURL + "/podcast/" + params.id + "/active")
                                    .then(() => {
                                        dispatch(setCurrentPodcast({...currentPodcast, active: !currentPodcast?.active}))
                                    })
                            }}/>
                        </div>
                        <h2 className="text-xl text-slate-600">{currentPodcast.author}</h2>
                        {<div className="flex gap-2">
                            {
                                currentPodcast.keywords && currentPodcast.keywords?.split(',').map((keyword, index) => {
                                    return <Chip key={index} index={index}>{keyword}</Chip>
                                })
                                }
                        </div>
                        }
                    </div>
                </div>
            </div>
            {currentPodcast.summary&&<div className="relative m-2">
                <div id="summary" className={`podcast-summary ${lineClamp?'line-clamp-3':''}`} dangerouslySetInnerHTML={removeHTML(currentPodcast.summary)}/>
                {(isOverflown('summary')||lineClamp)&&<div className="left-0 text-slate-600 underline  cursor-pointer "  onClick={()=>{
                    setLineClamp(!lineClamp)
                }}>{lineClamp?t('show-more'):t('show-less')}</div>}
            </div>
            }
            <hr className="border-gray-400"/>
            <div>
                {
                    selectedEpisodes.map((episode, index) => {
                        return <PodcastDetailItem episode={episode} key={index} index={index}/>
                    })
                }
            </div>
        </div>

    </Fragment>
}
