import { FC, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Toggle } from '@base-ui/react/toggle'
import { ToggleGroup } from '@base-ui/react/toggle-group'
import 'material-symbols/outlined.css'
import { applyThemeToDOM, getStoredThemePreference, onSystemThemeChange, setThemePreference, ThemePreference } from '../utils/theme'

const THEMES: ReadonlyArray<{ icon: string, translationKey: string, value: ThemePreference }> = [
    { icon: 'desktop_windows', translationKey: 'system', value: 'system' },
    { icon: 'light_mode', translationKey: 'light', value: 'light' },
    { icon: 'dark_mode', translationKey: 'dark', value: 'dark' },
]

/**
 * Three-way theme switcher. Uses Base UI primitives directly instead of
 * the shadcn ToggleGroupItem wrapper because the shadcn wrapper enforces
 * a "segmented pill" look (data-spacing=0 → rounded-none on inner items,
 * only first/last get rounded ends, fixed h-8 size). That collides with
 * the round icon-button look we want for theme picking.
 */
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
            // Base UI ToggleGroup is multi-select by default and exchanges
            // an array of selected values. `multiple={false}` constrains to
            // a single selection; value/onValueChange wrap in arrays.
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
                <Toggle
                    key={entry.value}
                    value={entry.value}
                    aria-label={t(entry.translationKey)}
                    className="grid place-items-center w-9 h-9 rounded-full ui-text hover:ui-text-hover data-[pressed]:ui-surface-muted data-[pressed]:ui-text outline-none transition-colors"
                >
                    <span className="material-symbols-outlined leading-none text-xl">{entry.icon}</span>
                </Toggle>
            ))}
        </ToggleGroup>
    )
}
