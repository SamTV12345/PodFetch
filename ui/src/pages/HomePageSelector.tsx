import {useTranslation} from "react-i18next";
import {useState} from "react";
import {Heading1} from "../components/Heading1";
import {UserAdminUsers} from "../components/UserAdminUsers";
import {UserAdminInvites} from "../components/UserAdminInvites";
import {Homepage} from "./Homepage";
import {PlaylistPage} from "./PlaylistPage";

type SelectableSection = 'home'|'playlist'
export const HomePageSelector = ()=>{
    const {t} = useTranslation()
    const [selectedSection, setSelectedSection] = useState<SelectableSection>('home')

    return (
        <>
            <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-x-6 gap-y-6 mb-6 xs:mb-10">
                <Heading1 className="">{t('homepage')}</Heading1>

                {/* Tabs */}
                <ul className="flex gap-2 border-b lg:border-none border-stone-200 text-sm text-stone-500 w-full lg:w-auto">
                    <li className={`cursor-pointer inline-block px-2 py-3 ${selectedSection === 'home' && 'border-b-2 border-mustard-600 text-mustard-600'}`} onClick={()=>setSelectedSection('home')}>
                        <span className="flex items-center gap-2">
                            <span className="material-symbols-outlined filled leading-5">home</span> {t('homepage')}
                        </span>
                    </li>
                    <li className={`cursor-pointer inline-block px-2 py-3 ${selectedSection === 'playlist' && 'border-b-2 border-mustard-600 text-mustard-600'}`} onClick={()=>setSelectedSection('playlist')}>
                        <span className="flex items-center gap-2">
                            <span className="material-symbols-outlined filled leading-5 text-xl">playlist_play</span> {t('playlists')}
                        </span>
                    </li>
                </ul>
            </div>

            {selectedSection === 'home' && (
                <Homepage />
            )}

            {selectedSection === 'playlist' && (
                <PlaylistPage />
            )}

        </>
    )
}
