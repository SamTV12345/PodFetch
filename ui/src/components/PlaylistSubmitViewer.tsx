import {useTranslation} from "react-i18next";

type PlaylistSubmitViewerProps = {
    playlistName: string
    episodeCount: number
}

export const PlaylistSubmitViewer = ({playlistName, episodeCount}: PlaylistSubmitViewerProps)=>{
    const {t} = useTranslation()

    return (
        <div className="mt-4 rounded-xl border ui-border p-4">
            <div className="text-xs ui-text-muted">{t('playlist-name')}</div>
            <div className="mt-1 text-base ui-text font-medium">
                {playlistName.length > 0 ? playlistName : "—"}
            </div>
            <div className="mt-4 text-xs ui-text-muted">{t('available-episodes')}</div>
            <div className="mt-1 text-base ui-text font-medium">
                {episodeCount}
            </div>
        </div>
    )
}
