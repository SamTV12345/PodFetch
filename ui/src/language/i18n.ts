import i18n from 'i18next'
import {initReactI18next} from "react-i18next";
import LanguageDetector from 'i18next-browser-languagedetector'
import da_translation from './json/da.json'
import de_translation from './json/de.json'
import en_translation from './json/en.json'
import fr_translation from './json/fr.json'
import pl_translation from './json/pl.json'
import es_translation from './json/es.json'

const resources = {
    da:{
        translation: da_translation
    },
    de: {
        translation: de_translation
    },
    en:{
        translation: en_translation
    },
    fr:{
        translation: fr_translation
    },
    pl:{
        translation: pl_translation
    },
    es:{
        translation: es_translation
    }
}

i18n
    .use(LanguageDetector)
    .use(initReactI18next)
    .init(
        {
            resources,
            fallbackLng: 'en'
        }
    )

export default i18n
