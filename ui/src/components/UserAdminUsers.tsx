import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import axios from 'axios'
import { useSnackbar } from 'notistack'
import { useAppDispatch, useAppSelector } from '../store/hooks'
import { setSelectedUser, setUsers } from '../store/CommonSlice'
import { setModalOpen } from '../store/ModalSlice'
import { apiURL, formatTime} from '../utils/Utilities'
import { User } from '../models/User'
import { ChangeRoleModal } from './ChangeRoleModal'
import { Loading } from './Loading'
import { ErrorIcon } from '../icons/ErrorIcon'
import 'material-symbols/outlined.css'

export const UserAdminUsers = () => {
    const dispatch = useAppDispatch()
    const users = useAppSelector(state => state.common.users)
    const [error, setError] = useState<boolean>()
    const {enqueueSnackbar} = useSnackbar()
    const { t } = useTranslation()

    useEffect(() => {
        axios.get(apiURL + '/users')
            .then(c => {
                dispatch(setUsers(c.data))
                setError(false)
            })
            .catch(() => setError(true))
    }, [])

    const deleteUser = (user: User) => {
        axios.delete(apiURL + '/users/' + user.username)
            .then(() => {
                enqueueSnackbar(t('user-deleted'), { variant: 'success' })
                dispatch(setUsers(users.filter(u => u.username !== user.username)))
            })
    }

    if (error === undefined) {
        return <Loading />
    }

    if (error) {
        return <ErrorIcon text={t('not-admin')} />
    }

    return (
        <div>
            <ChangeRoleModal />

            <div className={`
                scrollbox-x
                w-[calc(100vw-2rem)] ${/* viewport - padding */ ''}
                xs:w-[calc(100vw-4rem)] ${/* viewport - padding */ ''}
                md:w-[calc(100vw-18rem-4rem)] ${/* viewport - sidebar - padding */ ''}
            `}>
                <table className="text-left text-sm text-[--fg-color] w-full">
                    <thead>
                        <tr className="border-b border-[--border-color]">
                            <th scope="col" className="pr-2 py-3">
                                {t('username')}
                            </th>
                            <th scope="col" className="px-2 py-3">
                                {t('role')}
                            </th>
                            <th scope="col" className="px-2 py-3">
                                {t('created')}
                            </th>
                            <th scope="col" className="pl-2 py-3">
                            </th>
                        </tr>
                    </thead>
                    <tbody>
                        {users.map((v) => (
                            <tr className="border-b border-[--border-color]" key={v.id}>
                                <td className="pr-2 py-4">
                                    {v.username}
                                </td>
                                <td className="flex items-center px-2 py-4">
                                    {t(v.role)}

                                    <button className="flex ml-2" onClick={() => {
                                        dispatch(setSelectedUser({
                                            role: v.role,
                                            id: v.id,
                                            createdAt: v.createdAt,
                                            explicitConsent: v.explicitConsent,
                                            username: v.username
                                        }))

                                        dispatch(setModalOpen(true))
                                    }} title={t('change-role') || ''}>
                                        <span className="material-symbols-outlined text-[--fg-color] hover:text-[--fg-color-hover]">edit</span>
                                    </button>
                                </td>
                                <td className="px-2 py-4">
                                    {formatTime(v.createdAt)}
                                </td>
                                <td className="pl-2 py-4">
                                    <button className="flex items-center float-right text-[--danger-fg-color] hover:text-[--danger-fg-color-hover]" onClick={() => {
                                        deleteUser(v)
                                    }}>
                                        <span className="material-symbols-outlined mr-1">delete</span>
                                        {t('delete')}
                                    </button>
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </div>
    )
}
