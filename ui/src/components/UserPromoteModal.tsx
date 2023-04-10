import {Modal} from "./Modal";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {useTranslation} from "react-i18next";
import {setSelectedUser, setUsers} from "../store/CommonSlice";
import axios from "axios";
import {apiURL} from "../utils/Utilities";
import {enqueueSnackbar} from "notistack";
import {User} from "../models/User";

export const UserPromoteModal = () => {
    const selectedUser = useAppSelector(state=>state.common.selectedUser)
    const {t} = useTranslation()
    const dispatch = useAppDispatch()
    const users = useAppSelector(state=>state.common.users)
    function capitalizeFirstLetter(string: string|undefined) {
        if(string === undefined) return ""
        return string.charAt(0).toUpperCase() + string.slice(1);
    }

    const changeRole = () => {
        axios.put(apiURL+"/users/"+selectedUser?.username+"/role", {role: capitalizeFirstLetter(selectedUser?.role)})
            .then(()=>{
                enqueueSnackbar(t('role-changed'), {variant: "success"})
                const mapped_users = users.map(u=>{
                    if(u.username === selectedUser?.username) {
                        return {
                            ...u,
                            role: selectedUser.role
                        } satisfies User
                    }
                    return u
                })
                dispatch(setUsers(mapped_users))
            })
    }

    return <Modal headerText={t('change-role-user', {name: selectedUser?.username})!} onCancel={()=>{}} onAccept={()=>{}} onDelete={()=>{}} cancelText="Test" acceptText="Test123">
        <label htmlFor="countries" className="block mb-2 font-medium text-gray-900 dark:text-white">{t('select-role')}</label>
        <select id="countries" value={selectedUser?.role} onChange={(v)=> {
         dispatch(setSelectedUser({...selectedUser!, role: v.target.value}))
        }} className="border rounded-lg block w-full p-2.5 bg-gray-800 border-gray-600 placeholder-gray-400 text-white focus:ring-blue-500 focus:border-blue-500">
            <option value="admin">{t('admin')}</option>
            <option value="user">{t('user')}</option>
            <option value="uploader">{t('uploader')}</option>
        </select>
        <div className="flex">
            <div className=" flex-1"></div>
            <button className="bg-blue-700 p-2 text-white" onClick={changeRole}>{t('change-role')}</button>
        </div>
    </Modal>
}
