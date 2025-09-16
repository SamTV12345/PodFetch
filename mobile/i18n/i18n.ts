import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import translationEn from "./en.json";
import translationDe from "./de.json";
import { getLocales } from 'expo-localization';

const resources = {
    "de": { translation: translationDe },
    "en": { translation: translationEn },
};

i18n
    .use(initReactI18next)
    .init(
        {
            lng: getLocales()[0].languageCode!,
            resources,
            fallbackLng: 'en'
        }
    )

export default i18n
