import axios from "axios";
import TimeAgo from 'javascript-time-ago'
import de from 'javascript-time-ago/locale/de'


TimeAgo.addDefaultLocale(de)
const timeago = new TimeAgo('de-DE')
export const isLocalhost = Boolean(
    window.location.hostname === 'localhost' ||
    // [::1] is the IPv6 localhost address.
    window.location.hostname === '[::1]' ||
    // 127.0.0.0/8 are considered localhost for IPv4.
    window.location.hostname.match(
        /^127(?:\.(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)){3}$/
    )
);

export let apiURL= ''
export let uiURL= ''

if(isLocalhost && import.meta.env.DEV){
    apiURL="http://localhost:8000/api/v1"
    uiURL="http://localhost:5173/ui"
}
else {
    apiURL=window.location.protocol+"//"+window.location.hostname+"/api/v1"
    uiURL=window.location.protocol+"//"+window.location.hostname+"/ui"
}


export  const logCurrentPlaybackTime = (episodeId: string,timeInSeconds: number)=> {
        axios.post(apiURL+"/podcast/episode", {
            podcastEpisodeId: episodeId,
            time: Number(timeInSeconds.toFixed(0))
        })
}

export const formatTime = (isoDate: string) => {
    return timeago.format(new Date(isoDate))
}

export const removeHTML = (html: string) => {
    return html.replace(/<[^>]*>?/gm, '');
}
