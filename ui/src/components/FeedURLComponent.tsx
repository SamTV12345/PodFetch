import { FC } from 'react'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import axios, { AxiosResponse } from 'axios'
import { handleAddPodcast } from '../utils/ErrorSnackBarResponses'
import { Podcast } from '../store/CommonSlice'
import { CustomButtonPrimary } from './CustomButtonPrimary'

type FeedURLFormData = {
    feedUrl: string
}

export const FeedURLComponent: FC = () => {
    const { t } = useTranslation()

    const { register, watch, handleSubmit, formState: {} } = useForm<FeedURLFormData>({
        defaultValues: {
            feedUrl: ''
        }
    })

    const feedUrlWatched = watch('feedUrl')

    const onSubmit = (data: FeedURLFormData) => {
        axios.post(  '/podcast/feed', {
            rssFeedUrl: data.feedUrl
        }).then((v: AxiosResponse<Podcast>) => {
            handleAddPodcast(v.status, v.data.name, t)
        })
    }

    return (
        <form className="flex items-center gap-4" onSubmit={handleSubmit(onSubmit)}>
            <input {...register('feedUrl', {
                pattern: /^(http|https):\/\/[^ "]+$/,
            })} placeholder={t('rss-feed-url')!}
            className={"bg-[--input-bg-color] w-full px-4 py-2 rounded-lg text-sm text-[--input-fg-color] placeholder:text-[--input-fg-color-disabled]"} />

            <CustomButtonPrimary disabled={feedUrlWatched.trim().length === 0} type="submit">Add</CustomButtonPrimary>
        </form>
    )
}
