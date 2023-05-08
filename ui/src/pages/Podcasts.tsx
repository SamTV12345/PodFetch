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
import {Order, OrderCriteria} from "../models/Order";
import {Filter} from "../models/Filter";


interface PodcastsProps {
    onlyFavorites?: boolean
}

export const Podcasts:FC<PodcastsProps> = ({onlyFavorites})=>{
    const podcasts = useAppSelector(state=>state.common.podcasts)
    const dispatch = useAppDispatch()
    let location = useLocation();
    const [searchText, setSearchText] = useState<string>('')
    const [orderOfPodcasts, setOrderOfPodcasts] = useState<Order>(Order.ASC)
    const [latestPub, setLatestPub] = useState<OrderCriteria>(OrderCriteria.TITLE)
    const {t} = useTranslation()

    const refreshAllPodcasts = ()=>{
        axios.post(apiURL+"/podcast/all")
    }

    const performFilter =()=>{
        axios.get(apiURL + "/podcasts/search", {
            params: {
                title: searchText,
                order: orderOfPodcasts,
                orderOption: latestPub,
                favoredOnly: !!onlyFavorites
            }
        })
            .then((v: AxiosResponse<Podcast[]>) => {
                dispatch(setPodcasts(v.data))
            })
    }

    useDebounce(()=> {
        performFilter();
    },500, [searchText, orderOfPodcasts, latestPub])

    useEffect(()=>{
        axios.get(apiURL+"/podcasts/filter").then((c:AxiosResponse<Filter>)=>{
            if(c.data === null){
                setLatestPub(OrderCriteria.TITLE)
                setOrderOfPodcasts(Order.ASC)
                performFilter()
            }
            else{
                setLatestPub(c.data.filter === "PublishedDate"?OrderCriteria.PUBLISHEDDATE:OrderCriteria.TITLE)
                setOrderOfPodcasts(c.data.ascending?Order.ASC:Order.DESC)
                c.data.title&&setSearchText(c.data.title)
            }
        })
    },[location])


    return <div className="p-5">
        <AddPodcast/>
        <div className="flex flex-col md:flex-row gap-3">
                <span className="relative  w-full md:w-1/3">
                    <input type="text" value={searchText}  onChange={v => setSearchText(v.target.value)}
                           className="border-gray-400 w-full pl-10 pt-1 pb-1 border-2 rounded-2xl"/>
                    <span className="absolute left-2 top-1.5 scale-90">
                        <MaginifyingGlassIcon/>
                    </span>
                </span>
            <select  className="border text-sm rounded-lg block p-2.5 bg-gray-700 border-gray-600 placeholder-gray-400 text-white focus:ring-blue-500 focus:border-blue-500"
                     onChange={(v)=> {
                       setLatestPub(v.target.value as OrderCriteria)
                     }} value={latestPub}>
                <option value={OrderCriteria.PUBLISHEDDATE}>{t('sort-by-published-date')}</option>
                <option value={OrderCriteria.TITLE}>{t('sort-by-title')}</option>
            </select>
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth="1.5"
                 stroke="currentColor" className={`${orderOfPodcasts==Order.DESC?'rotate-180':''} w-6 h-6`} onClick={()=>{setOrderOfPodcasts(orderOfPodcasts==Order.DESC?Order.ASC:Order.DESC)}}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M4.5 10.5L12 3m0 0l7.5 7.5M12 3v18"/>
            </svg>
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
            {!onlyFavorites&&podcasts.map((podcast)=>{

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
