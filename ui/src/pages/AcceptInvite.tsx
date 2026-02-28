import { Controller, SubmitHandler, useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { useNavigate, useParams } from 'react-router-dom'
import { enqueueSnackbar } from 'notistack'
import { formatTime } from '../utils/Utilities'
import { CustomButtonPrimary } from '../components/CustomButtonPrimary'
import { CustomInput } from '../components/CustomInput'
import { Heading2 } from '../components/Heading2'
import { Loading } from '../components/Loading'
import { LoginData } from './Login'
import {$api} from "../utils/http";

export const AcceptInvite = () => {
    const { control, handleSubmit, formState: {} } = useForm<LoginData>()
    const navigate = useNavigate()
    const params = useParams()
    const { t } = useTranslation()
    const inviteQuery = $api.useQuery('get', '/api/v1/invites/{invite_id}', {
        params: {
            path: {
                invite_id: params.id!
            }
        }
    }, {retry: false})
    const createUserMutation = $api.useMutation('post', '/api/v1/users/')

    if (inviteQuery.isLoading) {
        return <Loading />
    }

    const invite = inviteQuery.data

    const onSubmit: SubmitHandler<LoginData> = (data) => {
        createUserMutation.mutateAsync({
            body: {
                username: data.username,
                password: data.password,
                inviteId: params.id!
            }
        }).then(() => {
            enqueueSnackbar(t('account-created'), { variant: 'success' })
            navigate('/')
        }).catch(() => {
            enqueueSnackbar(t('password-too-weak'), { variant: 'error' })
        })
    }

    if (!invite) {
        return <Loading />
    }

    return (
        <div className="flex flex-col items-center xs:justify-center bg-stone-900 h-full w-full">
            <span className="flex items-center gap-2 xs:mb-10 px-4 py-3 text-white">
                <span className="material-symbols-outlined ui-text-accent">auto_detect_voice</span>
                <span className="font-bold font-['Inter_variable']">Podfetch</span>
            </span>

            <div className="ui-surface max-w-sm p-8 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))] w-full">
                <Heading2 className="mb-10 text-center">
                    {t('create-account')}
                </Heading2>

                <dl className="grid xs:grid-cols-2 gap-5 mb-10">
                    <div>
                        <dt className="font-medium text-sm ui-text">
                            {t('role')}
                        </dt>
                        <dd className="text-sm ui-text-muted">
                            {t(invite.role)}
                        </dd>
                    </div>
                    <div>
                        <dt className="font-medium text-sm ui-text">
                            {t('explicit-content')}
                        </dt>
                        <dd className="text-sm ui-text-muted">
                            {invite.explicitConsent ? t('yes') : t('no')}
                        </dd>
                    </div>
                    <div>
                        <dt className="font-medium text-sm ui-text">
                            {t('invite-created')}
                        </dt>
                        <dd className="text-sm ui-text-muted">
                            {formatTime(invite.createdAt)}
                        </dd>
                    </div>
                    <div>
                        <dt className="font-medium text-sm ui-text">
                            {t('invite-expires-at')}
                        </dt>
                        <dd className="text-sm ui-text-muted">
                            {formatTime(invite.expiresAt)}
                        </dd>
                    </div>
                </dl>

                <form className="flex flex-col gap-6" onSubmit={handleSubmit(onSubmit)}>
                    <div className="flex flex-col gap-2">
                        <label className="text-sm ui-text" htmlFor="username">{t('username')!}</label>

                        <Controller
                        name="username"
                        control={control}
                        render={({ field: { name, onChange, value }}) => (
                            <CustomInput autoComplete="username" className="w-full" id="username" name={name} onChange={onChange} placeholder={t('your-username')!} value={value} required />
                        )} />
                    </div>
                    <div className="flex flex-col gap-2">
                        <label className="text-sm ui-text" htmlFor="password">{t('password')}</label>

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
