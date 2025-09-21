import "./utils/navigationUtils"
import React, { FC, PropsWithChildren, useEffect } from 'react'
import ReactDOM from 'react-dom/client'
import { I18nextProvider } from 'react-i18next'
import { RouterProvider } from 'react-router-dom'
import { SnackbarProvider } from 'notistack'
import { router } from './App'
import i18n from './language/i18n'
import { Loading } from './components/Loading'
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
import {UserManager, UserManagerSettings} from "oidc-client-ts";
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
            // Do OAuth2 flow
            const settings:  UserManagerSettings = {
                authority: config.oidcConfig.authority,
                client_id: config.oidcConfig.clientId,
                redirect_uri: window.location.href,
                post_logout_redirect_uri: postUrl,
                response_type: "code",
                scope: config.oidcConfig.scope + " read:user",
                automaticSilentRenew: true,
                filterProtocolClaims: true
            };
            const oidcClient = new UserManager(settings);
            oidcClient.signinRedirectCallback().then(resp=>{
                setAuth(resp.id_token!)
            })

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
