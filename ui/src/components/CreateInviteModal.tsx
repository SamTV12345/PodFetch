import { FC, useState } from 'react'
import * as Dialog from '@radix-ui/react-dialog'
import { useTranslation } from 'react-i18next'
import { enqueueSnackbar } from 'notistack'
import { CustomButtonPrimary } from './CustomButtonPrimary'
import { CustomSelect } from './CustomSelect'
import { Switcher } from './Switcher'
import 'material-symbols/outlined.css'
import { $api } from '../utils/http'
import { components } from '../../schema'

const roleOptions = [
    { translationKey: 'admin', value: 'admin' },
    { translationKey: 'user', value: 'user' },
    { translationKey: 'uploader', value: 'uploader' }
]

type CreateInviteModalProps = {
    open: boolean
    onOpenChange: (open: boolean) => void
    onCreated: (invite: components["schemas"]["Invite"]) => void
}

export const CreateInviteModal: FC<CreateInviteModalProps> = ({ open, onOpenChange, onCreated }) => {
    const [invite, setInvite] = useState<components["schemas"]["InvitePostModel"]>({ role: 'user', explicitConsent: false })
    const { t } = useTranslation()
    const createInviteMutation = $api.useMutation('post', '/api/v1/invites')

    return (
        <Dialog.Root open={open} onOpenChange={onOpenChange}>
            <Dialog.Portal>
                <Dialog.Overlay className="fixed inset-0 bg-[rgba(0,0,0,0.5)] backdrop-blur-sm z-30" />
                <Dialog.Content className="fixed inset-0 z-40 flex items-center justify-center p-4">
                    <div className="relative ui-surface max-w-5xl p-8 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))] w-full">
                        <Dialog.Title className="font-bold leading-tight! text-xl xs:text-2xl ui-text mb-4">{t('add-invite')}</Dialog.Title>
                        <Dialog.Close className="absolute top-4 right-4 bg-transparent">
                            <span className="material-symbols-outlined ui-modal-close hover:ui-modal-close-hover">close</span>
                        </Dialog.Close>

                        {/* Role select */}
                        <div className="mb-6">
                            <label className="block mb-2 text-sm ui-text" htmlFor="role">{t('role')}</label>
                            <CustomSelect className="text-left w-full" id="role" onChange={(v) => { setInvite({ ...invite, role: v as components["schemas"]["InvitePostModel"]["role"] }) }} options={roleOptions} placeholder={t('select-role')} value={invite.role} />
                        </div>

                        {/* Explicit content toggle */}
                        <div className="flex items-center gap-4 mb-6">
                            <label className="text-sm ui-text" htmlFor="allow-explicit-content">{t('allow-explicit-content')}</label>
                            <Switcher checked={invite.explicitConsent} id="allow-explicit-content" onChange={() => { setInvite({ ...invite, explicitConsent: !invite.explicitConsent }) }} />
                        </div>

                        <CustomButtonPrimary className="float-right" onClick={() => {
                            const modifiedInvite = {
                                role: invite.role,
                                explicitConsent: invite.explicitConsent
                            } satisfies components["schemas"]["InvitePostModel"]

                            createInviteMutation.mutateAsync({
                                body: modifiedInvite
                            }).then((v) => {
                                enqueueSnackbar(t('invite-created'), { variant: 'success' })
                                onCreated(v)
                                onOpenChange(false)
                            })
                        }}>{t('create-invite')}</CustomButtonPrimary>
                    </div>
                </Dialog.Content>
            </Dialog.Portal>
        </Dialog.Root>
    )
}
