import i18n from 'i18next'
import {initReactI18next} from "react-i18next";
import LanguageDetector from 'i18next-browser-languagedetector'
import de_translation from './json/de.json'
import en_translation from './json/en.json'


const resources = {
    de: {
        translation:de_translation
    },
    en:{
        translation: en_translation
    }
}

i18n
    .use(LanguageDetector)
    .use(initReactI18next)
    .init(
        {
            resources
        }
    )

export default i18n
