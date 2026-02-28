import { FC, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import * as ToggleGroup from '@radix-ui/react-toggle-group'
import 'material-symbols/outlined.css'
import { applyThemeToDOM, getStoredThemePreference, onSystemThemeChange, setThemePreference, ThemePreference } from '../utils/theme'

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

    const themes = [
        {
            icon: 'desktop_windows',
            translationKey: 'system',
            value: 'system'
        },
        {
            icon: 'light_mode',
            translationKey: 'light',
            value: 'light'
        },
        {
            icon: 'dark_mode',
            translationKey: 'dark',
            value: 'dark'
        }
    ]

    return (
        <ToggleGroup.Root className="flex items-center border ui-border p-0.5 rounded-full" defaultValue="" onValueChange={(v) => {
            if (v === 'system' || v === 'light' || v === 'dark') {
                setTheme(v)
            }
        }} value={theme} type="single" aria-label={t('theme')}>
            {themes.map((theme) =>
                <ToggleGroup.Item aria-label={t(theme.translationKey)} className="aspect-square p-2 rounded-full ui-text hover:ui-text-hover data-[state=on]:ui-surface-muted data-[state=on]:ui-text" key={theme.value} value={theme.value}>
                    <span className="material-symbols-outlined leading-none text-xl">{theme.icon}</span>
                </ToggleGroup.Item>
            )}
        </ToggleGroup.Root>
    )
}
