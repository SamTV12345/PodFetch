import {FC} from "react"
import {useForm} from "react-hook-form"
import {useTranslation} from "react-i18next"
import axios, {AxiosResponse} from "axios"
import {enqueueSnackbar} from "notistack"
import {apiURL} from "../utils/Utilities"
import {Podcast} from "../store/CommonSlice"
import {CustomButtonPrimary} from "./CustomButtonPrimary"

type FeedURLComponentProps = {

}

type FeedURLFormData  = {
    feedUrl: string
}

export const FeedURLComponent:FC<FeedURLComponentProps> = ()=>{
    const {t} = useTranslation()
    const {register, watch, handleSubmit, formState: {}} = useForm<FeedURLFormData>({defaultValues:{
         feedUrl: ''
        }})
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


    return <form className="flex items-center gap-4" onSubmit={handleSubmit(onSubmit)}>
        <input {...register('feedUrl',{
            pattern: /^(http|https):\/\/[^ "]+$/,
        })} placeholder={t('rss-feed-url')!}
        className={"bg-stone-100 w-full px-4 py-2 rounded-lg text-sm text-stone-600"}/>

        <CustomButtonPrimary disabled={feedUrlWatched.trim().length===0} type="submit">Add</CustomButtonPrimary>
    </form>
}
