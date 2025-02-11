import { useTranslation } from 'react-i18next'
import { Spinner } from './Spinner'

export const Loading = () => {
    const { t } = useTranslation()

    return (
        <div className="grid place-items-center h-full w-full">
            <div className="flex items-center gap-3" role="status">
                <Spinner/>

                <span className="text-(--fg-color)">{t('loading')}...</span>
            </div>
        </div>
    )
}
