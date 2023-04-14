import {useTranslation} from "react-i18next";
import {useEffect, useState} from "react";
import {User} from "../models/User";
import {apiURL, formatTime} from "../utils/Utilities";
import axios from "axios";
import {UserPromoteModal} from "../components/UserPromoteModal";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {setSelectedUser, setUsers} from "../store/CommonSlice";
import {setModalOpen} from "../store/ModalSlice";
import {Loading} from "../components/Loading";
import {ErrorIcon} from "../icons/ErrorIcon";

export const AdministrationUserPage = () => {
    const {t} = useTranslation()
    const users = useAppSelector(state=>state.common.users)
    const dispatch = useAppDispatch()
    const [error, setError] = useState<boolean>()

    useEffect(()=>{
        axios.get(apiURL+ "/users")
            .then(c=>{
                dispatch(setUsers(c.data))
                setError(false)
            }).catch(()=>setError(true))
    },[])

    const deleteUser = (user: User) => {
        axios.delete(apiURL+"/users/"+user.username)
            .then(()=>{
                dispatch(setUsers(users.filter(u=>u.username !== user.username)))
            })
    }

    if (error === undefined){
        return <Loading/>
    }

    if(error){
        return <ErrorIcon text={t('not-admin')}/>
    }

    return <div className="p-5">
        <h1 className="text-3xl text-center mt-2">{t('manage-users')}</h1>
        <UserPromoteModal/>

        <div className="relative overflow-x-auto">
            <table className="w-full text-sm text-left text-gray-400 rounded">
                <thead className="text-xs uppercase bg-gray-700 text-gray-400">
                <tr>
                    <th scope="col" className="px-6 py-3">
                        #
                    </th>
                    <th scope="col" className="px-6 py-3">
                        {t('username')}
                    </th>
                    <th scope="col" className="px-6 py-3">
                        {t('role')}
                    </th>
                    <th scope="col" className="px-6 py-3">
                        {t('created')}
                    </th>
                    <th>
                        {t('actions')}
                    </th>
                </tr>
                </thead>
                <tbody>
                {
                    users.map((v)=>

                        <tr className="bg-white border-b dark:bg-gray-800 dark:border-gray-700">
                            <th scope="row"
                                className="px-6 py-4 font-medium text-gray-900 whitespace-nowrap dark:text-white">
                                {v.id}
                            </th>
                            <td className="px-6 py-4">
                                {v.username}
                            </td>
                            <td className="px-6 py-4">
                                {v.role}
                            </td>
                            <td className="px-6 py-4">
                                {formatTime(v.createdAt)}
                            </td>
                            <td className=" py-4 flex gap-5">
                                <button title="Change role" onClick={()=>{
                                    dispatch(setSelectedUser({
                                        role: v.role,
                                        id: v.id,
                                        createdAt: v.createdAt,
                                        username: v.username
                                    }))

                                    dispatch(setModalOpen(true))
                                }}><i className="fa fa-ranking-star text-white"></i></button>
                                <button title="Ban/Delete user" onClick={()=>{
                                    deleteUser(v)
                                }}><i className="fa fa-ban text-red-700"></i></button>
                            </td>
                        </tr>
                    )}
                </tbody>
            </table>
        </div>
    </div>
}
