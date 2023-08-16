import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { UserAdminUsers } from '../components/UserAdminUsers'
import { UserAdminInvites } from '../components/UserAdminInvites'
import { Heading1 } from '../components/Heading1'
import 'material-symbols/outlined.css'

export const UserAdminPage = () => {
    const [selectedSection, setSelectedSection] = useState<string>('users')
    const { t } = useTranslation()

    return (
        <>
            <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-x-6 gap-y-6 mb-6 xs:mb-10">
                <Heading1 className="">{t('administration')}</Heading1>

                {/* Tabs */}
                <ul className="flex gap-2 border-b lg:border-none border-[--border-color] text-sm text-[--fg-secondary-color] w-full lg:w-auto">
                    <li className={`cursor-pointer inline-block px-2 py-3 ${selectedSection === 'users' && 'border-b-2 border-[--accent-color] text-[--accent-color]'}`} onClick={() => setSelectedSection('users')}>
                        <span className="flex items-center gap-2">
                            <span className="material-symbols-outlined filled leading-5">groups</span> {t('users')}
                        </span>
                    </li>
                    <li className={`cursor-pointer inline-block px-2 py-3 ${selectedSection === 'invites' && 'border-b-2 border-[--accent-color] text-[--accent-color]'}`} onClick={() => setSelectedSection('invites')}>
                        <span className="flex items-center gap-2">
                            <span className="material-symbols-outlined filled leading-5 text-xl">mail</span> {t('invites')}
                        </span>
                    </li>
                </ul>
            </div>

            {selectedSection === 'users' && (
                <UserAdminUsers />
            )}

            {selectedSection === 'invites' && (
                <UserAdminInvites />
            )}

        </>
    )
}
