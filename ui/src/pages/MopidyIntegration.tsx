import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useQueryClient } from '@tanstack/react-query'
import { $api } from '../utils/http'
import { CustomButtonPrimary } from '../components/CustomButtonPrimary'
import { CustomInput } from '../components/CustomInput'
import { useSnackbar } from '../utils/toast'

export const MopidyIntegration = () => {
    const { t } = useTranslation()
    const { enqueueSnackbar } = useSnackbar()
    const queryClient = useQueryClient()

    const [name, setName] = useState('')
    const [url, setUrl] = useState('')
    const [shared, setShared] = useState(true)

    const serversQuery = $api.useQuery('get', '/api/v1/mopidy/servers')
    const addServer = $api.useMutation('post', '/api/v1/mopidy/servers')
    const testServer = $api.useMutation('post', '/api/v1/mopidy/servers/test')
    const deleteServer = $api.useMutation('delete', '/api/v1/mopidy/servers/{id}')

    const invalidate = () =>
        queryClient.invalidateQueries({ queryKey: ['get', '/api/v1/mopidy/servers'] })

    const onTest = async () => {
        const result = await testServer.mutateAsync({ body: { url } })
        if (result.reachable) {
            enqueueSnackbar(t('mopidy-connection-ok', { version: result.version ?? '' }), { variant: 'success' })
        } else {
            enqueueSnackbar(t('mopidy-connection-failed', { error: result.error ?? '' }), { variant: 'error' })
        }
    }

    const onAdd = async () => {
        await addServer.mutateAsync({ body: { name, url, shared } })
        enqueueSnackbar(t('mopidy-server-added'), { variant: 'success' })
        setName('')
        setUrl('')
        invalidate()
    }

    const onDelete = async (id: string) => {
        await deleteServer.mutateAsync({ params: { path: { id } } })
        enqueueSnackbar(t('mopidy-server-deleted'), { variant: 'success' })
        invalidate()
    }

    return (
        <div className="flex flex-col gap-6 ui-text">
            <div className="flex flex-col gap-3 max-w-md">
                <label className="flex flex-col gap-1 text-sm ui-text">
                    {t('mopidy-server-name')}
                    <CustomInput value={name} onChange={(e) => setName(e.target.value)} />
                </label>
                <label className="flex flex-col gap-1 text-sm ui-text">
                    {t('mopidy-server-url')}
                    <CustomInput placeholder="http://mopidy.local:6680" value={url} onChange={(e) => setUrl(e.target.value)} />
                </label>
                <label className="flex items-center gap-2 text-sm ui-text">
                    <input type="checkbox" checked={shared} onChange={(e) => setShared(e.target.checked)} />
                    {t('mopidy-server-shared')}
                </label>
                <div className="flex gap-2">
                    <button
                        className="ui-bg-surface hover:bg-(--surface-hover) px-3 py-2 rounded-md text-sm ui-text"
                        onClick={onTest}
                        disabled={!url}
                    >
                        {t('mopidy-test-connection')}
                    </button>
                    <CustomButtonPrimary onClick={onAdd} disabled={!name || !url}>
                        {t('mopidy-add-server')}
                    </CustomButtonPrimary>
                </div>
            </div>

            {(serversQuery.data ?? []).length === 0 ? (
                <div className="text-sm ui-text-muted">{t('mopidy-no-servers')}</div>
            ) : (
                <table className="text-left text-sm w-full">
                    <thead>
                        <tr className="border-b ui-border">
                            <th className="px-2 py-3 ui-text">{t('mopidy-server-name')}</th>
                            <th className="px-2 py-3 ui-text">{t('mopidy-server-url')}</th>
                            <th className="px-2 py-3 ui-text">{t('actions')}</th>
                        </tr>
                    </thead>
                    <tbody>
                        {(serversQuery.data ?? []).map((server) => (
                            <tr key={server.id}>
                                <td className="px-2 py-4 ui-text">{server.name}</td>
                                <td className="px-2 py-4 ui-text">{server.url}</td>
                                <td className="px-2 py-4">
                                    <button
                                        className="ui-text-accent hover:underline"
                                        onClick={() => onDelete(server.id)}
                                    >
                                        {t('delete')}
                                    </button>
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            )}
        </div>
    )
}
