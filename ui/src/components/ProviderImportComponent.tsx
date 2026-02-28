import { FC, useState } from 'react'
import { useTranslation } from 'react-i18next'
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
import {$api} from "../utils/http";

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
    const addPodcastFromItunes = $api.useMutation('post', '/api/v1/podcasts/itunes')
    const addPodcastFromPodindex = $api.useMutation('post', '/api/v1/podcasts/podindex')
    const searchProviderMutation = $api.useMutation('get', '/api/v1/podcasts/{type_of}/{podcast}/search')

    const addPodcast = (podcast: AddPostPostModel) => {
        switch (selectedSearchType) {
            case "itunes": {
                addPodcastFromItunes.mutateAsync({
                    body: podcast
                })
                    .then(() => {
                        setModalOpen(false)
                        handleAddPodcast(200,
                            searchedPodcasts!.find((v) => v.id === podcast.trackId)?.title!, t)
                    })
                    .catch((err) => {
                        err.response && err.response.status && handleAddPodcast(err.response.status,
                            searchedPodcasts!.find((v) => v.id === podcast.trackId)?.title!, t)
                    })
                break
            }
            case "podindex": {
                addPodcastFromPodindex.mutateAsync({
                    body: podcast
                })
                    .then(() => {
                        setModalOpen(false)
                        handleAddPodcast(200,
                            searchedPodcasts!.find((v) => v.id === podcast.trackId)?.title!, t)
                    })
                    .catch((err) => {
                        err.response && err.response.status && handleAddPodcast(err.response.status,
                            searchedPodcasts!.find((v) => v.id === podcast.trackId)?.title!, t)
                    })
                break
            }
        }
    }

    useDebounce(() => {
        setLoading(true)
        selectedSearchType === 'itunes' ?
            searchProviderMutation.mutateAsync({
                params: {
                    path: {
                        type_of: 0,
                        podcast: searchText
                    }
                }
            }).then((v) => {
                    if (v && "resultCount" in v) {
                        const data = v
                        setLoading(false)
                        const agnosticModel: AgnosticPodcastDataModel[] = data!.results.map((podcast) => {
                            return {
                                title: podcast.collectionName!,
                                artist: podcast.artistName!,
                                id: podcast.trackId!,
                                imageUrl: podcast.artworkUrl600!
                            }
                        })
                        setSearchedPodcasts(agnosticModel)
                    }

            })
            : searchProviderMutation.mutateAsync({
                params: {
                    path: {
                        type_of: 1,
                        podcast: searchText
                    }
                }
            }).then((v) => {
                        if (v && "feeds" in v) {
                            setLoading(false)
                            let agnosticModel: AgnosticPodcastDataModel[] = v.feeds.map((podcast) => {
                                return {
                                    title: podcast.title!,
                                    artist: podcast.author!,
                                    id: podcast.id!,
                                    imageUrl: podcast.artwork!
                                }
                            })
                            setSearchedPodcasts(agnosticModel)
                        }

                })
    }, 2000, [searchText])

    return (
        <div className="flex flex-col gap-8">
            <span className="relative">
                <CustomInput type="text" value={searchText} placeholder={t('search-podcast')!} className="pl-10 w-full" onChange={(v) => setSearchText(v.target.value)} />

                <span className="material-symbols-outlined absolute left-2 top-2 ui-input-icon">search</span>
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
                                    <span className="font-bold leading-tight ui-text">{podcast.title}</span>
                                    <span className="leading-tight text-sm ui-text-muted">{podcast.artist}</span>
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
