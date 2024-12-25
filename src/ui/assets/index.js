window.onload = ()=>{
    const mapOfCookies = new Map();
    document.cookie.split(';').forEach((cookie)=>{
        let [key, value] = cookie.split('=');
        if (key.trim().length === 0) return;
        mapOfCookies.set(key.trim(), value);
    })

    handleLanguageChange(mapOfCookies)
    handleThemeToggle(mapOfCookies)
    handleNotificationClose()
}


function handleNotificationClose() {
    document.querySelectorAll('.notification-item .material-icons').forEach((icon) => {
        icon.addEventListener('click', (event) => {
            fetch('/api/v1/notifications/dismiss', {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    id: Number(icon.parentElement.getAttribute('data-id'))
                })
            })
                .then((response) => {
                    if (response.ok) {
                        icon.parentElement.remove()
                    }
                })
        })
    })
}

function persistCookies(mapOfCookies) {
    for (let [key, value] of mapOfCookies) {
        document.cookie = `${key}=${value};`
    }
}

function handleThemeToggle(mapOfCookies) {
    console.log("Cookies are", mapOfCookies)
    const currentTheme = mapOfCookies.get('theme') ?? 'light';
    console.log("Loaded theme is", mapOfCookies.get('theme'))
    const themeToggleMode = document.getElementById('mode-selector');
    const themeToggleButtons = themeToggleMode.querySelectorAll('button')


    function handleThemeChange(theme) {
        switch (theme) {
            case 'light':
                document.documentElement.classList.remove('dark')
                break;
            case 'dark':
                document.documentElement.classList.add('dark')
                break;
            default:
                window.matchMedia('(prefers-color-scheme: dark)').matches ? document.documentElement.classList.add('dark') : document.documentElement.classList.remove('dark')
        }
    }

    function unToggleButtons() {
        themeToggleButtons.forEach((button)=>{
            if (button.classList.contains('selected')) {
                button.classList.remove('selected')
            }
            handleThemeChange(button.id.split("-")[0])
        })
    }


    console.log("Current theme is: ", currentTheme)
    handleThemeChange(currentTheme)
    themeToggleButtons.forEach((button)=>{
        if (button.id.split('-')[0] === currentTheme) {
            button.classList.add('selected')
        }
    })

    themeToggleButtons.forEach((button)=>{
        button.addEventListener('click', ()=>{
            mapOfCookies.set('theme', button.id.split('-')[0]);
            unToggleButtons()
            button.classList.add('selected')
            handleThemeChange(button.id.split('-')[0])
            persistCookies(mapOfCookies)
        })
    })
}


function handleLanguageChange(mapOfCookies) {
    const notificationBell = document.getElementById('notification-bell');
    const notificationDropdown = document.getElementById('notification-dropdown');
    const languageSelect = document.getElementById('language-select')
    const languageSelectShow = document.querySelector('#language-select > span')
    let languageItems = document.getElementById('language-show')
    let languageChevron = document.querySelector('#language-select > .arrow')
    let languageCookieFound = false;

    languageSelectShow.innerText = mapOfCookies.get('language') || 'English';

    function handleToggleOfLanguage() {
        languageItems.classList.toggle('hidden')
        languageChevron.classList.toggle('rotate')
    }

    document.cookie.split(';').forEach((cookie)=>{
        let [key, value] = cookie.split('=');
        if(key === 'language'){
            languageCookieFound = true;
            languageSelectShow.innerText = value;
        }
    })
    if(!languageCookieFound){
        mapOfCookies.set('language', "English");
        persistCookies(mapOfCookies)
        languageSelectShow.innerText = 'English';
    }

    languageSelect.onclick = (e)=>{
        e.stopPropagation()
        handleToggleOfLanguage()
    }

    for (let item of languageItems.children) {
        item.onclick = ()=>{
            mapOfCookies.set('language', item.innerText);
            persistCookies(mapOfCookies)
            languageSelectShow.innerText = item.innerText;
            handleToggleOfLanguage()
        }
    }


    notificationBell.addEventListener('click', (event) => {
        event.stopPropagation();
        notificationDropdown.classList.toggle('show');
    });

    window.addEventListener('click', (event) => {
        if (!languageItems.classList.contains('hidden') && !languageItems.contains(event.target)) {
            handleToggleOfLanguage()
        }
        if (!notificationDropdown.contains(event.target)) {
            notificationDropdown.classList.remove('show');
        }
    });
}