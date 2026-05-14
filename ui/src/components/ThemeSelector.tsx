import { FC, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import 'material-symbols/outlined.css'
import { applyThemeToDOM, getStoredThemePreference, onSystemThemeChange, setThemePreference, ThemePreference } from '../utils/theme'

const THEMES: ReadonlyArray<{ icon: string, translationKey: string, value: ThemePreference }> = [
    { icon: 'desktop_windows', translationKey: 'system', value: 'system' },
    { icon: 'light_mode', translationKey: 'light', value: 'light' },
    { icon: 'dark_mode', translationKey: 'dark', value: 'dark' },
]

export const ThemeSelector: FC = () => {
    const [theme, setTheme] = useState<ThemePreference>(getStoredThemePreference())
    const { t } = useTranslation()

    /* Persist selection and sync DOM class */
    useEffect(() => {
        setThemePreference(theme)
    }, [theme])

    /* Keep `system` in sync with OS preference changes */
    useEffect(() => onSystemThemeChange(() => {
        if (getStoredThemePreference() === 'system') {
            applyThemeToDOM('system')
        }
    }), [])

    return (
        <ToggleGroup
            // Base UI ToggleGroup is multi-select by default and exchanges an
            // array of selected values. `multiple={false}` constrains to a
            // single selection; we still wrap value/onValueChange in arrays.
            multiple={false}
            value={[theme]}
            onValueChange={(values) => {
                const v = values[0]
                if (v === 'system' || v === 'light' || v === 'dark') {
                    setTheme(v)
                }
            }}
            className="flex items-center border ui-border p-0.5 rounded-full gap-0"
            aria-label={t('theme')}
        >
            {THEMES.map((entry) => (
                <ToggleGroupItem
                    key={entry.value}
                    value={entry.value}
                    aria-label={t(entry.translationKey)}
                    className="aspect-square p-2 rounded-full ui-text hover:ui-text-hover data-[pressed]:ui-surface-muted data-[pressed]:ui-text"
                >
                    <span className="material-symbols-outlined leading-none text-xl">{entry.icon}</span>
                </ToggleGroupItem>
            ))}
        </ToggleGroup>
    )
}
