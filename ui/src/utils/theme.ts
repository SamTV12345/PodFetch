export type ThemePreference = 'system' | 'light' | 'dark'

const themeStorageKey = 'theme'
const darkQuery = typeof window !== 'undefined' && 'matchMedia' in window
    ? window.matchMedia('(prefers-color-scheme: dark)')
    : undefined

export const getStoredThemePreference = (): ThemePreference => {
    if (typeof window === 'undefined') {
        return 'system'
    }

    const storedTheme = localStorage.getItem(themeStorageKey)
    if (storedTheme === 'light' || storedTheme === 'dark') {
        return storedTheme
    }
    return 'system'
}

const isDarkTheme = (theme: ThemePreference): boolean => {
    if (theme === 'dark') {
        return true
    }

    if (theme === 'light') {
        return false
    }

    return darkQuery?.matches ?? false
}

export const applyThemeToDOM = (theme: ThemePreference = getStoredThemePreference()) => {
    if (typeof document === 'undefined') {
        return
    }

    document.documentElement.classList.toggle('dark', isDarkTheme(theme))
}

export const setThemePreference = (theme: ThemePreference) => {
    if (typeof window === 'undefined') {
        return
    }

    if (theme === 'system') {
        localStorage.removeItem(themeStorageKey)
    } else {
        localStorage.setItem(themeStorageKey, theme)
    }
    applyThemeToDOM(theme)
}

export const onSystemThemeChange = (cb: () => void) => {
    if (!darkQuery) {
        return () => undefined
    }

    const handler = () => cb()
    darkQuery.addEventListener('change', handler)
    return () => darkQuery.removeEventListener('change', handler)
}
