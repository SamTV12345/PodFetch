import {FC, useEffect} from "react"
import {useLocation} from "react-router-dom"
import {useTranslation} from "react-i18next"
import axios, {AxiosResponse} from "axios"
import {useDebounce} from "../utils/useDebounce"
import {
    apiURL,
    getFiltersDefault,
    OrderCriteriaSortingType, TIME_ASCENDING, TIME_DESCENDING,
    TITLE_ASCENDING,
    TITLE_DESCENDING
} from "../utils/Utilities"
import {useAppDispatch, useAppSelector} from "../store/hooks"
import {Podcast, setFilters, setPodcasts} from "../store/CommonSlice"
import {setModalOpen} from "../store/ModalSlice"
import {Order} from "../models/Order"
import {Filter} from "../models/Filter"
import {Card} from "../components/Card"
import {AddPodcast} from "../components/AddPodcast"
import {ButtonPrimary} from "../components/ButtonPrimary"
import {CustomSelect} from "../components/CustomSelect"
import {Heading1} from "../components/Heading1"
import {Input} from "../components/Input"
import "material-symbols/outlined.css"

interface PodcastsProps {
    onlyFavorites?: boolean
}

export const Podcasts:FC<PodcastsProps> = ({onlyFavorites})=>{
    const podcasts = useAppSelector(state=>state.common.podcasts)
    const dispatch = useAppDispatch()
    let location = useLocation();
    const {t} = useTranslation()
    const filters = useAppSelector(state=>state.common.filters)
    const refreshAllPodcasts = ()=>{
        axios.post(apiURL+"/podcast/all")
    }

    const performFilter =()=>{
        axios.get(apiURL + "/podcasts/search", {
            params: {
                title: filters?.title,
                order: filters?.ascending?Order.ASC:Order.DESC,
                orderOption: filters?.filter=="PUBLISHEDDATE"?"PUBLISHEDDATE":"TITLE",
                favoredOnly: !!onlyFavorites
            }
        })
            .then((v: AxiosResponse<Podcast[]>) => {
                dispatch(setPodcasts(v.data))
            })
    }

    const orderOptions = [
        { value: JSON.stringify(TIME_ASCENDING), label: '1.1.-31.12' },
        { value: JSON.stringify(TIME_DESCENDING), label: '31.12-1.1' },
        { value: JSON.stringify(TITLE_ASCENDING), label: 'A-Z' },
        { value: JSON.stringify(TITLE_DESCENDING), label: 'Z-A' }
    ];

    useDebounce(()=> {
        performFilter();
    },500, [filters])

    useEffect(()=>{
        axios.get(apiURL+"/podcasts/filter")
            .then((c:AxiosResponse<Filter>)=>{
            if(c.data === null){
                dispatch(setFilters(getFiltersDefault()))
            }
            else{
                dispatch(setFilters({
                    ascending: c.data.ascending,
                    filter: c.data.filter,
                    title: c.data.title,
                    username: c.data.username,
                    onlyFavored: c.data.onlyFavored
                }))
            }
        })
    },[location])

    return <div className="px-8">
        <AddPodcast/>

        <div className="flex justify-between mb-10">
            <div className="flex gap-2 items-center">
                <Heading1>{t('all-subscriptions')}</Heading1>

                <span className="material-symbols-outlined cursor-pointer text-stone-800 hover:text-stone-600" onClick={()=>{
                    refreshAllPodcasts()
                }}>refresh</span>
            </div>

            <ButtonPrimary className="flex items-center" onClick={()=>{
                dispatch(setModalOpen(true))
            }}>
                <span className="material-symbols-outlined">add</span> {t('add-new')}
            </ButtonPrimary>
        </div>

        <div className="flex gap-4 mb-10">
            <span className="flex-1 relative">
                <Input type="text" placeholder={t('search')!} value={filters?.title} className="pl-10" onChange={v => dispatch(setFilters({...filters as Filter,title: v.target.value}))}/>

                <span className="material-symbols-outlined absolute left-2 top-2 text-stone-500">search</span>
            </span>

            <CustomSelect icon="sort" onChange={(v)=> {
                let converted = JSON.parse(v) as OrderCriteriaSortingType
                dispatch(setFilters({...filters as Filter, filter: converted.sorting, ascending: converted.ascending}))
            }} options={orderOptions} value={JSON.stringify({sorting: filters?.filter?.toUpperCase(), ascending: filters?.ascending})} />
        </div>

        <div className="grid grid-cols-1 gap-x-8 gap-y-12 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6">
            {!onlyFavorites&&podcasts.map((podcast)=>{
                return <Card podcast={podcast} key={podcast.id}/>
            })}

            {onlyFavorites&&podcasts.filter(podcast=>podcast.favorites).map((podcast)=>{
                return <Card podcast={podcast} key={podcast.id}/>
            })}
        </div>
    </div>
}
