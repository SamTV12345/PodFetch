import { FC, useState } from 'react'
import { Dialog, DialogContent, DialogTitle } from '@/components/ui/dialog'
import { useTranslation } from 'react-i18next'
import { enqueueSnackbar } from 'notistack'
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
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-5xl w-full sm:max-w-5xl">
                <DialogTitle className="font-bold leading-tight! text-xl xs:text-2xl ui-text mb-4">{t('add-invite')}</DialogTitle>

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
            </DialogContent>
        </Dialog>
    )
}
