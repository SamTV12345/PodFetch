import { expect, test, type APIRequestContext } from '@playwright/test'

// OIDC login against a strict provider (#2315), see servers/mock-oidc.mjs:
// state required, PKCE S256, asymmetric endpoints (/authorize,
// /api/oidc/token) that only the discovery document reveals, and the
// redirect URI on the login page (/ui/login) as the docs used to recommend.
// Runs against the second podfetch instance on :8001.

const PODFETCH = 'http://127.0.0.1:8001'
const IDP = 'http://127.0.0.1:9997'
// Somewhere in the app, not the login page, without provider parameters.
const IN_APP = new RegExp(`^${PODFETCH}/ui/(?!login)[^?]*$`)

let idp: APIRequestContext

const idpStats = async () =>
    (await (await idp.get('/__test/stats')).json()) as { authorize: number; token: number; rejectedAuthorize: number }

test.beforeAll(async ({ playwright }) => {
    idp = await playwright.request.newContext({ baseURL: IDP })

    // The endpoints are discovered in the background after startup.
    const podfetch = await playwright.request.newContext({ baseURL: PODFETCH })
    await expect
        .poll(async () => (await (await podfetch.get('/api/v1/sys/config')).json()).oidcConfig?.tokenEndpoint, {
            timeout: 60_000,
        })
        .toBe(`${IDP}/api/oidc/token`)
    await podfetch.dispose()
})

test.beforeEach(async () => {
    await idp.post('/__test/reset')
})

test.afterAll(async () => {
    await idp.dispose()
})

test('the server serves the discovered endpoints', async ({ request }) => {
    const config = await (await request.get(`${PODFETCH}/api/v1/sys/config`)).json()
    expect(config.oidcConfig.authorizationEndpoint).toBe(`${IDP}/authorize`)
    expect(config.oidcConfig.tokenEndpoint).toBe(`${IDP}/api/oidc/token`)
})

test('logs in with a strict provider and a redirect URI on the login page', async ({ page }) => {
    const authorizeRequests: URL[] = []
    page.on('request', request => {
        const url = new URL(request.url())
        if (url.origin === IDP && url.pathname === '/authorize') authorizeRequests.push(url)
    })
    const me = page.waitForResponse(response =>
        response.url().startsWith(`${PODFETCH}/api/v1/users/me`) && response.status() === 200)

    await page.goto(`${PODFETCH}/ui/`)

    // Back in the app - not stuck on the login page - and authenticated.
    await expect(page).toHaveURL(IN_APP)
    expect((await (await me).json()).username).toBe('oidc-e2e-user')

    expect(authorizeRequests).toHaveLength(1)
    const authorize = authorizeRequests[0]!
    expect(authorize.searchParams.get('state')!.length).toBeGreaterThanOrEqual(32)
    expect(authorize.searchParams.get('code_challenge_method')).toBe('S256')
    expect(await idpStats()).toMatchObject({ authorize: 1, token: 1, rejectedAuthorize: 0 })
})

test('rejects a callback whose state does not belong to this login', async ({ page }) => {
    await page.goto(`${PODFETCH}/ui/login?code=forged-code&state=forged-state`)

    await expect(page).toHaveURL(`${PODFETCH}/ui/login`)
    await expect(page.getByRole('alert')).toContainText('state mismatch')
    // The forged code is never sent to the provider.
    expect((await idpStats()).token).toBe(0)
})

test('a provider error ends on the login page instead of a redirect loop', async ({ page }) => {
    await idp.post('/__test/reject')

    await page.goto(`${PODFETCH}/ui/`)

    await expect(page).toHaveURL(`${PODFETCH}/ui/login`)
    await expect(page.getByRole('alert')).toContainText('access_denied')
    await expect(page.getByRole('alert')).toContainText('The user denied the request.')

    // No further round trips to the provider.
    await page.waitForTimeout(2_000)
    expect((await idpStats()).authorize).toBe(1)

    // Trying again works.
    const me = page.waitForResponse(response =>
        response.url().startsWith(`${PODFETCH}/api/v1/users/me`) && response.status() === 200)
    await page.getByRole('button', { name: 'OIDC Login' }).click()
    expect((await (await me).json()).username).toBe('oidc-e2e-user')
    await expect(page).toHaveURL(IN_APP)
    expect(await idpStats()).toMatchObject({ authorize: 2, token: 1 })
})
