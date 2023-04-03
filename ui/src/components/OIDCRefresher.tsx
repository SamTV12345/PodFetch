import React, {FC, PropsWithChildren, useEffect} from "react";
import {useAuth} from "react-oidc-context";
import axios from "axios";

export const OIDCRefresher:FC<PropsWithChildren> = ({children})=>{
    const auth = useAuth()
    const refreshInterval = 1000*60

    useEffect(()=>{

        const interval = setInterval(()=> {
            if (auth.user &&auth.user.expires_in&& auth.user.expires_in<70){
                auth.signinSilent()
                    .then(()=>{
                        axios.defaults.headers.common['Authorization'] = 'Bearer ' + auth.user?.access_token
                    })
            }
        }, refreshInterval)

        return ()=>clearInterval(interval)
    }, [])

    return <>
        {children}
    </>
}
