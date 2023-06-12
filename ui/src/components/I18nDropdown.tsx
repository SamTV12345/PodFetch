import {useState} from "react"
import i18n from "../language/i18n"
import {CustomSelect} from './CustomSelect'
import {LanguageIcon} from "../icons/LanguageIcon"
import "material-symbols/outlined.css"

const languageOptions = [
    { value: 'de-DE', label: 'Deutsch' },
    { value: 'en', label: 'English' },
    { value: 'fr', label: 'Français' }
]

export const Dropdown = ()=>{
    const [language, setLanguage] = useState<string>(i18n.language)

    return <CustomSelect iconClassName="text-stone-900" iconName="translate" onChange={(v)=>{setLanguage(v); i18n.changeLanguage(v)}} options={languageOptions} triggerClassName="border-0 text-stone-900" value={language}/>
}
