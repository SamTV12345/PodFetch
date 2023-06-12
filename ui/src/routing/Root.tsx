import {useEffect} from "react"
import axios from "axios"
import {Outlet, useNavigate} from "react-router-dom"
import {configWSUrl} from "../utils/Utilities"
import {useAppDispatch, useAppSelector} from "../store/hooks"
import {setLoginData} from "../store/CommonSlice"
import App from "../App"
import {AudioComponents} from "../components/AudioComponents"
import {Header} from "../components/Header"
import {Loading} from "../components/Loading"
import {MainContentPanel} from "../components/MainContentPanel"
import {Search} from "../components/Search"
import {Sidebar} from "../components/Sidebar"

export const Root = () => {
    const dispatch = useAppDispatch()
    const configModel = useAppSelector(state => state.common.configModel)
    const navigate = useNavigate()
    const auth = useAppSelector(state => state.common.loginData)

    const extractLoginData = (auth_local: string)=>{
        const test = atob(auth_local)
        const res = test.split(":")

        auth_local && dispatch(setLoginData({password: res[1], username: res[0]}))
        axios.defaults.headers.common['Authorization'] = 'Basic ' + auth_local;
    }

    useEffect(()=>{
        if(configModel){
            if(configModel.basicAuth){
                const auth_local =  localStorage.getItem('auth')
                const auth_session = sessionStorage.getItem('auth')
                if(auth_local == undefined && auth_session == undefined && !auth){
                    navigate("/login")
                }
                else if (auth_local && !auth){
                    extractLoginData(auth_local)
                }
                else if (auth_session && !auth){
                    extractLoginData(auth_session)
                }
                else if (auth){
                    axios.defaults.headers.common['Authorization'] = 'Basic ' + btoa(auth.username + ":" + auth.password);
                }
            }
            else if (configModel.oidcConfig && !axios.defaults.headers.common["Authorization"]){
                navigate("/login")
            }
        }
    },[configModel])

    if(!configModel || (configModel.basicAuth && !axios.defaults.headers.common["Authorization"]||(configModel.oidcConfigured&& !axios.defaults.headers.common["Authorization"]))){
        return <Loading/>
    }

    configWSUrl(configModel.serverUrl)

    return (
        <App>
            <Sidebar/>
            <MainContentPanel>
                <Header/>
                <div className="grid grid-rows-[1fr_auto] h-full">
                    <Outlet/>
                    <AudioComponents/>
                </div>
            </MainContentPanel>
            <Search/>
        </App>
    )
}
