import {FC} from 'react'
import useCommon from '../store/CommonSlice'

type SettingsInfoIconProps = {
    headerKey: string
    textKey: string,
    className?: string
}

export const SettingsInfoIcon: FC<SettingsInfoIconProps> = ({ textKey, headerKey, className }) => {
    const setInfoModalPodcastOpen = useCommon(state => state.setInfoModalPodcastOpen)
    const setInfoText = useCommon(state => state.setInfoText)
    const setInfoHeading = useCommon(state => state.setInfoHeading)

    return (
        <button type="button">
            <span
                className="material-symbols-outlined pointer active:scale-95"
                onClick={() => {
                    setInfoHeading(headerKey)
                    setInfoText(textKey)
                    setInfoModalPodcastOpen(true)
                }}
            >info</span>
        </button>
    )
}
