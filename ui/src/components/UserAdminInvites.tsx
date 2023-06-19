import {useEffect, useState} from "react"
import {useTranslation} from "react-i18next"
import axios from "axios"
import {useSnackbar} from "notistack"
import {useAppDispatch, useAppSelector} from "../store/hooks"
import {setCreateInviteModalOpen, setInvites} from "../store/CommonSlice"
import {apiURL, formatTime} from "../utils/Utilities"
import {CreateInviteModal} from "./CreateInviteModal"
import {CustomButtonPrimary} from "./CustomButtonPrimary"
import {Loading} from "./Loading"
import {ErrorIcon} from "../icons/ErrorIcon"
import "material-symbols/outlined.css"

export type Invite = {
    id: string,
    role: string,
    createdAt: string,
    acceptedAt: string,
    expiresAt: string
}

export const UserAdminInvites = () => {
    const {t} = useTranslation()
    const {enqueueSnackbar} = useSnackbar()
    const dispatch = useAppDispatch()
    const invites = useAppSelector(state=>state.common.invites)
    const [error,setError] = useState<boolean>()

    useEffect(()=>{
        axios.get(apiURL+"/users/invites").then(v=> {
            dispatch(setInvites(v.data))
            setError(false)
        }).catch(()=>{
                    setError(true)
                })
        }, [])

    if (error === undefined){
        return <Loading />
    }

    if(error){
        return <ErrorIcon text={t('not-admin')} />
    }

    return (
        <div>
            <CreateInviteModal />

            <CustomButtonPrimary className="flex items-center xs:float-right mb-4 xs:mb-10" onClick={()=>{
                dispatch(setCreateInviteModalOpen(true))
            }}>
                <span className="material-symbols-outlined leading-[0.875rem]">add</span> {t('add-new')}
            </CustomButtonPrimary>

            <div className={`
                scrollbox-x
                w-[calc(100vw-2rem)] ${/* viewport - padding */ ''}
                xs:w-[calc(100vw-4rem)] ${/* viewport - padding */ ''}
                md:w-[calc(100vw-18rem-4rem)] ${/* viewport - sidebar - padding */ ''}
            `}>
                <table className="text-left text-sm text-stone-900 w-full">
                    <thead>
                        <tr className="border-b border-stone-300">
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
                        {invites.map(i=>
                            <tr className="border-b border-stone-300" key={i.id}>
                                <td className="pr-2 py-4">
                                    <button className="text-left text-stone-900 hover:text-stone-600" onClick={()=>{
                                        axios.get(apiURL+"/users/invites/"+i.id+"/link").then(v=>{
                                            navigator.clipboard.writeText(v.data)
                                            enqueueSnackbar(t('invite-link-copied'), {autoHideDuration: 2000})
                                        })
                                    }} title={t('copy-link') || ''}>
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
                                    <button className="flex items-center float-right text-red-700 hover:text-red-500" onClick={()=>{
                                        axios.delete(apiURL+"/users/invites/"+i.id).then(()=>{
                                            enqueueSnackbar(t('invite-deleted'), {variant: "success"})
                                            dispatch(setInvites(invites.filter(v=>v.id !== i.id)))
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
