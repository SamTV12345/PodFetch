import {useForm} from "react-hook-form";
import axios, {AxiosResponse} from "axios";
import {Podcast} from "../store/CommonSlice";
import {handleAddPodcast} from "../utils/ErrorSnackBarResponses";
import {useTranslation} from "react-i18next";


type ManualImportData = {
    title: string,
    description: string,
}

export const ManualImport = ()=>{
    const { t } = useTranslation()
    const { register, watch, handleSubmit, formState: {} } = useForm<ManualImportData>({
        defaultValues: {
            feedUrl: ''
        }
    })

    const onSubmit = (data: ManualImportData) => {
        axios.post(  '/podcast/feed', {
            rssFeedUrl: data.feedUrl
        }).then((v: AxiosResponse<Podcast>) => {
            handleAddPodcast(v.status, v.data.name, t)
        })
    }

    return (
        <form  className="flex items-center gap-4" onSubmit={handleSubmit(onSubmit)}>

        </form>
    )
}
