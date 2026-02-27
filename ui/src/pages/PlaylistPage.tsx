import {useEffect, useState} from "react";
import {useTranslation} from "react-i18next";
import {enqueueSnackbar} from "notistack";
import {useNavigate} from "react-router-dom";
import {CustomButtonPrimary} from "../components/CustomButtonPrimary";
import {CustomButtonSecondary} from "../components/CustomButtonSecondary";
import {CreatePlaylistModal} from "../components/CreatePlaylistModal";
import usePlaylist from "../store/PlaylistSlice";
import {client} from "../utils/http";

export const PlaylistPage = () => {
    const {t} = useTranslation()
    const navigate = useNavigate()
    const playlist = usePlaylist(state => state.playlist)
    const setPlaylist = usePlaylist(state => state.setPlaylist)
    const setCreatePlaylistOpen = usePlaylist(state => state.setCreatePlaylistOpen)
    const setCurrentPlaylistToEdit = usePlaylist(state => state.setCurrentPlaylistToEdit)
    const [loading, setLoading] = useState(false)

    const loadPlaylists = async () => {
        setLoading(true)
        try {
            const response = await client.GET("/api/v1/playlist")
            setPlaylist(response.data ?? [])
        } finally {
            setLoading(false)
        }
    }

    useEffect(() => {
        void loadPlaylists()
    }, [])

    return (
        <div>
            <CreatePlaylistModal/>

            <div className="mb-6 flex flex-wrap items-center justify-between gap-3">
                <div>
                    <h2 className="text-xl font-semibold text-(--fg-color)">{t('playlists')}</h2>
                    <div className="text-sm text-(--fg-secondary-color)">
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

            {loading && (
                <div className="rounded-xl border border-(--border-color) p-6 text-(--fg-secondary-color)">
                    {t('loading-playlists')}
                </div>
            )}

            {!loading && playlist.length === 0 && (
                <div className="rounded-xl border border-dashed border-(--border-color) p-8 text-center">
                    <div className="text-(--fg-secondary-color)">{t('no-playlists-yet')}</div>
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
                        className="rounded-xl border border-(--border-color) bg-(--bg-color) p-4 shadow-[0_2px_10px_rgba(0,0,0,0.08)]"
                    >
                        <div className="mb-3">
                            <div className="line-clamp-1 text-lg font-semibold text-(--fg-color)">{item.name}</div>
                            <div className="text-xs text-(--fg-secondary-color)">
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
                                    const response = await client.GET("/api/v1/playlist/{playlist_id}", {
                                        params: {
                                            path: {playlist_id: String(item.id)}
                                        }
                                    })
                                    setCurrentPlaylistToEdit(response.data!)
                                    setCreatePlaylistOpen(true)
                                }}
                            >
                                {t('edit')}
                            </CustomButtonSecondary>
                            <button
                                className="ml-auto flex items-center text-sm text-(--danger-fg-color) hover:text-(--danger-fg-color-hover)"
                                onClick={async () => {
                                    await client.DELETE("/api/v1/playlist/{playlist_id}", {
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
