import { useTranslation } from 'react-i18next'
import { useAuth } from 'react-oidc-context'
import { useNavigate } from 'react-router-dom'
import axios from 'axios'
import { CustomButtonPrimary } from './CustomButtonPrimary'
import useCommon from "../store/CommonSlice";

export const OIDCButton = () => {
    const auth = useAuth()
    const navigate = useNavigate()
    const { t } = useTranslation()
    const setLoginData = useCommon(state => state.setLoginData)

    if (auth.isAuthenticated && auth.user) {
        axios.defaults.headers.common['Authorization'] = 'Bearer ' + auth.user.id_token;
        setLoginData({username: auth.user.profile.preferred_username, password: ''})
        navigate('/')
    }

    return (
        <CustomButtonPrimary className="text-center w-full" onClick={() => {
            auth.signinRedirect()
        }}>{t('oidc-login')}</CustomButtonPrimary>
    )
}
