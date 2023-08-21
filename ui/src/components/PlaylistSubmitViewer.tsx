import {CustomButtonPrimary} from "./CustomButtonPrimary";
import axios, {AxiosResponse} from "axios";
import {apiURL} from "../utils/Utilities";
import {PlaylistDto} from "../models/Playlist";
import {enqueueSnackbar} from "notistack";
import {setCreatePlaylistOpen, setPlaylist} from "../store/PlaylistSlice";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {useTranslation} from "react-i18next";
import {useFormContext} from "react-hook-form";

export const PlaylistSubmitViewer = ()=>{
    const dispatch = useAppDispatch()
    const {t} = useTranslation()
    const currentPlaylistToEdit = useAppSelector(state=>state.playlist.currentPlaylistToEdit)
    const playlists = useAppSelector(state=>state.playlist.playlist)

    const data = useFormContext()
    console.log(data)
    return <>
        <CustomButtonPrimary type="submit" className="float-right" onClick={()=>{
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
