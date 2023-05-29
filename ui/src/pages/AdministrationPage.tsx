import {useTranslation} from "react-i18next";
import {useNavigate} from "react-router-dom";

export const AdministrationPage = () => {
    const {t} = useTranslation()
    const navigate = useNavigate()

    return <div className="block static"><div className="grid grid-cols-1 md:grid-cols-2 p-5 gap-5">
        <button className="bg-slate-600 grid place-items-center text-3xl text-white" onClick={()=>{
            navigate('/administration/users')
        }}>
            <div className="flex gap-4">
                <i className="fa-solid fa-users"></i>
                {t('users')}
            </div>
        </button>
        <button className="bg-slate-600 grid place-items-center text-3xl" onClick={()=>{
            navigate('/administration/invites')
        }}>
            <div className="flex gap-4 text-white">
                <i className="fa-solid fa-envelope"></i>
                {t('invites')}
            </div>
        </button>
    </div></div>
}
