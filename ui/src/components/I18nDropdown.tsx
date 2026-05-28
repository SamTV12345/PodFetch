import { useState } from 'react'
import i18n from '../language/i18n'
import { CustomSelect } from './CustomSelect'
import { Languages } from 'lucide-react'

const languageOptions = [
    { value: 'da', label: 'Dansk' },
    { value: 'de-DE', label: 'Deutsch' },
    { value: 'en', label: 'English' },
    { value: 'fr', label: 'Français' },
    { value: 'pl', label: 'Polski' },
    { value: 'es', label: 'Español' },
    { value: 'zh', label: '中文' }
]

export const LanguageDropdown = () => {
    const [language, setLanguage] = useState<string>(i18n.language)

    /* Responsiveness handled via stylesheet */
    return <CustomSelect className="i18n-dropdown" icon={<Languages size={16} />} onChange={(v)=>{setLanguage(v); i18n.changeLanguage(v)}} options={languageOptions} value={language}/>
}
