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


interface PodcastsProps {
    onlyFavorites?: boolean
}

export const Podcasts:FC<PodcastsProps> = ({onlyFavorites})=>{
    const podcasts = useAppSelector(state=>state.common.podcasts)
    const dispatch = useAppDispatch()
    let location = useLocation();
    const [searchText, setSearchText] = useState<string>('')
    const [orderOfPodcasts, setOrderOfPodcasts] = useState<string>()
    const [latestPub, setLatestPub] = useState<boolean>(true)

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
            .then(v=>dispatch(setPodcasts(v.data)))
            .then()
    },500, [searchText, orderOfPodcasts])

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
        <div className="flex flex-1">
            <span className="relative  w-1/3">
                <input type="text" value={searchText}  onChange={v => setSearchText(v.target.value)}
                       className="border-gray-400 w-full pl-10 pt-1 pb-1 border-2 rounded-2xl"/>
                <span className="absolute left-2 top-1.5 scale-90">
                    <MaginifyingGlassIcon/>
                </span>
            </span>
            <select value={orderOfPodcasts} onChange={v=>setOrderOfPodcasts(v.target.value)} className="ml-5 border  text-sm rounded-lg
                    block p-2.5 bg-gray-700 border-gray-600
                    placeholder-gray-400 text-white focus:ring-blue-500 focus:border-blue-500">
                <option value="false">Aufsteigend</option>
                <option value="true">Absteigend</option>
            </select>
            <div className="flex-1"></div>
            <RefreshIcon onClick={()=>{
                refreshAllPodcasts()
            }}/>
        <button className="fa fa-plus bg-blue-900 text-white p-3" onClick={()=>{
            dispatch(setModalOpen(true))
        }}></button>
        </div>
        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 lg:grid-cols-5  xs:grid-cols-3 gap-2 pt-3">
            {!onlyFavorites&&podcasts.map((podcast, index)=>{

                return <Card podcast={podcast} key={index}/>
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
