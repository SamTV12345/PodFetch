import {CustomButtonPrimary} from "../components/CustomButtonPrimary";
import axios, {AxiosResponse} from "axios";
import {apiURL} from "../utils/Utilities";
import {useTranslation} from "react-i18next";
import {enqueueSnackbar} from "notistack";
import usePlaylist from "../store/PlaylistSlice";
import {useEffect} from "react";
import {PlaylistDto} from "../models/Playlist";
import {CreatePlaylistModal} from "../components/CreatePlaylistModal";
import {useNavigate} from "react-router-dom";

export const PlaylistPage = ()=>{
    const {t} = useTranslation()
    const setCreatePlaylistOpen = usePlaylist(state=>state.setCreatePlaylistOpen)
    const setCurrentPlaylistToEdit = usePlaylist(state=>state.setCurrentPlaylistToEdit)
    const setPlaylist = usePlaylist(state=>state.setPlaylist)
    const playlist = usePlaylist(state=>state.playlist)
    const navigate = useNavigate()

    useEffect(()=>{
        if (playlist.length ===0){
            axios.get(apiURL+"/playlist").then((response:AxiosResponse<PlaylistDto[]>)=>{
               setPlaylist(response.data)
            })
        }
    },[])

    return (
        <div>
           <CreatePlaylistModal/>

            <CustomButtonPrimary className="flex items-center xs:float-right mb-4 xs:mb-10" onClick={()=>{
               setCurrentPlaylistToEdit({name: '',items:[],id: -1} as PlaylistDto)
                setCreatePlaylistOpen(true)
            }}>
                <span className="material-symbols-outlined leading-[0.875rem]">add</span> {t('add-new')}
            </CustomButtonPrimary>

            <div className={`
                scrollbox-x 
                w-[calc(100vw-2rem)] ${/* viewport - padding */ ''}
                xs:w-[calc(100vw-4rem)] ${/* viewport - padding */ ''}
                md:w-[calc(100vw-18rem-4rem)] ${/* viewport - sidebar - padding */ ''}
            `}>
                <table className="text-left text-sm text-stone-900 w-full text-[--fg-color]">
                    <thead>
                    <tr className="border-b border-stone-300">
                        <th scope="col" className="pr-2 py-3 text-[--fg-color]">
                            {t('playlist-name')}
                        </th>
                    </tr>
                    </thead>
                    <tbody>
                    {playlist.map(i=>
                        <tr className="border-b border-stone-300 " key={i.id}>
                            <td className="px-2 py-4 flex items-center text-[--fg-color]">
                                {i.name}
                                <button className="flex ml-2" onClick={(e)=>{
                                    e.preventDefault()

                                    axios.get(apiURL+"/playlist/"+i.id).then((response:AxiosResponse<PlaylistDto>)=> {
                                        setCurrentPlaylistToEdit(response.data)
                                        setCreatePlaylistOpen(true)
                                    })
                                }} title={t('change-role')}>
                                    <span className="material-symbols-outlined text-[--fg-color] hover:text-stone-600">edit</span>
                                </button>
                            </td>
                            <td className="pl-2 py-4 gap-4">
                                <button className="flex float-left" onClick={(e)=>{
                                    e.preventDefault()
                                    setCurrentPlaylistToEdit(i)
                                   setCreatePlaylistOpen(true)

                                }} title={t('change-role')}>
                                    <span className="material-symbols-outlined hover:text-stone-600 text-[--fg-color]"
                                          onClick={()=>{
                                        navigate(i.id)
                                    }}>visibility</span>
                                </button>
                                <button className="flex float-right text-red-700 hover:text-red-500" onClick={(e)=>{
                                    e.preventDefault()
                                    axios.delete(apiURL+"/playlist/"+i.id).then(()=>{
                                        enqueueSnackbar(t('invite-deleted'), {variant: "success"})
                                        setPlaylist(playlist.filter(v=>v.id !== i.id))
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
