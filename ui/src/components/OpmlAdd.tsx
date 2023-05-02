import {AddTypes} from "../models/AddTypes";
import {setInProgress} from "../store/opmlImportSlice";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {FC, useEffect, useRef, useState} from "react";
import {FileItem, readFile} from "../utils/FileUtils";
import axios from "axios";
import {apiURL} from "../utils/Utilities";
import {useTranslation} from "react-i18next";

type OpmlAddProps = {
    selectedSearchType: AddTypes
}

export const OpmlAdd:FC<OpmlAddProps> = ({})=>{
    const opmlUploading = useAppSelector(state=>state.opmlImport.inProgress)
    const [files, setFiles] = useState<FileItem[]>([])
    const progress  = useAppSelector(state => state.opmlImport.progress)
    const [podcastsToUpload, setPodcastsToUpload] = useState<number>(0)
    const dispatch = useAppDispatch()
    const fileInputRef = useRef<HTMLInputElement>(null)
    const [dragState, setDragState] = useState<DragState>("none")
    type DragState = "none" | "allowed" | "invalid"
    const {t} = useTranslation()
    useEffect(()=>{
        if (progress.length===podcastsToUpload){
            dispatch(setInProgress(false))
        }
    },[progress])


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

    const uploadOpml = ()=>{
        let content = files[0].content
        const count = (content.match(/type="rss"/g) || []).length;
        setPodcastsToUpload(count)
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


    return <div className="flex flex-col gap-4">
        {files.length===0&&<><div className={`p-4 border-4 ${handleDropColor()} border-dashed border-gray-500 text-center w-full h-40 grid place-items-center cursor-pointer`}
                                  onDragEnter={() => setDragState("allowed")}
                                  onDragLeave={() => setDragState("none")}
                                  onDragOver={handleDragOver} onDrop={handleDrop}
                                  onClick={handleClick}>
            {t('drag-here')}
        </div>
            <input type={"file"} ref={fileInputRef} accept="application/xml, .opml" hidden onChange={(e)=>{
                handleInputChanged(e)}
            } /></>}
        {
            files.length > 0&& !opmlUploading && files.length===0 && <div>
                {t('following-file-uploaded')}
                <div className="ml-4" onClick={()=>{setFiles([])}}>{files[0].name}<i className="ml-5 fa-solid cursor-pointer active:scale-90 fa-x text-red-700"></i></div>
            </div>
        }
        {
            opmlUploading&& <>
                <div className="mt-4">
                    {t('progress')}: {progress.length}/{podcastsToUpload}
                </div>{ podcastsToUpload>0 && progress.length>0&&<div className="mt-2 w-full rounded-full h-2.5 bg-gray-700">

                <div className="bg-blue-600 h-2.5 rounded-full" style={{width:`${(progress.length/podcastsToUpload)*100}%`}}></div>
                {
                    !opmlUploading && <div>
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} className="w-6 h-6 text-slate-800">
                            <path strokeLinecap="round" strokeLinejoin="round" d="M4.5 12.75l6 6 9-13.5" />
                        </svg>
                    </div>
                }
            </div>
            }
            </>
        }


        <div className="flex">
            <div className="flex-1"/>
            <button className="bg-blue-800 p-2 disabled:bg-gray-800" disabled={files.length==0} onClick={()=>{
                dispatch(setInProgress(true))
                uploadOpml()}}>{t('upload-opml')}</button>
        </div>
    </div>
}
