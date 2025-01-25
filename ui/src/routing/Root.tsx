import { useEffect } from 'react'
import { Outlet, useNavigate } from 'react-router-dom'
import useCommon from '../store/CommonSlice'
import App from '../App'
import { AudioComponents } from '../components/AudioComponents'
import { EpisodeSearchModal } from '../components/EpisodeSearchModal'
import { Header } from '../components/Header'
import { Loading } from '../components/Loading'
import { MainContentPanel } from '../components/MainContentPanel'
import { Sidebar } from '../components/Sidebar'
import {configWSUrl} from "../utils/navigationUtils";

export const Root = () => {
    const configModel = useCommon(state => state.configModel)
    const auth = useCommon(state => state.loginData)
    const navigate = useNavigate()
    const setLoginData = useCommon(state => state.setLoginData)
    const extractLoginData = (auth_local: string) => {
        const test = atob(auth_local)
        const res = test.split(':')

        auth_local && setLoginData({ password: res[1], username: res[0] })
        useCommon.getState().setHeaders({ Authorization: 'Basic ' + auth_local })
    }
    const isAuth = useCommon(state=>state.isAuthenticated)

    useEffect(() => {
        if (isAuth) {
            return
        }
        if (configModel) {
            if (configModel.basicAuth) {
                const auth_local =  localStorage.getItem('auth')
                const auth_session = sessionStorage.getItem('auth')

                if (auth_local == undefined && auth_session == undefined && !auth) {
                    navigate('/login')
                } else if (auth_local && !auth) {
                    extractLoginData(auth_local)
                } else if (auth_session && !auth) {
                    extractLoginData(auth_session)
                } else if (auth) {
                    useCommon.getState().setHeaders({ Authorization: 'Basic ' + btoa(auth.username + ':' + auth.password) })
                }
            } else if (configModel.oidcConfig && !isAuth){
                navigate('/login')
            }
        }
    }, [configModel, isAuth])

    if (!configModel || (configModel.basicAuth && !isAuth || (configModel.oidcConfigured && !isAuth))) {
        return <Loading />
    }

    configWSUrl(configModel.serverUrl)



    return (
        <App>
            <div className="grid grid-cols-[1fr] md:grid-cols-[18rem_1fr] grid-rows-[1fr_auto]">
                <Sidebar />
                <MainContentPanel>
                    <Header />
                    <div className="grid grid-rows-[1fr_auto] pb-8">
                        <Outlet />
                    </div>
                </MainContentPanel>
                <AudioComponents />
                <EpisodeSearchModal />
            </div>
        </App>
    )
}
