import { useTranslation } from 'react-i18next'
import { Spinner } from './Spinner'
import {FC} from "react";
import {cn} from "../lib/utils";

type LoadingSpinnerProps = {
    className?: string
}

export const Loading: FC<LoadingSpinnerProps> = ({className}) => {
    const { t } = useTranslation()

    return (
        <div className={cn("grid place-items-center h-full w-full", className)}>
            <div className="flex items-center gap-3" role="status">
                <Spinner/>

                <span className="ui-text">{t('loading')}...</span>
            </div>
        </div>
    )
}
