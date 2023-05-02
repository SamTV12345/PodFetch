import {FC} from "react";
import {useTranslation} from "react-i18next";
import {useForm} from "react-hook-form";
import axios, {AxiosResponse} from "axios";
import {apiURL} from "../utils/Utilities";
import {enqueueSnackbar} from "notistack";
import {Podcast} from "../store/CommonSlice";

type FeedURLComponentProps = {

}

type FeedURLFormData  = {
    feedUrl: string
}
export const FeedURLComponent:FC<FeedURLComponentProps> = ()=>{
    const {t} = useTranslation()
    const {register, watch, handleSubmit, formState: {}} = useForm<FeedURLFormData>({defaultValues:{
         feedUrl: ''
        }});
    const feedUrlWatched = watch('feedUrl')

    const onSubmit = (data: FeedURLFormData) => {
            axios.post(apiURL+"/podcast/feed", {
                rssFeedUrl: data.feedUrl
            }).then((v:AxiosResponse<Podcast>)=>{
                enqueueSnackbar(t('podcast-added', {
                    name: v.data.name
                }),{variant: "success"})
            })
        }


    return <form onSubmit={handleSubmit(onSubmit)}>
        <div className="relative">
        <input {...register('feedUrl',{
            pattern: /^(http|https):\/\/[^ "]+$/,
        })} placeholder={t('rss-feed-url')!}
                  className={"border text-sm rounded-lg block w-full p-2.5 bg-gray-700 border-gray-600 placeholder-gray-400 text-white focus:ring-blue-500 focus:border-blue-500"}/>
        <button className="absolute  top-2 right-1.5 active:scale-90" type="submit">
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className={`${feedUrlWatched.trim().length>0? 'text-blue-600':''} w-6 h-6`}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M6 12L3.269 3.126A59.768 59.768 0 0121.485 12 59.77 59.77 0 013.27 20.876L5.999 12zm0 0h7.5" />
            </svg>
        </button>
        </div>
    </form>
}
