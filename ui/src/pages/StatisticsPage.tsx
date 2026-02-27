import {useMemo, useState} from "react";
import {useTranslation} from "react-i18next";
import {useQuery} from "@tanstack/react-query";
import {Heading1} from "../components/Heading1";
import {apiURL, HEADER_TO_USE} from "../utils/http";
import {getConfigFromHtmlFile} from "../utils/config";

type WeekdayStats = {
    dayIndex: number
    weekday: string
    listenedSeconds: number
}

type TopPodcastStats = {
    podcastId: number
    podcastName: string
    imageUrl: string
    listenedSeconds: number
    listenedEpisodes: number
}

type StatsOverview = {
    listenedPodcasts: number
    listenedEpisodes: number
    totalListenedSeconds: number
    topPodcasts: TopPodcastStats[]
    activeWeekdays: WeekdayStats[]
}

const formatDuration = (seconds: number) => {
    const s = Math.max(0, Math.floor(seconds))
    const hours = Math.floor(s / 3600)
    const minutes = Math.floor((s % 3600) / 60)
    if (hours > 0) {
        return `${hours}h ${minutes}m`
    }
    return `${minutes}m`
}

const createAuthHeaders = () => {
    const auth = localStorage.getItem('auth') || sessionStorage.getItem('auth')
    const config = getConfigFromHtmlFile()
    const headers = new Headers(HEADER_TO_USE)
    if (auth && config?.basicAuth) {
        headers.set('Authorization', 'Basic ' + auth)
    } else if (auth && config?.oidcConfigured) {
        headers.set('Authorization', 'Bearer ' + auth)
    }
    return headers
}

