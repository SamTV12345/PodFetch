import {FC} from "react"
import {useTranslation} from "react-i18next"
import axios from "axios"
import {AnimatePresence, motion} from "framer-motion"
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

    const trigger = () => <div className="flex items-center relative">
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
        } else {
            return <AnimatePresence>
                {notifications.map((notification) => (
                    <motion.div className="grid grid-cols-[1fr_auto] gap-2 last-of-type:!border-b-0 border-b-stone-200 px-5 text-sm text-stone-900"
                    key={notification.id}
                    initial={false}
                    animate={{ borderBottomWidth: '1px', maxHeight: '100%',  opacity: 1, paddingTop: '0.75rem', paddingBottom: '0.75rem' }}
                    exit={{ borderBottomWidth: 0, maxHeight: 0, opacity: 0, paddingBottom: 0, paddingTop: 0 }}
                    transition={{
                        opacity: { ease: 'linear', duration: 0.1 },
                        borderBottomWidth: { delay: 0.15, ease: 'easeOut', duration: 0.1 },
                        maxHeight: { delay: 0.15, ease: 'easeOut', duration: 0.1 },
                        paddingBottom: { delay: 0.15, ease: 'easeOut', duration: 0.1 },
                        paddingTop: { delay: 0.15, ease: 'easeOut', duration: 0.1 }
                    }}>
                        {notification.message}
 
                        <span className="material-symbols-outlined cursor-pointer text-stone-400 hover:text-stone-600" onClick={()=>{dismissNotification(notification)}}>close</span>
                    </motion.div>
                ))}
            </AnimatePresence>
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
