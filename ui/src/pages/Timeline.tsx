import {useTranslation} from "react-i18next";

export const Timeline = ()=>{
    const {t} = useTranslation()

    return <div className="p-3">
        <h1 className="font-bold text-2xl">{t('timeline')}</h1>

    </div>
}
