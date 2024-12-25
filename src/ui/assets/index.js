window.onload = ()=>{
    const mapOfCookies = new Map();
    document.cookie.split(';').forEach((cookie)=>{
        let [key, value] = cookie.split('=');
        if (key.trim().length === 0) return;
        mapOfCookies.set(key, value);
    })

    handleLanguageChange(mapOfCookies)
    handleThemeToggle(mapOfCookies)
}

function persistCookies(mapOfCookies) {
    for (let [key, value] of mapOfCookies) {
        document.cookie = `${key}=${value};`
    }
}

function handleThemeToggle(mapOfCookies) {
    const currentTheme = mapOfCookies.get('theme') || 'light';
    const themeToggleMode = document.getElementById('mode-selector');
    const themeToggleButtons = themeToggleMode.querySelectorAll('button')

    function unToggleButtons() {
        themeToggleButtons.forEach((button)=>{
            if (button.classList.contains('selected')) {
                button.classList.remove('selected')
            }
        })
    }

    themeToggleButtons.forEach((button)=>{
        if (button.id.split('-')[0] === currentTheme) {
            unToggleButtons()
            button.classList.add('selected')
        }
        button.addEventListener('click', (e)=>{
            mapOfCookies.set('theme', button.id.split('-')[0]);
            unToggleButtons()
            button.classList.add('selected')
            persistCookies(mapOfCookies)
        })
    })
}


function handleLanguageChange(mapOfCookies) {
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


    window.addEventListener('click', (event) => {
        if (!languageItems.classList.contains('hidden') && !languageItems.contains(event.target)) {
            handleToggleOfLanguage()
        }
    });
}