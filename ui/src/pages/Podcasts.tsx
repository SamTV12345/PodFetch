import {FC, useEffect, useState} from "react";
import axios, {AxiosResponse} from "axios";
import {apiURL} from "../utils/Utilities";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {Podcast, setPodcasts} from "../store/CommonSlice";
import {Card} from "../components/Card";
import {AddPodcast} from "../components/AddPodcast";
import {setModalOpen} from "../store/ModalSlice";
import {useLocation} from "react-router-dom";
import {RefreshIcon} from "../components/RefreshIcon";
import {MaginifyingGlassIcon} from "../icons/MaginifyingGlassIcon";
import {useDebounce} from "../utils/useDebounce";
import {useTranslation} from "react-i18next";


interface PodcastsProps {
    onlyFavorites?: boolean
}

export const Podcasts:FC<PodcastsProps> = ({onlyFavorites})=>{
    const podcasts = useAppSelector(state=>state.common.podcasts)
    const dispatch = useAppDispatch()
    let location = useLocation();
    const [searchText, setSearchText] = useState<string>('')
    const [orderOfPodcasts, setOrderOfPodcasts] = useState<boolean>()
    const [latestPub, setLatestPub] = useState<boolean>(true)
    const {t} = useTranslation()
    const refreshAllPodcasts = ()=>{
        axios.post(apiURL+"/podcast/all")
    }

    useDebounce(()=> {
        axios.get(apiURL+"/podcasts/search",{
            params:{
                title: searchText,
                orderOfPodcasts,
                latestPub
            }
        })
            .then(v=>{
                    dispatch(setPodcasts(v.data))
                })
    },500, [searchText, orderOfPodcasts, latestPub])

    useEffect(()=>{
        let url = apiURL+"/podcasts"
        if(onlyFavorites){
            url = apiURL+"/podcasts/favored"
        }
        axios.get(url)
            .then((response:AxiosResponse<Podcast[]>)=>{
                dispatch(setPodcasts(response.data))
            })
    },[location])

    return <div className="p-5">
        <AddPodcast/>
        <div className="flex flex-col md:flex-row gap-3">
                <span className="relative  w-1/3">
                    <input type="text" value={searchText}  onChange={v => setSearchText(v.target.value)}
                           className="border-gray-400 w-full pl-10 pt-1 pb-1 border-2 rounded-2xl"/>
                    <span className="absolute left-2 top-1.5 scale-90">
                        <MaginifyingGlassIcon/>
                    </span>
                </span>
                <div className="border-2 border-gray-500 bg-gray-800 p-1 text-white rounded grid grid-cols-[1fr_auto] gap-3">
                    <span className="ml-1">{t('descendant')}</span>
                    <input type={"checkbox"} checked={orderOfPodcasts} onChange={v=>setOrderOfPodcasts(v.target.checked)} className="m-1 w-4"/>
                </div>
                <div className="border-2 border-gray-500 bg-gray-800 p-1 text-white rounded grid grid-cols-[1fr_auto] gap-3">
                    <span>{t('sort-by-published-date')}</span>
                        <input type={"checkbox"} checked={latestPub} onChange={v=>setLatestPub(v.target.checked)} className="m-1 w-4"/>
                </div>
                <div className="flex-1"></div>
            <div className="grid grid-cols-2">
                    <RefreshIcon onClick={()=>{
                        refreshAllPodcasts()
                    }}/>
                    <button className="fa fa-plus bg-blue-900 text-white p-3" onClick={()=>{
                        dispatch(setModalOpen(true))
                    }}></button>
            </div>
        </div>
        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 lg:grid-cols-5  xs:grid-cols-3 gap-2 pt-3">
            {!onlyFavorites&&podcasts.map((podcast, index)=>{

                return <Card podcast={podcast} key={podcast.id}/>
            })
            }
            {
            onlyFavorites&&podcasts.filter(podcast=>podcast.favorites).map((podcast)=>{
                return <Card podcast={podcast} key={podcast.id}/>
                })
            }
        </div>
    </div>
}
