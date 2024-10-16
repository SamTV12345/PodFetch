import {Modal} from "./Modal";
import {useTranslation} from "react-i18next";
import {useWatchTogether} from "../store/Watch2Gether";
import {CustomInput} from "./CustomInput";
import {CustomButtonSecondary} from "./CustomButtonSecondary";
import {CustomButtonPrimary} from "./CustomButtonPrimary";
import axios from "axios";
import {useForm} from "react-hook-form";

export const Watch2GetherEditModal = ()=>{
    const {t} = useTranslation()
    const currentWatchTogether = useWatchTogether(state => state.currentWatchTogetherCreate)
    const {handleSubmit, register } = useForm<WatchTogetherCreate>({
        defaultValues: currentWatchTogether
    })

    const handleSendForm = (data: WatchTogetherCreate)=>{
        axios.post('/watch-together', data)
    }

    return <Modal headerText={t('create-podflix')} onCancel={() => {

    }} onAccept={() => {

    }} onDelete={() => {

    }} cancelText={t('cancel')!} acceptText={t('create')}>
        <form onSubmit={handleSubmit(handleSendForm)}>
            <div className="grid grid-cols-2">
                <label className="font-medium text-[--fg-color] flex gap-1 self-center">
                    {t('room-name')}
                </label>
                <CustomInput type="text" {...register('roomName', {
                    required: true
                })}  placeholder={t('room-name')}/>

            </div>
            <div className="text-right mt-5">
                <CustomButtonSecondary
                    className="border-transparent shadow-none hover:shadow-none text-base text-[--fg-color] hover:text-[--fg-color-hover]"
                    onClick={() => {

                    }}>{t('cancel')!}</CustomButtonSecondary>
                <CustomButtonPrimary
                    className="bg-mustard-600 hover:bg-[--danger-fg-color-hover] hover:shadow-[--danger-fg-color-hover] text-[--fg-color]"
                    onClick={() => {
                        axios.post(
                            "/watch-together",
                            currentWatchTogether,
                            {
                                headers: {
                                    "Content-Type": "application/json"
                                }
                            }
                        ).then((r) => {
                            console.log(r)
                        }
                        )
                    }}>{t('create')}</CustomButtonPrimary>
            </div>
        </form>
    </Modal>
}
