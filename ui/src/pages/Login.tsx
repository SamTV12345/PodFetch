import {useState} from "react"
import {Controller, SubmitHandler, useForm} from "react-hook-form"
import {useTranslation} from "react-i18next"
import {useNavigate} from "react-router-dom"
import axios, {AxiosError} from "axios"
import {useAppDispatch, useAppSelector} from "../store/hooks"
import {setLoginData} from "../store/CommonSlice"
import {apiURL} from "../utils/Utilities"
import {CustomButtonPrimary} from "../components/CustomButtonPrimary"
import {CustomCheckbox} from "../components/CustomCheckbox"
import {CustomInput} from "../components/CustomInput"
import {Heading2} from "../components/Heading2"
import {Loading} from "../components/Loading"
import {OIDCButton} from "../components/OIDCButton"
import "material-symbols/outlined.css"

export type LoginData = {
    username: string,
    password: string,
    rememberMe: boolean
}

export const Login = () => {
    const {t} = useTranslation()
    const dispatch = useAppDispatch()
    const configModel = useAppSelector(state => state.common.configModel)
    const navigate = useNavigate()
    const {control, register, handleSubmit, formState: {}} = useForm<LoginData>({
        defaultValues: {
            username: '',
            password: '',
            rememberMe: false
        }
    })
    const [alert, setAlert] = useState<string>()

    const onSubmit: SubmitHandler<LoginData> = (data, p) => {
        p?.preventDefault()

        axios.post(apiURL + "/login", data)
            .then(() => {
                const basicAuthString = btoa(data.username + ":" + data.password)
                if (data.rememberMe){
                    localStorage.setItem("auth", basicAuthString)
                }
                else{
                    sessionStorage.setItem("auth", basicAuthString)
                }
                dispatch(setLoginData(data))
                axios.defaults.headers.common['Authorization'] = 'Basic ' + basicAuthString;
                setTimeout(()=>navigate('/'), 100)

            })
            .catch((e: AxiosError<ErrorModel>) => {
               setAlert(e.response!.data.message)
            })
    }

    if (!configModel) {
        return <Loading/>
    }

    return (
        <div className="flex flex-col items-center xs:justify-center bg-stone-900 h-full w-full">
            <span className="flex items-center gap-2 xs:mb-10 px-4 py-3 text-white">
                <span className="material-symbols-outlined text-mustard-600">auto_detect_voice</span>
                <span className="font-bold font-['Inter_variable']">Podfetch</span>
            </span>

            <div className="bg-white max-w-sm p-8 rounded-2xl w-full">
                <Heading2 className="mb-10 text-center">
                    {t('sign-in')}
                </Heading2>

                {configModel?.basicAuth && (
                    <form className="flex flex-col gap-6" onSubmit={handleSubmit(onSubmit)}>

                        {alert && (
                            <div className="bg-red-100 px-6 py-5 rounded-lg relative text-sm text-red-700" role="alert">
                                <span className="block font-bold mb-2">{t('error-authenticating')}</span>

                                <span className="block sm:inline">{alert}</span>
                            </div>
                        )}

                        <div className="flex flex-col gap-2">
                            <label className="text-sm text-stone-900" htmlFor="username">{t('username')!}</label>

                            <Controller
                            name="username"
                            control={control}
                            render={({ field: { name, onChange, value }}) => (
                                <CustomInput autoComplete="username" className="w-full" id="username" name={name} onChange={onChange} placeholder={t('your-username')!} value={value} required />
                            )} />
                        </div>
                        <div className="flex flex-col gap-2">
                            <label className="text-sm text-stone-900" htmlFor="password">{t('password')}</label>

                            <Controller
                            name="password"
                            control={control}
                            render={({ field: { name, onChange, value }}) => (
                                <CustomInput autoComplete="current-password" className="w-full" id="password" name={name} onChange={onChange} placeholder="••••••••" type="password" value={value} required />
                            )} />
                        </div>
                        <div className="flex items-center">
                            <Controller
                            name="rememberMe"
                            control={control}
                            render={({ field: { name, onChange, value }}) => (
                                <CustomCheckbox aria-describedby="remember" id="remember" name={name} onChange={onChange} value ={value} />
                            )} />

                            <label className="ml-2 text-sm text-stone-600" htmlFor="remember">{t('remember-me')}</label>
                        </div>

                        <CustomButtonPrimary className="self-end" type="submit">{t('sign-in')}</CustomButtonPrimary>

                    </form>
                )}

                {configModel.oidcConfigured && configModel.oidcConfig && (
                    <OIDCButton />
                )}
            </div>
        </div>
    )
}
