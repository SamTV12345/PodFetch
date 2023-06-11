import {useAuth} from "react-oidc-context";
import {useNavigate} from "react-router-dom";
import axios from "axios";
import {useTranslation} from "react-i18next";

export const OIDCLogin = () => {
    const auth = useAuth()
    const navigate = useNavigate()
    const {t} = useTranslation()

    if (auth.isAuthenticated && auth.user){
        axios.defaults.headers.common['Authorization'] = 'Bearer ' + auth.user.access_token;
        navigate("/")
    }



    return  <button  className="bg-blue-600 rounded pt-2 pb-2 w-full hover:bg-blue-500 active:scale-95" onClick={()=>{
        auth.signinRedirect()
    }}>{t('oidc-login')}</button>
}
