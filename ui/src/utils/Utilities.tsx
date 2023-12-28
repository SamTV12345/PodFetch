import axios from "axios";
import TimeAgo from 'javascript-time-ago'
import sanitizeHtml, {IOptions} from 'sanitize-html'
import en from 'javascript-time-ago/locale/en'
import de from 'javascript-time-ago/locale/de'
import fr from 'javascript-time-ago/locale/fr'
import pl from 'javascript-time-ago/locale/pl'
import es from 'javascript-time-ago/locale/es'
import i18n from "i18next";
import useCommon, {PodcastEpisode} from "../store/CommonSlice";
import {PodcastWatchedModel} from "../models/PodcastWatchedModel";
import {Filter} from "../models/Filter";
import {OrderCriteria} from "../models/Order";
import {Episode} from "../models/Episode";

const defaultOptions: IOptions = {
    allowedTags: ['b', 'i', 'em', 'strong', 'a'],
    allowedAttributes: {
        'a': ['href', 'target']
    },
    allowedIframeHostnames: ['www.youtube.com']
};

i18n.on("languageChanged", (lng) => {
    timeago = new TimeAgo(lng)
})

TimeAgo.addDefaultLocale(en)
TimeAgo.addLocale(de)
TimeAgo.addLocale(pl)
TimeAgo.addLocale(es)
TimeAgo.addLocale(fr)

export const SKIPPED_TIME = 30
let timeago = new TimeAgo('en-US')

export let apiURL: string
export let uiURL: string
if (window.location.pathname.startsWith("/ui")) {
    apiURL = window.location.protocol + "//" + window.location.hostname + ":" + window.location.port + "/api/v1"
} else {
    //match everything before /ui
    const regex = /\/([^/]+)\/ui\//
    apiURL = window.location.protocol + "//" + window.location.hostname + ":" + window.location.port + "/" + regex.exec(window.location.href)![1] + "/api/v1"
}
uiURL = window.location.protocol + "//" + window.location.hostname + ":" + window.location.port + "/ui"

const wsEndpoint = "ws"

export const configWSUrl = (url: string) => {
    if (url.startsWith("http")) {
        return url.replace("http", "ws") + wsEndpoint
    }
    return url.replace("https", "wss") + wsEndpoint
}
export const logCurrentPlaybackTime = (episodeId: string, timeInSeconds: number) => {
    axios.post(apiURL + "/podcast/episode", {
        podcastEpisodeId: episodeId,
        time: Number(timeInSeconds.toFixed(0))
    })
}

export const formatTime = (isoDate: string) => {
    if(Number.isNaN(Date.parse(isoDate))) return ""
    if (isoDate === undefined) return ""
    return timeago.format(new Date(isoDate))
}

export const removeHTML = (html: string) => {
    html = html.split('<a').join('<a target="_blank"')
    return {
        __html: sanitizeHtml(html, defaultOptions)
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

export const preparePath = (path: string | undefined) => {
    if (path === undefined) return ""

    let pathToReturn = window.location.href.substring(0, window.location.href.indexOf('ui/')) + path.replaceAll(' ', '%20').replaceAll('#', '%23')

    if (useCommon.getState().loggedInUser?.apiKey && (useCommon.getState().configModel?.oidcConfig||useCommon.getState().configModel?.basicAuth)){
        if (pathToReturn.includes('?')) {
            pathToReturn += '&'
        }
        else {
            pathToReturn += '?'
        }
        pathToReturn += 'apiKey=' + useCommon.getState().loggedInUser?.apiKey
    }

    return pathToReturn
}

export const preparePodcastEpisode = (episode: PodcastEpisode, response: Episode) => {
    return {
        ...episode,
        local_url: preparePath(episode.local_url),
        local_image_url: preparePath(episode.local_image_url),
        time: response&&response.position?response.position: 0
    }
}


export const prependAPIKeyOnAuthEnabled = (url: string)=>{
    if (useCommon.getState().loggedInUser?.apiKey && (useCommon.getState().configModel?.oidcConfig||useCommon.getState().configModel?.basicAuth)) {
        if (url.includes('?')) {
            url += '&'
        }
        else {
            url += '?'
        }
        url += 'apiKey=' + useCommon.getState().loggedInUser?.apiKey
    }
    return url
}


export const prepareOnlinePodcastEpisode = (episode: PodcastEpisode, response: Episode) => {
    let online_url_with_proxy = window.location.href.substring(0, window.location.href.indexOf('ui/')) + 'proxy/podcast?episodeId=' + episode.episode_id

    console.log(useCommon.getState().loggedInUser?.apiKey)
    if (useCommon.getState().loggedInUser?.apiKey && (useCommon.getState().configModel?.oidcConfig||useCommon.getState().configModel?.basicAuth)) {
        if (online_url_with_proxy.includes('?')) {
            online_url_with_proxy += '&'
        }
        else {
            online_url_with_proxy += '?'
        }
        online_url_with_proxy += 'apiKey=' + useCommon.getState().loggedInUser?.apiKey
    }

    return {
        ...episode,
        local_url: online_url_with_proxy,
        local_image_url: episode.image_url,
        time: response&&response.position?response.position: 0
    }
}

export const getFiltersDefault = () => {
    return {
        ascending: true,
        filter: "PUBLISHEDDATE",
        onlyFavored: false,
        title: '',
        username: ''
    } satisfies Filter
}

export type OrderCriteriaSortingType = {
    sorting: OrderCriteria,
    ascending: boolean
}

export const TITLE_ASCENDING:OrderCriteriaSortingType = {
    sorting: OrderCriteria.TITLE,
    ascending: true
}

export const TIME_ASCENDING:OrderCriteriaSortingType = {
    sorting: OrderCriteria.PUBLISHEDDATE,
    ascending: true
}

export const TIME_DESCENDING:OrderCriteriaSortingType = {
    sorting: OrderCriteria.PUBLISHEDDATE,
    ascending: false
}

export const TITLE_DESCENDING:OrderCriteriaSortingType = {
    sorting: OrderCriteria.TITLE,
    ascending: false
}

export const decodeHTMLEntities = (() => {   const textArea = document.createElement('textarea');    return (message: string): string => {     textArea.innerHTML = message;     return textArea.value;   }; })();
