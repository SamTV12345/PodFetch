import {useAppDispatch, useAppSelector} from "../store/hooks";
import {setAddInviteModalOpen, setInvites} from "../store/CommonSlice";
import {AddInvite} from "../components/AddInvite";
import {useTranslation} from "react-i18next";
import {useEffect, useState} from "react";
import {apiURL, formatTime} from "../utils/Utilities";
import axios from "axios";
import {ErrorIcon} from "../icons/ErrorIcon";
import {Loading} from "../components/Loading";

export type Invite = {
    id: string,
    role: string,
    createdAt: string,
    acceptedAt: string,
    expiresAt: string
}

export const InviteAdministrationUserPage = () => {
    const dispatch = useAppDispatch()
    const {t} = useTranslation()
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
        return <Loading/>
    }

    if(error){
        return <ErrorIcon text={t('not-admin')}/>
    }

    return <div className="p-5">
        <AddInvite/>
            <h1 className="text-center text-3xl">Invite Administration User</h1>
        <div className="flex mb-5">
            <div className="flex-1"></div>
            <button className="fa fa-plus bg-blue-900 text-white p-3" onClick={()=>{
                dispatch(setAddInviteModalOpen(true))
            }}></button>
        </div>
            <div className="relative overflow-x-auto">
                <table className="w-full text-sm text-left text-gray-400 rounded">
                    <thead className="text-xs uppercase bg-gray-700 text-gray-400">
                    <tr>
                        <th scope="col" className="px-6 py-3">
                            #
                        </th>
                        <th scope="col" className="px-6 py-3">
                            {t('accepted-at')}
                        </th>
                        <th scope="col" className="px-6 py-3">
                            {t('role')}
                        </th>
                        <th scope="col" className="px-6 py-3">
                            {t('created')}
                        </th>
                        <th scope="col" className="px-6 py-3">
                            {t('expires-at')}
                        </th>
                        <th scope="col" className="px-6 py-3">
                            {t('actions')}
                        </th>
                    </tr>
                    </thead>
                    <tbody>
                    {
                        invites.map(i=>
                            <tr className="border-b bg-gray-800 border-gray-700">
                            <th scope="row"
                                className="px-6 py-4 font-medium whitespace-nowrap text-white">
                                {i.id}
                            </th>
                            <td className="px-6 py-4">
                                {i.acceptedAt?formatTime(i.acceptedAt):<i className="fa-solid fa-ban"></i>}
                            </td>
                            <td className="px-6 py-4">
                                {i.role}
                            </td>

                            <td className="px-6 py-4">
                                {formatTime(i.createdAt)}
                            </td>
                                <td className="px-6 py-4">
                                    {formatTime(i.expiresAt)}
                                </td>
                                <td className="">
                                    <button className="fa fa-trash bg-red-900 text-white p-3 rounded" onClick={()=>{
                                        axios.delete(apiURL+"/users/invites/"+i.id).then(()=>{
                                            dispatch(setInvites(invites.filter(v=>v.id !== i.id)))
                                        })
                                    }}></button>
                                    <button>
                                        <i className="fa-solid fa-copy text-2xl" onClick={()=>{
                                            axios.get(apiURL+"/users/invites/"+i.id+"/link").then(v=>{
                                                navigator.clipboard.writeText(v.data)
                                            })
                                        }}></i>
                                    </button>
                                </td>
                        </tr>
                        )
                    }
                    </tbody>
                </table>
            </div>
    </div>
}
