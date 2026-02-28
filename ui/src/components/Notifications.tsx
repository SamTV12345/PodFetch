import { FC, useEffect, useMemo, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { AnimatePresence, motion } from 'framer-motion'
import * as Popover from '@radix-ui/react-popover'
import 'material-symbols/outlined.css'
import { useQueryClient } from "@tanstack/react-query";
import { components } from "../../schema";
import { cn } from "../lib/utils";
import { formatTime, removeHTML } from '../utils/Utilities'
import { $api } from "../utils/http";
import { Skeleton } from "./ui/skeleton";

type NotificationModel = components["schemas"]["Notification"]

const unreadNotificationsQueryKey = ['get', '/api/v1/notifications/unread'] as const
const maxBadgeCount = 99

const notificationIcon = (type: string) => {
    switch (type) {
        case "Download":
            return 'download_done'
        default:
            return 'notifications'
    }
}

const NotificationText: FC<{ notification: NotificationModel }> = ({ notification }) => {
    const { t } = useTranslation()

    if (notification.typeOfMessage === "Download") {
        return <span dangerouslySetInnerHTML={removeHTML(t('notification.episode-now-available', { episode: notification.message }))} />
    }

    return <span dangerouslySetInnerHTML={removeHTML(notification.message)} />
}

const NotificationLoadingState = () => (
    <div className="p-4 space-y-3">
        {[0, 1, 2].map((row) => (
            <div className="grid grid-cols-[auto_1fr] items-center gap-3" key={row}>
                <Skeleton className="h-8 w-8 rounded-full" />
                <div className="space-y-2">
                    <Skeleton className="h-3 w-full max-w-[14rem]" />
                    <Skeleton className="h-3 w-16" />
                </div>
            </div>
        ))}
    </div>
)

export const Notifications: FC = () => {
    const queryClient = useQueryClient()
    const { t } = useTranslation()
    const [open, setOpen] = useState(false)
    const [bellPulse, setBellPulse] = useState(false)
    const [isClearingAll, setIsClearingAll] = useState(false)
    const previousUnreadCount = useRef(0)

    const notificationsQuery = $api.useQuery('get', '/api/v1/notifications/unread', {}, {
        refetchInterval: 45_000,
        refetchOnWindowFocus: true
    })
    const dismissNotificationMutation = $api.useMutation('put', '/api/v1/notifications/dismiss')

    const notifications = notificationsQuery.data ?? []
    const unreadCount = notifications.length
    const unreadCountLabel = unreadCount > maxBadgeCount ? `${maxBadgeCount}+` : `${unreadCount}`

    const orderedNotifications = useMemo(() => {
        return [...notifications].sort((a, b) => Date.parse(b.createdAt) - Date.parse(a.createdAt))
    }, [notifications])

    useEffect(() => {
        if (open) {
            notificationsQuery.refetch()
        }
    }, [open])

    useEffect(() => {
        if (unreadCount > previousUnreadCount.current) {
            setBellPulse(true)
            const timeout = setTimeout(() => setBellPulse(false), 900)
            previousUnreadCount.current = unreadCount
            return () => clearTimeout(timeout)
        }
        previousUnreadCount.current = unreadCount
    }, [unreadCount])

    const dismissNotification = async (notificationId: number) => {
        const previous = queryClient.getQueryData<NotificationModel[]>(unreadNotificationsQueryKey) ?? []

        queryClient.setQueryData<NotificationModel[]>(unreadNotificationsQueryKey, (oldData) => {
            return (oldData ?? []).filter(v => v.id !== notificationId)
        })

        try {
            await dismissNotificationMutation.mutateAsync({
                body: { id: notificationId }
            })
        } catch {
            queryClient.setQueryData(unreadNotificationsQueryKey, previous)
        }
    }

    const clearAllNotifications = async () => {
        if (orderedNotifications.length === 0 || isClearingAll) {
            return
        }

        setIsClearingAll(true)

        const previous = queryClient.getQueryData<NotificationModel[]>(unreadNotificationsQueryKey) ?? []
        queryClient.setQueryData(unreadNotificationsQueryKey, [])

        const result = await Promise.allSettled(
            previous.map((notification) => dismissNotificationMutation.mutateAsync({
                body: { id: notification.id }
            }))
        )

        if (result.some(v => v.status === 'rejected')) {
            queryClient.setQueryData(unreadNotificationsQueryKey, previous)
        }

        setIsClearingAll(false)
    }

    return (
        <Popover.Root open={open} onOpenChange={setOpen}>
            <Popover.Trigger asChild>
                <button
                    type="button"
                    className={cn(
                        "relative grid h-10 w-10 place-items-center rounded-full border ui-border ui-surface ui-text transition-all",
                        "hover:ui-text-accent hover:shadow-[0_0_0_4px_rgba(0,0,0,0.05)]",
                        "focus-visible:outline-none focus-visible:ring-2 ui-ring-accent",
                        unreadCount > 0 && "ui-text-accent ui-border-accent"
                    )}
                    aria-label={t('notifications-open')}
                    title={t('notifications-open') as string}
                >
                    <span className={cn("material-symbols-outlined leading-none", unreadCount > 0 && "filled")}>notifications</span>

                    {bellPulse && unreadCount > 0 && (
                        <span className="pointer-events-none absolute inset-0 rounded-full border-2 ui-border-accent animate-ping" />
                    )}

                    {unreadCount > 0 && (
                        <span className="absolute -right-1 -top-1 grid min-h-5 min-w-5 place-items-center rounded-full px-1 text-[0.625rem] font-semibold text-white ui-bg-accent">
                            {unreadCountLabel}
                        </span>
                    )}
                </button>
            </Popover.Trigger>

            <Popover.Portal>
                <Popover.Content
                    align="end"
                    sideOffset={12}
                    className="relative z-30 w-[22rem] max-w-[calc(100vw-2rem)] overflow-hidden rounded-2xl border ui-border ui-surface shadow-[0_12px_36px_rgba(0,0,0,var(--shadow-opacity))]"
                >
                    <div className="border-b ui-border-b px-4 py-3">
                        <div className="flex items-start justify-between gap-3">
                            <div className="min-w-0">
                                <div className="text-sm font-semibold ui-text">{t('notifications-title')}</div>
                                <div className="text-xs ui-text-muted">
                                    {unreadCount > 0 ? t('notifications-unread', { count: unreadCount }) : t('no-notifications')}
                                </div>
                            </div>

                            <div className="flex items-center gap-1">
                                <button
                                    type="button"
                                    className={cn("grid h-8 w-8 place-items-center rounded-full ui-text hover:ui-text-accent hover:ui-surface-muted transition-colors")}
                                    onClick={() => notificationsQuery.refetch()}
                                    aria-label={t('notifications-refresh')}
                                    title={t('notifications-refresh') as string}
                                >
                                    <span className={cn("material-symbols-outlined leading-none text-lg", notificationsQuery.isFetching && "animate-spin")}>refresh</span>
                                </button>

                                <button
                                    type="button"
                                    disabled={orderedNotifications.length === 0 || isClearingAll}
                                    onClick={clearAllNotifications}
                                    className={cn(
                                        "text-xs px-2.5 py-1.5 rounded-full border ui-border ui-text transition-colors",
                                        "hover:ui-surface-muted disabled:opacity-40 disabled:cursor-not-allowed"
                                    )}
                                >
                                    {t('clear-all')}
                                </button>
                            </div>
                        </div>
                    </div>

                    <div className="max-h-[24rem] overflow-y-auto">
                        {(notificationsQuery.isLoading || (notificationsQuery.isFetching && !notificationsQuery.data)) && <NotificationLoadingState />}

                        {notificationsQuery.isError && (
                            <div className="px-4 py-8 text-center">
                                <span className="material-symbols-outlined text-2xl ui-text-danger mb-2">error</span>
                                <p className="text-sm ui-text">{t('error-occured')}</p>
                            </div>
                        )}

                        {!notificationsQuery.isLoading && !notificationsQuery.isError && orderedNotifications.length === 0 && (
                            <div className="px-4 py-10 text-center">
                                <span className="material-symbols-outlined text-3xl ui-icon-muted mb-1">notifications_off</span>
                                <p className="text-sm ui-text">{t('no-notifications')}</p>
                                <p className="text-xs ui-text-muted mt-1">{t('notifications-empty-description')}</p>
                            </div>
                        )}

                        {!notificationsQuery.isLoading && !notificationsQuery.isError && orderedNotifications.length > 0 && (
                            <AnimatePresence initial={false}>
                                {orderedNotifications.map((notification) => (
                                    <motion.div
                                        key={notification.id}
                                        className="grid grid-cols-[auto_1fr_auto] items-start gap-3 px-4 py-3 border-b ui-border-b last:border-b-0"
                                        initial={{ opacity: 0, y: -6 }}
                                        animate={{ opacity: 1, y: 0 }}
                                        exit={{ opacity: 0, y: -4, height: 0, paddingTop: 0, paddingBottom: 0 }}
                                        transition={{ duration: 0.14, ease: 'easeOut' }}
                                    >
                                        <span className="material-symbols-outlined text-[1.1rem] leading-none mt-0.5 ui-text-accent">
                                            {notificationIcon(notification.typeOfMessage)}
                                        </span>

                                        <div className="min-w-0">
                                            <p className="text-sm leading-5 ui-text break-words">
                                                <NotificationText notification={notification} />
                                            </p>
                                            <p className="text-xs ui-text-muted mt-1">{formatTime(notification.createdAt)}</p>
                                        </div>

                                        <button
                                            type="button"
                                            className={cn(
                                                "grid h-7 w-7 place-items-center rounded-full ui-modal-close hover:ui-modal-close-hover",
                                                "hover:ui-surface-muted transition-colors"
                                            )}
                                            onClick={() => dismissNotification(notification.id)}
                                            aria-label={t('notifications-dismiss')}
                                            title={t('notifications-dismiss') as string}
                                        >
                                            <span className="material-symbols-outlined text-lg leading-none">close</span>
                                        </button>
                                    </motion.div>
                                ))}
                            </AnimatePresence>
                        )}
                    </div>

                    <Popover.Arrow className="ui-fill-inverse" />
                </Popover.Content>
            </Popover.Portal>
        </Popover.Root>
    )
}
