import {useTranslation} from "react-i18next";
import {useState} from "react";
import axios from "axios";
import {apiURL} from "../utils/Utilities";
import {enqueueSnackbar} from "notistack";
import {UpdateNameSettings} from "../models/UpdateNameSettings";

export const PodcastNaming = ()=>{
    const {t} = useTranslation()
    const [renameExistingPodcasts, setRenameExistingPodcasts] = useState<boolean>(true)
    const [renameIllegalCharacters, setRenameIllegalCharacters] = useState<boolean>(true)
    const [selectedReplacementOfColon, setSelectedReplacementOfColon] = useState<string>("dash")
    const [episodeFormat, setEpisodeFormat] = useState<string>("{}")



    return <div className="bg-slate-900 rounded p-5 text-white grid gap-4">
        <h2 className="text-2xl text-center">{t('podcast-naming')}</h2>
        <h3 className="text-xl ml-4">{t('rename-podcasts')}</h3>
        <div className="flex items-center ml-8 mt-2">
            <input checked={renameExistingPodcasts} id="checked-checkbox" type="checkbox" onChange={c=>setRenameExistingPodcasts(c.target.checked)}
                   className="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"/>
                <label htmlFor="checked-checkbox" className="ml-2 text-sm font-medium text-gray-900 dark:text-gray-300">{t('podfetch-use-existing-filenames')}</label>
        </div>
        <h3 className="text-xl ml-4">{t('replace-illegal-characters')}</h3>
        <div className="flex items-center ml-8 mt-2">
            <input checked={renameIllegalCharacters} onChange={()=>setRenameIllegalCharacters(!renameIllegalCharacters)} id="checked-checkbox" type="checkbox"
                   className="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"/>
            <label htmlFor="checked-checkbox" className="ml-2 text-sm font-medium text-gray-900 dark:text-gray-300">{t('replace-illegal-characters-description')}</label>
        </div>
        <h3 className="text-xl ml-4">{t('colon-replacement')}</h3>
        <div className="ml-8">
            <select id="colon-replace" className="border  text-sm rounded-lg
                              block p-2.5 bg-gray-700 border-gray-600 w-3/4
                            placeholder-gray-400 text-white focus:ring-blue-500 focus:border-blue-500" value={selectedReplacementOfColon} onChange={(e)=>setSelectedReplacementOfColon(e.target.value)}>
                <option value="dash">{t('dash-separated')}</option>
                <option value="online">{t('dash-separated-with-space')}</option>
            </select>
        </div>
        <h3 className="text-xl ml-4">{t('standard-episode-format')}</h3>
        <div className="ml-8">
            <input className="bg-gray-700 rounded p-1 w-2/4" value={episodeFormat} onChange={(v)=>setEpisodeFormat(v.target.value)}/>
        </div>
        <div className="flex">
            <div className="flex-1"></div>
            <button className="p-2 bg-blue-600 rounded hover:bg-blue-500" onClick={()=>{
                axios.put(apiURL+"/settings/name", {
                    replaceInvalidCharacters: renameIllegalCharacters,
                    replacementStrategy: selectedReplacementOfColon,
                    useExistingFilenames: renameExistingPodcasts,
                    episodeFormat
                } satisfies UpdateNameSettings)
                    .then(()=>{
                        enqueueSnackbar(t('settings-saved'), {variant: "success"})
                    })
            }}>
                {t('save')}
            </button>
        </div>
    </div>
}
