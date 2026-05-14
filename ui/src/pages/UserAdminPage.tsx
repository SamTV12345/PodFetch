import { useTranslation } from 'react-i18next'
import { Heading1 } from '../components/Heading1'
import { Mail, Users } from 'lucide-react'
import {NavLink, Outlet} from "react-router-dom";

export const UserAdminPage = () => {
    const { t } = useTranslation()

    return (
        <>
            <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-x-6 gap-y-6 mb-6 xs:mb-10">
                <Heading1 className="">{t('administration')}</Heading1>

                {/* Tabs */}
                <ul className="flex gap-2 border-b lg:border-none ui-border text-sm ui-text-muted w-full lg:w-auto settings-selector">
                    <li className="cursor-pointer inline-block px-2 py-3">
                        <NavLink to="users" className="block pb-1">
                            <span className="flex items-center gap-2">
                                <Users size={18} /> {t('users')}
                            </span>
                        </NavLink>
                    </li>
                    <li className="cursor-pointer inline-block px-2 py-3">
                        <NavLink to="invites" className="block pb-1">
                            <span className="flex items-center gap-2">
                                <Mail size={18} /> {t('invites')}
                            </span>
                        </NavLink>
                    </li>
                </ul>
            </div>

            <Outlet/>
        </>
    )
}
