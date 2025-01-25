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
import {components} from "../../schema";

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
export const VOLUME_STEP = 5
let timeago = new TimeAgo('en-US')


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

export const preparePodcastEpisode = (episode: components["schemas"]["PodcastEpisodeDto"], response: components["schemas"]["EpisodeDto"]) => {
    return {
        ...episode,
        local_url: episode.local_url,
        local_image_url: episode.local_image_url,
        time: response&&response.position?response.position: 0
    } satisfies components["schemas"]["PodcastEpisodeDto"] & {
        time: number
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


export const prepareOnlinePodcastEpisode = (episode: components["schemas"]["PodcastEpisodeDto"], response: components["schemas"]["EpisodeDto"]) => {
    let online_url_with_proxy = window.location.href.substring(0, window.location.href.indexOf('ui/')) + 'proxy/podcast?episodeId=' + episode.episode_id

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
    } satisfies components["schemas"]["PodcastEpisodeDto"] & {
        time: number
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

export const decodeHTMLEntities = (html: string): string => {
    const textArea = document.createElement('textarea');
    textArea.innerHTML = html;
    textArea.remove()
    return textArea.value;
}
