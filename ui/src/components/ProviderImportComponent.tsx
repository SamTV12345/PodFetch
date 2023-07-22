import {FC, useState} from "react";
import {useTranslation} from "react-i18next"
import axios, {AxiosError, AxiosResponse} from "axios"
import {useDebounce} from "../utils/useDebounce"
import {apiURL} from "../utils/Utilities"
import {useAppDispatch, useAppSelector} from "../store/hooks"
import {setSearchedPodcasts} from "../store/CommonSlice"
import {setModalOpen} from "../store/ModalSlice"
import {AddTypes} from "../models/AddTypes"
import {AgnosticPodcastDataModel, GeneralModel, PodIndexModel} from "../models/PodcastAddModel"
import {CustomButtonSecondary} from './CustomButtonSecondary'
import {CustomInput} from './CustomInput'
import {Spinner} from "./Spinner"
import "material-symbols/outlined.css"
import {handleAddPodcast} from "../utils/ErrorSnackBarResponses";

type ProviderImportComponent = {
    selectedSearchType: AddTypes
}

export type AddPostPostModel = {
    trackId: number,
    userId: number
}

export const ProviderImportComponent:FC<ProviderImportComponent> = ({selectedSearchType})=>{
    const dispatch = useAppDispatch()
    const searchedPodcasts = useAppSelector(state=>state.common.searchedPodcasts)
    const [loading, setLoading] = useState<boolean>()
    const [searchText, setSearchText] = useState<string>("")

    const addPodcast = (podcast:AddPostPostModel)=>{
        axios.post(apiURL+"/podcast/"+selectedSearchType,podcast)
            .then((err:any)=> {
                dispatch(setModalOpen(false))
                handleAddPodcast(err.response ? err.response.status: null, searchedPodcasts!.find((v)=>v.id === podcast.trackId)?.title!, t)
            })
            .catch((err:AxiosError)=>{
                handleAddPodcast(err.response ? err.response.status: null, searchedPodcasts!.find((v)=>v.id === podcast.trackId)?.title!, t)
            })
    }

    useDebounce(()=>{
            setLoading(true)
            selectedSearchType === "itunes"?
                axios.get(apiURL+"/podcasts/0/"+encodeURI(searchText)+"/search")
                    .then((v:AxiosResponse<GeneralModel>)=>{
                        setLoading(false)
                        const agnosticModel:AgnosticPodcastDataModel[] = v.data.results.map((podcast)=>{
                            return {
                                title: podcast.collectionName,
                                artist: podcast.artistName,
                                id: podcast.trackId,
                                imageUrl: podcast.artworkUrl600
                            }
                        })

                        dispatch(setSearchedPodcasts(agnosticModel))
                    }):axios.get(apiURL+"/podcasts/1/"+searchText+"/search")
                    .then((v:AxiosResponse<PodIndexModel>)=>{
                        setLoading(false)
                        let agnosticModel: AgnosticPodcastDataModel[] = v.data.feeds.map((podcast)=>{
                            return {
                                title: podcast.title,
                                artist: podcast.author,
                                id: podcast.id,
                                imageUrl: podcast.artwork
                            }
                        })
                        dispatch(setSearchedPodcasts(agnosticModel))
                    })
        },
        2000,[searchText])

    const {t} = useTranslation()

    return <div className="flex flex-col gap-8">
        <span className="relative">
            <CustomInput type="text" value={searchText} placeholder={t('search-podcast')!} className="pl-10 w-full" onChange={(v)=>setSearchText(v.target.value)}/>

            <span className="material-symbols-outlined absolute left-2 top-2 text-stone-500">search</span>
        </span>

        {loading?
            <div className="grid place-items-center">
                <Spinner className="w-12 h-12"/>
            </div> :searchedPodcasts &&
            <ul className="flex flex-col gap-6 max-h-80 pr-3 overflow-y-auto">
                {searchedPodcasts.map((podcast, index)=>{
                    return <li key={index} className="flex gap-4 items-center">
                        <div className="flex-1 flex flex-col gap-1">
                            <span className="font-bold leading-tight text-stone-900">{podcast.title}</span>
                            <span className="leading-tight text-sm text-stone-500">{podcast.artist}</span>
                        </div>
                        <div>
                            <CustomButtonSecondary className="flex" onClick={()=>{
                                addPodcast({
                                    trackId: podcast.id,
                                    userId:1
                                })
                            }}><span className="material-symbols-outlined leading-[0.875rem]">add</span></CustomButtonSecondary>
                        </div>
                    </li>
                })}
            </ul>
        }
    </div>
}
