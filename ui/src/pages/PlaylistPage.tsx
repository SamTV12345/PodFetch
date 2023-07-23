import {CustomButtonPrimary} from "../components/CustomButtonPrimary";
import axios, {AxiosResponse} from "axios";
import {apiURL, formatTime} from "../utils/Utilities";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {useTranslation} from "react-i18next";
import {enqueueSnackbar} from "notistack";
import {Simulate} from "react-dom/test-utils";
import play = Simulate.play;
import {setCreatePlaylistOpen, setCurrentPlaylistToEdit, setPlaylist} from "../store/PlaylistSlice";
import {useEffect, useState} from "react";
import {PlaylistDto} from "../models/Playlist";
import {CreatePlaylistModal} from "../components/CreatePlaylistModal";
import {useNavigate} from "react-router-dom";
import {setModalOpen} from "../store/ModalSlice";

export const PlaylistPage = ()=>{
    const dispatch = useAppDispatch()
    const {t} = useTranslation()
    const playlist = useAppSelector(state=>state.playlist.playlist)
    const navigate = useNavigate()
    const [creating, setCreating] = useState<boolean>(false)

    useEffect(()=>{
        if (playlist.length ===0){
            axios.get(apiURL+"/playlist").then((response:AxiosResponse<PlaylistDto[]>)=>{
                dispatch(setPlaylist(response.data))
            })
        }
    },[])

    return (
        <div>
           <CreatePlaylistModal/>

            <CustomButtonPrimary className="flex items-center xs:float-right mb-4 xs:mb-10" onClick={()=>{
                setCreating(true)
                dispatch(setCurrentPlaylistToEdit({name: '',items:[],id: -1} as PlaylistDto))
                dispatch(setCreatePlaylistOpen(true))
            }}>
                <span className="material-symbols-outlined leading-[0.875rem]">add</span> {t('add-new')}
            </CustomButtonPrimary>

            <div className={`
                scrollbox-x
                w-[calc(100vw-2rem)] ${/* viewport - padding */ ''}
                xs:w-[calc(100vw-4rem)] ${/* viewport - padding */ ''}
                md:w-[calc(100vw-18rem-4rem)] ${/* viewport - sidebar - padding */ ''}
            `}>
                <table className="text-left text-sm text-stone-900 w-full">
                    <thead>
                    <tr className="border-b border-stone-300">
                        <th scope="col" className="pr-2 py-3">
                            {t('playlist-name')}
                        </th>
                    </tr>
                    </thead>
                    <tbody>
                    {playlist.map(i=>
                        <tr className="border-b border-stone-300" key={i.id}>
                            <td className="px-2 py-4 flex items-center">
                                {i.name}
                                <button className="flex ml-2" onClick={(e)=>{
                                    e.preventDefault()

                                    axios.get(apiURL+"/playlist/"+i.id).then((response:AxiosResponse<PlaylistDto>)=> {
                                        dispatch(setCurrentPlaylistToEdit(response.data))
                                        setCreating(false)
                                        dispatch(setCreatePlaylistOpen(true))
                                    })
                                }} title={t('change-role')}>
                                    <span className="material-symbols-outlined text-stone-900 hover:text-stone-600">edit</span>
                                </button>
                            </td>
                            <td className="pl-2 py-4 gap-4">
                                <button className="flex float-left" onClick={(e)=>{
                                    e.preventDefault()
                                    dispatch(setCurrentPlaylistToEdit(i))

                                    dispatch(setModalOpen(true))
                                }} title={t('change-role')}>
                                    <span className="material-symbols-outlined text-stone-900 hover:text-stone-600"  onClick={()=>{
                                        navigate("/playlist/"+i.id)
                                    }}>visibility</span>
                                </button>
                                <button className="flex float-right text-red-700 hover:text-red-500" onClick={(e)=>{
                                    e.preventDefault()
                                    axios.delete(apiURL+"/playlist/"+i.id).then(()=>{
                                        enqueueSnackbar(t('invite-deleted'), {variant: "success"})
                                        dispatch(setPlaylist(playlist.filter(v=>v.id !== i.id)))
                                    })
                                }}>
                                    <span className="material-symbols-outlined mr-1">delete</span>
                                    {t('delete')}
                                </button>
                            </td>
                        </tr>
                    )}
                    </tbody>
                </table>
            </div>
        </div>)
}
