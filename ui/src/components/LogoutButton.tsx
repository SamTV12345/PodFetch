import {useAuth} from "react-oidc-context";
import {useTranslation} from "react-i18next";

export const LogoutButton = () => {
    const auth = useAuth()
    const {t} = useTranslation()
    return <button className="text-white bg-blue-600 rounded pt-2 pb-2 w-full hover:bg-blue-500 active:scale-95" onClick={()=>{
        auth.signoutRedirect()
    }}>{t('logout')}</button>
}
