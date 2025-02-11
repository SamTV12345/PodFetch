import { useState } from 'react'
import { Controller, SubmitHandler, useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { useNavigate } from 'react-router-dom'
import useCommon from '../store/CommonSlice'
import { CustomButtonPrimary } from '../components/CustomButtonPrimary'
import { CustomCheckbox } from '../components/CustomCheckbox'
import { CustomInput } from '../components/CustomInput'
import { Heading2 } from '../components/Heading2'
import { Loading } from '../components/Loading'
import { OIDCButton } from '../components/OIDCButton'
import 'material-symbols/outlined.css'
import {client} from "../utils/http";

export type LoginData = {
    username: string,
    password: string,
    rememberMe: boolean
}
export const Login = () => {
    const configModel = useCommon(state => state.configModel)
    const setLoginData = useCommon(state => state.setLoginData)
    const navigate = useNavigate()
    const [alert, setAlert] = useState<string>()
    const { t } = useTranslation()

    const { control, handleSubmit, formState: {} } = useForm<LoginData>({
        defaultValues: {
            username: '',
            password: '',
            rememberMe: false
        }
    })

    const onSubmit: SubmitHandler<LoginData> = (data, p) => {
        p?.preventDefault()

        client.POST("/api/v1/login", {
            body: {
                username: data.username,
                password: data.password
            }
        }).then(()=>{
            const basicAuthString = btoa(data.username + ':' + data.password)

            if (data.rememberMe) {
                localStorage.setItem('auth', basicAuthString)
            } else {
                sessionStorage.setItem('auth', basicAuthString)
            }

            setLoginData(data)

            useCommon.getState().setHeaders({ Authorization: 'Basic ' + basicAuthString })

            setTimeout(() => navigate('/'), 100)
        }).catch((e)=>{
            setAlert(e.toString())
        })
    }

    if (!configModel) {
        return <Loading />
    }

    return (
        <div className="flex flex-col items-center xs:justify-center bg-stone-900 h-full w-full">
            <span className="flex items-center gap-2 xs:mb-10 px-4 py-3 text-white">
                <span className="material-symbols-outlined text-(--accent-color)">auto_detect_voice</span>
                <span className="font-bold font-['Inter_variable']">Podfetch</span>
            </span>

            <div className="bg-(--bg-color) max-w-sm p-8 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))] w-full">
                <Heading2 className="mb-10 text-center">
                    {t('sign-in')}
                </Heading2>

                {configModel?.basicAuth && (
                    <form className="flex flex-col gap-6" onSubmit={handleSubmit(onSubmit)}>

                        {alert && (
                            <div className="bg-(--danger-bg-color) px-6 py-5 rounded-lg relative text-sm text-(--danger-fg-color)" role="alert">
                                <span className="block font-bold mb-2">{t('error-authenticating')}</span>

                                <span className="block sm:inline">{alert}</span>
                            </div>
                        )}

                        <div className="flex flex-col gap-2">
                            <label className="text-sm text-(--fg-color)" htmlFor="username">{t('username')!}</label>

                            <Controller
                            name="username"
                            control={control}
                            render={({ field: { name, onChange, value }}) => (
                                <CustomInput autoComplete="username" className="w-full" id="username" name={name} onChange={onChange} placeholder={t('your-username')!} value={value} required />
                            )} />
                        </div>
                        <div className="flex flex-col gap-2">
                            <label className="text-sm text-(--fg-color)" htmlFor="password">{t('password')}</label>

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

                            <label className="ml-2 text-sm text-(--fg-secondary-color)" htmlFor="remember">{t('remember-me')}</label>
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
