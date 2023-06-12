import {FC, useState} from "react"
import {useTranslation} from "react-i18next"
import axios from "axios"
import * as Popover from '@radix-ui/react-popover'
import {apiURL} from "../utils/Utilities"
import {useAppDispatch, useAppSelector} from "../store/hooks"
import {removeNotification} from "../store/CommonSlice"
import {Notification} from "../models/Notification"
import "material-symbols/outlined.css"

export const Notifications:FC = () => {
    const notifications = useAppSelector(state=>state.common.notifications)
    const dispatch = useAppDispatch()
    const {t}  = useTranslation()

    const trigger = () => <div className="relative">
        <span className="material-symbols-outlined cursor-pointer text-stone-900 hover:text-stone-600">notifications</span>

        {notifications.length>0 &&<div className="absolute top-0 right-0 border-2 border-white bg-red-700 h-3 w-3 rounded-full"/>}
    </div>

    const dismissNotification = (notification: Notification)=>{
        axios.put(apiURL+'/notifications/dismiss', {id: notification.id})
            .then(()=>{
                dispatch(removeNotification(notification.id))
            })}
            

    const displayNotifications = ()=>{
        if(notifications.length===0){
            return <div className="text-center place-items-center flex px-5 text-sm text-stone-500">
                {t('no-notifications')}
            </div>
        }
        else{
            return notifications.map((notification, index) => {
                return <div className="grid grid-cols-[1fr_auto] gap-2 border-b last-of-type:border-b-0 border-b-stone-200 px-5 py-3 text-sm text-stone-900" key={index}>
                    {notification.message}

                    <span className="material-symbols-outlined cursor-pointer text-stone-400 hover:text-stone-600" onClick={()=>{dismissNotification(notification)}}>close</span>
                </div>
            })
        }
    }

    return <Popover.Root>
        <Popover.Trigger>
            {trigger()}
        </Popover.Trigger>

        <Popover.Portal>
            <Popover.Content className="bg-white max-h-80 max-w-xs overflow-y-auto py-3 rounded-lg shadow-[0_4px_16px_rgba(0,0,0,0.2)]">
                {displayNotifications()}

                <Popover.Arrow className="fill-white" />
            </Popover.Content>
        </Popover.Portal>
    </Popover.Root>
}
