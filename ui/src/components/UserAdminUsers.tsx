import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useSnackbar } from 'notistack'
import useCommon from '../store/CommonSlice'
import { formatTime} from '../utils/Utilities'
import { User } from '../models/User'
import { ChangeRoleModal } from './ChangeRoleModal'
import { Loading } from './Loading'
import { ErrorIcon } from '../icons/ErrorIcon'
import 'material-symbols/outlined.css'
import {$api} from "../utils/http";
import {components} from "../../schema";

export const UserAdminUsers = () => {
    const setUsers = useCommon(state => state.setUsers)
    const users = useCommon(state => state.users)
    const [error, setError] = useState<boolean>()
    const {enqueueSnackbar} = useSnackbar()
    const { t } = useTranslation()
    const [changeRoleOpen, setChangeRoleOpen] = useState(false)
    const [userToEdit, setUserToEdit] = useState<components["schemas"]["UserSummary"] | undefined>(undefined)
    const usersQuery = $api.useQuery('get', '/api/v1/users', {}, {retry: false})
    const deleteUserMutation = $api.useMutation('delete', '/api/v1/users/{username}')

    useEffect(() => {
        if (usersQuery.isError) {
            setError(true)
            return
        }
        if (usersQuery.data) {
            setUsers(usersQuery.data)
            setError(false)
        }
    }, [setUsers, usersQuery.data, usersQuery.isError])

    const deleteUser = (user: User) => {
        deleteUserMutation.mutateAsync({
            params: {
                path: {
                    username: user.username
                }
            }
        }).then(() => {
            enqueueSnackbar(t('user-deleted'), { variant: 'success' })
            setUsers(users.filter(u => u.username !== user.username))
        })
    }

    if (error === undefined) {
        return <Loading />
    }

    if (error) {
        return <div className="w-full md:w-3/4 mx-auto">
            <ErrorIcon text={t('not-admin')} />
        </div>
    }

    return (
        <div>
            <ChangeRoleModal open={changeRoleOpen} onOpenChange={setChangeRoleOpen} user={userToEdit} onSuccess={(updatedUser) => { setUsers(users.map(u => u.username === updatedUser.username ? {...u, role: updatedUser.role, explicitConsent: updatedUser.explicitConsent} : u)) }} />

            <div className={`
                scrollbox-x
                w-[calc(100vw-2rem)] ${/* viewport - padding */ ''}
                xs:w-[calc(100vw-4rem)] ${/* viewport - padding */ ''}
                md:w-[calc(100vw-18rem-4rem)] ${/* viewport - sidebar - padding */ ''}
            `}>
                <table className="text-left text-sm ui-text w-full">
                    <thead>
                        <tr className="border-b ui-border">
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
                            <tr className="border-b ui-border" key={v.id}>
                                <td className="pr-2 py-4">
                                    {v.username}
                                </td>
                                <td className="flex items-center px-2 py-4">
                                    {t(v.role)}

                                    <button className="flex ml-2" onClick={() => {
                                        setUserToEdit({
                                            role: v.role,
                                            id: v.id,
                                            createdAt: v.createdAt,
                                            explicitConsent: v.explicitConsent,
                                            username: v.username
                                        })

                                        setChangeRoleOpen(true)
                                    }} title={t('change-role') || ''}>
                                        <span className="material-symbols-outlined ui-text hover:ui-text-hover">edit</span>
                                    </button>
                                </td>
                                <td className="px-2 py-4">
                                    {formatTime(v.createdAt)}
                                </td>
                                <td className="pl-2 py-4">
                                    <button className="flex items-center float-right ui-text-danger hover:ui-text-danger-hover" onClick={() => {
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
