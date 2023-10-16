import { useTranslation } from 'react-i18next'
import axios from 'axios'
import { enqueueSnackbar } from 'notistack'
import useCommon from '../store/CommonSlice'
import { apiURL } from '../utils/Utilities'
import { User } from '../models/User'
import { CustomButtonPrimary } from './CustomButtonPrimary'
import { CustomSelect } from './CustomSelect'
import { Modal } from './Modal'
import { Switcher } from './Switcher'
import useModal from "../store/ModalSlice";

const roleOptions = [
    { translationKey: 'admin', value: 'admin' },
    { translationKey: 'user', value: 'user' },
    { translationKey: 'uploader', value: 'uploader' }
]

export const ChangeRoleModal = () => {
    const selectedUser = useCommon(state => state.selectedUser)
    const users = useCommon(state => state.users)
    const { t } = useTranslation()
    const setModalOpen = useModal(state => state.setOpenModal)
    const setSelectedUser = useCommon(state => state.setSelectedUser)
    const setUsers = useCommon(state => state.setUsers)

    //setSelectedUser, setUsers
    const changeRole = () => {
        axios.put(apiURL + '/users/' + selectedUser?.username + '/role', {
            role: selectedUser?.role,
            explicitConsent: selectedUser?.explicitConsent
        })
            .then(() => {
                enqueueSnackbar(t('role-changed'), { variant: 'success' })

                const mapped_users = users.map(u => {
                    if (u.username === selectedUser?.username) {
                        return {
                            ...u,
                            role: selectedUser.role,
                            explicitConsent: selectedUser.explicitConsent
                        } satisfies User
                    }

                    return u
                })

                setUsers(mapped_users)
                setModalOpen(false)
            })
    }

    return (
        <Modal headerText={t('change-role-user', {name: selectedUser?.username})!} onCancel={() => {}} onAccept={() => {}} onDelete={() => {}} cancelText="Test" acceptText="Test123">

            {/* Role select */}
            <div className="mb-6">
                <label className="block mb-2 text-sm text-[--fg-color]" htmlFor="role">{t('role')}</label>
                <CustomSelect className="text-left w-full" id="role" onChange={(v) => {
                setSelectedUser({ ...selectedUser!, role: v })
                }} options={roleOptions} placeholder={t('select-role')} value={selectedUser?.role || ''} />
            </div>

            {/* Explicit content toggle */}
            <div className="flex items-center gap-4 mb-6">
                <label className="text-sm text-[--fg-color]" htmlFor="allow-explicit-content">{t('allow-explicit-content')}</label>
                <Switcher checked={selectedUser?.explicitConsent || false} id="allow-explicit-content"
                          setChecked={() => {setSelectedUser({ ...selectedUser!, explicitConsent: !selectedUser?.explicitConsent })}}/>
            </div>

            <CustomButtonPrimary className="float-right" onClick={changeRole}>{t('change-role')}</CustomButtonPrimary>

        </Modal>
    )
}
