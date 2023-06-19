import {useState} from "react"
import {createPortal} from "react-dom"
import {useTranslation} from "react-i18next"
import axios from "axios"
import {enqueueSnackbar} from "notistack"
import {useAppDispatch, useAppSelector} from "../store/hooks"
import {setCreateInviteModalOpen, setInvites} from "../store/CommonSlice"
import {apiURL} from "../utils/Utilities"
import {CustomButtonPrimary} from "./CustomButtonPrimary"
import {Heading2} from "./Heading2"
import {Switcher} from "./Switcher"
import "material-symbols/outlined.css"
import { CustomSelect } from './CustomSelect'

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
    const dispatch = useAppDispatch()
    const inviteModalOpen = useAppSelector(state=>state.common.createInviteModalOpen)
    const {t} = useTranslation()
    const [invite, setInvite] = useState<Invite>({role: "User", explicitConsent: false})
    const invites = useAppSelector(state=>state.common.invites)

    return createPortal(
        <div aria-hidden="true" id="defaultModal" onClick={()=>dispatch(setCreateInviteModalOpen(false))} className={`grid place-items-center fixed inset-0 bg-[rgba(0,0,0,0.5)] backdrop-blur overflow-x-hidden overflow-y-auto z-30 ${inviteModalOpen ? 'opacity-100' : 'opacity-0 pointer-events-none'}`} tabIndex={-1}>

            {/* Modal */}
            <div className="relative bg-white max-w-5xl p-8 rounded-2xl shadow-[0_4px_16px_rgba(0,0,0,0.2)]" onClick={e=>e.stopPropagation()}>

                {/* Close button */}
                <button type="button" className="absolute top-4 right-4 bg-transparent" data-modal-toggle="defaultModal" onClick={()=>dispatch(setCreateInviteModalOpen(false))}>
                    <span className="material-symbols-outlined text-stone-400 hover:text-stone-600">close</span>
                    <span className="sr-only">{t('closeModal')}</span>
                </button>

                <Heading2 className="mb-4">{t('add-invite')}</Heading2>

                {/* Role select */}
                <div className="mb-6">
                    <label className="block mb-2 text-sm text-stone-900" htmlFor="role">{t('role')}</label>
                    <CustomSelect className="text-left w-full" id="role" onChange={(v)=> {setInvite({...invite,role: v})}} options={roleOptions} placeholder={t('select-role')} value={invite.role} />
                </div>

                {/* Explicit content toggle */}
                <div className="flex items-center gap-4 mb-6">
                    <label className="text-sm text-stone-900" htmlFor="allow-explicit-content">{t('allow-explicit-content')}</label>
                    <Switcher checked={invite.explicitConsent} id="allow-explicit-content" setChecked={()=>{setInvite({...invite, explicitConsent: !invite.explicitConsent})}}/>
                </div>

                <CustomButtonPrimary className="float-right" onClick={()=>{
                    /* TODO: Inconsistency between GET (lowercase role) and PUT (capitalized role) */
                    axios.post(apiURL+'/users/invites', invite)
                        .then((v)=>{
                            enqueueSnackbar(t('invite-created'), {variant: "success"})
                            dispatch(setInvites([...invites,v.data]))
                            dispatch(setCreateInviteModalOpen(false))
                    })
                }}>{t('create-invite')}</CustomButtonPrimary>

            </div>

        </div>, document.getElementById('modal')!
    )
}
