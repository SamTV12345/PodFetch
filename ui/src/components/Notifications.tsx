import { FC } from 'react'
import { useTranslation } from 'react-i18next'
import { AnimatePresence, motion } from 'framer-motion'
import * as Popover from '@radix-ui/react-popover'
import { removeHTML} from '../utils/Utilities'
import useCommon from '../store/CommonSlice'
import 'material-symbols/outlined.css'
import {$api} from "../utils/http";
import {components} from "../../schema";
import {LoadingSkeletonSpan} from "./ui/LoadingSkeletonSpan";
import {useQueryClient} from "@tanstack/react-query";


const NotificationFormatter = (notification: components["schemas"]["Notification"]) => {
    const {t} = useTranslation()

    const decideMessage = ()=>{
        switch(notification.typeOfMessage) {
            case "Download":
                return <span dangerouslySetInnerHTML={removeHTML(t('notification.episode-now-available', {episode: notification.message}))}/>
        }
    }

    return decideMessage()
}

export const Notifications: FC = () => {
    const notifications = $api.useQuery('get','/api/v1/notifications/unread')
    const queryClient = useQueryClient()
    const { t }  = useTranslation()
    const dismissNotificationMutation = $api.useMutation('put', '/api/v1/notifications/dismiss')

    const trigger = () => (
        <div className="flex items-center relative">
            <span className="material-symbols-outlined cursor-pointer ui-text hover:ui-text-hover">notifications</span>

            {(notifications.isLoading || !notifications.data) ? <span>Loading</span> :notifications.data.length > 0 && <div className="absolute top-0 right-0 border-2 ui-border-contrast bg-red-700 h-3 w-3 rounded-full"/>}
        </div>
    )

    const dismissNotification = (notification: components["schemas"]["Notification"]) => {
        dismissNotificationMutation.mutateAsync({
            body: {
                id: notification.id
            }
        }).then(() => {
            queryClient.setQueryData(['get','/api/v1/notifications/unread'], (oldData: components["schemas"]["Notification"][]) => {
                return oldData.filter(n => n.id !== notification.id)
            })
        })
    }

    const DisplayNotification = () => {

        if (notifications.isLoading || !notifications.data) {
            return (
                <><LoadingSkeletonSpan/><LoadingSkeletonSpan/></>
            )
        }

        if (notifications.data.length === 0) {
            return (
                <div className="text-center place-items-center flex px-5 text-sm ui-text-disabled">
                    {t('no-notifications')}
                </div>
            )
        } else {
            return (
                <AnimatePresence>
                    {notifications.data.map((notification) => (
                        <motion.div className="grid grid-cols-[1fr_auto] gap-2 last-of-type:border-b-0! ui-border-b px-5 text-sm ui-text"
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

                            <span className="material-symbols-outlined cursor-pointer ui-modal-close hover:ui-modal-close-hover" onClick={()=>{dismissNotification(notification)}}>close</span>
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
                <Popover.Content className="relative ui-surface max-h-80 max-w-xs overflow-y-auto py-3 rounded-lg shadow-[0_4px_16px_rgba(0,0,0,var(--shadow-opacity))] z-30">
                    <div className="flex w-full">
                        <div className="grow"/>
                         <button className="ui-border-b flex active:scale-95
                         text-sm ui-text border-[2px] rounded-2xl  pl-2 pr-2 float-right mr-3 mb-3" onClick={()=>{
                             notifications.data?.forEach(n=>{
                                    dismissNotification(n)
                             })
                             queryClient.setQueryData(['get','/api/v1/notifications/unread'], []);
                         }}>{t('clear-all')}</button>
                    </div>
                    <div>
                       <DisplayNotification />
                    </div>
                    <Popover.Arrow className="ui-fill-inverse" />
                </Popover.Content>
            </Popover.Portal>
        </Popover.Root>
    )
}
