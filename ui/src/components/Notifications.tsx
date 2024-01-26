import { FC } from 'react'
import { useTranslation } from 'react-i18next'
import axios from 'axios'
import { AnimatePresence, motion } from 'framer-motion'
import * as Popover from '@radix-ui/react-popover'
import { apiURL } from '../utils/Utilities'
import useCommon from '../store/CommonSlice'
import { Notification } from '../models/Notification'
import 'material-symbols/outlined.css'


const NotificationFormatter = (notification: Notification) => {
    const {t} = useTranslation()

    const decideMessage = ()=>{
        switch(notification.typeOfMessage) {
            case "Download":
                return t('notification.episode-now-available', {episode: notification.message})
        }
    }

    return decideMessage()
}

export const Notifications: FC = () => {
    const notifications = useCommon(state => state.notifications)
    const { t }  = useTranslation()
    const removeNotification = useCommon(state => state.removeNotification)
    const setNotifications = useCommon(state => state.setNotifications)

    const trigger = () => (
        <div className="flex items-center relative">
            <span className="material-symbols-outlined cursor-pointer text-[--fg-color] hover:text-[--fg-color-hover]">notifications</span>

            {notifications.length > 0 && <div className="absolute top-0 right-0 border-2 border-[--bg-color] bg-red-700 h-3 w-3 rounded-full"/>}
        </div>
    )

    const dismissNotification = (notification: Notification) => {
        axios.put(apiURL + '/notifications/dismiss', { id: notification.id })
            .then(() => {
                removeNotification(notification.id)
            })
    }

    const DisplayNotification = () => {
        if (notifications.length === 0) {
            return (
                <div className="text-center place-items-center flex px-5 text-sm text-[--fg-color-disabled]">
                    {t('no-notifications')}
                </div>
            )
        } else {
            return (
                <AnimatePresence>
                    {notifications.map((notification) => (
                        <motion.div className="grid grid-cols-[1fr_auto] gap-2 last-of-type:!border-b-0 border-b-[--border-color] px-5 text-sm text-[--fg-color]"
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
                            <NotificationFormatter {...notification} />

                            <span className="material-symbols-outlined cursor-pointer text-[--modal-close-color] hover:text-[--modal-close-color-hover]" onClick={()=>{dismissNotification(notification)}}>close</span>
                        </motion.div>
                    ))}
                </AnimatePresence>
            )
        }
    }

    return (
        <Popover.Root>
            <Popover.Trigger>
                {trigger()}
            </Popover.Trigger>

            <Popover.Portal>
                <Popover.Content className="relative bg-[--bg-color] max-h-80 max-w-xs overflow-y-auto py-3 rounded-lg shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))] z-30">
                    <div className="flex w-full">
                        <div className="grow"/>
                         <button className="border-b-[--border-color] flex active:scale-95
                         text-sm text-[--fg-color] border-[2px] rounded-2xl  pl-2 pr-2 float-right mr-3 mb-3" onClick={()=>{
                             notifications.forEach(n=>{
                                    dismissNotification(n)
                             })
                             setNotifications([])
                         }}>{t('clear-all')}</button>
                    </div>
                    <div>
                       <DisplayNotification />
                    </div>
                    <Popover.Arrow className="fill-[--bg-color]" />
                </Popover.Content>
            </Popover.Portal>
        </Popover.Root>
    )
}
