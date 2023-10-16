import {CustomButtonPrimary} from "./CustomButtonPrimary";
import axios, {AxiosResponse} from "axios";
import {apiURL} from "../utils/Utilities";
import {PlaylistDto, PlaylistDtoPost, PlaylistItem} from "../models/Playlist";
import {enqueueSnackbar} from "notistack";
import usePlaylist from "../store/PlaylistSlice";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {useTranslation} from "react-i18next";
import {useFormContext} from "react-hook-form";

export const PlaylistSubmitViewer = ()=>{
    const dispatch = useAppDispatch()
    const {t} = useTranslation()
    const currentPlaylistToEdit = usePlaylist(state=>state.currentPlaylistToEdit)
    const playlists = usePlaylist(state=>state.playlist)
    const setCreatePlaylistOpen = usePlaylist(state=>state.setCreatePlaylistOpen)
    const setPlaylist = usePlaylist(state=>state.setPlaylist)

    const savePlaylist = ()=>{
        const idsToMap:PlaylistItem[] = currentPlaylistToEdit!.items.map(item=>{
            return{
                episode: item.podcastEpisode.id
            }})

        axios.post(apiURL+'/playlist', {
            name: currentPlaylistToEdit?.name!,
            items: idsToMap
        } satisfies PlaylistDtoPost)
            .then((v: AxiosResponse<PlaylistDto>)=>{
                setPlaylist([...playlists,v.data])
                setCreatePlaylistOpen(false)
            })
    }


    return <>
        <CustomButtonPrimary type="submit" className="float-right" onClick={()=>{
            savePlaylist()
        }}>{currentPlaylistToEdit?.id===-1?t('create-playlist'):t('update-playlist')}</CustomButtonPrimary>
        <br/>
    </>
}
