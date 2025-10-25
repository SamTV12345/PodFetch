import TimeAgo from 'javascript-time-ago'
import sanitizeHtml, {IOptions} from 'sanitize-html'
import en from 'javascript-time-ago/locale/en'
import de from 'javascript-time-ago/locale/de'
import fr from 'javascript-time-ago/locale/fr'
import pl from 'javascript-time-ago/locale/pl'
import es from 'javascript-time-ago/locale/es'
import i18n from "i18next";
import useCommon from "../store/CommonSlice";
import {Filter} from "../models/Filter";
import {OrderCriteria} from "../models/Order";
import {components} from "../../schema";
import type {AudioPlayerPlay} from "../store/AudioPlayerSlice";

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

export const preparePodcastEpisode = (episode: components["schemas"]["PodcastEpisodeDto"],  chapters: components['schemas']['PodcastEpisodeChapter'][], response?: components["schemas"]["EpisodeDto"],): AudioPlayerPlay => {
    return {
        podcastEpisode: {
          ...episode,
            local_url: episode.local_url,
            local_image_url: episode.local_image_url,
        },
        podcastHistoryItem: {
            ...response!,
            position: response === null? 0: response?.position?response.position: 0
        },
        chapters: chapters
    }
}

export const prepareOnlinePodcastEpisode = (episode: components["schemas"]["PodcastEpisodeDto"], chapters: components['schemas']['PodcastEpisodeChapter'][],  response?: components["schemas"]["EpisodeDto"]) : AudioPlayerPlay => {

    return {
        podcastEpisode: {
            ...episode,
        },
        podcastHistoryItem: {
            ...response!,
            position: response === null? 0: response?.position?response.position: 0
        },
        chapters: chapters
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


