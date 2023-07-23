import {FC, useEffect, useState} from "react"
import {createPortal} from "react-dom"
import {useTranslation} from "react-i18next"
import axios, {AxiosResponse} from "axios"
import {enqueueSnackbar} from "notistack"
import {useAppDispatch, useAppSelector} from "../store/hooks"
import {apiURL} from "../utils/Utilities"
import {CustomButtonPrimary} from "./CustomButtonPrimary"
import {Heading2} from "./Heading2"
import "material-symbols/outlined.css"
import {PlaylistDto, PlaylistDtoPost, PlaylistDtoPut, PlaylistItem} from "../models/Playlist";
import {setCreatePlaylistOpen, setPlaylist} from "../store/PlaylistSlice";
import {PlaylistData} from "./PlaylistData";
import {PlaylistSearchEpisode} from "./PlaylistSearchEpisode";




export const CreatePlaylistModal = () => {
    const dispatch = useAppDispatch()
    const playListOpen = useAppSelector(state=>state.playlist.createPlaylistOpen)
    const {t} = useTranslation()
    const playlists = useAppSelector(state=>state.playlist.playlist)
    const currentPlaylistToEdit = useAppSelector(state=>state.playlist.currentPlaylistToEdit)
    const [stage,setStage] = useState<number>(0)
    const createPlaylistOpen = useAppSelector(state=>state.playlist.createPlaylistOpen)

    {/* Reset to where the user opens the playlist again*/}
    useEffect(() => {
        createPlaylistOpen && setStage(0)
    }, [createPlaylistOpen]);

    const handlePlaylistCreateOrUpdate = ()=>{
        const itemsMappedToIDs = currentPlaylistToEdit!.items.map(item=>{
            return {
                episode: item.id
            } satisfies PlaylistItem
        })


        if (currentPlaylistToEdit && currentPlaylistToEdit.id !== -1){
            axios.put(apiURL+"/playlist/"+currentPlaylistToEdit.id, {
                id: currentPlaylistToEdit.id,
                name: currentPlaylistToEdit.name,
                items: itemsMappedToIDs
            } satisfies PlaylistDtoPut)
                .then((e:AxiosResponse<PlaylistDto>)=>{
                    dispatch(setPlaylist(playlists.map(p=>p.id===e.data.id?e.data:p)))
                    enqueueSnackbar(t('updated-playlist'), {variant: "success"})
                })
            return
        }

        if (currentPlaylistToEdit && currentPlaylistToEdit.id === -1) {
            axios.post(apiURL + "/playlist", {
                name: currentPlaylistToEdit.name,
                items: itemsMappedToIDs
            } satisfies PlaylistDtoPost)
                .then((e:AxiosResponse<PlaylistDto>) => {
                    dispatch(setPlaylist([...playlists, e.data]))
                    enqueueSnackbar(t('created-playlist'), {variant: "success"})
                })
        }
    }


    return createPortal(
        <div aria-hidden="true" id="defaultModal" onClick={()=>dispatch(setCreatePlaylistOpen(false))} className={`grid place-items-center fixed inset-0 bg-[rgba(0,0,0,0.5)] backdrop-blur overflow-x-hidden overflow-y-auto z-30 ${playListOpen ? 'opacity-100' : 'opacity-0 pointer-events-none'}`} tabIndex={-1}>

            {/* Modal */}
            <div className="relative bg-white max-w-5xl md:w-[50%] p-8 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,0.2)]" onClick={e=>e.stopPropagation()}>

                {/* Close button */}
                <button type="button" className="absolute top-4 right-4 bg-transparent" data-modal-toggle="defaultModal" onClick={()=>dispatch(setCreatePlaylistOpen(false))}>
                    <span className="material-symbols-outlined text-stone-400 hover:text-stone-600">close</span>
                    <span className="sr-only">{t('closeModal')}</span>
                </button>

                {/* Submit form for creating a playlist */}
                <form onSubmit={e=>{
                    e.preventDefault()
                    handlePlaylistCreateOrUpdate()
                }}>

                    <div className="mt-5 mb-5 ">
                    <Heading2 className="mb-4">{t('add-playlist')}</Heading2>
                    </div>

                    {/* Playlist data like name */}
                    {
                        stage === 0 && <PlaylistData/>
                    }

                    {/* Playlist items */}
                    {
                        stage === 1 &&
                        <PlaylistSearchEpisode/>
                    }

                    {/* Buttons for skipping to next step*/}
                    <div className="flex">
                    <button type="button">
                        <span className={`material-symbols-outlined ${stage===0&&'opacity-60'} text-mustard-600`} onClick={()=>{stage>=1&&setStage(stage-1)}}>arrow_back</span>
                    </button>
                        <div className="flex-1"></div>
                    <button type="button" onClick={()=>{stage<=1&&setStage(stage+1)}}>
                        <span className={`material-symbols-outlined ${stage===2&&'opacity-60'}  text-mustard-600`}>arrow_forward</span>
                    </button>
                    </div>

                    {stage === 2 &&
                        <><CustomButtonPrimary type="submit" className="float-right" onClick={()=>{
                            axios.post(apiURL+'/playlist', currentPlaylistToEdit)
                                .then((v: AxiosResponse<PlaylistDto>)=>{
                                    enqueueSnackbar(t('invite-created'), {variant: "success"})
                                    dispatch(setPlaylist([...playlists,v.data]))
                                    dispatch(setCreatePlaylistOpen(false))
                                })
                        }}>{currentPlaylistToEdit?.id===-1?t('create-playlist'):t('update-playlist')}</CustomButtonPrimary>
                            <br/>
                        </>
                    }
                </form>
            </div>
        </div>, document.getElementById('modal')!
    )
}
