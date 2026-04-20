import { FC, useCallback, useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigate, useParams } from 'react-router-dom'
import { useQueryClient } from '@tanstack/react-query'
import { $api } from '../utils/http'
import { Heading1 } from '../components/Heading1'
import { CustomButtonPrimary } from '../components/CustomButtonPrimary'
import { CustomSelect, Option } from '../components/CustomSelect'
import { handleAddPodcast } from '../utils/ErrorSnackBarResponses'
import type { components } from '../../schema'
import 'material-symbols/outlined.css'

type Tab = 'for-you' | 'trending' | 'charts' | 'categories'
const TABS: Tab[] = ['for-you', 'trending', 'charts', 'categories']
const DEFAULT_TAB: Tab = 'for-you'
type Feed = components['schemas']['Feed']
type ChartEntry = components['schemas']['ItunesChartEntry']
type Category = components['schemas']['CategoryDto']

// Derived from the six UI translations shipped in ui/src/language/json/.
// Language = ISO-639-1 code (Podcastindex `lang` filter).
// Country = ISO-3166-1 alpha-2 (iTunes RSS `country` segment).
const LANGUAGE_OPTIONS: Option[] = [
    { value: 'da', label: 'Dansk' },
    { value: 'de', label: 'Deutsch' },
    { value: 'en', label: 'English' },
    { value: 'es', label: 'Español' },
    { value: 'fr', label: 'Français' },
    { value: 'pl', label: 'Polski' },
]
const COUNTRY_OPTIONS: Option[] = [
    { value: 'dk', label: 'Danmark' },
    { value: 'de', label: 'Deutschland' },
    { value: 'us', label: 'USA' },
    { value: 'gb', label: 'United Kingdom' },
    { value: 'es', label: 'España' },
    { value: 'fr', label: 'France' },
    { value: 'pl', label: 'Polska' },
]