export const StatisticsPage = () => {
    const {t} = useTranslation()
    const [from, setFrom] = useState<string>("")
    const [to, setTo] = useState<string>("")

    const statsQuery = useQuery({
        queryKey: ['stats-overview', from, to],
        queryFn: async () => {
            const query = new URLSearchParams()
            if (from) {
                query.set("from", from)
            }
            if (to) {
                query.set("to", to)
            }
            const url = query.toString().length > 0
                ? `${apiURL}/api/v1/stats/overview?${query.toString()}`
                : `${apiURL}/api/v1/stats/overview`
            const response = await fetch(url, {headers: createAuthHeaders()})
            if (!response.ok) {
                throw new Error("Failed to load statistics")
            }
            return await response.json() as StatsOverview
        }
    })

    const weekdayMap = useMemo(() => ({
        monday: t('weekday-monday'),
        tuesday: t('weekday-tuesday'),
        wednesday: t('weekday-wednesday'),
        thursday: t('weekday-thursday'),
        friday: t('weekday-friday'),
        saturday: t('weekday-saturday'),
        sunday: t('weekday-sunday')
    }), [t])

    const maxWeekdaySeconds = Math.max(
        1,
        ...(statsQuery.data?.activeWeekdays ?? []).map((weekday) => weekday.listenedSeconds)
    )

    return (
        <div className="pb-20">
            <div className="mb-8 flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
                <div>
                    <Heading1>{t('stats-title')}</Heading1>
                    <p className="mt-2 text-sm text-(--fg-secondary-color)">{t('stats-subtitle')}</p>
                </div>

                <div className="grid grid-cols-1 xs:grid-cols-2 gap-3">
                    <label className="flex flex-col gap-1 text-xs text-(--fg-secondary-color)">
                        {t('stats-from')}
                        <input
                            type="date"
                            value={from}
                            onChange={(event) => setFrom(event.target.value)}
                            className="rounded-lg border border-(--border-color) bg-(--bg-color) px-3 py-2 text-sm text-(--fg-color) focus:outline-none focus:ring-2 focus:ring-(--accent-color)"
                        />
                    </label>
                    <label className="flex flex-col gap-1 text-xs text-(--fg-secondary-color)">
                        {t('stats-to')}
                        <input
                            type="date"
                            value={to}
                            onChange={(event) => setTo(event.target.value)}
                            className="rounded-lg border border-(--border-color) bg-(--bg-color) px-3 py-2 text-sm text-(--fg-color) focus:outline-none focus:ring-2 focus:ring-(--accent-color)"
                        />
                    </label>
                </div>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-4 gap-4 mb-10">
                <div className="rounded-xl border border-(--border-color) p-4 bg-(--bg-color)">
                    <p className="text-xs text-(--fg-secondary-color)">{t('stats-listened-podcasts')}</p>
                    <p className="mt-2 text-3xl font-semibold text-(--fg-color)">
                        {statsQuery.data?.listenedPodcasts ?? 0}
                    </p>
                </div>
                <div className="rounded-xl border border-(--border-color) p-4 bg-(--bg-color)">
                    <p className="text-xs text-(--fg-secondary-color)">{t('stats-listened-episodes')}</p>
                    <p className="mt-2 text-3xl font-semibold text-(--fg-color)">
                        {statsQuery.data?.listenedEpisodes ?? 0}
                    </p>
                </div>
                <div className="rounded-xl border border-(--border-color) p-4 bg-(--bg-color)">
                    <p className="text-xs text-(--fg-secondary-color)">{t('stats-total-listening-time')}</p>
                    <p className="mt-2 text-3xl font-semibold text-(--fg-color)">
                        {formatDuration(statsQuery.data?.totalListenedSeconds ?? 0)}
                    </p>
                </div>
                <div className="rounded-xl border border-(--border-color) p-4 bg-(--bg-color)">
                    <p className="text-xs text-(--fg-secondary-color)">{t('stats-active-days')}</p>
                    <p className="mt-2 text-3xl font-semibold text-(--fg-color)">
                        {(statsQuery.data?.activeWeekdays ?? []).filter((day) => day.listenedSeconds > 0).length}
                    </p>
                </div>
            </div>

            {statsQuery.isError && (
                <div className="mb-6 rounded-lg border border-(--danger-fg-color) bg-(--danger-bg-color) px-4 py-3 text-sm text-(--danger-fg-color)">
                    {t('error-occured')}
                </div>
            )}

            <div className="grid grid-cols-1 xl:grid-cols-2 gap-6">
                <div className="rounded-xl border border-(--border-color) p-5 bg-(--bg-color)">
                    <h2 className="mb-4 text-lg font-semibold text-(--fg-color)">{t('stats-top-podcasts')}</h2>
                    {statsQuery.isLoading && (
                        <p className="text-sm text-(--fg-secondary-color)">{t('loading')}</p>
                    )}
                    {!statsQuery.isLoading && (statsQuery.data?.topPodcasts?.length ?? 0) === 0 && (
                        <p className="text-sm text-(--fg-secondary-color)">{t('stats-no-data')}</p>
                    )}
                    <div className="flex flex-col gap-3">
                        {statsQuery.data?.topPodcasts?.map((podcast) => (
                            <div key={podcast.podcastId}
                                 className="flex items-center gap-3 rounded-lg border border-(--border-color) p-3">
                                <img
                                    src={podcast.imageUrl}
                                    alt={podcast.podcastName}
                                    className="h-12 w-12 rounded-md object-cover bg-(--input-bg-color)"
                                />
                                <div className="min-w-0 flex-1">
                                    <p className="line-clamp-1 text-sm font-medium text-(--fg-color)">{podcast.podcastName}</p>
                                    <p className="text-xs text-(--fg-secondary-color)">
                                        {t('stats-listened-episodes')}: {podcast.listenedEpisodes}
                                    </p>
                                </div>
                                <p className="text-xs font-medium text-(--accent-color)">
                                    {formatDuration(podcast.listenedSeconds)}
                                </p>
                            </div>
                        ))}
                    </div>
                </div>

                <div className="rounded-xl border border-(--border-color) p-5 bg-(--bg-color)">
                    <h2 className="mb-4 text-lg font-semibold text-(--fg-color)">{t('stats-weekday-activity')}</h2>
                    {statsQuery.isLoading && (
                        <p className="text-sm text-(--fg-secondary-color)">{t('loading')}</p>
                    )}
                    {!statsQuery.isLoading && (statsQuery.data?.activeWeekdays?.length ?? 0) === 0 && (
                        <p className="text-sm text-(--fg-secondary-color)">{t('stats-no-data')}</p>
                    )}
                    <div className="flex flex-col gap-3">
                        {statsQuery.data?.activeWeekdays?.map((weekday) => (
                            <div key={weekday.dayIndex} className="grid grid-cols-[88px_1fr_auto] items-center gap-3">
                                <span className="text-xs text-(--fg-secondary-color)">
                                    {weekdayMap[weekday.weekday as keyof typeof weekdayMap] ?? weekday.weekday}
                                </span>
                                <div className="h-2 rounded-full bg-(--input-bg-color) overflow-hidden">
                                    <div
                                        className="h-full rounded-full bg-(--accent-color)"
                                        style={{
                                            width: `${Math.max(4, (weekday.listenedSeconds / maxWeekdaySeconds) * 100)}%`
                                        }}
                                    />
                                </div>
                                <span className="text-xs text-(--fg-color)">{formatDuration(weekday.listenedSeconds)}</span>
                            </div>
                        ))}
                    </div>
                </div>
            </div>
        </div>
    )
}
