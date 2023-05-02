import {Spinner} from "./Spinner";
import {useTranslation} from "react-i18next";
import {useDebounce} from "../utils/useDebounce";
import axios, {AxiosResponse} from "axios";
import {apiURL} from "../utils/Utilities";
import {AgnosticPodcastDataModel, GeneralModel, PodIndexModel} from "../models/PodcastAddModel";
import {setSearchedPodcasts} from "../store/CommonSlice";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {FC, useState} from "react";
import {setModalOpen} from "../store/ModalSlice";
import {enqueueSnackbar} from "notistack";
import {AddTypes} from "../models/AddTypes";

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
    return <div className="flex flex-col gap-4">
    <input value={searchText} placeholder={t('search-podcast')!}
    className={"border text-sm rounded-lg block w-full p-2.5 bg-gray-700 border-gray-600 placeholder-gray-400 text-white focus:ring-blue-500 focus:border-blue-500"}
    onChange={(v)=>setSearchText(v.target.value)}/>
    <div className="border-2 border-gray-600 rounded p-5 max-h-80 overflow-y-scroll">
        {loading?<div className="grid place-items-center"><Spinner className="w-12 h-12"/></div>:
                searchedPodcasts&& searchedPodcasts.map((podcast, index)=>{
                    return <div key={index}>
                    <div className="flex">
                    <div className="flex-1 grid grid-rows-2">
                        <div>{podcast.title}</div>
                        <div className="text-sm">{podcast.artist}</div>
                    </div>
                    <div>
                    <button className="fa fa-plus bg-blue-900 text-white p-3" onClick={()=>{
                        addPodcast({
                            trackId: podcast.id,
                            userId:1
                        })
                    }}></button>
                    </div>
                    </div>
                    <hr className="border-gray-500"/>
                        </div>
                })
        }
        </div>
        </div>
}
