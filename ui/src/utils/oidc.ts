import type { components } from '../../schema'

/**
 * Browser side of the OIDC authorization code flow with PKCE (public client).
 *
 * - `state` is generated per login and checked on the callback (required by
 *   strict providers such as Pocket ID or Ory, and protects against CSRF).
 * - Authorization and token endpoint come from the server, which takes them
 *   from OIDC_AUTHORIZATION_ENDPOINT / OIDC_TOKEN_ENDPOINT or the provider's
 *   discovery document instead of guessing `<authority>/../token`.
 * - The callback is handled on any path, including `/ui/login`, which the
 *   docs used to recommend as redirect URI.
 */

export type OidcConfig = components['schemas']['OidcConfig']

export const PKCE_VERIFIER_KEY = 'pkce_code_verifier'
export const OIDC_STATE_KEY = 'oidc_state'
export const OIDC_ERROR_KEY = 'oidc_login_error'

const base64UrlEncode = (bytes: ArrayBuffer | Uint8Array) =>
    btoa(String.fromCharCode(...new Uint8Array(bytes)))
        .replace(/\+/g, '-')
        .replace(/\//g, '_')
        .replace(/=+$/, '')

/** Cryptographically random, URL-safe string (43 characters for 32 bytes). */
export const randomUrlSafeString = (byteLength = 32) => {
    const bytes = new Uint8Array(byteLength)
    window.crypto.getRandomValues(bytes)
    return base64UrlEncode(bytes)
}

export const createCodeChallenge = async (verifier: string) => {
    const digest = await window.crypto.subtle.digest('SHA-256', new TextEncoder().encode(verifier))
    return base64UrlEncode(digest)
}

export const buildAuthorizationUrl = (
    config: OidcConfig,
    params: { state: string; codeChallenge: string }
) => {
    const url = new URL(config.authorizationEndpoint)
    url.searchParams.set('client_id', config.clientId)
    url.searchParams.set('redirect_uri', config.redirectUri)
    url.searchParams.set('response_type', 'code')
    url.searchParams.set('scope', config.scope)
    url.searchParams.set('state', params.state)
    url.searchParams.set('code_challenge', params.codeChallenge)
    url.searchParams.set('code_challenge_method', 'S256')
    return url.toString()
}

export type OidcCallback =
    | { kind: 'none' }
    | { kind: 'code'; code: string; state: string | null }
    | { kind: 'error'; error: string; description: string | null }

/** What the provider sent back in the query string, if anything. */
export const readCallback = (search: string): OidcCallback => {
    const params = new URLSearchParams(search)
    const error = params.get('error')
    if (error) {
        return { kind: 'error', error, description: params.get('error_description') }
    }
    const code = params.get('code')
    if (code) {
        return { kind: 'code', code, state: params.get('state') }
    }
    return { kind: 'none' }
}

export const isExpectedState = (returned: string | null, expected: string | null) =>
    !!expected && returned === expected

const CALLBACK_PARAMS = ['code', 'state', 'iss', 'session_state', 'scope', 'error', 'error_description', 'error_uri']

/**
 * The URL to show after the callback: provider parameters removed, and a
 * callback on the login page continues to the app.
 */
export const urlAfterCallback = (location: Pick<Location, 'pathname' | 'search'>, appBasePath: string) => {
    const params = new URLSearchParams(location.search)
    CALLBACK_PARAMS.forEach(param => params.delete(param))
    const pathname = location.pathname.replace(/\/+$/, '').endsWith('/login') ? appBasePath : location.pathname
    const search = params.toString()
    return pathname + (search ? `?${search}` : '')
}

/** Redirect the browser to the provider's login page. */
export const startLogin = async (config: OidcConfig, storage: Storage = sessionStorage) => {
    const verifier = randomUrlSafeString(64)
    const state = randomUrlSafeString(32)
    storage.setItem(PKCE_VERIFIER_KEY, verifier)
    storage.setItem(OIDC_STATE_KEY, state)
    const codeChallenge = await createCodeChallenge(verifier)
    window.location.replace(buildAuthorizationUrl(config, { state, codeChallenge }))
}

export type TokenResponse = {
    id_token?: string
    refresh_token?: string
}

const postToTokenEndpoint = (config: OidcConfig, body: Record<string, string>) =>
    fetch(config.tokenEndpoint, {
        method: 'POST',
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: new URLSearchParams(body),
    })

export const exchangeCode = (config: OidcConfig, code: string, codeVerifier: string) =>
    postToTokenEndpoint(config, {
        grant_type: 'authorization_code',
        code,
        redirect_uri: config.redirectUri,
        client_id: config.clientId,
        code_verifier: codeVerifier,
    })

export const refreshTokens = (config: OidcConfig, refreshToken: string) =>
    postToTokenEndpoint(config, {
        grant_type: 'refresh_token',
        refresh_token: refreshToken,
        client_id: config.clientId,
    })

/** Remember a login error so the login page can show it after the redirect. */
export const rememberLoginError = (message: string, storage: Storage = sessionStorage) =>
    storage.setItem(OIDC_ERROR_KEY, message)

export const takeLoginError = (storage: Storage = sessionStorage) => {
    const message = storage.getItem(OIDC_ERROR_KEY)
    storage.removeItem(OIDC_ERROR_KEY)
    return message
}
