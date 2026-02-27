export const registerPwaServiceWorker = () => {
    if (!('serviceWorker' in navigator)) {
        return
    }

    if (import.meta.env.DEV) {
        return
    }

    window.addEventListener('load', () => {
        const swUrl = `${import.meta.env.BASE_URL}sw.js`
        navigator.serviceWorker.register(swUrl, { scope: import.meta.env.BASE_URL }).catch((error) => {
            console.error('Service worker registration failed', error)
        })
    })
}
