import {Modal} from "./Modal";
import {useState} from "react";
import {useDebounce} from "../utils/useDebounce";
import {apiURL} from "../utils/Utilities";
import axios, {AxiosResponse} from "axios";
import {AgnosticPodcastDataModel, GeneralModel, PodIndexModel} from "../models/PodcastAddModel";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {setSearchedPodcasts} from "../store/CommonSlice";
import {setModalOpen} from "../store/ModalSlice";
import {useTranslation} from "react-i18next";

export const AddPodcast = ()=>{
    const [searchText, setSearchText] = useState<string>("")
    const dispatch = useAppDispatch()
    const searchedPodcasts = useAppSelector(state=>state.common.searchedPodcasts)
    const {t} = useTranslation()
    const [selectedSearchType, setSelectedSearchType] = useState<"itunes"|"podindex">("itunes")
    const configModel = useAppSelector(state=>state.common.configModel)

    type AddPostPostModel = {
        trackId: number,
        userId: number
    }
    useDebounce(()=>{
        selectedSearchType === "itunes"?
            axios.get(apiURL+"/podcasts/0/"+searchText+"/search")
                .then((v:AxiosResponse<GeneralModel>)=>{

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

    const addPodcast = (podcast:AddPostPostModel)=>{
        axios.post(apiURL+"/podcast/"+selectedSearchType,podcast).then(()=>{
            dispatch(setModalOpen(false))
        })
    }

    return <Modal onCancel={()=>{}} onAccept={()=>{}} headerText={t('add-podcast')} onDelete={()=>{}}  cancelText={"Abbrechen"} acceptText={"Hinzufügen"} >
        <div>
            {configModel?.podindexConfigured&&<ul id="podcast-add-decider" className="flex flex-wrap text-sm font-medium text-center text-gray-500 border-b border-gray-200 dark:border-gray-700 dark:text-gray-400">
                <li className="mr-2">
                    <div className={`cursor-pointer inline-block p-4 rounded-t-lg ${selectedSearchType=== "itunes"&& 'active'}`} onClick={()=>setSelectedSearchType("itunes")}>iTunes</div>
                </li>
                <li className="mr-2">
                    <div
                       className={`cursor-pointer inline-block p-4 rounded-t-lg hover:bg-gray-800 hover:text-gray-300 ${selectedSearchType === "podindex" && 'active'}`} onClick={()=>setSelectedSearchType("podindex")}>PodIndex</div>
                </li>
            </ul>}
            <div className="flex flex-col gap-4">
                <input value={searchText} placeholder={t('search-podcast')!}
                       className={"border text-sm rounded-lg block w-full p-2.5 bg-gray-700 border-gray-600 placeholder-gray-400 text-white focus:ring-blue-500 focus:border-blue-500"}
                       onChange={(v)=>setSearchText(v.target.value)}/>
                <div className="border-2 border-gray-600 rounded p-5 max-h-80 overflow-y-scroll">
                {
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
        </div>
    </Modal>
}
