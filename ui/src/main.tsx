import React, {FC, PropsWithChildren, useEffect} from 'react'
import ReactDOM from 'react-dom/client'
import {router} from './App'
import './index.css'
import {Provider} from "react-redux";
import {store} from "./store/store";
import {I18nextProvider} from "react-i18next";
import i18n from "./language/i18n";
import "@fortawesome/fontawesome-free/css/all.min.css";
import "@fontsource/roboto"
import "@fontsource/anton"
import {RouterProvider} from "react-router-dom";
import axios, {AxiosResponse} from "axios";
import {apiURL} from "./utils/Utilities";
import {ConfigModel} from "./models/SysInfo";
import {setConfigModel} from "./store/CommonSlice";
import {useAppDispatch, useAppSelector} from "./store/hooks";
import {AuthProvider, useAuth} from "react-oidc-context";
import {Loading} from "./components/Loading";

const AuthWrapper:FC<PropsWithChildren> = ({children})=>{
    const dispatch = useAppDispatch()
    const configModel = useAppSelector(state=>state.common.configModel)

    useEffect(()=>{
        axios.get(apiURL+"/sys/config").then((v:AxiosResponse<ConfigModel>)=>{
            dispatch(setConfigModel(v.data))
        })
    },[])

    if(configModel===undefined){
        return <Loading/>
    }

    if(configModel.oidcConfigured && configModel.oidcConfig){
        return <AuthProvider client_id={configModel.oidcConfig.clientId} authority={configModel.oidcConfig.authority} scope={configModel.oidcConfig.scope}
                      redirect_uri={configModel.oidcConfig.redirectUri}>
            <OIDCRefresher>
                {children}
            </OIDCRefresher>
        </AuthProvider>
    }

    return <>{children}</>
}


const refreshInterval = 1000*60
const OIDCRefresher:FC<PropsWithChildren> = ({children})=>{
    const auth = useAuth()

    useEffect(()=>{

        setInterval(()=> {
            if (auth.user &&auth.user.expires_in&& auth.user.expires_in<60){
                console.log("Refreshing token")
                auth.signinSilent()
                    .then(()=>{
                        axios.defaults.headers.common['Authorization'] = 'Bearer ' + auth.user?.access_token
                    })
            }
        }, refreshInterval)
    }, [auth])

    return <>
        {children}
    </>
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
      <I18nextProvider i18n={i18n}>
          <Provider store={store}>
              <AuthWrapper>
                <RouterProvider router={router}/>
              </AuthWrapper>
          </Provider>
      </I18nextProvider>
  </React.StrictMode>,
)
