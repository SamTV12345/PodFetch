import {Modal} from "./Modal";
import {useTranslation} from "react-i18next";
import {useWatchTogether} from "../store/Watch2Gether";
import {CustomInput} from "./CustomInput";
import {CustomButtonSecondary} from "./CustomButtonSecondary";
import {CustomButtonPrimary} from "./CustomButtonPrimary";
import axios from "axios";
import {Controller, useForm} from "react-hook-form";

export const Watch2GetherEditModal = ()=>{
    const {t} = useTranslation()
    const currentWatchTogether = useWatchTogether(state => state.currentWatchTogetherCreate)
    const {handleSubmit, register, control } = useForm<WatchTogetherCreate>({
        defaultValues: currentWatchTogether
    })

    const handleSendForm = (data: WatchTogetherCreate)=>{
        console.log("Sent", data)
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

                <Controller
                    name="roomName"
                    control={control}
                    render={({ field: { name, onChange, value }}) => (
                        <CustomInput autoComplete="roomName" className="w-full" id="username" name={name} onChange={onChange} placeholder={t('room-name')!} value={value} required />
                    )} />

            </div>
            <div className="text-right mt-5">
                <CustomButtonSecondary
                    className="border-transparent shadow-none hover:shadow-none text-base text-[--fg-color] hover:text-[--fg-color-hover]"
                    onClick={() => {

                    }}>{t('cancel')!}</CustomButtonSecondary>
                <CustomButtonPrimary
                    className="bg-mustard-600  text-[--fg-color]"
                    type="submit">{t('create')}</CustomButtonPrimary>
            </div>
        </form>
    </Modal>
}
