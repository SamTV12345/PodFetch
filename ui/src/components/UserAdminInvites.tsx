import { useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useSnackbar } from 'notistack'
import useCommon from '../store/CommonSlice'
import {formatTime } from '../utils/Utilities'
import { CreateInviteModal } from './CreateInviteModal'
import { CustomButtonPrimary } from './CustomButtonPrimary'
import { CustomSelect, Option } from './CustomSelect'
import { Loading } from './Loading'
import { ErrorIcon } from '../icons/ErrorIcon'
import 'material-symbols/outlined.css'
import copy from 'copy-text-to-clipboard'
import {client} from "../utils/http";

export type Invite = {
    id: string,
    role: string,
    createdAt: string,
    acceptedAt: string,
    expiresAt: string
}

enum InviteTypeSelection {
    all = 'all',
    pending = 'pending',
    accepted = 'accepted',
    expired = 'expired'
}

const INVITE_KEY_PREFIX = 'invite-status-'

const options: Option[] = [
    {
        label: 'All',
        value: InviteTypeSelection.all,
        translationKey: INVITE_KEY_PREFIX + InviteTypeSelection.all
    },
    {
        value: InviteTypeSelection.accepted,
        label: 'Accepted',
        translationKey: INVITE_KEY_PREFIX + InviteTypeSelection.accepted
    },
    {
        value: InviteTypeSelection.pending,
        label: 'Pending',
        translationKey: INVITE_KEY_PREFIX + InviteTypeSelection.pending
    },
    {
        value: InviteTypeSelection.expired,
        label: 'Expired',
        translationKey: INVITE_KEY_PREFIX + InviteTypeSelection.expired
    }
]


export const UserAdminInvites = () => {
    const invites = useCommon(state => state.invites)
    const { enqueueSnackbar } = useSnackbar()
    const [error, setError] = useState<boolean>()
    const [selectedInviteType, setSelectedInviteType] = useState<InviteTypeSelection>(InviteTypeSelection.all)
    const { t } = useTranslation()
    const setCreateInviteModalOpen = useCommon(state => state.setCreateInviteModalOpen)
    const setInvites = useCommon(state => state.setInvites)

    const filteredInvites = useMemo(() => {
       switch (selectedInviteType) {
            case InviteTypeSelection.all:
                return invites
            case InviteTypeSelection.pending:
                return invites.filter(v => v.acceptedAt == null && v.expiresAt > new Date().toISOString())
            case InviteTypeSelection.accepted:
                return invites.filter(v => v.acceptedAt !== null)
            case InviteTypeSelection.expired:
                return invites.filter(v => v.expiresAt < new Date().toISOString() && v.acceptedAt == null)
       }
    }, [invites, selectedInviteType])

    useEffect(() => {
        client.GET("/api/v1/invites").then((v)=>{
            setInvites(v.data!)
            setError(false)
        }).catch(()=>{
            setError(true)
        })
    }, [])

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
            <CreateInviteModal />

            <CustomSelect options={options} value={selectedInviteType} onChange={v => setSelectedInviteType(v as InviteTypeSelection)} />

            <CustomButtonPrimary className="flex items-center xs:float-right mb-4 xs:mb-10" onClick={() => {
                setCreateInviteModalOpen(true)
            }}>
                <span className="material-symbols-outlined leading-[0.875rem]">add</span> {t('add-new')}
            </CustomButtonPrimary>

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
                                ID
                            </th>
                            <th scope="col" className="px-2 py-3">
                                {t('role')}
                            </th>
                            <th scope="col" className="px-2 py-3">
                                {t('created')}
                            </th>
                            <th scope="col" className="px-2 py-3">
                                {t('expires-at')}
                            </th>
                            <th scope="col" className="px-2 py-3">
                                {t('accepted-at')}
                            </th>
                            <th scope="col" className="pl-2 py-3">
                            </th>
                        </tr>
                    </thead>
                    <tbody>
                        {filteredInvites.map(i=>
                            <tr className="border-b border-[--border-color]" key={i.id}>
                                <td className="pr-2 py-4">
                                    <button className="text-left text-[--fg-color] hover:text-[--fg-color-hover]" onClick={() => {
                                        client.GET("/api/v1/invites/{invite_id}/link", {
                                            params: {
                                                path: {
                                                    invite_id: i.id
                                                }
                                            }
                                        }).then((v)=>{
                                            copy(v.data!)
                                            enqueueSnackbar(t('invite-link-copied'), { autoHideDuration: 2000, variant: 'success'})
                                        })
                                    }} title={t('copy-link')}>
                                        {i.id}
                                        <span className="material-symbols-outlined align-middle ml-1">link</span>
                                    </button>
                                </td>
                                <td className="px-2 py-4">
                                    {t(i.role)}
                                </td>
                                <td className="px-2 py-4">
                                    {formatTime(i.createdAt)}
                                </td>
                                <td className="px-2 py-4">
                                    {formatTime(i.expiresAt)}
                                </td>
                                <td className="px-2 py-4">
                                    {i.acceptedAt ? (
                                        formatTime(i.acceptedAt)
                                    ) : (
                                        <span>-</span>
                                    )}
                                </td>
                                <td className="pl-2 py-4">
                                    <button className="flex items-center float-right text-[--danger-fg-color] hover:text-[--danger-fg-color-hover]" onClick={() => {
                                        client.DELETE("/api/v1/invites/{invite_id}", {
                                            params: {
                                                path: {
                                                    invite_id: i.id
                                                }
                                            }
                                        }).then(() => {
                                            enqueueSnackbar(t('invite-deleted'), { variant: 'success' })
                                            setInvites(invites.filter(v => v.id !== i.id))
                                        })
                                    }}>
                                        <span className="material-symbols-outlined mr-1">delete</span>
                                        {t('delete')}
                                    </button>
                                </td>
                            </tr>
                        )}
                    </tbody>
                </table>
            </div>
        </div>
    )
}
