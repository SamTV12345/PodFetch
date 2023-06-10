import React, {FC, PropsWithChildren, useEffect} from "react";
import {useAuth} from "react-oidc-context";
import axios from "axios";
import useOnMount from "../hooks/useOnMount";

export const OIDCRefresher:FC<PropsWithChildren> = ({children})=>{
    const auth = useAuth()
    const refreshInterval = 1000*60

    useOnMount(()=>{
        const interval = setInterval(()=> {
            if (auth.user &&auth.user.expires_in&& auth.user.expires_in<70){
                auth.signinSilent()
                    .then(()=>{
                    })
            }
        }, refreshInterval)

        return ()=>clearInterval(interval)
    })

    useEffect(()=>{
        if(auth.user?.access_token){
            axios.defaults.headers.common['Authorization'] = 'Bearer ' + auth.user.access_token;
        }
    },[auth.user?.access_token])

    return <>
        {children}
    </>
}
