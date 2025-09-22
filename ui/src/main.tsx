import "./utils/navigationUtils"
import React from 'react'
import ReactDOM from 'react-dom/client'
import { I18nextProvider } from 'react-i18next'
import { RouterProvider } from 'react-router-dom'
import {enqueueSnackbar, SnackbarProvider} from 'notistack'
import { router } from './App'
import i18n from './language/i18n'
import '@fortawesome/fontawesome-free/css/all.min.css'
import '@fontsource-variable/inter'
import '@fontsource/roboto'
import '@fontsource/anton'
import '@fontsource/poppins/400.css'
import '@fontsource/poppins/400-italic.css'
import '@fontsource/poppins/500.css'
import '@fontsource/poppins/500-italic.css'
import '@fontsource/poppins/700.css'
import '@fontsource/poppins/700-italic.css'
import './index.css'
import {client} from "./utils/http";
import {QueryClient, QueryClientProvider} from "@tanstack/react-query";
import {setAuth, setLogin} from "./utils/login";
import {getConfigFromHtmlFile} from "./utils/config";

const config = getConfigFromHtmlFile()


if (config) {
    const postUrl = config.serverUrl + "ui/login"
    if (!window.location.pathname.endsWith('login')) {
        setLogin({
            rememberMe: false,
            loginType: 'oidc'
        })
        if (config.oidcConfigured && config.oidcConfig) {
            function base64UrlEncode(str: ArrayBuffer) {
                return btoa(String.fromCharCode(...new Uint8Array(str)))
                    .replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
            }

            async function generateCodeChallenge(verifier: string) {
                const data = new TextEncoder().encode(verifier);
                const digest = await window.crypto.subtle.digest('SHA-256', data);
                return base64UrlEncode(digest);
            }

            function generateCodeVerifier(length = 128) {
                const array = new Uint8Array(length);
                window.crypto.getRandomValues(array);
                return Array.from(array, b => ('0' + (b % 36).toString(36)).slice(-1)).join('');
            }

            let redirectUri = window.location.href
            redirectUri = redirectUri.replace(/[?&](code|state|iss|session_state)=[^&]*/g, '')

            if (window.location.search.includes('code=')) {
                console.log('Redirecting to', window.location.href, "with client" + config.oidcConfig.clientId)
                try {

                    const codeVerifier = sessionStorage.getItem('pkce_code_verifier') || '';
                const resp = await fetch(config.oidcConfig?.authority + "/../token", {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/x-www-form-urlencoded'
                    },
                    body: new URLSearchParams({
                        grant_type: 'authorization_code',
                        code: new URLSearchParams(window.location.search).get('code') || '',
                        redirect_uri: redirectUri,
                        client_id: config.oidcConfig.clientId,
                        code_verifier: codeVerifier
                    })})
                if (resp.ok) {
                    const tokenResponse = await resp.json()
                    if (tokenResponse.id_token) {
                        setAuth(tokenResponse.id_token)
                        const params = new URLSearchParams(window.location.search);
                        params.delete('code');
                        params.delete('state');
                        params.delete('iss');
                        params.delete('session_state');
                        const newSearch = params.toString();
                        const newUrl = window.location.pathname + (newSearch ? '?' + newSearch : '');

                        window.history.replaceState({}, '', newUrl);
                    }
                    setInterval(()=>{
                        if (tokenResponse.refresh_token) {
                            fetch(config.oidcConfig?.authority + "/../token", {
                                method: 'POST',
                                headers: {
                                    'Content-Type': 'application/x-www-form-urlencoded'
                                },
                                body: new URLSearchParams({
                                    grant_type: 'refresh_token',
                                    refresh_token: tokenResponse.refresh_token,
                                    client_id: config.oidcConfig?.clientId ?? ''
                                })
                            }).then((refreshResp)=>{
                                if (refreshResp.ok) {
                                    refreshResp.json().then((refreshData)=>{
                                        if (refreshData.id_token) {
                                            setAuth(refreshData.id_token)
                                        }
                                    })
                                }
                            })
                        }
                    }, config.oidcConfig.refreshInterval)
                } else {
                    enqueueSnackbar('Error during OIDC login: ' + resp.statusText, {variant: 'error'})
                }
                } catch (e) {

                }
            } else {
                const codeVerifier = generateCodeVerifier();
                sessionStorage.setItem('pkce_code_verifier', codeVerifier);
                const codeChallenge = await generateCodeChallenge(codeVerifier)
                const requestUrl = `${config.oidcConfig.authority}?client_id=${config.oidcConfig.clientId}&redirect_uri=${encodeURIComponent(redirectUri)}&response_type=code&scope=${encodeURIComponent(config.oidcConfig.scope!)}&code_challenge=${codeChallenge}&code_challenge_method=S256`
                console.error(requestUrl)
                window.location.replace(requestUrl)
            }
        } else if (config.basicAuth) {
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
            client.POST('/api/v1/login', {
                body: {
                    username: basicAuthDecoded.split(':')[0]!,
                    password: basicAuthDecoded.split(':')[1]!
                }
            }).then(()=>{
                console.log('Logged in successfully')
            }).catch(()=>{
                window.location.href = postUrl
            })
        }
    }
}


const queryClient = new QueryClient()
ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
    <React.StrictMode>
        <QueryClientProvider client={queryClient}>
        <I18nextProvider i18n={i18n}>
            <SnackbarProvider maxSnack={4} >
                <RouterProvider router={router} />
            </SnackbarProvider>
        </I18nextProvider>
        </QueryClientProvider>
    </React.StrictMode>
)
