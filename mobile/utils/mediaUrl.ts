import { useStore } from '@/store/store';

/**
 * Fügt den API-Key als Query-Parameter an eine URL an,
 * wenn Basic Auth aktiv ist und ein API-Key verfügbar ist.
 */
export const appendApiKeyToUrl = (url: string, apiKey?: string | null): string => {
    if (!apiKey) return url;

    try {
        const urlObj = new URL(url);
        urlObj.searchParams.set('apiKey', apiKey);
        return urlObj.toString();
    } catch {
        const separator = url.includes('?') ? '&' : '?';
        return `${url}${separator}apiKey=${encodeURIComponent(apiKey)}`;
    }
};

/**
 * Erstellt die vollständige Media-URL mit Auth-Informationen.
 * Verwendet den gespeicherten API-Key aus dem Store.
 */
export const getAuthenticatedMediaUrl = (
    baseUrl: string,
    serverUrl: string | null,
    apiKey?: string | null
): string => {
    if (!serverUrl) return baseUrl;

    const isAbsoluteUrl = (url: string) => url.startsWith('http://') || url.startsWith('https://');

    let fullUrl: string;

    if (isAbsoluteUrl(baseUrl)) {
        fullUrl = baseUrl;
    } else {
        fullUrl = serverUrl.replace(/\/$/, '') + baseUrl;
    }

    if (apiKey) {
        return appendApiKeyToUrl(fullUrl, apiKey);
    }

    return fullUrl;
};

/**
 * Hook um die Media-URL mit Auth zu erhalten.
 */
export const useAuthenticatedUrl = () => {
    const serverUrl = useStore((state) => state.serverUrl);
    const userApiKey = useStore((state) => state.userApiKey);
    const authType = useStore((state) => state.authType);

    return (url: string): string => {
        if (!url) return '';

        // API-Key nur anhängen wenn Basic Auth aktiv ist
        const apiKey = authType === 'basic' ? userApiKey : null;

        return getAuthenticatedMediaUrl(url, serverUrl, apiKey);
    };
};

