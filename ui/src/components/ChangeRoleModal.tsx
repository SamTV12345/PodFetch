import { FC, useEffect, useState } from 'react'
import * as Dialog from '@radix-ui/react-dialog'
import { useTranslation } from 'react-i18next'
import { enqueueSnackbar } from 'notistack'
import { User } from '../models/User'
import { CustomButtonPrimary } from './CustomButtonPrimary'
import { CustomSelect } from './CustomSelect'
import { Switcher } from './Switcher'
import { $api } from '../utils/http'
import { components } from '../../schema'

const roleOptions = [
    { translationKey: 'admin', value: 'admin' },
    { translationKey: 'user', value: 'user' },
    { translationKey: 'uploader', value: 'uploader' }
]

type ChangeRoleModalProps = {
    open: boolean
    onOpenChange: (open: boolean) => void
    user: components["schemas"]["UserSummary"] | undefined
    onSuccess: (updatedUser: User) => void
}

export const ChangeRoleModal: FC<ChangeRoleModalProps> = ({ open, onOpenChange, user, onSuccess }) => {
    const { t } = useTranslation()
    const [role, setRole] = useState(user?.role || '')
    const [explicitConsent, setExplicitConsent] = useState(user?.explicitConsent || false)
    const changeRoleMutation = $api.useMutation('put', '/api/v1/users/{username}/role')

    useEffect(() => {
        if (user) {
            setRole(user.role)
            setExplicitConsent(user.explicitConsent)
        }
    }, [user])

    const changeRole = () => {
        changeRoleMutation.mutateAsync({
            params: {
                path: {
                    username: user?.username as string
                }
            },
            body: {
                role: role as any,
                explicitConsent: explicitConsent
            }
        })
            .then(() => {
                enqueueSnackbar(t('role-changed'), { variant: 'success' })
                onSuccess({
                    ...user!,
                    role,
                    explicitConsent
                } satisfies User)
                onOpenChange(false)
            })
    }

    return (
        <Dialog.Root open={open} onOpenChange={onOpenChange}>
            <Dialog.Portal>
                <Dialog.Overlay className="fixed inset-0 bg-[rgba(0,0,0,0.5)] backdrop-blur-sm z-30" />
                <Dialog.Content className="fixed inset-0 z-40 flex items-center justify-center p-4">
                    <div className="relative ui-surface max-w-lg p-8 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))] w-full">
                        <Dialog.Title className="font-bold leading-tight! text-xl xs:text-2xl ui-text mb-4">{t('change-role-user', { name: user?.username })}</Dialog.Title>
                        <Dialog.Close className="absolute top-4 right-4 bg-transparent">
                            <span className="material-symbols-outlined ui-modal-close hover:ui-modal-close-hover">close</span>
                        </Dialog.Close>

                        {/* Role select */}
                        <div className="mb-6">
                            <label className="block mb-2 text-sm ui-text" htmlFor="role">{t('role')}</label>
                            <CustomSelect className="text-left w-full" id="role" onChange={(v) => {
                                setRole(v)
                            }} options={roleOptions} placeholder={t('select-role')} value={role} />
                        </div>

                        {/* Explicit content toggle */}
                        <div className="flex items-center gap-4 mb-6">
                            <label className="text-sm ui-text" htmlFor="allow-explicit-content">{t('allow-explicit-content')}</label>
                            <Switcher checked={explicitConsent} id="allow-explicit-content"
                                      onChange={() => { setExplicitConsent(!explicitConsent) }} />
                        </div>

                        <CustomButtonPrimary className="float-right" onClick={changeRole}>{t('change-role')}</CustomButtonPrimary>
                    </div>
                </Dialog.Content>
            </Dialog.Portal>
        </Dialog.Root>
    )
}
