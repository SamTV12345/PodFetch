import { FC, useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import axios from 'axios'
import { apiURL } from '../utils/Utilities'
import { FileItem, readFile } from '../utils/FileUtils'
import useOpmlImport from '../store/opmlImportSlice'
import { AddTypes } from '../models/AddTypes'
import { CustomButtonPrimary } from './CustomButtonPrimary'

type DragState = "none" | "allowed" | "invalid"

type OpmlAddProps = {
    selectedSearchType: AddTypes
}

export const OpmlAdd: FC<OpmlAddProps> = ({}) => {
    const opmlUploading = useOpmlImport(state => state.inProgress)
    const progress  = useOpmlImport(state => state.progress)
    const setInProgress = useOpmlImport(state => state.setInProgress)
    const fileInputRef = useRef<HTMLInputElement>(null)
    const [dragState, setDragState] = useState<DragState>("none")
    const [files, setFiles] = useState<FileItem[]>([])
    const [podcastsToUpload, setPodcastsToUpload] = useState<number>(0)
    const { t } = useTranslation()

    useEffect(() => {
        if (progress.length === podcastsToUpload) {
            setInProgress(false)
        }
    }, [progress])

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

    const uploadOpml = () => {
        let content = files[0].content
        const count = (content.match(/type="rss"/g) || []).length

        setPodcastsToUpload(count)

        axios.post(apiURL + '/podcast/opml', {
            content: files[0].content
        })
    }

    const handleDragOver = (e: React.DragEvent) => {
        e.preventDefault()
        e.dataTransfer.dropEffect = 'copy'
    }

    const handleDropColor = () => {
        switch (dragState) {
            case 'none':
                return ''
            case 'allowed':
                return 'bg-stone-100'
            case 'invalid':
                return 'border-solid border-red-500'
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

        setDragState('none')
    }

    return (
        <div className="flex flex-col gap-4 items-end">
            {/* Default state */
            files.length === 0 &&
                <>
                    <div
                        className={`flex flex-col justify-center gap-2 border border-dashed border-[--border-color] cursor-pointer p-4 text-center rounded-lg h-40 w-full hover:bg-[--input-bg-color] ${handleDropColor()}`}
                        onDragEnter={() => setDragState('allowed')}
                        onDragLeave={() => setDragState('none')}
                        onDragOver={handleDragOver}
                        onDrop={handleDrop}
                        onClick={handleClick}
                    >
                        <span className="material-symbols-outlined !text-4xl text-[--fg-secondary-icon-color]">upload</span>
                        <span className="text-sm text-[--fg-secondary-color]">{t('drag-here')}</span>
                    </div>
                    <input type={'file'} ref={fileInputRef} accept="application/xml, .opml" hidden onChange={(e)=>{
                        handleInputChanged(e)}
                    } />
                </>
            }
            {/* File(s) selected */
            files.length > 0 && !opmlUploading && files.length === 0 &&
                <div className="leading-[1.75] text-sm text-[--fg-color] w-full">
                    {t('following-file-uploaded')}
                    <div className="" onClick={() => {setFiles([])}}>
                        {files[0].name}<i className="ml-5 fa-solid cursor-pointer active:scale-90 fa-x text-red-700"></i>
                    </div>
                </div>
            }
            {/* Upload in progress */
            opmlUploading &&
                <div className="pt-4 pb-6 w-full">
                    <span className="block text-center text-sm text-[--fg-color]">
                        {t('progress')}: {progress.length}/{podcastsToUpload}
                    </span>

                    {podcastsToUpload > 0 && progress.length > 0 &&
                        <div className="bg-[--slider-bg-color] h-2.5 mt-2  rounded-full w-full">
                            <div className="bg-[--slider-fg-color] h-2.5 rounded-full" style={{width: `${(progress.length / podcastsToUpload) * 100}%`}}></div>
                            {!opmlUploading &&
                                <div>
                                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} className="w-6 h-6 text-slate-800">
                                        <path strokeLinecap="round" strokeLinejoin="round" d="M4.5 12.75l6 6 9-13.5" />
                                    </svg>
                                </div>
                            }
                        </div>
                    }
                </div>
            }

            <CustomButtonPrimary disabled={files.length === 0} onClick={() => {
                setInProgress(true)
                uploadOpml()
            }}>
                {t('upload-opml')}
            </CustomButtonPrimary>
        </div>
    )
}
