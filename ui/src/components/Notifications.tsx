import {BellIcon} from "./BellIcon";
import {useState} from "react";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {Notification} from "../models/Notification";
import axios from "axios";
import {apiURL} from "../utils/Utilities";
import {removeNotification, setNotifications} from "../store/CommonSlice";

export const Notifications = () => {
    const notifications = useAppSelector(state=>state.common.notifications)
    const [open, setOpen] = useState<boolean>(false)
    const dispatch = useAppDispatch()

    const dismissNotification = (notification: Notification)=>{
        axios.put(apiURL+'/notifications/dismiss', {id: notification.id})
            .then(c=>{
                dispatch(removeNotification(notification.id))
            })}

    const displayNotifications = ()=>{
        if(notifications.length===0){
            return <div className="text-center h-20 place-items-center flex text-white ml-2 mr-2 pt-2 text-gray-500">
                No notifications
            </div>
        }
        else{
            return notifications.map((notification, index) => {
                return <div className="relative flex text-white ml-2 mr-2 border-b-2 border-b-amber-800 pt-4" key={index}>
                    {notification.message}
                    <div className="absolute right-0 top-0 cursor-pointer" onClick={()=>{dismissNotification(notification)}}>
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-4 h-4">
                            <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </div>
                </div>
            })
        }
    }

    return <div className="relative">
        <BellIcon onClick={()=>setOpen(!open)}/>
        {notifications.length>0 &&<div className="absolute top-0 right-0 w-2 h-2 bg-red-500 rounded-full"/>}
        <div className="absolute" id="positionwrapper">
        <div className={`absolute bottom-0 z-50 w-screen md:w-60 bg-black rounded invisible opacity-0 ${open?'notification-modal-visible':'hidden'}  transition-opacity`} id='notification-modal'>
            <div className="flex flex-col gap-1 bg-gray-700 overflow-y-auto max-h-80" id="notification-body">
                {displayNotifications()}
            </div>
        </div>
</div>
    </div>
}
