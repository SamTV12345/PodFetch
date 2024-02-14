import "./utils/navigationUtils"
import React, { FC, PropsWithChildren, useEffect } from 'react'
import ReactDOM from 'react-dom/client'
import { I18nextProvider } from 'react-i18next'
import { AuthProvider } from 'react-oidc-context'
import { RouterProvider } from 'react-router-dom'
import axios, { AxiosResponse } from 'axios'
import { SnackbarProvider } from 'notistack'
import { router } from './App'
import useCommon from './store/CommonSlice'
import i18n from './language/i18n'
import { ConfigModel } from './models/SysInfo'
import { Loading } from './components/Loading'
import { OIDCRefresher } from './components/OIDCRefresher'
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
import './assets/scss/style.scss'
export let apiURL: string
export let uiURL: string
if (window.location.pathname.startsWith("/ui")) {
    apiURL = window.location.protocol + "//" + window.location.hostname + ":" + window.location.port + "/api/v1"
} else {
    //match everything before /ui
    const regex = /\/([^/]+)\/ui\//
    apiURL = window.location.protocol + "//" + window.location.hostname + ":" + window.location.port + "/" + regex.exec(window.location.href)![1] + "/api/v1"
}
uiURL = window.location.protocol + "//" + window.location.hostname + ":" + window.location.port + "/ui"

const AuthWrapper: FC<PropsWithChildren> = ({ children }) => {
    const configModel = useCommon(state => state.configModel)
    const setConfigModel = useCommon(state => state.setConfigModel)

    useEffect(() => {
        axios.defaults.baseURL = apiURL
        axios.get('/sys/config').then((v: AxiosResponse<ConfigModel>) => {
            setConfigModel(v.data)
        })
    }, [])

    if (configModel === undefined) {
        return <Loading />
    }

    if (configModel.oidcConfigured && configModel.oidcConfig) {
        return (
            <AuthProvider client_id={configModel.oidcConfig.clientId} authority={configModel.oidcConfig.authority} scope={configModel.oidcConfig.scope} redirect_uri={configModel.oidcConfig.redirectUri}>
                <OIDCRefresher>
                    {children}
                </OIDCRefresher>
            </AuthProvider>
        )
    }

    return (
        <>
            {children}
        </>
    )
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
    <React.StrictMode>
        <I18nextProvider i18n={i18n}>
                <AuthWrapper>
                    <SnackbarProvider maxSnack={4} >
                        <RouterProvider router={router} />
                    </SnackbarProvider>
                </AuthWrapper>
        </I18nextProvider>
    </React.StrictMode>,
)
