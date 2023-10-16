import { FC, useEffect, useMemo } from 'react'
import { useLocation } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import axios, { AxiosResponse } from 'axios'
import { useDebounce } from '../utils/useDebounce'
import {
    apiURL,
    getFiltersDefault,
    OrderCriteriaSortingType, TIME_ASCENDING, TIME_DESCENDING,
    TITLE_ASCENDING,
    TITLE_DESCENDING
} from '../utils/Utilities'
import useCommon, { Podcast,  } from '../store/CommonSlice'
import { Order } from '../models/Order'
import { Filter } from '../models/Filter'
import { AddPodcastModal } from '../components/AddPodcastModal'
import { CustomButtonPrimary } from '../components/CustomButtonPrimary'
import { CustomInput } from '../components/CustomInput'
import { CustomSelect } from '../components/CustomSelect'
import { Heading1 } from '../components/Heading1'
import { PodcastCard } from '../components/PodcastCard'
import 'material-symbols/outlined.css'
import useModal from "../store/ModalSlice";

interface PodcastsProps {
    onlyFavorites?: boolean
}

const orderOptions = [
    { value: JSON.stringify(TIME_ASCENDING), label: '1.1.-31.12' },
    { value: JSON.stringify(TIME_DESCENDING), label: '31.12-1.1' },
    { value: JSON.stringify(TITLE_ASCENDING), label: 'A-Z' },
    { value: JSON.stringify(TITLE_DESCENDING), label: 'Z-A' }
]

export const Podcasts: FC<PodcastsProps> = ({ onlyFavorites }) => {
    const filters = useCommon(state => state.filters)
    const podcasts = useCommon(state => state.podcasts)
    let location = useLocation();
    const { t } = useTranslation()
    const setModalOpen = useModal(state => state.setOpenModal)
    const setFilters = useCommon(state => state.setFilters)
    const setPodcasts = useCommon(state => state.setPodcasts)

    const memorizedSelection = useMemo(() => {
        return JSON.stringify({sorting: filters?.filter?.toUpperCase(), ascending: filters?.ascending})
    }, [filters])

    const refreshAllPodcasts = () => {
        axios.post(apiURL + '/podcast/all')
    }

    const performFilter = () => {
        if (filters === undefined) {
            return
        }

        axios.get(apiURL + '/podcasts/search', {
            params: {
                title: filters?.title,
                order: filters?.ascending?Order.ASC:Order.DESC,
                orderOption: filters?.filter?.toUpperCase(),
                favoredOnly: !!onlyFavorites
            }
        })
            .then((v: AxiosResponse<Podcast[]>) => {
                setPodcasts(v.data)
            })
    }

    useDebounce(() => {
        performFilter()
    },500, [filters])

    useEffect(() => {
        axios.get(apiURL + '/podcasts/filter')
            .then((c: AxiosResponse<Filter>) => {
                if (c.data === null) {
                    setFilters(getFiltersDefault())
                } else {
                    setFilters({
                        ascending: c.data.ascending,
                        filter: c.data.filter,
                        title: c.data.title,
                        username: c.data.username,
                        onlyFavored: c.data.onlyFavored
                    })
                }
            })
    }, [location])

    return (
        <div>
            <AddPodcastModal />

            {/* Title and Add button */}
            <div className="flex flex-col xs:flex-row items-start xs:items-center justify-between gap-4 mb-10">
                <div className="flex gap-2 items-center">
                    <Heading1>{t('all-subscriptions')}</Heading1>

                    <span className="material-symbols-outlined cursor-pointer text-[--fg-icon-color] hover:text-[--fg-icon-color-hover]" onClick={()=>{
                        refreshAllPodcasts()
                    }}>refresh</span>
                </div>

                <CustomButtonPrimary className="flex items-center" onClick={() => {
                   setModalOpen(true)
                }}>
                    <span className="material-symbols-outlined leading-[0.875rem]">add</span> {t('add-new')}
                </CustomButtonPrimary>
            </div>

            {/* Search/sort */}
            <div className="flex flex-col md:flex-row gap-4 mb-10">
                <span className="flex-1 relative">
                    <CustomInput className="pl-10 w-full" type="text" onChange={v =>
                        setFilters({...filters as Filter,title: v.target.value})} placeholder={t('search')!} value={filters?.title || ''} />

                    <span className="material-symbols-outlined absolute left-2 top-2 text-[--input-icon-color]">search</span>
                </span>

                <CustomSelect iconName="sort" onChange={(v) => {
                    let converted = JSON.parse(v) as OrderCriteriaSortingType
                    setFilters({...filters as Filter, filter: converted.sorting, ascending: converted.ascending})
                }} options={orderOptions} placeholder={t('sort-by')} value={memorizedSelection} />
            </div>

            {/* Podcast list */}
            <div className="grid grid-cols-1 xs:grid-cols-2 sm:grid-cols-3 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 gap-x-8 gap-y-12">
                {!onlyFavorites && podcasts.map((podcast) => {
                    return <PodcastCard podcast={podcast} key={podcast.id} />
                })}

                {onlyFavorites && podcasts.filter(podcast => podcast.favorites).map((podcast) => {
                    return <PodcastCard podcast={podcast} key={podcast.id} />
                })}
            </div>
        </div>
    )
}
