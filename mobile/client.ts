import {paths, components} from "@/schema";
import createClient from "openapi-react-query";
import createFetchClient, { Middleware } from "openapi-fetch";
import { useStore } from "@/store/store";
import uuid from 'react-native-uuid';


const fetchClient = createFetchClient<paths>({
    baseUrl: "",
});

const baseUrlMiddleware: Middleware = {
    async onRequest({ request }) {
        const state = useStore.getState();
        const serverUrl = state.serverUrl;

        if (serverUrl) {
            let pathname: string;
            try {
                const url = new URL(request.url);
                pathname = url.pathname + url.search;
            } catch {
                pathname = request.url;
            }

            const fullUrl = serverUrl.replace(/\/$/, '') + pathname;
            const newRequest = new Request(fullUrl, request);

            if (state.authType === 'basic' && state.basicAuthUsername && state.basicAuthPassword) {
                const credentials = btoa(`${state.basicAuthUsername}:${state.basicAuthPassword}`);
                newRequest.headers.set('Authorization', `Basic ${credentials}`);
            } else if (state.authType === 'oidc' && state.oidcAccessToken) {
                newRequest.headers.set('Authorization', `Bearer ${state.oidcAccessToken}`);
            }

            return newRequest;
        }
        return request;
    },
};

fetchClient.use(baseUrlMiddleware);

export const $api = createClient(fetchClient);

const createAuthenticatedClient = (serverUrl: string, username: string, password: string) => {
    const client = createFetchClient<paths>({
        baseUrl: serverUrl.replace(/\/$/, ''),
        headers: {
            'Authorization': `Basic ${btoa(`${username}:${password}`)}`,
        },
    });
    return client;
};

export type ServerConfigResult = {
    success: true;
    config: components["schemas"]["ConfigModel"];
} | {
    success: false;
    error: string;
};

export const validatePodFetchServer = async (url: string): Promise<ServerConfigResult> => {
    try {
        let normalizedUrl = url.trim();
        if (!normalizedUrl.startsWith('http://') && !normalizedUrl.startsWith('https://')) {
            normalizedUrl = 'http://' + normalizedUrl;
        }
        normalizedUrl = normalizedUrl.replace(/\/$/, '');

        // Verwende baseFetchClient ohne Auth
        const tempClient = createFetchClient<paths>({
            baseUrl: normalizedUrl,
        });

        const { data, error } = await tempClient.GET('/api/v1/sys/config');

        if (data) {
            return { success: true, config: data };
        }

        return { success: false, error: error ? String(error) : 'Server returned an error' };
    } catch (error) {
        console.error('Server validation failed:', error);
        return { success: false, error: 'Connection failed' };
    }
};

export const validateBasicAuth = async (
    url: string,
    username: string,
    password: string
): Promise<boolean> => {
    try {
        let normalizedUrl = url.trim().replace(/\/$/, '');
        if (!normalizedUrl.startsWith('http://') && !normalizedUrl.startsWith('https://')) {
            normalizedUrl = 'http://' + normalizedUrl;
        }

        const client = createAuthenticatedClient(normalizedUrl, username, password);
        const { data, error } = await client.GET('/api/v1/podcasts');

        return !error && !!data;
    } catch (error) {
        console.error('Basic auth validation failed:', error);
        return false;
    }
};

export const fetchUserProfile = async (
    serverUrl: string,
    username: string,
    password: string
): Promise<components["schemas"]["UserWithAPiKey"] | null> => {
    try {
        const normalizedUrl = serverUrl.replace(/\/$/, '');
        const client = createAuthenticatedClient(normalizedUrl, username, password);

        const { data, error } = await client.GET('/api/v1/users/{username}', {
            params: {
                path: { username },
            },
        });

        if (error) {
            console.error('Failed to fetch user profile:', error);
            return null;
        }

        return data ?? null;
    } catch (error) {
        console.error('Failed to fetch user profile:', error);
        return null;
    }
};

export const exchangeOidcCode = async (
    tokenEndpoint: string,
    code: string,
    clientId: string,
    redirectUri: string,
    codeVerifier?: string
): Promise<{
    access_token: string;
    refresh_token?: string;
    expires_in?: number;
} | null> => {
    try {
        const body = new URLSearchParams({
            grant_type: 'authorization_code',
            code,
            client_id: clientId,
            redirect_uri: redirectUri,
            ...(codeVerifier && { code_verifier: codeVerifier }),
        });

        const response = await fetch(tokenEndpoint, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/x-www-form-urlencoded',
            },
            body: body.toString(),
        });

        if (response.ok) {
            return await response.json();
        }
        return null;
    } catch (error) {
        console.error('OIDC token exchange failed:', error);
        return null;
    }
};

export const refreshOidcToken = async (
    tokenEndpoint: string,
    refreshToken: string,
    clientId: string
): Promise<{
    access_token: string;
    refresh_token?: string;
    expires_in?: number;
} | null> => {
    try {
        const body = new URLSearchParams({
            grant_type: 'refresh_token',
            refresh_token: refreshToken,
            client_id: clientId,
        });

        const response = await fetch(tokenEndpoint, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/x-www-form-urlencoded',
            },
            body: body.toString(),
        });

        if (response.ok) {
            return await response.json();
        }
        return null;
    } catch (error) {
        console.error('OIDC token refresh failed:', error);
        return null;
    }
};

export const updateUserProfile = async (
    serverUrl: string,
    username: string,
    password: string,
    updateData: components["schemas"]["UserCoreUpdateModel"]
): Promise<components["schemas"]["UserWithAPiKey"] | null> => {
    try {
        const normalizedUrl = serverUrl.replace(/\/$/, '');
        const client = createAuthenticatedClient(normalizedUrl, username, password);

        const { data, error } = await client.PUT('/api/v1/users/{username}', {
            params: {
                path: { username: updateData.username },
            },
            body: updateData,
        });

        if (error) {
            console.error('Failed to update user profile:', error);
            return null;
        }

        return data ?? null;
    } catch (error) {
        console.error('Failed to update user profile:', error);
        return null;
    }
};

export const generateNewApiKey = (): string => {
    const generatedUUID = uuid.v4()
    return generatedUUID.replace(/-/g, '');
};

