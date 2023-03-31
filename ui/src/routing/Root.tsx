import {useEffect} from "react";
import {useAppDispatch, useAppSelector} from "../store/hooks";
import {Header} from "../components/Header";
import {SideBar} from "../components/SideBar";
import {Outlet} from "react-router-dom";
import {AudioComponents} from "../components/AudioComponents";
import {Search} from "../components/Search";
import axios, {AxiosResponse} from "axios";
import {apiURL, configWSUrl} from "../utils/Utilities";
import {ConfigModel} from "../models/SysInfo";
import {setConfigModel} from "../store/CommonSlice";
import {Loading} from "../components/Loading";
import App from "../App";


export const Root = () => {
    const sideBarCollapsed = useAppSelector(state => state.common.sideBarCollapsed)
    const dispatch = useAppDispatch()
    const configModel = useAppSelector(state => state.common.configModel)
    useEffect(()=>{
        axios.get(apiURL+"/sys/config").then((v:AxiosResponse<ConfigModel>)=>{
            dispatch(setConfigModel(v.data))
        })
    },[])

    if(!configModel){
        return <Loading/>
    }

    console.log(configModel.serverUrl)
    configWSUrl(configModel.serverUrl)

    return <App>
        <div className="grid  grid-rows-[auto_1fr] h-full md:grid-cols-[300px_1fr]">
            <Header/>
            <SideBar/>
            <div
                className={`col-span-6 md:col-span-5 ${sideBarCollapsed ? 'xs:col-span-5' : 'hidden'} md:block w-full overflow-x-auto`}>
                <div className="grid grid-rows-[1fr_auto] h-full ">
                    <Outlet/>
                    <AudioComponents/>
                </div>
            </div>
        </div>
        <Search/>
    </App>
}
