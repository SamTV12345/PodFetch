import {useState} from "react"
import i18n from "../language/i18n"
import {CustomSelect} from './CustomSelect'
import "material-symbols/outlined.css"

const languageOptions = [
    { value: 'de-DE', label: 'Deutsch' },
    { value: 'en', label: 'English' },
    { value: 'fr', label: 'Français' }
]

export const Dropdown = ()=>{
    const [language, setLanguage] = useState<string>(i18n.language)

    /* Responsiveness handled via stylesheet */
    return <CustomSelect className="i18n-dropdown" iconName="translate" onChange={(v)=>{setLanguage(v); i18n.changeLanguage(v)}} options={languageOptions} value={language}/>
}
