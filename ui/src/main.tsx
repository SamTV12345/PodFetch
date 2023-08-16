import React, { FC, PropsWithChildren, useEffect } from 'react'
import ReactDOM from 'react-dom/client'
import { I18nextProvider } from 'react-i18next'
import { AuthProvider } from 'react-oidc-context'
import { Provider } from 'react-redux'
import { RouterProvider } from 'react-router-dom'
import axios, { AxiosResponse } from 'axios'
import { SnackbarProvider } from 'notistack'
import { router } from './App'
import { store } from './store/store'
import { useAppDispatch, useAppSelector } from './store/hooks'
import { setConfigModel } from './store/CommonSlice'
import i18n from './language/i18n'
import { apiURL } from './utils/Utilities'
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

const AuthWrapper: FC<PropsWithChildren> = ({ children }) => {
    const dispatch = useAppDispatch()
    const configModel = useAppSelector(state => state.common.configModel)

    useEffect(() => {
        axios.get(apiURL + '/sys/config').then((v: AxiosResponse<ConfigModel>) => {
            dispatch(setConfigModel(v.data))
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
            <Provider store={store}>
                <AuthWrapper>
                    <SnackbarProvider maxSnack={4} >
                        <RouterProvider router={router} />
                    </SnackbarProvider>
                </AuthWrapper>
            </Provider>
        </I18nextProvider>
    </React.StrictMode>,
)
