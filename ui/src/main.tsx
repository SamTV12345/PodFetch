import "./utils/navigationUtils"
import React from 'react'
import ReactDOM from 'react-dom/client'
import { I18nextProvider } from 'react-i18next'
import { RouterProvider } from 'react-router-dom'
import {Toaster} from '@/components/ui/sonner'
import { router } from './App'
import i18n from './language/i18n'
import '@fontsource-variable/geist'
import './index.css'
import {$api} from "./utils/http";
import {QueryClientProvider} from "@tanstack/react-query";
import {setAuth, setLogin} from "./utils/login";
import {
    exchangeCode,
    isExpectedState,
    OIDC_STATE_KEY,
    PKCE_VERIFIER_KEY,
    readCallback,
    refreshTokens,
    rememberLoginError,
    startLogin,
    type TokenResponse,
    urlAfterCallback
} from "./utils/oidc";
import {getConfigFromHtmlFile} from "./utils/config";
import {queryClient} from "./utils/socketio";
import {registerPwaServiceWorker} from "./utils/pwa";
import {applyThemeToDOM} from "./utils/theme";

const config = getConfigFromHtmlFile()
applyThemeToDOM()
registerPwaServiceWorker()


if (config) {
    const postUrl = window.location.origin + import.meta.env.BASE_URL + "login"
    const onLoginPage = window.location.pathname.replace(/\/+$/, '').endsWith('login')
    const onInvitePage = window.location.pathname.includes('/invite/')
    const oidcConfig = config.oidcConfigured ? config.oidcConfig : undefined
    // The provider may redirect to any page, including the login page (the
    // redirect URI the docs used to recommend), so look at the callback first.
    const oidcCallback = oidcConfig ? readCallback(window.location.search) : { kind: 'none' as const }

    const failOidcLogin = (message: string) => {
        console.error('OIDC login failed:', message)
        sessionStorage.removeItem(PKCE_VERIFIER_KEY)
        sessionStorage.removeItem(OIDC_STATE_KEY)
        rememberLoginError(message)
        // Stay on the login page instead of redirecting to the provider
        // again - that would loop if the provider keeps rejecting us.
        window.location.replace(postUrl)
        throw new Error(message)
    }

    if (oidcConfig && !onInvitePage && (oidcCallback.kind !== 'none' || !onLoginPage)) {
        setLogin({
            rememberMe: false,
            loginType: 'oidc'
        })

        if (oidcCallback.kind === 'error') {
            failOidcLogin(i18n.t('oidc-login-error', {
                status: oidcCallback.description ? `${oidcCallback.error}: ${oidcCallback.description}` : oidcCallback.error
            }))
        } else if (oidcCallback.kind === 'code') {
            const expectedState = sessionStorage.getItem(OIDC_STATE_KEY)
            if (!isExpectedState(oidcCallback.state, expectedState)) {
                failOidcLogin(i18n.t('oidc-state-mismatch'))
            }
            const codeVerifier = sessionStorage.getItem(PKCE_VERIFIER_KEY) || ''
            sessionStorage.removeItem(OIDC_STATE_KEY)
            sessionStorage.removeItem(PKCE_VERIFIER_KEY)

            let resp: Response | undefined
            try {
                resp = await exchangeCode(oidcConfig, oidcCallback.code, codeVerifier)
            } catch (e) {
                failOidcLogin(i18n.t('oidc-login-error', { status: String(e) }))
            }
            if (!resp?.ok) {
                failOidcLogin(i18n.t('oidc-login-error', { status: `${resp?.status} ${resp?.statusText}` }))
            }
            const tokenResponse: TokenResponse = await resp!.json()
            if (!tokenResponse.id_token) {
                failOidcLogin(i18n.t('oidc-login-error', { status: 'no id_token in token response' }))
            }
            setAuth(tokenResponse.id_token!)
            // The router was created (with the callback URL) when App was
            // imported, so let it navigate - replaceState alone would leave
            // it on the login route for callbacks on /ui/login.
            const target = urlAfterCallback(window.location, import.meta.env.BASE_URL)
            const base = import.meta.env.BASE_URL.replace(/\/+$/, '')
            await router.navigate(target.startsWith(base) ? target.slice(base.length) || '/' : target, { replace: true })

            let refreshToken = tokenResponse.refresh_token
            if (refreshToken) {
                setInterval(() => {
                    if (!refreshToken) return
                    refreshTokens(oidcConfig, refreshToken).then(async (refreshResp) => {
                        if (!refreshResp.ok) return
                        const refreshData: TokenResponse = await refreshResp.json()
                        if (refreshData.id_token) {
                            setAuth(refreshData.id_token)
                        }
                        // Providers may rotate refresh tokens.
                        if (refreshData.refresh_token) {
                            refreshToken = refreshData.refresh_token
                        }
                    }).catch(() => {})
                }, oidcConfig.refreshInterval)
            }
        } else {
            await startLogin(oidcConfig)
        }
    } else if (!onLoginPage && !onInvitePage && config.basicAuth) {
        setLogin({
            rememberMe: false,
            loginType: 'basic'
        })
        let basicAuth = sessionStorage.getItem('auth') || localStorage.getItem('auth')

        if (!basicAuth || atob(basicAuth).split(':').length !== 2) {
            window.location.replace(postUrl)
            throw new Error('No basic auth found')
        }
        const basicAuthDecoded = atob(basicAuth)
        queryClient.fetchQuery($api.queryOptions('post', '/api/v1/login', {
            body: {
                username: basicAuthDecoded.split(':')[0]!,
                password: basicAuthDecoded.split(':')[1]!
            }
        }, {retry: false})).then(()=>{
            console.log('Logged in successfully')
        }).catch(()=>{
            window.location.href = postUrl
        })
    }
}



ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
    <React.StrictMode>
        <QueryClientProvider client={queryClient}>
        <I18nextProvider i18n={i18n}>
            <RouterProvider router={router} />
            <Toaster position="bottom-right" richColors closeButton />
        </I18nextProvider>
        </QueryClientProvider>
    </React.StrictMode>
)
