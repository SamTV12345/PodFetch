import {useTranslation} from "react-i18next"
import axios from "axios"
import {enqueueSnackbar} from "notistack"
import {useAppDispatch, useAppSelector} from "../store/hooks"
import {setSelectedUser, setUsers} from "../store/CommonSlice"
import {setModalOpen} from "../store/ModalSlice"
import {apiURL, capitalizeFirstLetter} from "../utils/Utilities"
import {User} from "../models/User"
import {CustomButtonPrimary} from "./CustomButtonPrimary"
import {CustomSelect} from "./CustomSelect"
import {Modal} from "./Modal"
import {Switcher} from "./Switcher"

const roleOptions = [
    { translationKey: 'admin', value: 'admin' },
    { translationKey: 'user', value: 'user' },
    { translationKey: 'uploader', value: 'uploader' }
]

export const ChangeRoleModal = () => {
    const selectedUser = useAppSelector(state=>state.common.selectedUser)
    const {t} = useTranslation()
    const dispatch = useAppDispatch()
    const users = useAppSelector(state=>state.common.users)

    const changeRole = () => {
        /* TODO: Inconsistency between GET (lowercase role) and PUT (capitalized role) */
        axios.put(apiURL+"/users/"+selectedUser?.username+"/role", {
            role: capitalizeFirstLetter(selectedUser?.role),
            explicitConsent: selectedUser?.explicitConsent
        })
            .then(()=>{
                enqueueSnackbar(t('role-changed'), {variant: "success"})

                const mapped_users = users.map(u=>{
                    if(u.username === selectedUser?.username) {
                        return {
                            ...u,
                            role: selectedUser.role,
                            explicitConsent: selectedUser.explicitConsent
                        } satisfies User
                    }
                    return u
                })

                dispatch(setUsers(mapped_users))
                dispatch(setModalOpen(false))
            })
    }

    return (
        <Modal headerText={t('change-role-user', {name: selectedUser?.username})!} onCancel={()=>{}} onAccept={()=>{}} onDelete={()=>{}} cancelText="Test" acceptText="Test123">

            {/* Role select */}
            <div className="mb-6">
                <label className="block mb-2 text-sm text-stone-900" htmlFor="role">{t('role')}</label>
                <CustomSelect className="text-left w-full" id="role" onChange={(v)=> {
                dispatch(setSelectedUser({...selectedUser!, role: v}))
                }} options={roleOptions} placeholder={t('select-role')} value={selectedUser?.role || ''} />
            </div>

            {/* Explicit content toggle */}
            <div className="flex items-center gap-4 mb-6">
                <label className="text-sm text-stone-900" htmlFor="allow-explicit-content">{t('allow-explicit-content')}</label>
                <Switcher checked={selectedUser?.explicitConsent || false} id="allow-explicit-content" setChecked={()=>{dispatch(setSelectedUser({...selectedUser!, explicitConsent: !selectedUser?.explicitConsent}))}}/>
            </div>

            <CustomButtonPrimary className="float-right" onClick={changeRole}>{t('change-role')}</CustomButtonPrimary>

        </Modal>
    )
}
