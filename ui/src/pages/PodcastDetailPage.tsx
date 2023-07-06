import {Fragment, useEffect, useState} from "react"
import {useParams} from "react-router-dom"
import {useTranslation} from "react-i18next"
import axios, {AxiosResponse} from "axios"
import {apiURL, removeHTML} from "../utils/Utilities"
import {useAppDispatch, useAppSelector} from "../store/hooks"
import {Podcast, setCurrentDetailedPodcastId, setSelectedEpisodes} from "../store/CommonSlice"
import {setCurrentPodcast} from "../store/AudioPlayerSlice"
import {Chip} from "../components/Chip"
import {Heading1} from "../components/Heading1"
import {Heading2} from "../components/Heading2"
import {PodcastDetailItem} from "../components/PodcastDetailItem"
import {PodcastInfoModal} from "../components/PodcastInfoModal"
import {Switcher} from "../components/Switcher"
import "material-symbols/outlined.css"

export const PodcastDetailPage = () => {
    const currentPodcast = useAppSelector(state => state.audioPlayer.currentPodcast)
    const params = useParams()
    const selectedEpisodes = useAppSelector(state => state.common.selectedEpisodes)
    const dispatch = useAppDispatch()
    const [lineClamp, setLineClamp] = useState(true)
    const {t} = useTranslation()
    const configModel = useAppSelector(state => state.common.configModel)

    useEffect(() => {
        if(params&&!isNaN(parseFloat(params.id as string))) {
            dispatch(setCurrentDetailedPodcastId(Number(params.id)))
        }

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

        <div className="max-w-4xl">
            <PodcastInfoModal/>

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
                " src={currentPodcast.image_url} alt=""/>

                {/* Title and refresh icon */}
                <div className="
                    order-2
                    col-start-1 col-end-4 row-start-2 row-end-3
                    sm:col-start-1 sm:col-end-3
                    lg:col-start-2 lg:col-end-3 
                    self-start xs:self-end
                ">
                    <Heading1 className="inline align-middle mr-2">{currentPodcast.name}</Heading1>

                    <span className="material-symbols-outlined inline cursor-pointer align-middle text-stone-800 hover:text-stone-600" onClick={() => {
                        axios.post(apiURL + "/podcast/" + params.id + "/refresh")
                            .then(() => {
                                console.log("Refreshed")
                            })
                    }}>refresh</span>
                </div>

                {/* Metadata */}
                <div className="
                    order-3
                    col-start-2 col-end-4 row-start-3 row-end-4
                    sm:col-start-2 sm:col-end-3
                    self-start flex flex-col gap-2
                ">
                    <span className="block text-stone-500">{currentPodcast.author}</span>

                    {<div className="flex gap-2">
                        {currentPodcast.keywords && currentPodcast.keywords?.split(',').map((keyword, index) => {
                            return <Chip key={index} index={index}>{keyword}</Chip>
                        })}
                    </div>}

                    <span className="material-symbols-outlined inline cursor-pointer text-stone-800 hover:text-stone-600" onClick={()=>{window.open(configModel?.rssFeed+"/"+params.id)}}>rss_feed</span>
                </div>

                {/* Toggle */}
                <div className="
                    order-1
                    col-start-1 col-end-4 row-start-1 row-end-2
                    sm:col-start-3 sm:col-end-4 sm:row-start-2 sm:row-end-3
                    justify-self-end self-end sm:self-start
                    flex gap-3 items-center
                ">
                    <span className="text-xs text-stone-500">{t('active')}</span>

                    <Switcher checked={currentPodcast.active} setChecked={()=>{
                        axios.put(apiURL + "/podcast/" + params.id + "/active")
                            .then(() => {
                                dispatch(setCurrentPodcast({...currentPodcast, active: !currentPodcast?.active}))
                            })
                    }}/>
                </div>
            </div>

            {/* Description */
            currentPodcast.summary &&
                <div className="relative leading-[1.75] mb-8 text-sm text-stone-900">
                    <div id="summary" className={lineClamp?'line-clamp-3':''} dangerouslySetInnerHTML={removeHTML(currentPodcast.summary)}/>
                    {(isOverflown('summary')||lineClamp)&&<div className="cursor-pointer underline text-mustard-600 hover:text-mustard-500"  onClick={()=>{
                        setLineClamp(!lineClamp)
                    }}>
                        {lineClamp?t('show-more'):t('show-less')}
                    </div>}
                </div>
            }

            {/* Episode list */}
            <div>
                <Heading2 className="mb-8">{t('available-episodes')}</Heading2>

                {selectedEpisodes.map((episode, index) => {
                    return <PodcastDetailItem episode={episode} key={index} index={index} episodesLength={selectedEpisodes.length}/>
                })}
            </div>
        </div>

    </Fragment>
}
