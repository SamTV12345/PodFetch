import { afterEach, describe, expect, it, vi } from 'vitest'
import {
    buildAuthorizationUrl,
    createCodeChallenge,
    isExpectedState,
    OIDC_STATE_KEY,
    PKCE_VERIFIER_KEY,
    randomUrlSafeString,
    readCallback,
    rememberLoginError,
    startLogin,
    takeLoginError,
    urlAfterCallback,
    type OidcConfig,
} from './oidc'

const pocketId: OidcConfig = {
    authority: 'https://id.example.com',
    authorizationEndpoint: 'https://id.example.com/authorize',
    tokenEndpoint: 'https://id.example.com/api/oidc/token',
    clientId: 'podfetch client',
    redirectUri: 'https://podfetch.example.com/ui/login',
    scope: 'openid profile email',
    jwksUri: 'https://id.example.com/.well-known/jwks.json',
    refreshInterval: 120000,
}

describe('buildAuthorizationUrl', () => {
    it('uses the authorization endpoint and sends state and PKCE parameters', () => {
        const url = new URL(buildAuthorizationUrl(pocketId, { state: 'the-state', codeChallenge: 'the-challenge' }))

        expect(url.origin + url.pathname).toBe('https://id.example.com/authorize')
        expect(url.searchParams.get('client_id')).toBe('podfetch client')
        expect(url.searchParams.get('redirect_uri')).toBe('https://podfetch.example.com/ui/login')
        expect(url.searchParams.get('response_type')).toBe('code')
        expect(url.searchParams.get('scope')).toBe('openid profile email')
        expect(url.searchParams.get('state')).toBe('the-state')
        expect(url.searchParams.get('code_challenge')).toBe('the-challenge')
        expect(url.searchParams.get('code_challenge_method')).toBe('S256')
    })

    it('keeps query parameters that are part of the configured endpoint', () => {
        const url = new URL(buildAuthorizationUrl(
            { ...pocketId, authorizationEndpoint: 'https://idp.example.com/oauth2/auth?tenant=home' },
            { state: 's', codeChallenge: 'c' },
        ))
        expect(url.searchParams.get('tenant')).toBe('home')
        expect(url.searchParams.get('state')).toBe('s')
    })
})

describe('readCallback', () => {
    it('detects an authorization code with state', () => {
        expect(readCallback('?code=abc&state=xyz&iss=https%3A%2F%2Fid.example.com')).toEqual({
            kind: 'code', code: 'abc', state: 'xyz',
        })
    })

    it('detects provider errors instead of treating them as "not logged in yet"', () => {
        expect(readCallback('?error=invalid_state&error_description=The+state+is+missing')).toEqual({
            kind: 'error', error: 'invalid_state', description: 'The state is missing',
        })
    })

    it('returns none for unrelated query strings', () => {
        expect(readCallback('?tab=episodes')).toEqual({ kind: 'none' })
        expect(readCallback('')).toEqual({ kind: 'none' })
    })
})

describe('isExpectedState', () => {
    it('only accepts the stored state', () => {
        expect(isExpectedState('abc', 'abc')).toBe(true)
        expect(isExpectedState('abc', 'other')).toBe(false)
        expect(isExpectedState(null, 'abc')).toBe(false)
        // No login was started from this browser tab.
        expect(isExpectedState('abc', null)).toBe(false)
        expect(isExpectedState(null, null)).toBe(false)
    })
})

describe('urlAfterCallback', () => {
    it('continues to the app when the callback landed on the login page', () => {
        expect(urlAfterCallback({ pathname: '/ui/login', search: '?code=a&state=b' }, '/ui/')).toBe('/ui/')
        expect(urlAfterCallback({ pathname: '/ui/login/', search: '?code=a&state=b' }, '/ui/')).toBe('/ui/')
    })

    it('stays on other pages and keeps unrelated parameters', () => {
        expect(urlAfterCallback(
            { pathname: '/ui/podcasts', search: '?code=a&state=b&session_state=c&iss=d&tab=1' },
            '/ui/',
        )).toBe('/ui/podcasts?tab=1')
    })
})

describe('random values and PKCE', () => {
    it('creates URL-safe random strings that differ', () => {
        const first = randomUrlSafeString(32)
        expect(first).toMatch(/^[A-Za-z0-9_-]{43}$/)
        expect(randomUrlSafeString(32)).not.toBe(first)
    })

    it('computes the S256 code challenge (RFC 7636 appendix B)', async () => {
        expect(await createCodeChallenge('dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk'))
            .toBe('E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM')
    })
})

describe('startLogin', () => {
    afterEach(() => {
        vi.unstubAllGlobals()
        sessionStorage.clear()
    })

    it('stores state and verifier and redirects with the same state', async () => {
        const replace = vi.fn()
        vi.stubGlobal('location', { ...window.location, replace })

        await startLogin(pocketId)

        const state = sessionStorage.getItem(OIDC_STATE_KEY)
        const verifier = sessionStorage.getItem(PKCE_VERIFIER_KEY)
        expect(state).toMatch(/^[A-Za-z0-9_-]{43}$/)
        expect(verifier!.length).toBeGreaterThanOrEqual(43)
        expect(verifier!.length).toBeLessThanOrEqual(128)

        const target = new URL(replace.mock.calls[0]![0])
        expect(target.searchParams.get('state')).toBe(state)
        expect(target.searchParams.get('code_challenge')).toBe(await createCodeChallenge(verifier!))
    })
})

describe('login errors', () => {
    it('are shown once', () => {
        rememberLoginError('invalid_state')
        expect(takeLoginError()).toBe('invalid_state')
        expect(takeLoginError()).toBeNull()
    })
})
