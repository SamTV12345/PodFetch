import { useEffect, useCallback, useRef } from 'react';
import { useStore } from '@/store/store';
import { refreshOidcToken } from '@/client';

/**
 * Hook that automatically refreshes OIDC tokens before they expire.
 * Should be used in the root layout or a persistent component.
 */
export const useAuthRefresh = () => {
    const {
        authType,
        oidcAccessToken,
        oidcRefreshToken,
        oidcTokenExpiry,
        serverConfig,
        setOidcAccessToken,
        setOidcRefreshToken,
        setOidcTokenExpiry,
        clearAuth,
    } = useStore();

    const refreshTimeoutRef = useRef<NodeJS.Timeout | null>(null);

    const performRefresh = useCallback(async () => {
        if (!oidcRefreshToken || !serverConfig?.oidcConfig) {
            console.warn('Cannot refresh: missing refresh token or OIDC config');
            return false;
        }

        try {
            const tokenEndpoint = `${serverConfig.oidcConfig.authority}/protocol/openid-connect/token`;
            const result = await refreshOidcToken(
                tokenEndpoint,
                oidcRefreshToken,
                serverConfig.oidcConfig.clientId
            );

            if (result) {
                setOidcAccessToken(result.access_token);
                if (result.refresh_token) {
                    setOidcRefreshToken(result.refresh_token);
                }
                if (result.expires_in) {
                    setOidcTokenExpiry(Date.now() + result.expires_in * 1000);
                }
                return true;
            }
        } catch (error) {
            console.error('Token refresh failed:', error);
        }

        // If refresh fails, clear auth and force re-login
        clearAuth();
        return false;
    }, [oidcRefreshToken, serverConfig, setOidcAccessToken, setOidcRefreshToken, setOidcTokenExpiry, clearAuth]);

    useEffect(() => {
        // Only set up refresh for OIDC auth
        if (authType !== 'oidc' || !oidcTokenExpiry || !oidcAccessToken) {
            return;
        }

        // Clear any existing timeout
        if (refreshTimeoutRef.current) {
            clearTimeout(refreshTimeoutRef.current);
        }

        // Calculate time until token expires
        const now = Date.now();
        const timeUntilExpiry = oidcTokenExpiry - now;

        // Refresh 5 minutes before expiry, or immediately if already expired
        const refreshBuffer = 5 * 60 * 1000; // 5 minutes
        const refreshIn = Math.max(0, timeUntilExpiry - refreshBuffer);

        if (refreshIn === 0 && timeUntilExpiry > 0) {
            // Token expires soon but not yet - refresh now
            performRefresh();
        } else if (timeUntilExpiry <= 0) {
            // Token already expired - try to refresh
            performRefresh();
        } else {
            refreshTimeoutRef.current = setTimeout(() => {
                performRefresh();
            }, refreshIn);
        }

        return () => {
            if (refreshTimeoutRef.current) {
                clearTimeout(refreshTimeoutRef.current);
            }
        };
    }, [authType, oidcTokenExpiry, oidcAccessToken, performRefresh]);

    return {
        performRefresh,
        isAuthenticated: authType !== 'none' && (
            (authType === 'basic' && useStore.getState().basicAuthUsername && useStore.getState().basicAuthPassword) ||
            (authType === 'oidc' && oidcAccessToken)
        ),
    };
};

