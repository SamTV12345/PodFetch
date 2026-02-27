import { FC } from 'react'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { handleAddPodcast } from '../utils/ErrorSnackBarResponses'
import { CustomButtonPrimary } from './CustomButtonPrimary'
import {$api} from "../utils/http";

type FeedURLFormData = {
    feedUrl: string
}

export const FeedURLComponent: FC = () => {
    const { t } = useTranslation()
    const addFeedMutation = $api.useMutation('post', '/api/v1/podcasts/feed')

    const { register, handleSubmit, formState: {
        isDirty, isValid
    } } = useForm<FeedURLFormData>({
        defaultValues: {
            feedUrl: ''
        }
    })


    const onSubmit = (data: FeedURLFormData) => {
        addFeedMutation.mutateAsync({
            body: {
                rssFeedUrl: data.feedUrl
            }
        })
            .then((v: any) => {
                handleAddPodcast(200, v.name, t)
            })
    }

    return (
        <form className="flex items-center gap-4" onSubmit={handleSubmit(onSubmit)}>
            <input {...register('feedUrl', {
                pattern: /^(http|https):\/\/[^ "]+$/,
            })} placeholder={t('rss-feed-url')!}
            className={"bg-(--input-bg-color) w-full px-4 py-2 rounded-lg text-sm text-(--input-fg-color) placeholder:text-(--input-fg-color-disabled)"} />

            <CustomButtonPrimary disabled={!isDirty || !isValid} type="submit">{t('add')}</CustomButtonPrimary>
        </form>
    )
}
