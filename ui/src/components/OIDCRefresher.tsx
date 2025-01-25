import { FC, PropsWithChildren, useEffect } from 'react'
import { useAuth } from 'react-oidc-context'
import axios from 'axios'
import useOnMount from '../hooks/useOnMount'
import useCommon from "../store/CommonSlice";

export const OIDCRefresher: FC<PropsWithChildren> = ({ children }) => {
    const auth = useAuth()
    const refreshInterval = 1000 * 60

    useOnMount(() => {
        const interval = setInterval(() => {
            if (auth.user && auth.user.expires_in && auth.user.expires_in < 70) {
                auth.signinSilent()
                    .then(() => {
                    })
            }
        }, refreshInterval)

        return () => clearInterval(interval)
    })

    useEffect(() => {
        if (auth.user?.id_token) {
            useCommon.getState().setHeaders({Authorization: 'Bearer ' + auth.user.id_token})
        }
    }, [auth.user?.id_token])

    return (
        <>
            {children}
        </>
    )
}
