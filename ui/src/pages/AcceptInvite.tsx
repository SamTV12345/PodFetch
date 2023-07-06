import {useEffect, useState} from "react"
import {Controller, SubmitHandler, useForm} from "react-hook-form"
import {useTranslation} from "react-i18next"
import {useNavigate, useParams} from "react-router-dom"
import axios from "axios"
import {enqueueSnackbar} from "notistack"
import {apiURL, formatTime} from "../utils/Utilities"
import {CustomButtonPrimary} from "../components/CustomButtonPrimary"
import {CustomInput} from "../components/CustomInput"
import {Heading2} from "../components/Heading2"
import {Loading} from "../components/Loading"
import {LoginData} from "./Login"

export const AcceptInvite = () => {
    const navigate = useNavigate();
    const params = useParams()
    const {t} = useTranslation()
    const [invite, setInvite] = useState<Invite>()
    const [errored, setErrored] = useState<boolean>(false)
    const {control, handleSubmit, formState: {}} = useForm<LoginData>();

    type Invite = {
        id: string,
        role: string,
        createdAt: string,
        acceptedAt: string,
        expiresAt: string,
        explicitContent: boolean
    }

    useEffect(() => {
        axios.get(apiURL+"/users/invites/"+params.id).then((res) => {
            setInvite(res.data)
            })
            .catch(() => {
                setErrored(true)
            })
    },[])

    if (!invite && !errored) {
        return <Loading/>
    }

    const onSubmit: SubmitHandler<LoginData> = (data) => {
        axios.post(apiURL+"/users/", {
            username: data.username,
            password: data.password,
            inviteId: params.id
        }).then(() => {
            enqueueSnackbar(t('account-created'), {variant: "success"})
            navigate('/')
        }).catch(() => {
            enqueueSnackbar(t('password-too-weak'), {variant: "error"})
        })
    }

    if (!invite) {
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
                    {t('create-account')}
                </Heading2>

                <dl className="grid xs:grid-cols-2 gap-5 mb-10">
                    <div>
                        <dt className="font-medium text-sm">
                            {t('role')}
                        </dt>
                        <dd className="text-sm text-stone-500">
                            {t(invite.role)}
                        </dd>
                    </div>
                    <div>
                        <dt className="font-medium text-sm">
                            {t('explicit-content')}
                        </dt>
                        <dd className="text-sm text-stone-500">
                            {invite.explicitContent ? t('yes') : t('no')}
                        </dd>
                    </div>
                    <div>
                        <dt className="font-medium text-sm">
                            {t('invite-created')}
                        </dt>
                        <dd className="text-sm text-stone-500">
                            {formatTime(invite.createdAt)}
                        </dd>
                    </div>
                    <div>
                        <dt className="font-medium text-sm">
                            {t('invite-expires-at')}
                        </dt>
                        <dd className="text-sm text-stone-500">
                            {formatTime(invite.expiresAt)}
                        </dd>
                    </div>
                </dl>

                <form className="flex flex-col gap-6" onSubmit={handleSubmit(onSubmit)}>
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

                    <CustomButtonPrimary className="self-end" type="submit">{t('create')}</CustomButtonPrimary>
                </form>
            </div>
        </div>
    )
}
