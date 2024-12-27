import audioPlayerInstance, {getAudioPlayerInstance} from './audio_player.js'

window.addEventListener('load', ()=>{
    const menuItemsSelect = document.querySelectorAll('.main-page-header span')
    menuItemsSelect.forEach(span=>{
        span.addEventListener('click', ()=>{
            console.log('clicked')
            menuItemsSelect.forEach(span=>{
                if (span.classList.contains('active')) {
                    span.classList.remove('active')
                }
            })
            span.classList.toggle('active')
        })
    })

    const audioPlayerButtons = document.querySelectorAll('.recent-listened i')
    audioPlayerButtons.forEach(button=>{
        button.addEventListener('click', ()=>{
            const audioPlayerInstance = getAudioPlayerInstance()
            audioPlayerInstance.loadSource(button.parentElement.getAttribute('data-url'))
            audioPlayerInstance.getSource().mediaElement.play()
            audioPlayerInstance.setVolume(0.5)
        })
    })

    const recentlyAdded = document.querySelectorAll('.recently-added i')
    recentlyAdded.forEach(button=>{
        button.addEventListener('click', ()=>{
            const audioPlayerInstance = getAudioPlayerInstance()
            audioPlayerInstance.loadSource(button.parentElement.getAttribute('data-url'))
            audioPlayerInstance.getSource().mediaElement.play()
            audioPlayerInstance.setVolume(0.5)
        })
    })
})