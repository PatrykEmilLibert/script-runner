import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import plTranslation from '/locales/pl/translation.json';

const resources = {
  pl: {
    translation: plTranslation
  }
};

i18n
  .use(initReactI18next)
  .init({
    resources,
    lng: 'pl',
    fallbackLng: 'pl',
    interpolation: {
      escapeValue: false
    }
  });

export default i18n;
