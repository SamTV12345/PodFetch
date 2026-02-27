import {useTranslation} from "react-i18next";

type PlaylistSubmitViewerProps = {
    playlistName: string
    episodeCount: number
}

export const PlaylistSubmitViewer = ({playlistName, episodeCount}: PlaylistSubmitViewerProps)=>{
    const {t} = useTranslation()

    return (
        <div className="mt-4 rounded-xl border border-(--border-color) p-4">
            <div className="text-xs text-(--fg-secondary-color)">{t('playlist-name')}</div>
            <div className="mt-1 text-base text-(--fg-color) font-medium">
                {playlistName.length > 0 ? playlistName : "—"}
            </div>
            <div className="mt-4 text-xs text-(--fg-secondary-color)">{t('available-episodes')}</div>
            <div className="mt-1 text-base text-(--fg-color) font-medium">
                {episodeCount}
            </div>
        </div>
    )
}
