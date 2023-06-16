import {FC, useState} from "react";
import {useTranslation} from "react-i18next"
import axios, {AxiosResponse} from "axios"
import {enqueueSnackbar} from "notistack"
import {useDebounce} from "../utils/useDebounce"
import {apiURL} from "../utils/Utilities"
import {useAppDispatch, useAppSelector} from "../store/hooks"
import {setSearchedPodcasts} from "../store/CommonSlice"
import {setModalOpen} from "../store/ModalSlice"
import {AddTypes} from "../models/AddTypes"
import {AgnosticPodcastDataModel, GeneralModel, PodIndexModel} from "../models/PodcastAddModel"
import {ButtonSecondary} from './ButtonSecondary'
import {CustomInput} from './CustomInput'
import {Spinner} from "./Spinner"
import "material-symbols/outlined.css"

type ProviderImportComponent = {
    selectedSearchType: AddTypes
}

type AddPostPostModel = {
    trackId: number,
    userId: number
}

export const ProviderImportComponent:FC<ProviderImportComponent> = ({selectedSearchType})=>{
    const dispatch = useAppDispatch()
    const searchedPodcasts = useAppSelector(state=>state.common.searchedPodcasts)
    const [loading, setLoading] = useState<boolean>()
    const [searchText, setSearchText] = useState<string>("")

    const addPodcast = (podcast:AddPostPostModel)=>{
        axios.post(apiURL+"/podcast/"+selectedSearchType,podcast).then(()=>{
            dispatch(setModalOpen(false))
        }).catch(()=>enqueueSnackbar(t('not-admin-or-uploader'),{variant: "error"}))
    }
    useDebounce(()=>{
            setLoading(true)
            selectedSearchType === "itunes"?
                axios.get(apiURL+"/podcasts/0/"+searchText+"/search")
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
            </div>
        :searchedPodcasts&&
            <ul className="flex flex-col gap-6 max-h-80 pr-3 overflow-y-scroll">
                {searchedPodcasts.map((podcast, index)=>{
                    return <li key={index} className="flex gap-4 items-center">
                        <div className="flex-1 flex flex-col gap-1">
                            <span className="font-bold leading-tight text-stone-900">{podcast.title}</span>
                            <span className="leading-tight text-sm text-stone-500">{podcast.artist}</span>
                        </div>
                        <div>
                            <ButtonSecondary className="flex" onClick={()=>{
                                addPodcast({
                                    trackId: podcast.id,
                                    userId:1
                                })
                            }}><span className="material-symbols-outlined leading-[0.875rem]">add</span></ButtonSecondary>
                        </div>
                    </li>
                })}
            </ul>
        }
    </div>
}
