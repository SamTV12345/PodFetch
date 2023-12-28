import {FC, useEffect} from "react";
import {useTranslation} from "react-i18next";
import {Heading1} from "../components/Heading1";
import {CustomInput} from "../components/CustomInput";
import useCommon from "../store/CommonSlice";
import {Controller, useForm} from "react-hook-form";
import {CustomCheckbox} from "../components/CustomCheckbox";
import {CustomButtonPrimary} from "../components/CustomButtonPrimary";
import axios from "axios";
import {apiURL} from "../utils/Utilities";

type UserManagementPageProps = {

}


export const UserManagementPage: FC<UserManagementPageProps> = () => {
    const {t} = useTranslation()
    const loggedInUser = useCommon(state => state.loggedInUser)

    type UsermanagementForm = {
        username: string,
        password: string,
        apiKey: string,
    }

    const {control, handleSubmit,formState, setValue} = useForm<UsermanagementForm>({
        defaultValues: {
            username: '',
            password: '',
            apiKey: ''
        }})

    useEffect(() => {
        if (loggedInUser) {
            setValue('username', loggedInUser.username)
        }
    }, [loggedInUser])

    const update_settings = (data: UsermanagementForm) => {

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
                    </div>

                    <CustomButtonPrimary className="float-right" onClick={() => {

                    }}>{t('save')}</CustomButtonPrimary>
                </form>
            </div>
        </div>
    )
}
