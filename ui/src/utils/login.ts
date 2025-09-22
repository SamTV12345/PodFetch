type LoginObject = {
    loginType: 'oidc' | 'basic',
    rememberMe: boolean
}


export const LoginKey = 'login'

export const getLogin = (): LoginObject | null => {
    const item = localStorage.getItem(LoginKey) || sessionStorage.getItem(LoginKey)
    if (item) {
        return JSON.parse(item) as LoginObject
    }
    return null
}


export const setAuth = (auth: string) => {
    const login = getLogin()
    if (login) {
        if (login.loginType === 'basic') {
            if (login.rememberMe) {
                localStorage.setItem('auth', auth)
            } else {
                sessionStorage.setItem('auth', auth)
            }
        } else if (login.loginType === 'oidc') {
            if (login.rememberMe) {
                localStorage.setItem('auth', auth)
            } else {
                sessionStorage.setItem('auth', auth)
            }
        }
    }
}

export const setLogin = (login: LoginObject) => {
    localStorage.setItem('login', JSON.stringify(login))
}


export const removeLogin = () => {
    sessionStorage.removeItem('auth')
    localStorage.removeItem('auth')
}