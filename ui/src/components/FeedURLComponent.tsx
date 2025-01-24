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

    const { register, handleSubmit, formState: {
        isDirty, isValid
    } } = useForm<FeedURLFormData>({
        defaultValues: {
            feedUrl: ''
        }
    })


    const onSubmit = (data: FeedURLFormData) => {
        axios.post(  '/podcasts/feed', {
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

            <CustomButtonPrimary disabled={!isDirty || !isValid} type="submit">{t('add')}</CustomButtonPrimary>
        </form>
    )
}
