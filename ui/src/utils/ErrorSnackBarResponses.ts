import {enqueueSnackbar} from "notistack";
import {TFunction} from "i18next";

export const handleAddPodcast = (resp: number, podcast: string, t: TFunction)=>{

    switch (resp) {
        case 409:
            enqueueSnackbar(t('already-added',{
                name:podcast
            }),{variant: "error"})
            break
        case 403:
            enqueueSnackbar(t('not-admin-or-uploader'),{variant: "error"})
            break
        default:
            enqueueSnackbar(t('not-admin-or-uploader'),{variant: "error"})
    }
}
