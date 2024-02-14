import {FC, useEffect} from "react";
import {useTranslation} from "react-i18next";
import {Heading1} from "../components/Heading1";
import {CustomInput} from "../components/CustomInput";
import useCommon, {LoggedInUser} from "../store/CommonSlice";
import {Controller, useForm} from "react-hook-form";
import {CustomCheckbox} from "../components/CustomCheckbox";
import {CustomButtonPrimary} from "../components/CustomButtonPrimary";
import axios, {AxiosResponse} from "axios";
import {v4} from "uuid";
import {enqueueSnackbar} from "notistack";

type UserManagementPageProps = {

}


export const UserManagementPage: FC<UserManagementPageProps> = () => {
    const {t} = useTranslation()
    const loggedInUser = useCommon(state => state.loggedInUser)
    type UsermanagementForm = {
        username: string,
        password?: string,
        apiKey: string,
    }

    const {control, handleSubmit,formState, setValue} = useForm<UsermanagementForm>({
        defaultValues: {
            username: '',
            apiKey: ''
        }})

    useEffect(() => {
        if (loggedInUser) {
            setValue('username', loggedInUser.username)
        }
    }, [loggedInUser])

    const update_settings = (data: UsermanagementForm) => {
        if (data.password === '') {
            delete data.password
        }
        axios.put('/users/'+loggedInUser?.username, data)
            .then((c:AxiosResponse<LoggedInUser>)=>{
                useCommon.getState().setLoggedInUser(c.data)
                enqueueSnackbar(t('user-settings-updated'), {variant: 'success'})
            })
            .catch(e=>enqueueSnackbar(e.response.data.error, {variant: 'error'}))
    }

    return (
        <div className="md:w-3/6">
            <Heading1>{t('profile')}</Heading1>
            <div className="mt-5">
                <form onSubmit={handleSubmit(update_settings)}>
                    <div className="grid grid-cols-2 gap-5 mb-5">
                        <label className="ml-2 mt-2 text-sm text-[--fg-secondary-color]"
                               htmlFor="username">{t('username')}</label>
                        <Controller name="username" control={control}
                                    render={({field: {name, onChange, value}}) => (
                                        <CustomInput id="username" name={name}
                                                     onChange={onChange}
                                                     value={value}/>
                                    )}/>
                        <label className="ml-2 mt-2 text-sm text-[--fg-secondary-color]"
                               htmlFor="password">{t('password')}</label>
                        <Controller name="password" control={control}
                                    render={({field: {name, onChange, value}}) => (
                                        <CustomInput id="password" name={name}
                                                     onChange={onChange}
                                                     value={value}/>
                                    )}/>
                        <label className="ml-2 mt-2 text-sm text-[--fg-secondary-color]"
                               htmlFor="apiKey">{t('api-key')}</label>
                        <Controller name="apiKey" control={control}
                                    render={({field: {name, onChange, value}}) => (
                                        <div className="block relative">
                                            <CustomInput disabled={true} className="w-full" id="apiKey" name={name}
                                                     onChange={onChange}
                                                     value={value}/>
                                            <button type="button" className="material-symbols-outlined absolute right-2 top-1.5 text-[--fg-color]" onClick={()=>{
                                                setValue("apiKey", v4().replace(/-/g, ''))
                                            }}>cached</button>
                                        </div>
                                    )}/>
                    </div>

                    <CustomButtonPrimary type="submit" className="float-right" onClick={() => {

                    }}>{t('save')}</CustomButtonPrimary>
                </form>
            </div>
        </div>
    )
}
