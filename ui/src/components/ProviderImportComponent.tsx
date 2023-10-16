import { FC, useState } from 'react'
import { useTranslation } from 'react-i18next'
import axios, { AxiosError, AxiosResponse } from 'axios'
import { apiURL } from '../utils/Utilities'
import { useDebounce } from '../utils/useDebounce'
import { handleAddPodcast } from '../utils/ErrorSnackBarResponses'
import useCommon from '../store/CommonSlice'
import { AddTypes } from '../models/AddTypes'
import { AgnosticPodcastDataModel, GeneralModel, PodIndexModel } from '../models/PodcastAddModel'
import { CustomButtonSecondary } from './CustomButtonSecondary'
import { CustomInput } from './CustomInput'
import { Spinner } from './Spinner'
import 'material-symbols/outlined.css'
import useModal from "../store/ModalSlice";

type ProviderImportComponent = {
    selectedSearchType: AddTypes
}

export type AddPostPostModel = {
    trackId: number,
    userId: number
}

export const ProviderImportComponent: FC<ProviderImportComponent> = ({ selectedSearchType }) => {
    const setSearchedPodcasts = useCommon(state => state.setSearchedPodcasts)
    const searchedPodcasts = useCommon(state => state.searchedPodcasts)
    const [loading, setLoading] = useState<boolean>()
    const [searchText, setSearchText] = useState<string>('')
    const { t } = useTranslation()
    const setModalOpen = useModal(state => state.setOpenModal)

    const addPodcast = (podcast: AddPostPostModel) => {
        axios.post(apiURL + '/podcast/' + selectedSearchType, podcast)
            .then((err: any) => {
                setModalOpen(false)
                err.response.status && handleAddPodcast(err.response.status,
                    searchedPodcasts!.find((v) => v.id === podcast.trackId)?.title!, t)
            })
            .catch((err: AxiosError) => {
                err.response && err.response.status && handleAddPodcast(err.response.status,
                    searchedPodcasts!.find((v) => v.id === podcast.trackId)?.title!, t)
            })
    }

    useDebounce(() => {
        setLoading(true)
        selectedSearchType === 'itunes' ?
            axios.get(apiURL + '/podcasts/0/' + encodeURI(searchText) + '/search')
                .then((v: AxiosResponse<GeneralModel>) => {
                    setLoading(false)
                    const agnosticModel: AgnosticPodcastDataModel[] = v.data.results.map((podcast) => {
                        return {
                            title: podcast.collectionName,
                            artist: podcast.artistName,
                            id: podcast.trackId,
                            imageUrl: podcast.artworkUrl600
                        }
                    })

                    setSearchedPodcasts(agnosticModel)
                })
            : axios.get(apiURL + '/podcasts/1/' + searchText + '/search')
                .then((v: AxiosResponse<PodIndexModel>) => {
                    setLoading(false)
                    let agnosticModel: AgnosticPodcastDataModel[] = v.data.feeds.map((podcast) => {
                        return {
                            title: podcast.title,
                            artist: podcast.author,
                            id: podcast.id,
                            imageUrl: podcast.artwork
                        }
                    })
                    setSearchedPodcasts(agnosticModel)
                })
    }, 2000, [searchText])

    return (
        <div className="flex flex-col gap-8">
            <span className="relative">
                <CustomInput type="text" value={searchText} placeholder={t('search-podcast')!} className="pl-10 w-full" onChange={(v) => setSearchText(v.target.value)} />

                <span className="material-symbols-outlined absolute left-2 top-2 text-[--input-icon-color]">search</span>
            </span>

            {loading ? (
                <div className="grid place-items-center">
                    <Spinner className="w-12 h-12"/>
                </div>
            ) : searchedPodcasts && (
                <ul className="flex flex-col gap-6 max-h-80 pr-3 overflow-y-auto">
                    {searchedPodcasts.map((podcast, index) => {
                        return (
                            <li key={index} className="flex gap-4 items-center">
                                <div className="flex-1 flex flex-col gap-1">
                                    <span className="font-bold leading-tight text-[--fg-color]">{podcast.title}</span>
                                    <span className="leading-tight text-sm text-[--fg-secondary-color]">{podcast.artist}</span>
                                </div>
                                <div>
                                    <CustomButtonSecondary className="flex" onClick={() => {
                                        addPodcast({
                                            trackId: podcast.id,
                                            userId:1
                                        })
                                    }}><span className="material-symbols-outlined leading-[0.875rem]">add</span></CustomButtonSecondary>
                                </div>
                            </li>
                        )
                    })}
                </ul>
            )}
        </div>
    )
}
