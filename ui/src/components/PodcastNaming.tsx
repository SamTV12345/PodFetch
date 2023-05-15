import {useTranslation} from "react-i18next";
import {FC, useState} from "react";
import axios from "axios";
import {apiURL} from "../utils/Utilities";
import {enqueueSnackbar} from "notistack";
import {UpdateNameSettings} from "../models/UpdateNameSettings";
import {SubmitHandler, useForm} from "react-hook-form";
import {LoginData} from "./LoginComponent";
import {Setting} from "../models/Setting";

type PodcastNamingProps = {
    settings: Setting
}
export const PodcastNaming:FC<PodcastNamingProps> = ({settings})=>{
    const {t} = useTranslation()
    const {register, watch, handleSubmit, formState: {}} = useForm<UpdateNameSettings>({defaultValues:{
        replacementStrategy: settings.replacementStrategy,
            episodeFormat: settings.episodeFormat,
            replaceInvalidCharacters: settings.replaceInvalidCharacters,
            useExistingFilenames: settings.useExistingFilenames,
            podcastFormat: settings.podcastFormat
        }})

    const update_settings:SubmitHandler<UpdateNameSettings> = (data)=>{
        axios.put(apiURL+"/settings/name", data satisfies UpdateNameSettings)
            .then(()=>{
                enqueueSnackbar(t('settings-saved'), {variant: "success"})
            })
    }
    return <form onSubmit={handleSubmit(update_settings)}>
        <div className="bg-slate-900 rounded p-5 text-white grid gap-4">
        <h2 className="text-2xl text-center">{t('podcast-naming')}</h2>
        <h3 className="text-xl ml-4">{t('rename-podcasts')}</h3>
        <div className="flex items-center ml-8 mt-2">
            <input {...register('useExistingFilenames')} id="checked-checkbox" type="checkbox" className="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"/>
                <label htmlFor="checked-checkbox" className="ml-2 text-sm font-medium text-gray-900 dark:text-gray-300">{t('podfetch-use-existing-filenames')}</label>
        </div>
        <h3 className="text-xl ml-4">{t('replace-illegal-characters')}</h3>
        <div className="flex items-center ml-8 mt-2">
            <input {...register('replaceInvalidCharacters')} id="checked-checkbox" type="checkbox"
                   className="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"/>
            <label htmlFor="checked-checkbox" className="ml-2 text-sm font-medium text-gray-900 dark:text-gray-300">{t('replace-illegal-characters-description')}</label>
        </div>
        <h3 className="text-xl ml-4">{t('colon-replacement')}</h3>
        <div className="ml-8">
            <select id="colon-replace" className="border  text-sm rounded-lg
                              block p-2.5 bg-gray-700 border-gray-600 w-3/4
                            placeholder-gray-400 text-white focus:ring-blue-500 focus:border-blue-500" {...register("replacementStrategy")}>
                <option value="replace-with-dash">{t('dash-separated')}</option>
                <option value="replace-with-dash-and-underscore">{t('dash-separated-with-underscore')}</option>
                <option value="remove">{t('remove')}</option>
            </select>
        </div>
        <h3 className="text-xl ml-4">{t('standard-episode-format')}</h3>
        <div className="ml-8">
            <input className="bg-gray-700 rounded p-1 w-2/4" {...register("episodeFormat")}/>
        </div>
            <h3 className="text-xl ml-4">{t('standard-podcast-format')}</h3>
            <div className="ml-8">
                <input className="bg-gray-700 rounded p-1 w-2/4" {...register("podcastFormat")}/>
            </div>
        <div className="flex">
            <div className="flex-1"></div>
            <button className="p-2 bg-blue-600 rounded hover:bg-blue-500" type="submit">
                {t('save')}
            </button>
        </div>
    </div>
    </form>
}
