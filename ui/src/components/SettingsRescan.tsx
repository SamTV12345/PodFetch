import { FC, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useSnackbar } from '@/utils/toast'
import { CustomButtonPrimary } from './CustomButtonPrimary'
import { CustomCheckbox } from './CustomCheckbox'
import { SettingsInfoIcon } from './SettingsInfoIcon'
import { $api } from '../utils/http'
import { components } from '../../schema'

type RescanOptionsModel = components['schemas']['RescanOptions']

const OPTION_KEYS = [
    'applyFilenames',
    'applyTranscode',
    'applyCovers',
    'applyMetadata',
] as const

export const SettingsRescan: FC = () => {
    const { t } = useTranslation()
    const { enqueueSnackbar } = useSnackbar()
    const rescanEpisodesMutation = $api.useMutation('post', '/api/v1/settings/rescan-episodes')
    const [options, setOptions] = useState<Required<RescanOptionsModel>>({
        applyFilenames: false,
        applyTranscode: false,
        applyCovers: false,
        applyMetadata: false,
    })

    const setOption = (key: typeof OPTION_KEYS[number], value: boolean) => {
        setOptions((prev) => ({ ...prev, [key]: value }))
    }

    return (
        <div className="flex flex-col gap-6 ui-text">
            <p className="ui-text-muted">
                {t('rescan-audio-files-description')}
            </p>

            <div className="flex flex-col gap-3">
                {OPTION_KEYS.map((key) => (
                    <div key={key} className="flex items-center gap-3">
                        <CustomCheckbox
                            id={`rescan-${key}`}
                            value={options[key]}
                            onChange={(checked) => setOption(key, checked === true)}
                        />
                        <label htmlFor={`rescan-${key}`} className="flex gap-1 cursor-pointer">
                            {t(`rescan-option-${key}`)}
                            <SettingsInfoIcon
                                headerKey={`rescan-option-${key}`}
                                textKey={`rescan-option-${key}-explanation`}
                            />
                        </label>
                    </div>
                ))}
            </div>

            <div>
                <CustomButtonPrimary
                    loading={rescanEpisodesMutation.isPending}
                    onClick={async () => {
                        await rescanEpisodesMutation.mutateAsync({ body: options })
                        enqueueSnackbar(t('rescan-done'), { variant: 'success' })
                    }}>
                    {t('rescan-audio-files')}
                </CustomButtonPrimary>
            </div>
        </div>
    )
}
