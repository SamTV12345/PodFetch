import {useTranslation} from "react-i18next"
import {Heading1} from "../components/Heading1"
import {EpisodeSearch} from "../components/EpisodeSearch"

export const EpisodeSearchPage = () => {
    const {t} = useTranslation()

    return (
        <>
            <Heading1 className="mb-10">{t('search-episodes')}</Heading1>

            <EpisodeSearch showBlankState={true} />
        </>
    )
}
