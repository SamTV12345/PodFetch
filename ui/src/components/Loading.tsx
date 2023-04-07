import {useTranslation} from "react-i18next";
import {Spinner} from "./Spinner";

export const Loading = () => {
    const {t} = useTranslation()
    return <div className=" grid w-full h-full place-items-center">
        <div role="status" className="border-2 w-full p-2 rounded bg-gray-800 md:w-auto">
            <div className="flex gap-2">
                <Spinner/>
                <div>
                    <div className="flex  align-baseline">
                        <div className="mt-1 text-white">{t('loading')}</div>
                        <div className="flex dots-distance ml-1 dots-margin-bottom">
                            <div className="flex flex-col-reverse">
                                <div className="bg-slate-500  p-1 w-1 h-1 rounded-full animate-bounce bottom-0"></div>
                            </div>
                            <div className="flex flex-col-reverse">
                                <div className="bg-slate-600 p-1 w-1 h-1 rounded-full animate-bounce"></div>
                            </div>
                            <div className="flex flex-col-reverse">
                            <div className="bg-slate-700  p-1 w-1 h-1 rounded-full animate-bounce"></div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            <span className="sr-only">Loading...</span>
        </div>
    </div>
}
