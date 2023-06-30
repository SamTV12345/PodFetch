import {lazy} from "react"

export const PodcastViewLazyLoad = lazy(()=>import('../pages/Podcasts').then(module=> {
    return{default:module["Podcasts"]}
}))

export const PodcastDetailViewLazyLoad = lazy(()=>import('../pages/PodcastDetailPage').then(module=> {
    return{default:module["PodcastDetailPage"]}
}))

export const PodcastInfoViewLazyLoad = lazy(()=>import('../pages/PodcastInfoPage').then(module=> {
    return{default:module["PodcastInfoPage"]}
}))

export const SettingsViewLazyLoad = lazy(()=>import('../pages/SettingsPage').then(module=> {
    return{default:module["SettingsPage"]}
}))

export const UserAdminViewLazyLoad = lazy(()=>import('../pages/UserAdminPage').then(module=> {
    return{default:module["UserAdminPage"]}
}))

export const TimeLineViewLazyLoad = lazy(()=>import('../pages/Timeline').then(module=> {
    return{default:module["Timeline"]}
}))

export const EpisodeSearchViewLazyLoad = lazy(()=>import('../pages/EpisodeSearchPage').then(module=> {
    return{default:module["EpisodeSearchPage"]}
}))
