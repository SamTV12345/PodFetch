import {useEffect} from "react";
import {useTranslation} from "react-i18next";
import {enqueueSnackbar} from "notistack";
import {useNavigate} from "react-router-dom";
import {CustomButtonPrimary} from "../components/CustomButtonPrimary";
import {CustomButtonSecondary} from "../components/CustomButtonSecondary";
import {CreatePlaylistModal} from "../components/CreatePlaylistModal";
import usePlaylist from "../store/PlaylistSlice";
import {$api} from "../utils/http";

export const PlaylistPage = () => {
    const {t} = useTranslation()
    const navigate = useNavigate()
    const playlist = usePlaylist(state => state.playlist)
    const setPlaylist = usePlaylist(state => state.setPlaylist)
    const setCreatePlaylistOpen = usePlaylist(state => state.setCreatePlaylistOpen)
    const setCurrentPlaylistToEdit = usePlaylist(state => state.setCurrentPlaylistToEdit)
    const playlistsQuery = $api.useQuery('get', '/api/v1/playlist')
    const playlistDetailQuery = $api.useMutation('get', '/api/v1/playlist/{playlist_id}')
    const deletePlaylistMutation = $api.useMutation('delete', '/api/v1/playlist/{playlist_id}')

    useEffect(() => {
        setPlaylist(playlistsQuery.data ?? [])
    }, [playlistsQuery.data, setPlaylist])

    return (
        <div>
            <CreatePlaylistModal/>

            <div className="mb-6 flex flex-wrap items-center justify-between gap-3">
                <div>
                    <h2 className="text-xl font-semibold ui-text">{t('playlists')}</h2>
                    <div className="text-sm ui-text-muted">
                        {t('playlist-page-description')}
                    </div>
                </div>
                <CustomButtonPrimary
                    className="flex items-center"
                    onClick={() => {
                        setCurrentPlaylistToEdit({name: '', items: [], id: String(-1)})
                        setCreatePlaylistOpen(true)
                    }}
                >
                    <span className="material-symbols-outlined leading-[0.875rem] mr-1">add</span>
                    {t('add-new')}
                </CustomButtonPrimary>
            </div>

            {playlistsQuery.isLoading && (
                <div className="rounded-xl border ui-border p-6 ui-text-muted">
                    {t('loading-playlists')}
                </div>
            )}

            {!playlistsQuery.isLoading && playlist.length === 0 && (
                <div className="rounded-xl border border-dashed ui-border p-8 text-center">
                    <div className="ui-text-muted">{t('no-playlists-yet')}</div>
                    <CustomButtonSecondary
                        className="mt-4"
                        onClick={() => {
                            setCurrentPlaylistToEdit({name: '', items: [], id: String(-1)})
                            setCreatePlaylistOpen(true)
                        }}
                    >
                        {t('create-playlist')}
                    </CustomButtonSecondary>
                </div>
            )}

            <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
                {playlist.map(item => (
                    <div
                        key={item.id}
                        className="rounded-xl border ui-border ui-surface p-4 shadow-[0_2px_10px_rgba(0,0,0,0.08)]"
                    >
                        <div className="mb-3">
                            <div className="line-clamp-1 text-lg font-semibold ui-text">{item.name}</div>
                            <div className="text-xs ui-text-muted">
                                {t('item_other', {count: item.items.length})}
                            </div>
                        </div>
                        <div className="flex items-center gap-2">
                            <CustomButtonPrimary
                                className="px-3 py-2"
                                onClick={() => navigate(item.id)}
                            >
                                {t('open')}
                            </CustomButtonPrimary>
                            <CustomButtonSecondary
                                className="px-3 py-2"
                                onClick={async () => {
                                    const response = await playlistDetailQuery.mutateAsync({
                                        params: {
                                            path: {playlist_id: String(item.id)}
                                        }
                                    })
                                    setCurrentPlaylistToEdit(response)
                                    setCreatePlaylistOpen(true)
                                }}
                            >
                                {t('edit')}
                            </CustomButtonSecondary>
                            <button
                                className="ml-auto flex items-center text-sm ui-text-danger hover:ui-text-danger-hover"
                                onClick={async () => {
                                    await deletePlaylistMutation.mutateAsync({
                                        params: {
                                            path: {playlist_id: String(item.id)}
                                        }
                                    })
                                    enqueueSnackbar(t('invite-deleted'), {variant: "success"})
                                    setPlaylist(playlist.filter(v => v.id !== item.id))
                                }}
                            >
                                <span className="material-symbols-outlined mr-1">delete</span>
                                {t('delete')}
                            </button>
                        </div>
                    </div>
                ))}
            </div>
        </div>
    )
}