export const DiscoverPage: FC = () => {
    const { t } = useTranslation()
    const queryClient = useQueryClient()
    const navigate = useNavigate()
    const { tab: tabParam } = useParams<{ tab?: string }>()
    const tab: Tab = (TABS as string[]).includes(tabParam ?? '') ? (tabParam as Tab) : DEFAULT_TAB
    const setTab = useCallback(
        (next: Tab) => navigate(`/discover/${next}`),
        [navigate],
    )
    const [chartCountry, setChartCountry] = useState('')
    const [chartGenre, setChartGenre] = useState('')
    const [trendingCat, setTrendingCat] = useState('')

    const addFeedMutation = $api.useMutation('post', '/api/v1/podcasts/feed')
    const addItunesPodcastMutation = $api.useMutation('post', '/api/v1/podcasts/itunes')
    const updateLocaleMutation = $api.useMutation('put', '/api/v1/users/me/locale')

    const meQuery = $api.useSuspenseQuery('get', '/api/v1/users/{username}', {
        params: { path: { username: 'me' } },
    })

    useEffect(() => {
        if (meQuery.data?.country && !chartCountry) {
            setChartCountry(meQuery.data.country)
        }
    }, [meQuery.data?.country, chartCountry])

    const categoriesQuery = $api.useSuspenseQuery(
        'get',
        '/api/v1/discover/categories',
        {},
        { staleTime: Infinity },
    )

    const forYouQuery = $api.useQuery(
        'get',
        '/api/v1/discover/for-you',
        { params: { query: { limit: 50 } } },
        { enabled: tab === 'for-you' },
    )

    const trendingQuery = $api.useQuery(
        'get',
        '/api/v1/discover/trending',
        {
            params: {
                query: {
                    limit: 50,
                    ...(trendingCat ? { cat: trendingCat } : {}),
                    ...(meQuery.data?.language ? { lang: meQuery.data.language } : {}),
                },
            },
        },
        { enabled: tab === 'trending' },
    )

    const chartsQuery = $api.useQuery(
        'get',
        '/api/v1/discover/charts',
        {
            params: {
                query: {
                    country: chartCountry,
                    limit: 100,
                    ...(chartGenre ? { genre: Number(chartGenre) } : {}),
                },
            },
        },
        { enabled: tab === 'charts' && chartCountry.length >= 2 },
    )

    const categoryOptions: Option[] = useMemo(
        () =>
            (categoriesQuery.data ?? []).map((c: Category) => ({
                value: c.podindexId.toString(),
                label: c.name,
            })),
        [categoriesQuery.data],
    )

    const genreOptions: Option[] = useMemo(() => {
        const unique = new Map<string, string>()
        ;(categoriesQuery.data ?? []).forEach((c: Category) => {
            const key = c.itunesGenreId.toString()
            if (!unique.has(key)) {
                unique.set(key, c.name)
            }
        })
        return Array.from(unique.entries()).map(([value, label]) => ({ value, label }))
    }, [categoriesQuery.data])

    const subscribeByUrl = useCallback(
        async (url?: string | null) => {
            if (!url) return
            try {
                const res = await addFeedMutation.mutateAsync({
                    body: { rssFeedUrl: url },
                })
                const name = (res as { name?: string })?.name ?? url
                handleAddPodcast(200, name, t)
            } catch {
                // snackbar emitted by http.ts middleware
            }
        },
        [addFeedMutation, t],
    )

    const subscribeByItunesId = useCallback(
        async (rawId: string, fallbackName: string) => {
            const trackId = Number(rawId)
            if (!Number.isFinite(trackId)) return
            try {
                const res = await addItunesPodcastMutation.mutateAsync({
                    body: { trackId, userId: 1 },
                })
                const name = (res as { name?: string })?.name ?? fallbackName
                handleAddPodcast(200, name, t)
            } catch {
                // snackbar emitted by http.ts middleware
            }
        },
        [addItunesPodcastMutation, t],
    )

    const saveLocale = useCallback(
        async (country: string, language: string) => {
            await updateLocaleMutation.mutateAsync({
                body: { country: country || null, language: language || null },
            })
            await Promise.all([
                queryClient.invalidateQueries({
                    queryKey: ['get', '/api/v1/users/{username}'],
                }),
                queryClient.invalidateQueries({
                    queryKey: ['get', '/api/v1/discover/for-you'],
                }),
                queryClient.invalidateQueries({
                    queryKey: ['get', '/api/v1/discover/trending'],
                }),
            ])
        },
        [updateLocaleMutation, queryClient],
    )

    return (
        <div className="p-4 md:p-8">
            <Heading1>{t('discover')}</Heading1>

            <LocaleBar
                initialCountry={meQuery.data?.country ?? ''}
                initialLanguage={meQuery.data?.language ?? ''}
                saving={updateLocaleMutation.isPending}
                onSave={saveLocale}
            />

            <div role="tablist" className="flex gap-2 mt-6 mb-4 border-b ui-border">
                <TabButton label={t('discover-for-you')} active={tab === 'for-you'} onClick={() => setTab('for-you')} />
                <TabButton label={t('discover-trending')} active={tab === 'trending'} onClick={() => setTab('trending')} />
                <TabButton label={t('discover-charts')} active={tab === 'charts'} onClick={() => setTab('charts')} />
                <TabButton label={t('discover-categories')} active={tab === 'categories'} onClick={() => setTab('categories')} />
            </div>

            {tab === 'for-you' && (
                <ForYouTab
                    loading={forYouQuery.isLoading}
                    feeds={forYouQuery.data ?? []}
                    onSubscribe={subscribeByUrl}
                    emptyText={t('discover-for-you-empty')}
                />
            )}
            {tab === 'trending' && (
                <section>
                    <div className="flex items-center gap-3 mb-4">
                        <label className="text-sm ui-text-muted">{t('discover-category')}</label>
                        <CustomSelect
                            options={categoryOptions}
                            value={trendingCat}
                            onChange={(v) => setTrendingCat(v)}
                            placeholder={t('discover-all-categories')}
                        />
                    </div>
                    <FeedGrid
                        loading={trendingQuery.isLoading}
                        feeds={trendingQuery.data?.feeds ?? []}
                        onSubscribe={subscribeByUrl}
                    />
                </section>
            )}
            {tab === 'charts' && (
                <section>
                    <div className="flex flex-wrap items-center gap-3 mb-4">
                        <label className="text-sm ui-text-muted">{t('discover-country')}</label>
                        <CustomSelect
                            options={COUNTRY_OPTIONS}
                            value={chartCountry}
                            onChange={(v) => setChartCountry(v)}
                            placeholder={t('discover-country')}
                        />
                        <label className="text-sm ui-text-muted">{t('discover-genre')}</label>
                        <CustomSelect
                            options={genreOptions}
                            value={chartGenre}
                            onChange={(v) => setChartGenre(v)}
                            placeholder={t('discover-all-categories')}
                        />
                    </div>
                    {chartCountry.length < 2 ? (
                        <p className="ui-text-muted">{t('discover-charts-country-hint')}</p>
                    ) : (
                        <ChartsGrid
                            loading={chartsQuery.isLoading}
                            entries={chartsQuery.data ?? []}
                            onSubscribe={subscribeByItunesId}
                        />
                    )}
                </section>
            )}
            {tab === 'categories' && (
                <section className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3">
                    {(categoriesQuery.data ?? []).map((c: Category) => (
                        <button
                            key={c.podindexId}
                            className="ui-surface rounded-xl py-4 text-left px-4 hover:ui-bg-accent-subtle"
                            onClick={() => {
                                setTrendingCat(c.podindexId.toString())
                                setTab('trending')
                            }}
                        >
                            {c.name}
                        </button>
                    ))}
                </section>
            )}
        </div>
    )
}

