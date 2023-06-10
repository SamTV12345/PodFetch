import axios from "axios";
import TimeAgo from 'javascript-time-ago'
import sanitizeHtml, {IOptions} from 'sanitize-html'
import en from 'javascript-time-ago/locale/en'
import de from 'javascript-time-ago/locale/de'
import fr from 'javascript-time-ago/locale/fr'
import i18n from "i18next";
import {PodcastEpisode} from "../store/CommonSlice";
import {PodcastWatchedModel} from "../models/PodcastWatchedModel";
import {Filter} from "../models/Filter";

const defaultOptions: IOptions = {
    allowedTags: ['b', 'i', 'em', 'strong', 'a'],
    allowedAttributes: {
        'a': ['href']
    },
    allowedIframeHostnames: ['www.youtube.com']
};

i18n.on("languageChanged", (lng) => {
    timeago = new TimeAgo(lng)
})
TimeAgo.addDefaultLocale(de)
TimeAgo.addLocale(en)
TimeAgo.addLocale(fr)

export const SKIPPED_TIME = 30
let timeago = new TimeAgo('en-US')

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

export const formatTime = (isoDate: Date) => {
    return timeago.format(isoDate)
}

export const removeHTML = (html: string) => {
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


export const capitalizeFirstLetter = (string: string | undefined) => {
    if (string === undefined) return ""
    return string.charAt(0).toUpperCase() + string.slice(1);
}

export const preparePath = (path: string | undefined) => {
    if (path === undefined) return ""

    return window.location.href.substring(0, window.location.href.indexOf('ui/')) + path.replaceAll(' ', '%20').replaceAll('#', '%23')
}

export const preparePodcastEpisode = (episode: PodcastEpisode, response: PodcastWatchedModel) => {
    return {
        ...episode,
        local_url: preparePath(episode.local_url),
        local_image_url: preparePath(episode.local_image_url),
        time: response.watchedTime
    }
}


export const prepareOnlinePodcastEpisode = (episode: PodcastEpisode, response: PodcastWatchedModel) => {
    const strippedImage = episode.image_url.replace(/\?.*$/, "");
    const test = window.location.href.substring(0, window.location.href.indexOf('ui/')) + 'proxy/podcast?episodeId=' + episode.episode_id

    return {
        ...episode,
        local_url: test,
        local_image_url: strippedImage,
        time: response.watchedTime
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

