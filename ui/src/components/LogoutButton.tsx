import {useAuth} from "react-oidc-context";
import {useTranslation} from "react-i18next";

export const LogoutButton = () => {
    const auth = useAuth()
    const {t} = useTranslation()
    return <button className="text-white" onClick={()=>{
        auth.signoutRedirect()
    }}>{t('logout')}</button>
}
