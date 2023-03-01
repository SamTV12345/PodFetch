import {Modal} from "./Modal";
import {useState} from "react";
import {useDebounce} from "../utils/useDebounce";
import {apiURL} from "../utils/Utilities";
import axios, {AxiosResponse} from "axios";
import {GeneralModel} from "../models/PodcastAddModel";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {setSearchedPodcasts} from "../store/CommonSlice";
import {setModalOpen} from "../store/ModalSlice";

export const AddPodcast = ()=>{
    const [searchText, setSearchText] = useState<string>("")
    const dispatch = useAppDispatch()
    const searchedPodcasts = useAppSelector(state=>state.common.searchedPodcasts)


    type AddPostPostModel = {
        trackId: number,
        userId: number
    }
    useDebounce(()=>{
            axios.get(apiURL+"/podcasts/"+searchText+"/search")
                .then((v:AxiosResponse<GeneralModel>)=>{
                    dispatch(setSearchedPodcasts(v.data))
                })
        },
        2000,[searchText])

    const addPodcast = (podcast:AddPostPostModel)=>{
        axios.post(apiURL+"/podcast",podcast).then((v)=>{
            dispatch(setModalOpen(false))
        })
    }

    return <Modal onCancel={()=>{}} onAccept={()=>{}} headerText="Podcast hinzufügen" onDelete={()=>{}}  cancelText={"Abbrechen"} acceptText={"Hinzufügen"} >
        <div>
            <div className="flex flex-col gap-4">
                <input value={searchText} placeholder="Podcast suchen"
                       className={"border text-sm rounded-lg block w-full p-2.5 bg-gray-700 border-gray-600 placeholder-gray-400 text-white focus:ring-blue-500 focus:border-blue-500"}
                       onChange={(v)=>setSearchText(v.target.value)}/>
                <div className="border-2 border-gray-600 rounded p-5 max-h-80 overflow-y-scroll">
                {
                    searchedPodcasts&& searchedPodcasts.results.map((podcast, index)=>{
                        return <div key={index}>
                            <div className="flex">
                                <div className="flex-1 grid grid-rows-2">
                                    <div>{podcast.collectionName}</div>
                                    <div className="text-sm">{podcast.artistName}</div>
                                </div>
                                <div>
                                <button className="fa fa-plus bg-blue-900 text-white p-3" onClick={()=>{
                                    addPodcast({
                                        trackId: podcast.trackId,
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
