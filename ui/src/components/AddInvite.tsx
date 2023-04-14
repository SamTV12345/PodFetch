import {createPortal} from "react-dom";
import {
    setAddInviteModalOpen, setInvites,
} from "../store/CommonSlice";
import {apiURL} from "../utils/Utilities";
import axios from "axios";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {setModalOpen} from "../store/ModalSlice";
import {useTranslation} from "react-i18next";
import {useState} from "react";


type Invite = {
    role: string,
    explicitConsent: boolean
}

export const AddInvite = () => {
    const dispatch = useAppDispatch()
    const inviteModalOpen = useAppSelector(state=>state.common.addInviteModalOpen)
    const {t} = useTranslation()
    const [invite, setInvite] = useState<Invite>({role: "User", explicitConsent: false})
    const invites = useAppSelector(state=>state.common.invites)


    return createPortal( <div id="defaultModal" tabIndex={-1} aria-hidden="true" onClick={()=>dispatch(setAddInviteModalOpen(false))}
                              className={`overflow-y-auto overflow-x-hidden fixed top-0 right-0 left-0 z-50 md:inset-0 h-modal md:h-full
             ${!inviteModalOpen&&'pointer-events-none'}
              z-40 ${inviteModalOpen?'opacity-100':'opacity-0'}`}>
        <div className="grid place-items-center h-screen ">
            <div className={`bg-gray-800 max-w-5xl ${inviteModalOpen?'opacity-100':'opacity-0'}`} onClick={e=>e.stopPropagation()}>
                <div className="flex justify-between items-start p-4 rounded-t border-b border-gray-600">
                    <h3 className="text-xl font-semibold text-white">
                        {t('add-invite')}
                    </h3>
                    <button type="button" className="text-gray-400 bg-transparent rounded-lg text-sm p-1.5 ml-auto inline-flex items-center hover:bg-gray-600 hover:text-white" data-modal-toggle="defaultModal" onClick={()=>dispatch(setModalOpen(false))}>
                        <svg aria-hidden="true" className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"><path fillRule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clipRule="evenodd"></path></svg>
                        <span className="sr-only">{t('closeModal')}</span>
                    </button>
                </div>
                <div className="p-6 space-y-6 text-base leading-relaxed text-gray-400">
                    <div className="grid grid-cols-2">
                        <select id="roles" value={invite.role} onChange={(v)=> {setInvite({...invite,role: v.target.value})}} className="border rounded-lg block w-full p-2.5 bg-gray-800 border-gray-600 placeholder-gray-400 text-white focus:ring-blue-500 focus:border-blue-500">
                            <option value="Admin">{t('admin')}</option>
                            <option value="User">{t('user')}</option>
                            <option value="Uploader">{t('uploader')}</option>
                        </select>
                    </div>
                    <div>
                        <label className="relative inline-flex items-center cursor-pointer">
                            <input type="checkbox" checked={invite.explicitConsent} onChange={()=>{setInvite({...invite, explicitConsent: !invite.explicitConsent})}} className="sr-only peer"/>
                                <div
                                    className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-blue-600"></div>
                                <span
                                    className="ml-3 text-sm font-medium text-gray-900 dark:text-gray-300">{t('allow-explicit-content')}</span>
                        </label>
                    </div>
                    <div className="flex">
                        <div className="flex-1"></div>
                        <button className="bg-slate-500 text-white p-2 rounded"
                        onClick={()=>{
                            axios.post(apiURL+'/users/invites', invite)
                                .then((v)=>{
                                    dispatch(setInvites([...invites,v.data]))
                                    dispatch(setAddInviteModalOpen(false))
                            })
                        }}
                        >{t('create-invite')}</button>
                    </div>
                </div>

                </div>
            </div>
        </div>, document.getElementById('modal')!)
}