const TabButton: FC<{ label: string; active: boolean; onClick: () => void }> = ({ label, active, onClick }) => (
    <button
        role="tab"
        aria-selected={active}
        onClick={onClick}
        className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
            active ? 'ui-text-accent border-[var(--color-mustard-500)]' : 'ui-text-muted border-transparent hover:ui-text'
        }`}
    >
        {label}
    </button>
)

const LocaleBar: FC<{
    initialCountry: string
    initialLanguage: string
    saving: boolean
    onSave: (country: string, language: string) => Promise<void>
}> = ({ initialCountry, initialLanguage, saving, onSave }) => {
    const { t } = useTranslation()
    const [country, setCountry] = useState(initialCountry)
    const [language, setLanguage] = useState(initialLanguage)

    useEffect(() => setCountry(initialCountry), [initialCountry])
    useEffect(() => setLanguage(initialLanguage), [initialLanguage])

    const dirty = country !== initialCountry || language !== initialLanguage

    return (
        <div className="ui-surface rounded-xl p-4 mt-4 flex flex-wrap items-center gap-3">
            <span className="text-sm ui-text-muted">{t('discover-locale-hint')}</span>
            <label className="text-sm ui-text-muted">{t('discover-country')}</label>
            <CustomSelect
                options={COUNTRY_OPTIONS}
                value={country}
                onChange={(v) => setCountry(v)}
                placeholder={t('discover-country')}
            />
            <label className="text-sm ui-text-muted">{t('discover-language')}</label>
            <CustomSelect
                options={LANGUAGE_OPTIONS}
                value={language}
                onChange={(v) => setLanguage(v)}
                placeholder={t('discover-language')}
            />
            <CustomButtonPrimary
                disabled={!dirty || saving}
                onClick={() => {
                    onSave(country, language)
                }}
            >
                {t('save')}
            </CustomButtonPrimary>
        </div>
    )
}

const ForYouTab: FC<{
    loading: boolean
    feeds: Feed[]
    onSubscribe: (url?: string | null) => void
    emptyText: string
}> = ({ loading, feeds, onSubscribe, emptyText }) => {
    if (loading) return <p className="ui-text-muted">…</p>
    if (feeds.length === 0) return <p className="ui-text-muted">{emptyText}</p>
    return <FeedGrid loading={false} feeds={feeds} onSubscribe={onSubscribe} />
}

const FeedGrid: FC<{ loading: boolean; feeds: Feed[]; onSubscribe: (url?: string | null) => void }> = ({
    loading,
    feeds,
    onSubscribe,
}) => {
    const { t } = useTranslation()
    if (loading) return <p className="ui-text-muted">…</p>
    if (feeds.length === 0) return <p className="ui-text-muted">{t('discover-no-results')}</p>
    return (
        <ul className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {feeds.map((feed) => (
                <li key={feed.id ?? feed.url ?? feed.title} className="ui-surface rounded-xl p-4 flex gap-4">
                    {feed.artwork || feed.image ? (
                        <img
                            src={feed.artwork ?? feed.image ?? ''}
                            alt=""
                            loading="lazy"
                            className="w-20 h-20 rounded-lg object-cover flex-shrink-0"
                        />
                    ) : (
                        <div className="w-20 h-20 rounded-lg ui-bg-accent-subtle flex-shrink-0" />
                    )}
                    <div className="flex flex-col min-w-0 flex-1">
                        <span className="font-medium truncate">{feed.title ?? '—'}</span>
                        <span className="text-sm ui-text-muted truncate">{feed.author ?? ''}</span>
                        <CustomButtonPrimary
                            className="mt-auto self-start px-3 py-1.5"
                            disabled={!feed.url}
                            onClick={() => onSubscribe(feed.url)}
                        >
                            {t('add')}
                        </CustomButtonPrimary>
                    </div>
                </li>
            ))}
        </ul>
    )
}

const ChartsGrid: FC<{
    loading: boolean
    entries: ChartEntry[]
    onSubscribe: (id: string, fallbackName: string) => void
}> = ({ loading, entries, onSubscribe }) => {
    const { t } = useTranslation()
    if (loading) return <p className="ui-text-muted">…</p>
    if (entries.length === 0) return <p className="ui-text-muted">{t('discover-no-results')}</p>
    return (
        <ul className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {entries.map((entry) => (
                <li key={entry.id} className="ui-surface rounded-xl p-4 flex gap-4">
                    {entry.image ? (
                        <img
                            src={entry.image}
                            alt=""
                            loading="lazy"
                            className="w-20 h-20 rounded-lg object-cover flex-shrink-0"
                        />
                    ) : (
                        <div className="w-20 h-20 rounded-lg ui-bg-accent-subtle flex-shrink-0" />
                    )}
                    <div className="flex flex-col min-w-0 flex-1">
                        <span className="font-medium truncate">{entry.name}</span>
                        <span className="text-sm ui-text-muted truncate">{entry.artist ?? ''}</span>
                        <div className="flex items-center justify-between gap-2 mt-auto">
                            <span className="text-xs ui-text-muted truncate">{entry.genre ?? ''}</span>
                            <CustomButtonPrimary
                                className="self-start px-3 py-1.5"
                                onClick={() => onSubscribe(entry.id, entry.name)}
                            >
                                {t('add')}
                            </CustomButtonPrimary>
                        </div>
                    </div>
                </li>
            ))}
        </ul>
    )
}
