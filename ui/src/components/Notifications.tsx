import { FC, useEffect, useMemo, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { Bell, BellOff, CheckCircle2, CircleAlert, RotateCw, X } from 'lucide-react'
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
            return <CheckCircle2 size={18} className="ui-text-accent" />
        default:
            return <Bell size={18} className="ui-text-accent" />
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

    const dismissNotification = async (notificationId: string) => {
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
        <Popover open={open} onOpenChange={setOpen}>
            <PopoverTrigger
                render={
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
                        <Bell size={20} className={cn(unreadCount > 0 && "fill-current")} />

                        {bellPulse && unreadCount > 0 && (
                            <span className="pointer-events-none absolute inset-0 rounded-full border-2 ui-border-accent animate-ping" />
                        )}

                        {unreadCount > 0 && (
                            <span className="absolute -right-1 -top-1 grid min-h-5 min-w-5 place-items-center rounded-full px-1 text-[0.625rem] font-semibold text-white ui-bg-accent">
                                {unreadCountLabel}
                            </span>
                        )}
                    </button>
                }
            />

            <PopoverContent
                align="end"
                sideOffset={12}
                className="w-[22rem] max-w-[calc(100vw-2rem)] overflow-hidden rounded-2xl border ui-border ui-surface shadow-[0_12px_36px_rgba(0,0,0,var(--shadow-opacity))] p-0"
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
                                    <RotateCw size={18} className={cn(notificationsQuery.isFetching && "animate-spin")} />
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
                                <CircleAlert size={24} className="ui-text-danger mb-2 inline-block" />
                                <p className="text-sm ui-text">{t('error-occured')}</p>
                            </div>
                        )}

                        {!notificationsQuery.isLoading && !notificationsQuery.isError && orderedNotifications.length === 0 && (
                            <div className="px-4 py-10 text-center">
                                <BellOff size={32} className="ui-icon-muted mb-1 inline-block" />
                                <p className="text-sm ui-text">{t('no-notifications')}</p>
                                <p className="text-xs ui-text-muted mt-1">{t('notifications-empty-description')}</p>
                            </div>
                        )}

                        {!notificationsQuery.isLoading && !notificationsQuery.isError && orderedNotifications.length > 0 && (
                            <>
                                {orderedNotifications.map((notification) => (
                                    // Entry-only animation via tw-animate-css. The previous
                                    // framer-motion exit animation (height collapse + fade)
                                    // is dropped - the dismiss button is explicit, so an
                                    // instant remove is acceptable and saves a 140 KB dep.
                                    <div
                                        key={notification.id}
                                        className="grid grid-cols-[auto_1fr_auto] items-start gap-3 px-4 py-3 border-b ui-border-b last:border-b-0 animate-in fade-in slide-in-from-top-2 duration-150"
                                    >
                                        <span className="mt-0.5">
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
                                            <X size={18} />
                                        </button>
                                    </div>
                                ))}
                            </>
                        )}
                    </div>

            </PopoverContent>
        </Popover>
    )
}
