import {Modal} from "./Modal";
import {useEffect, useRef, useState} from "react";
import {useDebounce} from "../utils/useDebounce";
import {apiURL} from "../utils/Utilities";
import axios, {AxiosResponse} from "axios";
import {AgnosticPodcastDataModel, GeneralModel, PodIndexModel} from "../models/PodcastAddModel";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {setSearchedPodcasts} from "../store/CommonSlice";
import {setModalOpen} from "../store/ModalSlice";
import {useTranslation} from "react-i18next";
import {FileItem, readFile} from "../utils/FileUtils";

export const AddPodcast = ()=>{
    const [searchText, setSearchText] = useState<string>("")
    const dispatch = useAppDispatch()
    const searchedPodcasts = useAppSelector(state=>state.common.searchedPodcasts)
    const {t} = useTranslation()
    const [selectedSearchType, setSelectedSearchType] = useState<"itunes"|"podindex"|"opml">("itunes")
    const configModel = useAppSelector(state=>state.common.configModel)
    const [dragState, setDragState] = useState<DragState>("none")
    type DragState = "none" | "allowed" | "invalid"
    const fileInputRef = useRef<HTMLInputElement>(null)
    const [files, setFiles] = useState<FileItem[]>([])

    type AddPostPostModel = {
        trackId: number,
        userId: number
    }

    const handleDragOver = (e: React.DragEvent) => {
        e.preventDefault()
        e.dataTransfer.dropEffect = "copy"
    }
    const handleDropColor =()=> {
        switch (dragState) {
            case "none":
                return "border-double"
            case "allowed":
                return "border-dashed"
            case "invalid":
                return "border-solid border-red-500"
        }
    }

    const uploadOpml = ()=>{
        axios.post(apiURL+"/podcast/opml", {
            content: files[0].content
        })
            .then((v)=>{
                console.log(v)
            })
            .catch((e)=>{
                console.log(e)
            })
    }

    const handleDrop = (e: React.DragEvent) => {
        e.preventDefault()

        const fileList: Promise<FileItem>[] = []
        for (const f of e.dataTransfer.files) {
            fileList.push(readFile(f))
        }
        Promise.all(fileList).then(e => {
            setFiles(e)
        })
        setDragState("none")
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

    const handleClick = () => {
        fileInputRef.current?.click()
    }

    const handleInputChanged = (e: any) => {
        uploadFiles(e.target.files[0])
    }


    const uploadFiles = (files: File) => {
        const fileList: Promise<FileItem>[] = []
        fileList.push(readFile(files))

        Promise.all(fileList).then(e => {
            setFiles(e)
        })
    }

    return <Modal onCancel={()=>{}} onAccept={()=>{}} headerText={t('add-podcast')} onDelete={()=>{}}  cancelText={"Abbrechen"} acceptText={"Hinzufügen"} >
        <div>
            <ul id="podcast-add-decider" className="flex flex-wrap text-sm font-medium text-center text-gray-500 border-b border-gray-200 dark:border-gray-700 dark:text-gray-400">
                <li className="mr-2">
                    <div className={`cursor-pointer inline-block p-4 rounded-t-lg ${selectedSearchType=== "itunes"&& 'active'}`} onClick={()=>setSelectedSearchType("itunes")}>iTunes</div>
                </li>
                {configModel?.podindexConfigured&&<li className="mr-2">
                    <div
                       className={`cursor-pointer inline-block p-4 rounded-t-lg hover:bg-gray-800 hover:text-gray-300 ${selectedSearchType === "podindex" && 'active'}`} onClick={()=>setSelectedSearchType("podindex")}>PodIndex</div>
                </li>}
                <li>
                    <div
                        className={`cursor-pointer inline-block p-4 rounded-t-lg hover:bg-gray-800 hover:text-gray-300 ${selectedSearchType === "opml" && 'active'}`} onClick={()=>setSelectedSearchType("opml")}>OPML-File</div>
                </li>
            </ul>
            {selectedSearchType!=="opml"&&<div className="flex flex-col gap-4">
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
            </div>}
            {
                selectedSearchType==="opml"&&<div className="flex flex-col gap-4">
            {files.length===0&&<><div className={`p-4 border-4 ${handleDropColor()} border-dashed border-gray-500 text-center w-full h-40 grid place-items-center cursor-pointer`}
                         onDragEnter={() => setDragState("allowed")}
                         onDragLeave={() => setDragState("none")}
                         onDragOver={handleDragOver} onDrop={handleDrop}
                         onClick={handleClick}>
                        {t('drag-here')}
                    </div>
                    <input type={"file"} ref={fileInputRef} accept="application/xml" hidden onChange={(e)=>{
                        handleInputChanged(e)}
                    } /></>}
                    {
                        files.length > 0 && <div>
                            {t('following-file-uploaded')}
                            <div className="ml-4" onClick={()=>{setFiles([])}}>{files[0].name}<i className="ml-5 fa-solid cursor-pointer active:scale-90 fa-x text-red-700"></i></div>
                        </div>
                    }
                    <div className="flex">
                        <div className="flex-1"/>
                        <button className="bg-blue-800 p-2 disabled:bg-gray-800" disabled={files.length==0} onClick={()=>{uploadOpml()}}>Upload OPML</button>
                    </div>
                </div>
            }
        </div>
    </Modal>
}
