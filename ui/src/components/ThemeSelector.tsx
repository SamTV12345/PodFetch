import { FC, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import * as ToggleGroup from '@radix-ui/react-toggle-group'
import 'material-symbols/outlined.css'

export const ThemeSelector: FC = () => {
    const [theme, setTheme] = useState<string>()
    const { t } = useTranslation()

    /* Initialize state from local storage */
    useEffect(() => {
        setTheme(localStorage.theme || 'system')
    }, [])

    /* Update local storage whenever state changes */
    useEffect(() => {
        switch (theme) {
            case 'system':
                localStorage.removeItem('theme')
                break
            case 'light':
                localStorage.theme = 'light'
                break
            case 'dark':
                localStorage.theme = 'dark'
                break
        }

        updateDOM()
    }, [theme])

    /* Update CSS class in DOM accordingly */
    const updateDOM = () => {
        if (localStorage.theme === 'dark' || (!('theme' in localStorage) && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
            document.documentElement.classList.add('dark')
        } else {
            document.documentElement.classList.remove('dark')
        }
    }

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
        <ToggleGroup.Root className="flex items-center border border-[--border-color] p-0.5 rounded-full" defaultValue="" onValueChange={(v) => { if (v) setTheme(v) }} value={theme} type="single" aria-label={t('theme')}>
            {themes.map((theme) =>
                <ToggleGroup.Item aria-label={t(theme.translationKey)} className="aspect-square p-2 rounded-full text-[--fg-color] hover:text-[--fg-color-hover] data-[state=on]:bg-[--border-color] data-[state=on]:text-[--fg-color]" key={theme.value} value={theme.value}>
                    <span className="material-symbols-outlined leading-none text-xl">{theme.icon}</span>
                </ToggleGroup.Item>
            )}
        </ToggleGroup.Root>
    )
}
