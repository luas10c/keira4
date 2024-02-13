import i18n from 'i18next'

i18n.init({
  lng: localStorage.getItem('@keira:lang') ?? 'en-US',
  fallbackLng: 'en-US'
})
