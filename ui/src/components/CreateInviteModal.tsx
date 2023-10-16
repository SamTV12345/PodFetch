import { useState } from 'react'
import { createPortal } from 'react-dom'
import { useTranslation } from 'react-i18next'
import axios from 'axios'
import { enqueueSnackbar } from 'notistack'
import useCommon from '../store/CommonSlice'
import { apiURL } from '../utils/Utilities'
import { CustomButtonPrimary } from './CustomButtonPrimary'
import { CustomSelect } from './CustomSelect'
import { Heading2 } from './Heading2'
import { Switcher } from './Switcher'
import 'material-symbols/outlined.css'

type Invite = {
    role: string,
    explicitConsent: boolean
}

const roleOptions = [
    { translationKey: 'admin', value: 'Admin' },
    { translationKey: 'user', value: 'User' },
    { translationKey: 'uploader', value: 'Uploader' }
]

export const CreateInviteModal = () => {
    const inviteModalOpen = useCommon(state => state.createInviteModalOpen)
    const invites = useCommon(state => state.invites)
    const [invite, setInvite] = useState<Invite>({ role: 'User', explicitConsent: false })
    const { t } = useTranslation()
    const setCreateInviteModalOpen = useCommon(state => state.setCreateInviteModalOpen)
    const setInvites = useCommon(state => state.setInvites)

    return createPortal(
        <div aria-hidden="true" id="defaultModal" onClick={() => setCreateInviteModalOpen(false)} className={`grid place-items-center fixed inset-0 bg-[rgba(0,0,0,0.5)] backdrop-blur overflow-x-hidden overflow-y-auto z-30 ${inviteModalOpen ? 'opacity-100' : 'opacity-0 pointer-events-none'}`} tabIndex={-1}>

            {/* Modal */}
            <div className="relative bg-[--bg-color] max-w-5xl p-8 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,0.2)]" onClick={e => e.stopPropagation()}>

                {/* Close button */}
                <button type="button" className="absolute top-4 right-4 bg-transparent" data-modal-toggle="defaultModal" onClick={() => setCreateInviteModalOpen(false)}>
                    <span className="material-symbols-outlined text-[--modal-close-color] hover:text-[--modal-close-color-hover]">close</span>
                    <span className="sr-only">{t('closeModal')}</span>
                </button>

                <Heading2 className="mb-4">{t('add-invite')}</Heading2>

                {/* Role select */}
                <div className="mb-6">
                    <label className="block mb-2 text-sm text-[--fg-color]" htmlFor="role">{t('role')}</label>
                    <CustomSelect className="text-left w-full" id="role" onChange={(v) => {setInvite({ ...invite, role: v })}} options={roleOptions} placeholder={t('select-role')} value={invite.role} />
                </div>

                {/* Explicit content toggle */}
                <div className="flex items-center gap-4 mb-6">
                    <label className="text-sm text-[--fg-color]" htmlFor="allow-explicit-content">{t('allow-explicit-content')}</label>
                    <Switcher checked={invite.explicitConsent} id="allow-explicit-content" setChecked={() => {setInvite({ ...invite, explicitConsent: !invite.explicitConsent })}}/>
                </div>

                <CustomButtonPrimary className="float-right" onClick={() => {
                    const modifiedInvite = {
                        role: invite.role.toLowerCase(),
                        explicitConsent: invite.explicitConsent
                    } satisfies Invite
                    axios.post(apiURL + '/users/invites', modifiedInvite)
                        .then((v) => {
                            enqueueSnackbar(t('invite-created'), { variant: 'success' })
                            setInvites([...invites,v.data])
                            setCreateInviteModalOpen(false)
                        })
                }}>{t('create-invite')}</CustomButtonPrimary>

            </div>

        </div>, document.getElementById('modal')!
    )
}
