import axios from "axios";
import TimeAgo from 'javascript-time-ago'
import de from 'javascript-time-ago/locale/de'
import sanitizeHtml,{IOptions} from 'sanitize-html'

const defaultOptions: IOptions = {
    allowedTags: [ 'b', 'i', 'em', 'strong', 'a' ],
    allowedAttributes: {
        'a': [ 'href' ]
    },
    allowedIframeHostnames: ['www.youtube.com']
};

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

export let apiURL: string
export let uiURL: string

if(isLocalhost && import.meta.env.DEV){
    apiURL="http://localhost:8000/api/v1"
    uiURL="http://localhost:5173/ui"
}
else {
    apiURL=window.location.protocol+"//"+window.location.hostname+":"+window.location.port+"/api/v1"
    uiURL=window.location.protocol+"//"+window.location.hostname+":"+window.location.port+"/ui"
}

const wsEndpoint = "ws"

export const configWSUrl = (url: string) => {
    if(url.startsWith("http")){
        return url.replace("http","ws")+wsEndpoint
    }
    return url.replace("https","wss")+wsEndpoint
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
    return {
        __html:sanitizeHtml(html, defaultOptions)
    }
}


export const isJsonString = (str: string) => {
    try {
        JSON.parse(str);
    } catch (e) {
        return false;
    }
    return true;
}
